use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::error::ProspectError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub created_at: String,
    pub original_name: String,
}

/// Create a backup of a prospect file with a timestamp suffix
pub fn backup_prospect(src: &Path, backup_dir: &Path) -> Result<PathBuf, ProspectError> {
    if !backup_dir.exists() {
        fs::create_dir_all(backup_dir)?;
    }

    let stem = src.file_stem().unwrap_or_default().to_string_lossy();
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}.json", stem, timestamp);
    let backup_path = backup_dir.join(&backup_name);

    fs::copy(src, &backup_path)?;

    Ok(backup_path)
}

/// Restore a prospect from a backup file
pub fn restore_prospect(backup_path: &Path, dest_dir: &Path) -> Result<PathBuf, ProspectError> {
    let file_name = backup_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    // Strip the timestamp suffix to get the original name
    let original_name = extract_original_name(&file_name);
    let dest_path = dest_dir.join(&original_name);

    fs::copy(backup_path, &dest_path)?;

    Ok(dest_path)
}

/// List all backup files in a directory
pub fn list_backups(backup_dir: &Path) -> Result<Vec<BackupEntry>, ProspectError> {
    let mut backups = Vec::new();

    if !backup_dir.exists() {
        return Ok(backups);
    }

    for entry in fs::read_dir(backup_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let metadata = fs::metadata(&path)?;
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let created_at = metadata
            .created()
            .or_else(|_| metadata.modified())
            .map(|t| {
                let dt: chrono::DateTime<Local> = t.into();
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_default();

        backups.push(BackupEntry {
            original_name: extract_original_name(&file_name),
            file_name,
            file_path: path.to_string_lossy().to_string(),
            file_size: metadata.len(),
            created_at,
        });
    }

    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(backups)
}

fn extract_original_name(backup_filename: &str) -> String {
    // Pattern: "Name_20260415_221530.json" -> "Name.json"
    let without_ext = backup_filename.trim_end_matches(".json");

    // Try to find timestamp pattern at the end: _YYYYMMDD_HHMMSS
    if without_ext.len() > 16 {
        let potential_ts = &without_ext[without_ext.len() - 16..];
        if potential_ts.starts_with('_')
            && potential_ts[1..9].chars().all(|c| c.is_ascii_digit())
            && potential_ts[9..10] == *"_"
            && potential_ts[10..].chars().all(|c| c.is_ascii_digit())
        {
            return format!("{}.json", &without_ext[..without_ext.len() - 16]);
        }
    }

    // No timestamp found, return as-is
    backup_filename.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_original_name() {
        assert_eq!(
            extract_original_name("Pomoyka_20260415_221530.json"),
            "Pomoyka.json"
        );
        assert_eq!(
            extract_original_name("MyProspect_20260101_120000.json"),
            "MyProspect.json"
        );
        assert_eq!(
            extract_original_name("NoTimestamp.json"),
            "NoTimestamp.json"
        );
    }
}
