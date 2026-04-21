use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};

use super::error::ProspectError;
use super::types::ComponentSummary;

// ────────────────────────────────────────────────────────────
// UE4 Property Types
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PropertyValue {
    Int(i32),
    Int64(i64),
    UInt32(u32),
    UInt64(u64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Str(String),
    Name(String),
    Enum {
        enum_type: String,
        enum_value: String,
    },
    Byte {
        enum_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        enum_value: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        byte_value: Option<u8>,
    },
    Struct {
        struct_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        guid: Option<[u8; 16]>,
        properties: Vec<Property>,
    },
    Array {
        inner_type: String,
        items: ArrayItems,
    },
    Map {
        key_type: String,
        value_type: String,
        entries: Vec<MapEntry>,
    },
    /// Raw bytes for property types we don't fully parse
    Raw {
        prop_type: String,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ArrayItems {
    Bytes(Vec<u8>),
    Ints(Vec<i32>),
    Floats(Vec<f32>),
    Structs {
        struct_name: String,
        struct_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        guid: Option<[u8; 16]>,
        items: Vec<Vec<Property>>,
    },
    Names(Vec<String>),
    Strs(Vec<String>),
    Enums {
        enum_type: String,
        values: Vec<String>,
    },
    /// Fallback for array types we don't handle
    RawItems {
        inner_type: String,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapEntry {
    pub key: PropertyValue,
    pub value: PropertyValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    #[serde(flatten)]
    pub value: PropertyValue,
}

// ────────────────────────────────────────────────────────────
// Reading UE4 FString (length-prefixed, null-terminated)
// ────────────────────────────────────────────────────────────

fn read_fstring(cursor: &mut Cursor<&[u8]>) -> Result<String, ProspectError> {
    let len = cursor.read_i32::<LittleEndian>().map_err(|e| {
        ProspectError::PropertyParse {
            offset: cursor.position(),
            message: format!("Failed to read string length: {}", e),
        }
    })?;

    if len == 0 {
        return Ok(String::new());
    }

    if len > 0 {
        // UTF-8 string
        let mut buf = vec![0u8; len as usize];
        cursor.read_exact(&mut buf).map_err(|e| {
            ProspectError::PropertyParse {
                offset: cursor.position(),
                message: format!("Failed to read string data (len={}): {}", len, e),
            }
        })?;
        // Remove null terminator
        if let Some(last) = buf.last() {
            if *last == 0 {
                buf.pop();
            }
        }
        Ok(String::from_utf8_lossy(&buf).to_string())
    } else {
        // Negative length means UTF-16
        let char_count = (-len) as usize;
        let mut utf16 = Vec::with_capacity(char_count);
        for _ in 0..char_count {
            utf16.push(cursor.read_u16::<LittleEndian>().map_err(|e| {
                ProspectError::PropertyParse {
                    offset: cursor.position(),
                    message: format!("Failed to read UTF-16 char: {}", e),
                }
            })?);
        }
        // Remove null terminator
        if let Some(last) = utf16.last() {
            if *last == 0 {
                utf16.pop();
            }
        }
        Ok(String::from_utf16_lossy(&utf16))
    }
}

fn write_fstring(cursor: &mut Cursor<Vec<u8>>, s: &str) -> Result<(), ProspectError> {
    if s.is_empty() {
        cursor.write_i32::<LittleEndian>(0).map_err(|e| {
            ProspectError::PropertyParse {
                offset: cursor.position(),
                message: format!("Failed to write string length: {}", e),
            }
        })?;
        return Ok(());
    }

    let len = s.len() as i32 + 1; // +1 for null terminator
    cursor.write_i32::<LittleEndian>(len).map_err(|e| {
        ProspectError::PropertyParse {
            offset: cursor.position(),
            message: format!("Failed to write string length: {}", e),
        }
    })?;
    cursor.write_all(s.as_bytes()).map_err(|e| {
        ProspectError::PropertyParse {
            offset: cursor.position(),
            message: format!("Failed to write string data: {}", e),
        }
    })?;
    cursor.write_u8(0).map_err(|e| {
        ProspectError::PropertyParse {
            offset: cursor.position(),
            message: format!("Failed to write null terminator: {}", e),
        }
    })?;
    Ok(())
}

// ────────────────────────────────────────────────────────────
// Parse a None-terminated property list
// ────────────────────────────────────────────────────────────

pub fn read_properties(cursor: &mut Cursor<&[u8]>) -> Result<Vec<Property>, ProspectError> {
    let mut properties = Vec::new();

    loop {
        let name = read_fstring(cursor)?;
        if name == "None" || name.is_empty() {
            break;
        }

        let prop_type = read_fstring(cursor)?;
        let data_size = cursor.read_u64::<LittleEndian>().map_err(|e| {
            ProspectError::PropertyParse {
                offset: cursor.position(),
                message: format!("Failed to read property data size for '{}': {}", name, e),
            }
        })?;

        let value = read_property_value(cursor, &prop_type, data_size, &name)?;
        properties.push(Property { name, value });
    }

    Ok(properties)
}

fn read_property_value(
    cursor: &mut Cursor<&[u8]>,
    prop_type: &str,
    data_size: u64,
    prop_name: &str,
) -> Result<PropertyValue, ProspectError> {
    match prop_type {
        "IntProperty" => read_int_property(cursor),
        "Int64Property" => read_int64_property(cursor),
        "UInt32Property" => read_uint32_property(cursor),
        "UInt64Property" => read_uint64_property(cursor),
        "FloatProperty" => read_float_property(cursor),
        "DoubleProperty" => read_double_property(cursor),
        "BoolProperty" => read_bool_property(cursor),
        "StrProperty" => read_str_property(cursor),
        "NameProperty" => read_name_property(cursor),
        "EnumProperty" => read_enum_property(cursor),
        "ByteProperty" => read_byte_property(cursor, data_size),
        "StructProperty" => read_struct_property(cursor, data_size),
        "ArrayProperty" => read_array_property(cursor, data_size, prop_name),
        "MapProperty" => read_map_property(cursor, data_size),
        _ => {
            // Unknown property type - read raw bytes
            // Skip the tag byte (0x00 separator after size)
            let tag_byte = cursor.read_u8().map_err(|e| ProspectError::PropertyParse {
                offset: cursor.position(),
                message: format!("Failed to read tag byte for unknown type '{}': {}", prop_type, e),
            })?;
            let _ = tag_byte;

            let mut data = vec![0u8; data_size as usize];
            cursor.read_exact(&mut data).map_err(|e| {
                ProspectError::PropertyParse {
                    offset: cursor.position(),
                    message: format!(
                        "Failed to read raw data for '{}' (type={}, size={}): {}",
                        prop_name, prop_type, data_size, e
                    ),
                }
            })?;
            Ok(PropertyValue::Raw {
                prop_type: prop_type.to_string(),
                data,
            })
        }
    }
}

// ────────────────────────────────────────────────────────────
// Individual property type readers
// ────────────────────────────────────────────────────────────

fn read_int_property(cursor: &mut Cursor<&[u8]>) -> Result<PropertyValue, ProspectError> {
    let _tag = cursor.read_u8()?; // separator byte
    let value = cursor.read_i32::<LittleEndian>()?;
    Ok(PropertyValue::Int(value))
}

fn read_int64_property(cursor: &mut Cursor<&[u8]>) -> Result<PropertyValue, ProspectError> {
    let _tag = cursor.read_u8()?;
    let value = cursor.read_i64::<LittleEndian>()?;
    Ok(PropertyValue::Int64(value))
}

fn read_uint32_property(cursor: &mut Cursor<&[u8]>) -> Result<PropertyValue, ProspectError> {
    let _tag = cursor.read_u8()?;
    let value = cursor.read_u32::<LittleEndian>()?;
    Ok(PropertyValue::UInt32(value))
}

fn read_uint64_property(cursor: &mut Cursor<&[u8]>) -> Result<PropertyValue, ProspectError> {
    let _tag = cursor.read_u8()?;
    let value = cursor.read_u64::<LittleEndian>()?;
    Ok(PropertyValue::UInt64(value))
}

fn read_float_property(cursor: &mut Cursor<&[u8]>) -> Result<PropertyValue, ProspectError> {
    let _tag = cursor.read_u8()?;
    let value = cursor.read_f32::<LittleEndian>()?;
    Ok(PropertyValue::Float(value))
}

fn read_double_property(cursor: &mut Cursor<&[u8]>) -> Result<PropertyValue, ProspectError> {
    let _tag = cursor.read_u8()?;
    let value = cursor.read_f64::<LittleEndian>()?;
    Ok(PropertyValue::Double(value))
}

fn read_bool_property(cursor: &mut Cursor<&[u8]>) -> Result<PropertyValue, ProspectError> {
    // BoolProperty is special: the value is in the tag, not in the data section
    let value = cursor.read_u8()? != 0;
    let _tag = cursor.read_u8()?; // separator
    Ok(PropertyValue::Bool(value))
}

fn read_str_property(cursor: &mut Cursor<&[u8]>) -> Result<PropertyValue, ProspectError> {
    let _tag = cursor.read_u8()?;
    let value = read_fstring(cursor)?;
    Ok(PropertyValue::Str(value))
}

fn read_name_property(cursor: &mut Cursor<&[u8]>) -> Result<PropertyValue, ProspectError> {
    let _tag = cursor.read_u8()?;
    let value = read_fstring(cursor)?;
    Ok(PropertyValue::Name(value))
}

fn read_enum_property(cursor: &mut Cursor<&[u8]>) -> Result<PropertyValue, ProspectError> {
    let enum_type = read_fstring(cursor)?;
    let _tag = cursor.read_u8()?;
    let enum_value = read_fstring(cursor)?;
    Ok(PropertyValue::Enum {
        enum_type,
        enum_value,
    })
}

fn read_byte_property(
    cursor: &mut Cursor<&[u8]>,
    _data_size: u64,
) -> Result<PropertyValue, ProspectError> {
    let enum_type = read_fstring(cursor)?;
    let _tag = cursor.read_u8()?;

    if enum_type == "None" {
        // Simple byte value
        let value = cursor.read_u8()?;
        Ok(PropertyValue::Byte {
            enum_type,
            enum_value: None,
            byte_value: Some(value),
        })
    } else {
        // Named enum byte
        let enum_value = read_fstring(cursor)?;
        Ok(PropertyValue::Byte {
            enum_type,
            enum_value: Some(enum_value),
            byte_value: None,
        })
    }
}

fn read_struct_property(
    cursor: &mut Cursor<&[u8]>,
    data_size: u64,
) -> Result<PropertyValue, ProspectError> {
    let struct_type = read_fstring(cursor)?;

    // Read GUID (16 bytes)
    let mut guid = [0u8; 16];
    cursor.read_exact(&mut guid)?;
    let _tag = cursor.read_u8()?;

    let has_guid = guid != [0u8; 16];

    let start_pos = cursor.position();

    // Some struct types are "special" - they have a fixed binary layout
    let properties = match struct_type.as_str() {
        "Vector" => {
            vec![
                Property {
                    name: "X".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Y".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Z".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
            ]
        }
        "Rotator" => {
            vec![
                Property {
                    name: "Pitch".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Yaw".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Roll".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
            ]
        }
        "Quat" => {
            vec![
                Property {
                    name: "X".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Y".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Z".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "W".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
            ]
        }
        "Transform" => {
            // Transform = Quat rotation + Vector translation + Vector scale
            let rotation = vec![
                Property {
                    name: "X".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Y".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Z".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "W".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
            ];
            let translation = vec![
                Property {
                    name: "X".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Y".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Z".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
            ];
            let scale = vec![
                Property {
                    name: "X".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Y".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
                Property {
                    name: "Z".to_string(),
                    value: PropertyValue::Double(cursor.read_f64::<LittleEndian>()?),
                },
            ];
            vec![
                Property {
                    name: "Rotation".to_string(),
                    value: PropertyValue::Struct {
                        struct_type: "Quat".to_string(),
                        guid: None,
                        properties: rotation,
                    },
                },
                Property {
                    name: "Translation".to_string(),
                    value: PropertyValue::Struct {
                        struct_type: "Vector".to_string(),
                        guid: None,
                        properties: translation,
                    },
                },
                Property {
                    name: "Scale3D".to_string(),
                    value: PropertyValue::Struct {
                        struct_type: "Vector".to_string(),
                        guid: None,
                        properties: scale,
                    },
                },
            ]
        }
        "Guid" => {
            let mut bytes = [0u8; 16];
            cursor.read_exact(&mut bytes)?;
            vec![Property {
                name: "Guid".to_string(),
                value: PropertyValue::Raw {
                    prop_type: "Guid".to_string(),
                    data: bytes.to_vec(),
                },
            }]
        }
        "DateTime" => {
            let ticks = cursor.read_i64::<LittleEndian>()?;
            vec![Property {
                name: "Ticks".to_string(),
                value: PropertyValue::Int64(ticks),
            }]
        }
        "LinearColor" => {
            vec![
                Property {
                    name: "R".to_string(),
                    value: PropertyValue::Float(cursor.read_f32::<LittleEndian>()?),
                },
                Property {
                    name: "G".to_string(),
                    value: PropertyValue::Float(cursor.read_f32::<LittleEndian>()?),
                },
                Property {
                    name: "B".to_string(),
                    value: PropertyValue::Float(cursor.read_f32::<LittleEndian>()?),
                },
                Property {
                    name: "A".to_string(),
                    value: PropertyValue::Float(cursor.read_f32::<LittleEndian>()?),
                },
            ]
        }
        "IntPoint" => {
            vec![
                Property {
                    name: "X".to_string(),
                    value: PropertyValue::Int(cursor.read_i32::<LittleEndian>()?),
                },
                Property {
                    name: "Y".to_string(),
                    value: PropertyValue::Int(cursor.read_i32::<LittleEndian>()?),
                },
            ]
        }
        _ => {
            // Generic struct: read None-terminated property list
            read_properties(cursor)?
        }
    };

    // Verify we consumed exactly data_size bytes
    let consumed = cursor.position() - start_pos;
    if consumed != data_size {
        log::warn!(
            "Struct '{}' consumed {} bytes but expected {}",
            struct_type,
            consumed,
            data_size
        );
        // Seek to expected position
        cursor.set_position(start_pos + data_size);
    }

    Ok(PropertyValue::Struct {
        struct_type,
        guid: if has_guid { Some(guid) } else { None },
        properties,
    })
}

fn read_array_property(
    cursor: &mut Cursor<&[u8]>,
    data_size: u64,
    prop_name: &str,
) -> Result<PropertyValue, ProspectError> {
    let inner_type = read_fstring(cursor)?;
    let _tag = cursor.read_u8()?;

    let start_pos = cursor.position();
    let count = cursor.read_i32::<LittleEndian>()? as usize;

    let items = match inner_type.as_str() {
        "ByteProperty" => {
            let mut bytes = vec![0u8; count];
            cursor.read_exact(&mut bytes)?;
            ArrayItems::Bytes(bytes)
        }
        "IntProperty" => {
            let mut ints = Vec::with_capacity(count);
            for _ in 0..count {
                ints.push(cursor.read_i32::<LittleEndian>()?);
            }
            ArrayItems::Ints(ints)
        }
        "FloatProperty" => {
            let mut floats = Vec::with_capacity(count);
            for _ in 0..count {
                floats.push(cursor.read_f32::<LittleEndian>()?);
            }
            ArrayItems::Floats(floats)
        }
        "NameProperty" => {
            let mut names = Vec::with_capacity(count);
            for _ in 0..count {
                names.push(read_fstring(cursor)?);
            }
            ArrayItems::Names(names)
        }
        "StrProperty" => {
            let mut strs = Vec::with_capacity(count);
            for _ in 0..count {
                strs.push(read_fstring(cursor)?);
            }
            ArrayItems::Strs(strs)
        }
        "EnumProperty" => {
            let mut values = Vec::with_capacity(count);
            for _ in 0..count {
                values.push(read_fstring(cursor)?);
            }
            ArrayItems::Enums {
                enum_type: String::new(),
                values,
            }
        }
        "StructProperty" => {
            // Struct arrays have a special header
            let struct_array_name = read_fstring(cursor)?;
            let _struct_type_tag = read_fstring(cursor)?; // "StructProperty"
            let _struct_total_size = cursor.read_u64::<LittleEndian>()?;
            let struct_type = read_fstring(cursor)?;

            let mut guid = [0u8; 16];
            cursor.read_exact(&mut guid)?;
            let _sep = cursor.read_u8()?;

            let has_guid = guid != [0u8; 16];
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                let props = read_properties(cursor)?;
                items.push(props);
            }

            ArrayItems::Structs {
                struct_name: struct_array_name,
                struct_type,
                guid: if has_guid { Some(guid) } else { None },
                items,
            }
        }
        _ => {
            // Unknown array inner type - read remaining as raw bytes
            let remaining = data_size - (cursor.position() - start_pos);
            let mut data = vec![0u8; remaining as usize];
            cursor.read_exact(&mut data)?;
            ArrayItems::RawItems {
                inner_type: inner_type.clone(),
                data,
            }
        }
    };

    // Verify consumption
    let consumed = cursor.position() - start_pos;
    if consumed != data_size {
        log::warn!(
            "Array '{}' (inner={}) consumed {} bytes but expected {}",
            prop_name,
            inner_type,
            consumed,
            data_size
        );
        cursor.set_position(start_pos + data_size);
    }

    Ok(PropertyValue::Array {
        inner_type,
        items,
    })
}

fn read_map_property(
    cursor: &mut Cursor<&[u8]>,
    data_size: u64,
) -> Result<PropertyValue, ProspectError> {
    let key_type = read_fstring(cursor)?;
    let value_type = read_fstring(cursor)?;
    let _tag = cursor.read_u8()?;

    let start_pos = cursor.position();
    let _removal_count = cursor.read_i32::<LittleEndian>()?;
    let count = cursor.read_i32::<LittleEndian>()? as usize;

    let mut entries = Vec::with_capacity(count);

    for _ in 0..count {
        let key = match read_map_type_value(cursor, &key_type) {
            Ok(k) => k,
            Err(_) => {
                log::warn!(
                    "Map with key type '{}' or value type '{}' not fully supported, truncating entries",
                    key_type,
                    value_type
                );
                break;
            }
        };
        let value = match read_map_type_value(cursor, &value_type) {
            Ok(v) => v,
            Err(_) => {
                log::warn!(
                    "Map with key type '{}' or value type '{}' not fully supported, truncating entries",
                    key_type,
                    value_type
                );
                break;
            }
        };
        entries.push(MapEntry { key, value });
    }

    let consumed = cursor.position() - start_pos;
    if consumed != data_size {
        log::warn!(
            "Map consumed {} bytes but expected {}",
            consumed,
            data_size
        );
        cursor.set_position(start_pos + data_size);
    }

    Ok(PropertyValue::Map {
        key_type,
        value_type,
        entries,
    })
}

fn read_map_type_value(
    cursor: &mut Cursor<&[u8]>,
    type_name: &str,
) -> Result<PropertyValue, ProspectError> {
    match type_name {
        "IntProperty" => Ok(PropertyValue::Int(cursor.read_i32::<LittleEndian>()?)),
        "Int64Property" => Ok(PropertyValue::Int64(cursor.read_i64::<LittleEndian>()?)),
        "FloatProperty" => Ok(PropertyValue::Float(cursor.read_f32::<LittleEndian>()?)),
        "StrProperty" => Ok(PropertyValue::Str(read_fstring(cursor)?)),
        "NameProperty" => Ok(PropertyValue::Name(read_fstring(cursor)?)),
        "StructProperty" => {
            let properties = read_properties(cursor)?;
            Ok(PropertyValue::Struct {
                struct_type: "MapStruct".to_string(),
                guid: None,
                properties,
            })
        }
        _ => Err(ProspectError::UnsupportedPropertyType(format!(
            "Unsupported map type: {}",
            type_name
        ))),
    }
}

// ────────────────────────────────────────────────────────────
// Property writing (serialization)
// ────────────────────────────────────────────────────────────

pub fn write_properties(cursor: &mut Cursor<Vec<u8>>, properties: &[Property]) -> Result<(), ProspectError> {
    for prop in properties {
        write_fstring(cursor, &prop.name)?;

        match &prop.value {
            PropertyValue::Int(v) => {
                write_fstring(cursor, "IntProperty")?;
                cursor.write_u64::<LittleEndian>(4)?;
                cursor.write_u8(0)?;
                cursor.write_i32::<LittleEndian>(*v)?;
            }
            PropertyValue::Int64(v) => {
                write_fstring(cursor, "Int64Property")?;
                cursor.write_u64::<LittleEndian>(8)?;
                cursor.write_u8(0)?;
                cursor.write_i64::<LittleEndian>(*v)?;
            }
            PropertyValue::UInt32(v) => {
                write_fstring(cursor, "UInt32Property")?;
                cursor.write_u64::<LittleEndian>(4)?;
                cursor.write_u8(0)?;
                cursor.write_u32::<LittleEndian>(*v)?;
            }
            PropertyValue::UInt64(v) => {
                write_fstring(cursor, "UInt64Property")?;
                cursor.write_u64::<LittleEndian>(8)?;
                cursor.write_u8(0)?;
                cursor.write_u64::<LittleEndian>(*v)?;
            }
            PropertyValue::Float(v) => {
                write_fstring(cursor, "FloatProperty")?;
                cursor.write_u64::<LittleEndian>(4)?;
                cursor.write_u8(0)?;
                cursor.write_f32::<LittleEndian>(*v)?;
            }
            PropertyValue::Double(v) => {
                write_fstring(cursor, "DoubleProperty")?;
                cursor.write_u64::<LittleEndian>(8)?;
                cursor.write_u8(0)?;
                cursor.write_f64::<LittleEndian>(*v)?;
            }
            PropertyValue::Bool(v) => {
                write_fstring(cursor, "BoolProperty")?;
                cursor.write_u64::<LittleEndian>(0)?;
                cursor.write_u8(if *v { 1 } else { 0 })?;
                cursor.write_u8(0)?;
            }
            PropertyValue::Str(v) => {
                write_fstring(cursor, "StrProperty")?;
                let size = compute_fstring_size(v);
                cursor.write_u64::<LittleEndian>(size as u64)?;
                cursor.write_u8(0)?;
                write_fstring(cursor, v)?;
            }
            PropertyValue::Name(v) => {
                write_fstring(cursor, "NameProperty")?;
                let size = compute_fstring_size(v);
                cursor.write_u64::<LittleEndian>(size as u64)?;
                cursor.write_u8(0)?;
                write_fstring(cursor, v)?;
            }
            PropertyValue::Enum {
                enum_type,
                enum_value,
            } => {
                write_fstring(cursor, "EnumProperty")?;
                let size = compute_fstring_size(enum_value);
                cursor.write_u64::<LittleEndian>(size as u64)?;
                write_fstring(cursor, enum_type)?;
                cursor.write_u8(0)?;
                write_fstring(cursor, enum_value)?;
            }
            PropertyValue::Byte {
                enum_type,
                enum_value,
                byte_value,
            } => {
                write_fstring(cursor, "ByteProperty")?;
                if enum_type == "None" {
                    cursor.write_u64::<LittleEndian>(1)?;
                    write_fstring(cursor, enum_type)?;
                    cursor.write_u8(0)?;
                    cursor.write_u8(byte_value.unwrap_or(0))?;
                } else {
                    let size = compute_fstring_size(enum_value.as_deref().unwrap_or(""));
                    cursor.write_u64::<LittleEndian>(size as u64)?;
                    write_fstring(cursor, enum_type)?;
                    cursor.write_u8(0)?;
                    write_fstring(cursor, enum_value.as_deref().unwrap_or(""))?;
                }
            }
            PropertyValue::Struct {
                struct_type,
                guid,
                properties: props,
            } => {
                write_fstring(cursor, "StructProperty")?;

                // We need to compute size first by writing to a temp buffer
                let data_bytes = {
                    let mut temp = Cursor::new(Vec::new());
                    write_struct_data(&mut temp, struct_type, props)?;
                    temp.into_inner()
                };

                cursor.write_u64::<LittleEndian>(data_bytes.len() as u64)?;
                write_fstring(cursor, struct_type)?;

                let guid_bytes = guid.unwrap_or([0u8; 16]);
                cursor.write_all(&guid_bytes)?;
                cursor.write_u8(0)?;

                cursor.write_all(&data_bytes)?;
            }
            PropertyValue::Array { inner_type, items } => {
                write_fstring(cursor, "ArrayProperty")?;

                let data_bytes = {
                    let mut temp = Cursor::new(Vec::new());
                    write_array_data(&mut temp, inner_type, items, &prop.name)?;
                    temp.into_inner()
                };

                cursor.write_u64::<LittleEndian>(data_bytes.len() as u64)?;
                write_fstring(cursor, inner_type)?;
                cursor.write_u8(0)?;

                cursor.write_all(&data_bytes)?;
            }
            PropertyValue::Map {
                key_type,
                value_type,
                entries,
            } => {
                write_fstring(cursor, "MapProperty")?;

                let data_bytes = {
                    let mut temp = Cursor::new(Vec::new());
                    temp.write_i32::<LittleEndian>(0)?; // removal count
                    temp.write_i32::<LittleEndian>(entries.len() as i32)?;
                    for entry in entries {
                        write_map_type_value(&mut temp, &entry.key, key_type)?;
                        write_map_type_value(&mut temp, &entry.value, value_type)?;
                    }
                    temp.into_inner()
                };

                cursor.write_u64::<LittleEndian>(data_bytes.len() as u64)?;
                write_fstring(cursor, key_type)?;
                write_fstring(cursor, value_type)?;
                cursor.write_u8(0)?;

                cursor.write_all(&data_bytes)?;
            }
            PropertyValue::Raw { prop_type, data } => {
                write_fstring(cursor, prop_type)?;
                cursor.write_u64::<LittleEndian>(data.len() as u64)?;
                cursor.write_u8(0)?;
                cursor.write_all(data)?;
            }
        }
    }

    // Write None terminator
    write_fstring(cursor, "None")?;

    Ok(())
}

fn write_struct_data(
    cursor: &mut Cursor<Vec<u8>>,
    struct_type: &str,
    properties: &[Property],
) -> Result<(), ProspectError> {
    match struct_type {
        "Vector" => {
            for prop in properties {
                if let PropertyValue::Double(v) = &prop.value {
                    cursor.write_f64::<LittleEndian>(*v)?;
                }
            }
        }
        "Rotator" => {
            for prop in properties {
                if let PropertyValue::Double(v) = &prop.value {
                    cursor.write_f64::<LittleEndian>(*v)?;
                }
            }
        }
        "Quat" => {
            for prop in properties {
                if let PropertyValue::Double(v) = &prop.value {
                    cursor.write_f64::<LittleEndian>(*v)?;
                }
            }
        }
        "Transform" => {
            // Rotation (Quat), Translation (Vector), Scale3D (Vector)
            for prop in properties {
                if let PropertyValue::Struct { properties: inner, .. } = &prop.value {
                    for inner_prop in inner {
                        if let PropertyValue::Double(v) = &inner_prop.value {
                            cursor.write_f64::<LittleEndian>(*v)?;
                        }
                    }
                }
            }
        }
        "Guid" => {
            if let Some(prop) = properties.first() {
                if let PropertyValue::Raw { data, .. } = &prop.value {
                    cursor.write_all(data)?;
                }
            }
        }
        "DateTime" => {
            if let Some(prop) = properties.first() {
                if let PropertyValue::Int64(v) = &prop.value {
                    cursor.write_i64::<LittleEndian>(*v)?;
                }
            }
        }
        "LinearColor" => {
            for prop in properties {
                if let PropertyValue::Float(v) = &prop.value {
                    cursor.write_f32::<LittleEndian>(*v)?;
                }
            }
        }
        "IntPoint" => {
            for prop in properties {
                if let PropertyValue::Int(v) = &prop.value {
                    cursor.write_i32::<LittleEndian>(*v)?;
                }
            }
        }
        _ => {
            // Generic struct: write None-terminated property list
            write_properties(cursor, properties)?;
        }
    }
    Ok(())
}

fn write_array_data(
    cursor: &mut Cursor<Vec<u8>>,
    _inner_type: &str,
    items: &ArrayItems,
    _prop_name: &str,
) -> Result<(), ProspectError> {
    match items {
        ArrayItems::Bytes(bytes) => {
            cursor.write_i32::<LittleEndian>(bytes.len() as i32)?;
            cursor.write_all(bytes)?;
        }
        ArrayItems::Ints(ints) => {
            cursor.write_i32::<LittleEndian>(ints.len() as i32)?;
            for v in ints {
                cursor.write_i32::<LittleEndian>(*v)?;
            }
        }
        ArrayItems::Floats(floats) => {
            cursor.write_i32::<LittleEndian>(floats.len() as i32)?;
            for v in floats {
                cursor.write_f32::<LittleEndian>(*v)?;
            }
        }
        ArrayItems::Names(names) => {
            cursor.write_i32::<LittleEndian>(names.len() as i32)?;
            for name in names {
                write_fstring(cursor, name)?;
            }
        }
        ArrayItems::Strs(strs) => {
            cursor.write_i32::<LittleEndian>(strs.len() as i32)?;
            for s in strs {
                write_fstring(cursor, s)?;
            }
        }
        ArrayItems::Enums { values, .. } => {
            cursor.write_i32::<LittleEndian>(values.len() as i32)?;
            for v in values {
                write_fstring(cursor, v)?;
            }
        }
        ArrayItems::Structs {
            struct_name,
            struct_type,
            guid,
            items: struct_items,
        } => {
            cursor.write_i32::<LittleEndian>(struct_items.len() as i32)?;

            // Write struct array header
            write_fstring(cursor, struct_name)?;
            write_fstring(cursor, "StructProperty")?;

            // Compute total size of all struct items
            let items_data = {
                let mut temp = Cursor::new(Vec::new());
                for item in struct_items {
                    write_properties(&mut temp, item)?;
                }
                temp.into_inner()
            };

            cursor.write_u64::<LittleEndian>(items_data.len() as u64)?;
            write_fstring(cursor, struct_type)?;

            let guid_bytes = guid.unwrap_or([0u8; 16]);
            cursor.write_all(&guid_bytes)?;
            cursor.write_u8(0)?;

            cursor.write_all(&items_data)?;
        }
        ArrayItems::RawItems { data, .. } => {
            cursor.write_all(data)?;
        }
    }
    Ok(())
}

fn write_map_type_value(
    cursor: &mut Cursor<Vec<u8>>,
    value: &PropertyValue,
    _type_hint: &str,
) -> Result<(), ProspectError> {
    match value {
        PropertyValue::Int(v) => cursor.write_i32::<LittleEndian>(*v)?,
        PropertyValue::Int64(v) => cursor.write_i64::<LittleEndian>(*v)?,
        PropertyValue::Float(v) => cursor.write_f32::<LittleEndian>(*v)?,
        PropertyValue::Str(v) => write_fstring(cursor, v)?,
        PropertyValue::Name(v) => write_fstring(cursor, v)?,
        PropertyValue::Struct { properties, .. } => {
            write_properties(cursor, properties)?;
        }
        _ => {
            return Err(ProspectError::UnsupportedPropertyType(format!(
                "Cannot write map value type: {:?}",
                value
            )));
        }
    }
    Ok(())
}

fn compute_fstring_size(s: &str) -> usize {
    if s.is_empty() {
        4 // just the i32 length (0)
    } else {
        4 + s.len() + 1 // i32 length + bytes + null terminator
    }
}

// ────────────────────────────────────────────────────────────
// Prospect Blob Manager (lazy loading + dirty tracking)
// ────────────────────────────────────────────────────────────

pub struct ProspectBlob {
    /// Top-level properties (Version, LobbyPrivacy, ProspectMapName, etc.)
    pub top_level_props: Vec<Property>,
    /// Component entries with raw binary data
    pub components: Vec<ComponentData>,
}

pub struct ComponentData {
    pub class_name: String,
    pub raw_data: Vec<u8>,
    pub parsed: Option<Vec<Property>>,
    pub dirty: bool,
}

impl ProspectBlob {
    /// Parse the outer structure of the blob, extracting component metadata
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProspectError> {
        let mut cursor = Cursor::new(data);
        let all_props = read_properties(&mut cursor)?;

        let mut top_level_props = Vec::new();
        let mut components = Vec::new();

        for prop in all_props {
            if prop.name == "StateRecorderBlobs" {
                if let PropertyValue::Array {
                    items: ArrayItems::Structs { items, .. },
                    ..
                } = prop.value
                {
                    for item_props in items {
                        let mut class_name = String::new();
                        let mut binary_data: Vec<u8> = Vec::new();

                        for p in &item_props {
                            match (&p.name[..], &p.value) {
                                ("ComponentClassName", PropertyValue::Str(s)) => {
                                    class_name = s.clone();
                                }
                                ("BinaryData", PropertyValue::Array { items: ArrayItems::Bytes(bytes), .. }) => {
                                    binary_data = bytes.clone();
                                }
                                _ => {}
                            }
                        }

                        components.push(ComponentData {
                            class_name,
                            raw_data: binary_data,
                            parsed: None,
                            dirty: false,
                        });
                    }
                }
            } else {
                top_level_props.push(prop);
            }
        }

        Ok(ProspectBlob {
            top_level_props,
            components,
        })
    }

    /// Get summary of all components
    pub fn component_summaries(&self) -> Vec<ComponentSummary> {
        self.components
            .iter()
            .enumerate()
            .map(|(i, c)| ComponentSummary {
                index: i,
                class_name: c.class_name.clone(),
                data_size: c.raw_data.len(),
            })
            .collect()
    }

    /// Lazily parse a single component's binary data
    pub fn parse_component(&mut self, index: usize) -> Result<&Vec<Property>, ProspectError> {
        if index >= self.components.len() {
            return Err(ProspectError::ComponentNotFound(index));
        }

        if self.components[index].parsed.is_none() {
            let data = &self.components[index].raw_data;
            if data.is_empty() {
                self.components[index].parsed = Some(Vec::new());
            } else {
                let mut cursor = Cursor::new(data.as_slice());
                let props = read_properties(&mut cursor)?;
                self.components[index].parsed = Some(props);
            }
        }

        Ok(self.components[index].parsed.as_ref().unwrap())
    }

    /// Get the version from top-level props
    pub fn version(&self) -> Option<i32> {
        self.top_level_props.iter().find_map(|p| {
            if p.name == "Version" {
                if let PropertyValue::Int(v) = &p.value {
                    return Some(*v);
                }
            }
            None
        })
    }

    /// Get the map name from top-level props
    pub fn map_name(&self) -> Option<String> {
        self.top_level_props.iter().find_map(|p| {
            if p.name == "ProspectMapName" {
                if let PropertyValue::Str(v) = &p.value {
                    return Some(v.clone());
                }
            }
            None
        })
    }

    /// Get lobby privacy from top-level props
    pub fn lobby_privacy(&self) -> Option<String> {
        self.top_level_props.iter().find_map(|p| {
            if p.name == "LobbyPrivacy" {
                if let PropertyValue::Enum { enum_value, .. } = &p.value {
                    return Some(enum_value.clone());
                }
            }
            None
        })
    }

    /// Serialize the entire blob back to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, ProspectError> {
        let mut cursor = Cursor::new(Vec::new());

        // Rebuild the StateRecorderBlobs array
        let mut blob_items: Vec<Vec<Property>> = Vec::new();
        for component in &self.components {
            let binary_data = if component.dirty {
                // Re-serialize the parsed (and modified) properties
                if let Some(props) = &component.parsed {
                    let mut temp = Cursor::new(Vec::new());
                    write_properties(&mut temp, props)?;
                    temp.into_inner()
                } else {
                    component.raw_data.clone()
                }
            } else {
                // Use original raw data
                component.raw_data.clone()
            };

            let item = vec![
                Property {
                    name: "ComponentClassName".to_string(),
                    value: PropertyValue::Str(component.class_name.clone()),
                },
                Property {
                    name: "BinaryData".to_string(),
                    value: PropertyValue::Array {
                        inner_type: "ByteProperty".to_string(),
                        items: ArrayItems::Bytes(binary_data),
                    },
                },
            ];
            blob_items.push(item);
        }

        // Build the full property list
        let mut all_props = Vec::new();

        // StateRecorderBlobs must come first (matching original order)
        all_props.push(Property {
            name: "StateRecorderBlobs".to_string(),
            value: PropertyValue::Array {
                inner_type: "StructProperty".to_string(),
                items: ArrayItems::Structs {
                    struct_name: "StateRecorderBlobs".to_string(),
                    struct_type: "StateRecorderBlob".to_string(),
                    guid: None,
                    items: blob_items,
                },
            },
        });

        // Then the rest of the top-level props
        for prop in &self.top_level_props {
            all_props.push(prop.clone());
        }

        write_properties(&mut cursor, &all_props)?;

        Ok(cursor.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn load_test_blob() -> Option<Vec<u8>> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
        let path = PathBuf::from(manifest_dir)
            .parent()?
            .parent()?
            .join("Pomoyka.json");

        if !path.exists() {
            return None;
        }

        let (_, decompressed) = super::super::envelope::read_prospect_blob(&path).ok()?;
        Some(decompressed)
    }

    #[test]
    fn test_parse_outer_structure() {
        let Some(data) = load_test_blob() else {
            eprintln!("Test file not found, skipping");
            return;
        };

        let blob = ProspectBlob::from_bytes(&data).unwrap();

        assert!(blob.components.len() > 100, "Expected many components");
        println!("Total components: {}", blob.components.len());
        println!("Version: {:?}", blob.version());
        println!("Map name: {:?}", blob.map_name());

        // Count components by type
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for c in &blob.components {
            *type_counts.entry(c.class_name.clone()).or_default() += 1;
        }
        let mut counts: Vec<_> = type_counts.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1));
        for (name, count) in &counts[..counts.len().min(10)] {
            println!("  {} ({})", name, count);
        }
    }

    #[test]
    fn test_parse_individual_components() {
        let Some(data) = load_test_blob() else {
            eprintln!("Test file not found, skipping");
            return;
        };

        let mut blob = ProspectBlob::from_bytes(&data).unwrap();

        let mut parse_errors = 0;
        let total = blob.components.len();

        for i in 0..total {
            match blob.parse_component(i) {
                Ok(_) => {}
                Err(e) => {
                    parse_errors += 1;
                    if parse_errors <= 5 {
                        eprintln!(
                            "Parse error for component {} ({}): {}",
                            i, blob.components[i].class_name, e
                        );
                    }
                }
            }
        }

        println!(
            "Parsed {}/{} components successfully ({} errors)",
            total - parse_errors,
            total,
            parse_errors
        );
        assert!(
            parse_errors < total / 10,
            "Too many parse errors: {}/{}",
            parse_errors,
            total
        );
    }

    #[test]
    fn test_round_trip_blob() {
        let Some(data) = load_test_blob() else {
            eprintln!("Test file not found, skipping");
            return;
        };

        let blob = ProspectBlob::from_bytes(&data).unwrap();
        let rewritten = blob.to_bytes().unwrap();

        if data != rewritten {
            // Find first difference
            let min_len = data.len().min(rewritten.len());
            for i in 0..min_len {
                if data[i] != rewritten[i] {
                    eprintln!(
                        "First diff at byte {}: original=0x{:02x}, rewritten=0x{:02x}",
                        i, data[i], rewritten[i]
                    );
                    break;
                }
            }
            if data.len() != rewritten.len() {
                eprintln!(
                    "Length mismatch: original={}, rewritten={}",
                    data.len(),
                    rewritten.len()
                );
            }
            panic!("Round-trip failed: data does not match");
        }
    }
}
