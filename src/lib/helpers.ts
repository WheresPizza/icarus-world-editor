/** Format bytes into human-readable size */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

/** Format elapsed time (seconds) into human-readable duration */
export function formatElapsedTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

/** Extract a short class name from a full UE4 path */
export function shortClassName(fullPath: string): string {
  // "/Script/Icarus.DeployableRecorderComponent" -> "Deployable"
  const parts = fullPath.split(".");
  const last = parts[parts.length - 1] || fullPath;
  return last
    .replace("RecorderComponent", "")
    .replace("Recorder", "")
    .replace("Component", "");
}

/** Group components by class name */
export function groupByClass<T extends { class_name: string }>(
  items: T[]
): Map<string, T[]> {
  const groups = new Map<string, T[]>();
  for (const item of items) {
    const existing = groups.get(item.class_name);
    if (existing) {
      existing.push(item);
    } else {
      groups.set(item.class_name, [item]);
    }
  }
  return groups;
}
