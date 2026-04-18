use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::State;

use crate::prospect::backup;
use crate::prospect::domain;
use crate::prospect::envelope;
use crate::prospect::error::ProspectError;
use crate::prospect::property_engine::{ArrayItems, Property, PropertyValue, ProspectBlob};
use crate::prospect::types::*;

pub struct AppState {
    pub config: AppConfig,
    pub open_prospects: HashMap<String, OpenProspect>,
}

pub struct OpenProspect {
    pub file_path: PathBuf,
    pub info: ProspectInfo,
    pub blob: ProspectBlob,
    pub original_key: String,
    pub info_dirty: bool,
}

// ────────────────────────────────────────────────────────────
// Prospect listing
// ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_prospects(
    dir: String,
    _state: State<'_, Mutex<AppState>>,
) -> Result<Vec<ProspectSummary>, String> {
    let path = PathBuf::from(&dir);
    envelope::list_prospect_files(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn auto_detect_prospects_dir() -> Option<String> {
    envelope::auto_detect_prospects_dir().map(|p| p.to_string_lossy().to_string())
}

// ────────────────────────────────────────────────────────────
// Prospect loading
// ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_prospect_overview(
    path: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<ProspectOverview, String> {
    let file_path = PathBuf::from(&path);

    let (file, decompressed) = envelope::read_prospect_blob(&file_path).map_err(|e| e.to_string())?;

    let blob = ProspectBlob::from_bytes(&decompressed).map_err(|e| e.to_string())?;

    let overview = ProspectOverview {
        prospect_info: file.prospect_info.clone(),
        blob_version: blob.version(),
        lobby_privacy: blob.lobby_privacy(),
        prospect_map_name: blob.map_name(),
        components: blob.component_summaries(),
        total_components: blob.components.len(),
    };

    // Store in state for later access
    let prospect_id = file.prospect_info.prospect_id.clone();
    let mut state = state.lock().unwrap();
    state.open_prospects.insert(
        prospect_id,
        OpenProspect {
            file_path,
            info: file.prospect_info,
            blob,
            original_key: file.prospect_blob.key,
            info_dirty: false,
        },
    );

    Ok(overview)
}

// ────────────────────────────────────────────────────────────
// Component access (lazy loading)
// ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_component_details(
    prospect_id: String,
    index: usize,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<Property>, String> {
    let mut state = state.lock().unwrap();
    let prospect = state
        .open_prospects
        .get_mut(&prospect_id)
        .ok_or_else(|| format!("Prospect '{}' not loaded", prospect_id))?;

    let props = prospect
        .blob
        .parse_component(index)
        .map_err(|e| e.to_string())?;

    Ok(props.clone())
}

#[tauri::command]
pub fn get_domain_view(
    prospect_id: String,
    index: usize,
    state: State<'_, Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    let mut state = state.lock().unwrap();
    let prospect = state
        .open_prospects
        .get_mut(&prospect_id)
        .ok_or_else(|| format!("Prospect '{}' not loaded", prospect_id))?;

    let class_name = prospect.blob.components[index].class_name.clone();
    let props = prospect
        .blob
        .parse_component(index)
        .map_err(|e| e.to_string())?;

    // Depending on component type, return a domain-specific view
    if class_name.contains("PlayerState") || class_name.contains("Container") {
        let items = domain::extract_inventory_items(props);
        Ok(serde_json::json!({
            "type": "inventory",
            "items": items
        }))
    } else if class_name.contains("Deployable") || class_name.contains("BuildingGrid") {
        if let Some(deployable) = domain::extract_deployable(props) {
            Ok(serde_json::json!({
                "type": "deployable",
                "data": deployable
            }))
        } else {
            Ok(serde_json::json!({
                "type": "generic",
                "properties": props
            }))
        }
    } else {
        Ok(serde_json::json!({
            "type": "generic",
            "properties": props
        }))
    }
}

// ────────────────────────────────────────────────────────────
// Editing
// ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn update_metadata(
    prospect_id: String,
    info: ProspectInfo,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let prospect = state
        .open_prospects
        .get_mut(&prospect_id)
        .ok_or_else(|| format!("Prospect '{}' not loaded", prospect_id))?;

    prospect.info = info;
    prospect.info_dirty = true;
    Ok(())
}

#[tauri::command]
pub fn update_component_property(
    prospect_id: String,
    component_index: usize,
    property_path: String,
    value: serde_json::Value,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let prospect = state
        .open_prospects
        .get_mut(&prospect_id)
        .ok_or_else(|| format!("Prospect '{}' not loaded", prospect_id))?;

    // Ensure component is parsed
    prospect
        .blob
        .parse_component(component_index)
        .map_err(|e| e.to_string())?;

    // Update the property at the given path
    let component = &mut prospect.blob.components[component_index];
    if let Some(props) = &mut component.parsed {
        update_property_at_path(props, &property_path, &value).map_err(|e| e.to_string())?;
        component.dirty = true;
    }

    Ok(())
}

fn update_property_at_path(
    properties: &mut Vec<Property>,
    path: &str,
    value: &serde_json::Value,
) -> Result<(), ProspectError> {
    let parts: Vec<&str> = path.splitn(2, '.').collect();
    let name = parts[0];

    // Handle array indexing: "ArrayName[0]"
    let (prop_name, array_index) = if let Some(bracket_pos) = name.find('[') {
        let idx_str = &name[bracket_pos + 1..name.len() - 1];
        let idx: usize = idx_str
            .parse()
            .map_err(|_| ProspectError::PropertyPathNotFound(path.to_string()))?;
        (&name[..bracket_pos], Some(idx))
    } else {
        (name, None)
    };

    let prop = properties
        .iter_mut()
        .find(|p| p.name == prop_name)
        .ok_or_else(|| ProspectError::PropertyPathNotFound(path.to_string()))?;

    if parts.len() == 1 && array_index.is_none() {
        // Leaf: update the value
        update_property_value(&mut prop.value, value)?;
        return Ok(());
    }

    // Navigate deeper
    let remaining = if parts.len() > 1 { parts[1] } else { "" };

    match &mut prop.value {
        PropertyValue::Struct { properties: inner, .. } => {
            update_property_at_path(inner, remaining, value)
        }
        PropertyValue::Array { items, .. } => {
            if let Some(idx) = array_index {
                match items {
                    ArrayItems::Structs { items: struct_items, .. } => {
                        if idx < struct_items.len() {
                            if remaining.is_empty() {
                                return Err(ProspectError::PropertyPathNotFound(
                                    "Cannot set entire array element".to_string(),
                                ));
                            }
                            update_property_at_path(&mut struct_items[idx], remaining, value)
                        } else {
                            Err(ProspectError::PropertyPathNotFound(format!(
                                "Array index {} out of bounds",
                                idx
                            )))
                        }
                    }
                    _ => Err(ProspectError::PropertyPathNotFound(
                        "Array indexing only supported for struct arrays".to_string(),
                    )),
                }
            } else {
                Err(ProspectError::PropertyPathNotFound(
                    "Expected array index".to_string(),
                ))
            }
        }
        _ => Err(ProspectError::PropertyPathNotFound(format!(
            "Cannot navigate into property type at '{}'",
            prop_name
        ))),
    }
}

fn update_property_value(
    prop_value: &mut PropertyValue,
    json_value: &serde_json::Value,
) -> Result<(), ProspectError> {
    use PropertyValue;

    match prop_value {
        PropertyValue::Int(v) => {
            *v = json_value.as_i64().unwrap_or(0) as i32;
        }
        PropertyValue::Int64(v) => {
            *v = json_value.as_i64().unwrap_or(0);
        }
        PropertyValue::Float(v) => {
            *v = json_value.as_f64().unwrap_or(0.0) as f32;
        }
        PropertyValue::Double(v) => {
            *v = json_value.as_f64().unwrap_or(0.0);
        }
        PropertyValue::Bool(v) => {
            *v = json_value.as_bool().unwrap_or(false);
        }
        PropertyValue::Str(v) | PropertyValue::Name(v) => {
            *v = json_value.as_str().unwrap_or("").to_string();
        }
        PropertyValue::Enum { enum_value, .. } => {
            *enum_value = json_value.as_str().unwrap_or("").to_string();
        }
        _ => {
            return Err(ProspectError::UnsupportedPropertyType(
                "Cannot directly update this property type".to_string(),
            ));
        }
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────
// Saving
// ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn save_prospect(
    prospect_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let mut state = state.lock().unwrap();

    // Auto-backup before save if configured
    let should_backup = state.config.auto_backup_on_save;
    let backup_dir = state.config.backup_dir.clone();

    let prospect = state
        .open_prospects
        .get(&prospect_id)
        .ok_or_else(|| format!("Prospect '{}' not loaded", prospect_id))?;

    let file_path = prospect.file_path.clone();

    if should_backup {
        if let Some(ref backup_dir) = backup_dir {
            let _ = backup::backup_prospect(&file_path, &PathBuf::from(backup_dir));
        }
    }

    let prospect = state
        .open_prospects
        .get(&prospect_id)
        .ok_or_else(|| format!("Prospect '{}' not loaded", prospect_id))?;

    let blob_bytes = prospect.blob.to_bytes().map_err(|e| e.to_string())?;

    envelope::write_prospect(&file_path, &prospect.info, &blob_bytes, &prospect.original_key)
        .map_err(|e| e.to_string())?;

    // Mark everything as clean
    let prospect = state
        .open_prospects
        .get_mut(&prospect_id)
        .ok_or_else(|| format!("Prospect '{}' not loaded", prospect_id))?;
    prospect.info_dirty = false;
    for component in &mut prospect.blob.components {
        component.dirty = false;
    }

    Ok(file_path.to_string_lossy().to_string())
}

// ────────────────────────────────────────────────────────────
// Backup operations
// ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn backup_prospect_cmd(
    prospect_id: String,
    backup_dir: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let state = state.lock().unwrap();
    let prospect = state
        .open_prospects
        .get(&prospect_id)
        .ok_or_else(|| format!("Prospect '{}' not loaded", prospect_id))?;

    let backup_path =
        backup::backup_prospect(&prospect.file_path, &PathBuf::from(&backup_dir))
            .map_err(|e| e.to_string())?;

    Ok(backup_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn restore_prospect_cmd(
    backup_path: String,
    dest_dir: String,
) -> Result<String, String> {
    let dest = backup::restore_prospect(&PathBuf::from(&backup_path), &PathBuf::from(&dest_dir))
        .map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub fn list_backups_cmd(backup_dir: String) -> Result<Vec<backup::BackupEntry>, String> {
    backup::list_backups(&PathBuf::from(&backup_dir)).map_err(|e| e.to_string())
}

// ────────────────────────────────────────────────────────────
// Config
// ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_config(state: State<'_, Mutex<AppState>>) -> AppConfig {
    let state = state.lock().unwrap();
    state.config.clone()
}

#[tauri::command]
pub fn set_config(
    config: AppConfig,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    state.config = config;
    Ok(())
}
