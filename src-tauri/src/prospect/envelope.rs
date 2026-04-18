use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use super::error::ProspectError;
use super::types::{ProspectBlobEnvelope, ProspectFile, ProspectInfo, ProspectSummary};

/// Read only the ProspectInfo from a file (fast, no blob parsing)
pub fn read_prospect_info(path: &Path) -> Result<ProspectInfo, ProspectError> {
    let contents = fs::read_to_string(path)?;
    let file: ProspectFile = serde_json::from_str(&contents)?;
    Ok(file.prospect_info)
}

/// Read the full prospect file and decompress the blob
pub fn read_prospect_blob(path: &Path) -> Result<(ProspectFile, Vec<u8>), ProspectError> {
    let contents = fs::read_to_string(path)?;
    let file: ProspectFile = serde_json::from_str(&contents)?;

    let compressed = BASE64
        .decode(&file.prospect_blob.binary_blob)
        .map_err(ProspectError::Base64)?;

    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    if decompressed.len() as u64 != file.prospect_blob.uncompressed_length {
        return Err(ProspectError::InvalidFile(format!(
            "Decompressed length mismatch: expected {}, got {}",
            file.prospect_blob.uncompressed_length,
            decompressed.len()
        )));
    }

    Ok((file, decompressed))
}

/// Write a prospect file with new blob data
pub fn write_prospect(
    path: &Path,
    info: &ProspectInfo,
    blob_data: &[u8],
    original_key: &str,
) -> Result<(), ProspectError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(blob_data)?;
    let compressed = encoder.finish()?;

    let hash = {
        let mut hasher = Sha1::new();
        hasher.update(&compressed);
        format!("{:x}", hasher.finalize())
    };

    let blob_b64 = BASE64.encode(&compressed);

    let envelope = ProspectBlobEnvelope {
        key: original_key.to_string(),
        hash,
        total_length: compressed.len() as u64,
        data_length: compressed.len() as u64,
        uncompressed_length: blob_data.len() as u64,
        binary_blob: blob_b64,
    };

    let file = ProspectFile {
        prospect_info: info.clone(),
        prospect_blob: envelope,
    };

    let json = serde_json::to_string_pretty(&file)?;
    fs::write(path, json)?;

    Ok(())
}

/// Scan a directory for prospect JSON files and return summaries
pub fn list_prospect_files(dir: &Path) -> Result<Vec<ProspectSummary>, ProspectError> {
    let mut prospects = Vec::new();

    if !dir.exists() || !dir.is_dir() {
        return Ok(prospects);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        match read_prospect_info(&path) {
            Ok(info) => {
                let metadata = fs::metadata(&path)?;
                prospects.push(ProspectSummary {
                    file_name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    file_path: path.to_string_lossy().to_string(),
                    file_size: metadata.len(),
                    prospect_info: info,
                });
            }
            Err(e) => {
                log::warn!("Skipping invalid prospect file {:?}: {}", path, e);
            }
        }
    }

    Ok(prospects)
}

/// Try to auto-detect the ICARUS prospects directory
pub fn auto_detect_prospects_dir() -> Option<PathBuf> {
    // Windows: %LocalAppData%\Icarus\Saved\PlayerData\<SteamID>\Prospects
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let icarus_dir = PathBuf::from(&local_app_data)
            .join("Icarus")
            .join("Saved")
            .join("PlayerData");

        if icarus_dir.exists() {
            // Find the first SteamID subdirectory that has a Prospects folder
            if let Ok(entries) = fs::read_dir(&icarus_dir) {
                for entry in entries.flatten() {
                    let prospects_dir = entry.path().join("Prospects");
                    if prospects_dir.exists() && prospects_dir.is_dir() {
                        return Some(prospects_dir);
                    }
                }
            }
        }
    }

    // macOS/Linux fallback: check if the user has set up ICARUS via Proton/Wine
    // This is unlikely but handle gracefully
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_file_path() -> PathBuf {
        // Navigate up from src-tauri to the project root, then find Pomoyka.json
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("Pomoyka.json")
    }

    #[test]
    fn test_read_prospect_info() {
        let path = test_file_path();
        if !path.exists() {
            eprintln!("Test file not found at {:?}, skipping", path);
            return;
        }

        let info = read_prospect_info(&path).unwrap();
        assert_eq!(info.prospect_id, "Pomoyka");
        assert_eq!(info.prospect_dt_key, "Outpost006_Olympus");
        assert_eq!(info.difficulty, "Medium");
        assert_eq!(info.prospect_state, "Active");
        assert_eq!(info.associated_members.len(), 2);
    }

    #[test]
    fn test_read_and_decompress_blob() {
        let path = test_file_path();
        if !path.exists() {
            eprintln!("Test file not found at {:?}, skipping", path);
            return;
        }

        let (file, decompressed) = read_prospect_blob(&path).unwrap();
        assert_eq!(decompressed.len() as u64, file.prospect_blob.uncompressed_length);
        assert_eq!(decompressed.len(), 12780144);
    }

    #[test]
    fn test_round_trip_envelope() {
        let path = test_file_path();
        if !path.exists() {
            eprintln!("Test file not found at {:?}, skipping", path);
            return;
        }

        let (file, decompressed) = read_prospect_blob(&path).unwrap();

        // Write to a temp file
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_roundtrip_prospect.json");

        write_prospect(
            &temp_path,
            &file.prospect_info,
            &decompressed,
            &file.prospect_blob.key,
        )
        .unwrap();

        // Re-read and verify
        let (file2, decompressed2) = read_prospect_blob(&temp_path).unwrap();
        assert_eq!(decompressed, decompressed2);
        assert_eq!(
            file.prospect_info.prospect_id,
            file2.prospect_info.prospect_id
        );

        // Clean up
        let _ = fs::remove_file(&temp_path);
    }
}
