import { invoke } from "@tauri-apps/api/core";
import type {
  ProspectSummary,
  ProspectOverview,
  Property,
  BackupEntry,
  AppConfig,
  ProspectInfo,
  SearchHit,
  ProspectDiff,
  InventoryView,
  ServerConfig,
  ServerStatusResponse,
} from "./types";

export async function listProspects(dir: string): Promise<ProspectSummary[]> {
  return invoke("list_prospects", { dir });
}

export async function autoDetectProspectsDir(): Promise<string | null> {
  return invoke("auto_detect_prospects_dir");
}

export async function getProspectOverview(
  path: string
): Promise<ProspectOverview> {
  return invoke("get_prospect_overview", { path });
}

export async function getComponentDetails(
  prospectId: string,
  index: number
): Promise<Property[]> {
  return invoke("get_component_details", { prospectId, index });
}

export async function getDomainView(
  prospectId: string,
  index: number
): Promise<unknown> {
  return invoke("get_domain_view", { prospectId, index });
}

export async function updateMetadata(
  prospectId: string,
  info: ProspectInfo
): Promise<void> {
  return invoke("update_metadata", { prospectId, info });
}

export async function updateComponentProperty(
  prospectId: string,
  componentIndex: number,
  propertyPath: string,
  value: unknown
): Promise<void> {
  return invoke("update_component_property", {
    prospectId,
    componentIndex,
    propertyPath,
    value,
  });
}

export async function saveProspect(prospectId: string): Promise<string> {
  return invoke("save_prospect", { prospectId });
}

export async function backupProspect(
  prospectId: string,
  backupDir: string
): Promise<string> {
  return invoke("backup_prospect_cmd", { prospectId, backupDir });
}

export async function restoreProspect(
  backupPath: string,
  destDir: string
): Promise<string> {
  return invoke("restore_prospect_cmd", { backupPath, destDir });
}

export async function listBackups(backupDir: string): Promise<BackupEntry[]> {
  return invoke("list_backups_cmd", { backupDir });
}

export async function getConfig(): Promise<AppConfig> {
  return invoke("get_config");
}

export async function setConfig(config: AppConfig): Promise<void> {
  return invoke("set_config", { config });
}

export async function searchComponents(
  prospectId: string,
  query: string
): Promise<SearchHit[]> {
  return invoke("search_components", { prospectId, query });
}

export async function diffProspects(
  idA: string,
  idB: string
): Promise<ProspectDiff> {
  return invoke("diff_prospects", { idA, idB });
}

export async function getInventoryView(prospectId: string): Promise<InventoryView> {
  return invoke("get_inventory_view", { prospectId });
}

export async function updateInventorySlot(
  prospectId: string,
  componentIdx: number,
  slotIndex: number,
  itemKey: string,
  quantity: number
): Promise<void> {
  return invoke("update_inventory_slot", {
    prospectId,
    componentIdx,
    slotIndex,
    itemKey,
    quantity,
  });
}

export async function deleteInventorySlot(
  prospectId: string,
  componentIdx: number,
  slotIndex: number
): Promise<void> {
  return invoke("delete_inventory_slot", { prospectId, componentIdx, slotIndex });
}

export async function addInventoryItem(
  prospectId: string,
  componentIdx: number,
  itemKey: string,
  quantity: number
): Promise<void> {
  return invoke("add_inventory_item", { prospectId, componentIdx, itemKey, quantity });
}

export async function detectServer(): Promise<string | null> {
  return invoke("detect_server");
}

export async function getServerConfig(): Promise<ServerConfig> {
  return invoke("get_server_config");
}

export async function setServerConfig(config: ServerConfig): Promise<void> {
  return invoke("set_server_config", { config });
}

export async function startServer(prospectId: string): Promise<void> {
  return invoke("start_server", { prospectId });
}

export async function stopServer(): Promise<void> {
  return invoke("stop_server");
}

export async function getServerStatus(): Promise<ServerStatusResponse> {
  return invoke("get_server_status");
}
