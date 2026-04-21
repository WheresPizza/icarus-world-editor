use serde::{Deserialize, Serialize};

use super::property_engine::{Property, PropertyValue};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProspectDiff {
    pub metadata_changes: Vec<FieldDiff>,
    pub added_components: Vec<String>,
    pub removed_components: Vec<String>,
    pub modified_components: Vec<ComponentDiff>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentDiff {
    pub component_name: String,
    pub component_class: String,
    pub property_changes: Vec<PropertyDiff>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyDiff {
    pub path: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldDiff {
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

/// Compare two property lists and collect diffs
pub fn diff_properties(
    props_a: &[Property],
    props_b: &[Property],
    prefix: &str,
    diffs: &mut Vec<PropertyDiff>,
) {
    for prop_a in props_a {
        let path = if prefix.is_empty() {
            prop_a.name.clone()
        } else {
            format!("{}.{}", prefix, prop_a.name)
        };

        if let Some(prop_b) = props_b.iter().find(|p| p.name == prop_a.name) {
            // Both have this property - compare values
            let val_a = property_value_to_string(&prop_a.value);
            let val_b = property_value_to_string(&prop_b.value);

            if val_a != val_b {
                // Try to recurse into structs
                match (&prop_a.value, &prop_b.value) {
                    (
                        PropertyValue::Struct { properties: inner_a, .. },
                        PropertyValue::Struct { properties: inner_b, .. },
                    ) => {
                        diff_properties(inner_a, inner_b, &path, diffs);
                    }
                    _ => {
                        diffs.push(PropertyDiff {
                            path,
                            old_value: val_a,
                            new_value: val_b,
                        });
                    }
                }
            }
        } else {
            // Property in A but not in B
            diffs.push(PropertyDiff {
                path,
                old_value: property_value_to_string(&prop_a.value),
                new_value: String::from("<removed>"),
            });
        }
    }

    // Properties in B but not in A
    for prop_b in props_b {
        let path = if prefix.is_empty() {
            prop_b.name.clone()
        } else {
            format!("{}.{}", prefix, prop_b.name)
        };
        if !props_a.iter().any(|p| p.name == prop_b.name) {
            diffs.push(PropertyDiff {
                path,
                old_value: String::from("<added>"),
                new_value: property_value_to_string(&prop_b.value),
            });
        }
    }
}

pub fn property_value_to_string(value: &PropertyValue) -> String {
    match value {
        PropertyValue::Int(v) => v.to_string(),
        PropertyValue::Int64(v) => v.to_string(),
        PropertyValue::UInt32(v) => v.to_string(),
        PropertyValue::UInt64(v) => v.to_string(),
        PropertyValue::Float(v) => format!("{:.4}", v),
        PropertyValue::Double(v) => format!("{:.4}", v),
        PropertyValue::Bool(v) => v.to_string(),
        PropertyValue::Str(v) | PropertyValue::Name(v) => v.clone(),
        PropertyValue::Enum { enum_value, .. } => enum_value.clone(),
        PropertyValue::Byte { byte_value: Some(b), .. } => b.to_string(),
        PropertyValue::Byte { enum_value: Some(e), .. } => e.clone(),
        PropertyValue::Byte { .. } => String::from("<byte>"),
        PropertyValue::Struct { struct_type, properties, .. } => {
            format!("{{{}:{} fields}}", struct_type, properties.len())
        }
        PropertyValue::Array { inner_type, items, .. } => {
            let count = match items {
                super::property_engine::ArrayItems::Bytes(b) => b.len(),
                super::property_engine::ArrayItems::Ints(v) => v.len(),
                super::property_engine::ArrayItems::Floats(v) => v.len(),
                super::property_engine::ArrayItems::Names(v) => v.len(),
                super::property_engine::ArrayItems::Strs(v) => v.len(),
                super::property_engine::ArrayItems::Enums { values, .. } => values.len(),
                super::property_engine::ArrayItems::Structs { items, .. } => items.len(),
                super::property_engine::ArrayItems::RawItems { .. } => 0,
            };
            format!("[Array<{}>: {} items]", inner_type, count)
        }
        PropertyValue::Map { key_type, value_type, entries } => {
            format!("{{Map<{},{}>: {} entries}}", key_type, value_type, entries.len())
        }
        PropertyValue::Raw { prop_type, data } => {
            format!("[Raw<{}>: {} bytes]", prop_type, data.len())
        }
    }
}
