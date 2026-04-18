use serde::{Deserialize, Serialize};

/// Top-level prospect file structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ProspectFile {
    #[serde(rename = "ProspectInfo")]
    pub prospect_info: ProspectInfo,
    #[serde(rename = "ProspectBlob")]
    pub prospect_blob: ProspectBlobEnvelope,
}

/// Prospect metadata - the clean JSON header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProspectInfo {
    #[serde(rename = "ProspectID")]
    pub prospect_id: String,
    #[serde(rename = "ClaimedAccountID")]
    pub claimed_account_id: String,
    #[serde(rename = "ClaimedAccountCharacter")]
    pub claimed_account_character: i32,
    #[serde(rename = "ProspectDTKey")]
    pub prospect_dt_key: String,
    #[serde(rename = "FactionMissionDTKey")]
    pub faction_mission_dt_key: String,
    #[serde(rename = "LobbyName")]
    pub lobby_name: String,
    #[serde(rename = "ExpireTime")]
    pub expire_time: i64,
    #[serde(rename = "ProspectState")]
    pub prospect_state: String,
    #[serde(rename = "AssociatedMembers")]
    pub associated_members: Vec<AssociatedMember>,
    #[serde(rename = "Cost")]
    pub cost: i32,
    #[serde(rename = "Reward")]
    pub reward: i32,
    #[serde(rename = "Difficulty")]
    pub difficulty: String,
    #[serde(rename = "Insurance")]
    pub insurance: bool,
    #[serde(rename = "NoRespawns")]
    pub no_respawns: bool,
    #[serde(rename = "ElapsedTime")]
    pub elapsed_time: i64,
    #[serde(rename = "SelectedDropPoint")]
    pub selected_drop_point: i32,
    #[serde(rename = "CustomSettings")]
    pub custom_settings: Vec<CustomSetting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociatedMember {
    #[serde(rename = "AccountName")]
    pub account_name: String,
    #[serde(rename = "CharacterName")]
    pub character_name: String,
    #[serde(rename = "UserID")]
    pub user_id: String,
    #[serde(rename = "ChrSlot")]
    pub chr_slot: i32,
    #[serde(rename = "Experience")]
    pub experience: i64,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Settled")]
    pub settled: bool,
    #[serde(rename = "IsCurrentlyPlaying")]
    pub is_currently_playing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSetting {
    #[serde(rename = "SettingRowName")]
    pub setting_row_name: String,
    #[serde(rename = "SettingValue")]
    pub setting_value: serde_json::Value,
}

/// The blob envelope from the JSON file
#[derive(Debug, Serialize, Deserialize)]
pub struct ProspectBlobEnvelope {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Hash")]
    pub hash: String,
    #[serde(rename = "TotalLength")]
    pub total_length: u64,
    #[serde(rename = "DataLength")]
    pub data_length: u64,
    #[serde(rename = "UncompressedLength")]
    pub uncompressed_length: u64,
    #[serde(rename = "BinaryBlob")]
    pub binary_blob: String,
}

/// Summary of a prospect for the library view (no blob data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProspectSummary {
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub prospect_info: ProspectInfo,
}

/// Overview of a loaded prospect's blob structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ProspectOverview {
    pub prospect_info: ProspectInfo,
    pub blob_version: Option<i32>,
    pub lobby_privacy: Option<String>,
    pub prospect_map_name: Option<String>,
    pub components: Vec<ComponentSummary>,
    pub total_components: usize,
}

/// Summary of a single component in the blob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSummary {
    pub index: usize,
    pub class_name: String,
    pub data_size: usize,
}

/// Configuration for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub prospects_dir: Option<String>,
    pub backup_dir: Option<String>,
    pub auto_backup_on_save: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            prospects_dir: None,
            backup_dir: None,
            auto_backup_on_save: true,
        }
    }
}
