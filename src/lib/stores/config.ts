import { writable } from "svelte/store";
import type { AppConfig } from "../types";

export const config = writable<AppConfig>({
  prospects_dir: null,
  backup_dir: null,
  auto_backup_on_save: true,
});
