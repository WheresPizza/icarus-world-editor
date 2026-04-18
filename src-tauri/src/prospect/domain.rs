use serde::{Deserialize, Serialize};

use super::property_engine::{Property, PropertyValue};

/// Simplified inventory item for the specialized view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub slot_index: i32,
    pub item_name: String,
    pub stack_count: i32,
    pub durability: Option<f32>,
}

/// Simplified deployable record for the specialized view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployableRecord {
    pub class_name: String,
    pub position: [f64; 3],
    pub rotation: [f64; 4],
    pub scale: [f64; 3],
}

/// Simplified player state for the specialized view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStateData {
    pub properties: Vec<Property>,
}

/// Attempt to extract inventory items from component properties
pub fn extract_inventory_items(properties: &[Property]) -> Vec<InventoryItem> {
    let mut items = Vec::new();

    for prop in properties {
        if prop.name == "SavedInventories" || prop.name == "Items" {
            if let PropertyValue::Array { items: array_items, .. } = &prop.value {
                if let super::property_engine::ArrayItems::Structs { items: struct_items, .. } = array_items {
                    for (i, item_props) in struct_items.iter().enumerate() {
                        let mut item = InventoryItem {
                            slot_index: i as i32,
                            item_name: String::new(),
                            stack_count: 1,
                            durability: None,
                        };

                        for p in item_props {
                            match (p.name.as_str(), &p.value) {
                                ("StaticItemDataRowName" | "ItemRowName", PropertyValue::Name(s) | PropertyValue::Str(s)) => {
                                    item.item_name = s.clone();
                                }
                                ("StackCount" | "ItemCount", PropertyValue::Int(v)) => {
                                    item.stack_count = *v;
                                }
                                ("Durability" | "CurrentDurability", PropertyValue::Float(v)) => {
                                    item.durability = Some(*v);
                                }
                                ("SlotIndex", PropertyValue::Int(v)) => {
                                    item.slot_index = *v;
                                }
                                _ => {}
                            }
                        }

                        if !item.item_name.is_empty() {
                            items.push(item);
                        }
                    }
                }
            }
        }

        // Recurse into nested structs
        if let PropertyValue::Struct { properties: inner, .. } = &prop.value {
            items.extend(extract_inventory_items(inner));
        }
    }

    items
}

/// Attempt to extract deployable info from component properties
pub fn extract_deployable(properties: &[Property]) -> Option<DeployableRecord> {
    let mut class_name = String::new();
    let mut position = [0.0f64; 3];
    let mut rotation = [0.0f64; 4];
    let mut scale = [1.0f64; 3];

    for prop in properties {
        match (prop.name.as_str(), &prop.value) {
            ("ObjectFName" | "ComponentClassName", PropertyValue::Str(s) | PropertyValue::Name(s)) => {
                if class_name.is_empty() {
                    class_name = s.clone();
                }
            }
            ("ActorTransform", PropertyValue::Struct { properties: transform_props, .. }) => {
                for tp in transform_props {
                    match tp.name.as_str() {
                        "Translation" => {
                            if let PropertyValue::Struct { properties: vec_props, .. } = &tp.value {
                                extract_vector(vec_props, &mut position);
                            }
                        }
                        "Rotation" => {
                            if let PropertyValue::Struct { properties: quat_props, .. } = &tp.value {
                                extract_quat(quat_props, &mut rotation);
                            }
                        }
                        "Scale3D" => {
                            if let PropertyValue::Struct { properties: vec_props, .. } = &tp.value {
                                extract_vector(vec_props, &mut scale);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if class_name.is_empty() {
        return None;
    }

    Some(DeployableRecord {
        class_name,
        position,
        rotation,
        scale,
    })
}

fn extract_vector(properties: &[Property], out: &mut [f64; 3]) {
    for prop in properties {
        match (prop.name.as_str(), &prop.value) {
            ("X", PropertyValue::Double(v)) => out[0] = *v,
            ("Y", PropertyValue::Double(v)) => out[1] = *v,
            ("Z", PropertyValue::Double(v)) => out[2] = *v,
            ("X", PropertyValue::Float(v)) => out[0] = *v as f64,
            ("Y", PropertyValue::Float(v)) => out[1] = *v as f64,
            ("Z", PropertyValue::Float(v)) => out[2] = *v as f64,
            _ => {}
        }
    }
}

fn extract_quat(properties: &[Property], out: &mut [f64; 4]) {
    for prop in properties {
        match (prop.name.as_str(), &prop.value) {
            ("X", PropertyValue::Double(v)) => out[0] = *v,
            ("Y", PropertyValue::Double(v)) => out[1] = *v,
            ("Z", PropertyValue::Double(v)) => out[2] = *v,
            ("W", PropertyValue::Double(v)) => out[3] = *v,
            ("X", PropertyValue::Float(v)) => out[0] = *v as f64,
            ("Y", PropertyValue::Float(v)) => out[1] = *v as f64,
            ("Z", PropertyValue::Float(v)) => out[2] = *v as f64,
            ("W", PropertyValue::Float(v)) => out[3] = *v as f64,
            _ => {}
        }
    }
}
