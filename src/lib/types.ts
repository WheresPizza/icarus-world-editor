export interface AssociatedMember {
  AccountName: string;
  CharacterName: string;
  UserID: string;
  ChrSlot: number;
  Experience: number;
  Status: string;
  Settled: boolean;
  IsCurrentlyPlaying: boolean;
}

export interface CustomSetting {
  SettingRowName: string;
  SettingValue: unknown;
}

export interface ProspectInfo {
  ProspectID: string;
  ClaimedAccountID: string;
  ClaimedAccountCharacter: number;
  ProspectDTKey: string;
  FactionMissionDTKey: string;
  LobbyName: string;
  ExpireTime: number;
  ProspectState: string;
  AssociatedMembers: AssociatedMember[];
  Cost: number;
  Reward: number;
  Difficulty: string;
  Insurance: boolean;
  NoRespawns: boolean;
  ElapsedTime: number;
  SelectedDropPoint: number;
  CustomSettings: CustomSetting[];
}

export interface ProspectSummary {
  file_name: string;
  file_path: string;
  file_size: number;
  prospect_info: ProspectInfo;
}

export interface ComponentSummary {
  index: number;
  class_name: string;
  data_size: number;
}

export interface ProspectOverview {
  prospect_info: ProspectInfo;
  blob_version: number | null;
  lobby_privacy: string | null;
  prospect_map_name: string | null;
  components: ComponentSummary[];
  total_components: number;
}

export interface Property {
  name: string;
  type: string;
  value: unknown;
}

export interface BackupEntry {
  file_name: string;
  file_path: string;
  file_size: number;
  created_at: string;
  original_name: string;
}

export interface AppConfig {
  prospects_dir: string | null;
  backup_dir: string | null;
  auto_backup_on_save: boolean;
}

export interface SearchHit {
  component_idx: number;
  component_name: string;
  component_class: string;
  property_path: string;
  value_preview: string;
}

export interface FieldDiff {
  field: string;
  old_value: string;
  new_value: string;
}

export interface PropertyDiff {
  path: string;
  old_value: string;
  new_value: string;
}

export interface ComponentDiff {
  component_name: string;
  component_class: string;
  property_changes: PropertyDiff[];
}

export interface ProspectDiff {
  metadata_changes: FieldDiff[];
  added_components: string[];
  removed_components: string[];
  modified_components: ComponentDiff[];
}

// View state
export type ViewMode = "library" | "detail" | "component";

export interface AppViewState {
  mode: ViewMode;
  selectedProspectId: string | null;
  selectedProspectPath: string | null;
  selectedComponentIndex: number | null;
}
