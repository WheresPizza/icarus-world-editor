<script lang="ts">
  import type { ServerConfig, ServerStatusResponse } from "../types";
  import * as api from "../api";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  let { prospectId }: {
    prospectId: string;
  } = $props();

  let expanded = $state(false);
  let status = $state<ServerStatusResponse>({ status: "Stopped", pid: null, uptime_secs: null, log_lines: [] });
  let config = $state<ServerConfig>({
    executable_path: null,
    server_name: "ICARUS Dedicated Server",
    port: 17777,
    max_players: 8,
    password: null,
    admin_password: null,
  });
  let detectedExe = $state<string | null>(null);
  let logLines = $state<Array<{ line: string; level: string }>>([]);
  let errorsOnly = $state(false);
  let loading = $state(false);
  let errorMsg = $state<string | null>(null);
  let pollInterval: ReturnType<typeof setInterval> | null = null;
  let unlistenLog: (() => void) | null = null;

  const displayedLines = $derived(
    errorsOnly ? logLines.filter(l => l.level === "error") : logLines
  );

  async function init() {
    try {
      [detectedExe, config, status] = await Promise.all([
        api.detectServer(),
        api.getServerConfig(),
        api.getServerStatus(),
      ]);
    } catch (e) {
      // ignore
    }
  }

  function startPolling() {
    if (pollInterval) return;
    pollInterval = setInterval(async () => {
      try {
        status = await api.getServerStatus();
      } catch { /* ignore */ }
    }, 2000);
  }

  function stopPolling() {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  }

  async function handleStart() {
    loading = true;
    errorMsg = null;
    try {
      await api.startServer(prospectId);
      status = await api.getServerStatus();
      startPolling();
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      loading = false;
    }
  }

  async function handleStop() {
    loading = true;
    errorMsg = null;
    try {
      await api.stopServer();
      status = await api.getServerStatus();
      stopPolling();
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      loading = false;
    }
  }

  async function handleSaveConfig() {
    loading = true;
    try {
      await api.setServerConfig(config);
      errorMsg = null;
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await init();
    // Listen for log events
    unlistenLog = await listen<{ line: string; level: string }>("server://log", (event) => {
      logLines = [...logLines.slice(-999), event.payload];
    });
    if (status.status === "Running") startPolling();
  });

  onDestroy(() => {
    stopPolling();
    unlistenLog?.();
  });

  function formatUptime(secs: number | null): string {
    if (secs === null) return "";
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = secs % 60;
    if (h > 0) return `${h}h ${m}m`;
    if (m > 0) return `${m}m ${s}s`;
    return `${s}s`;
  }
</script>

<div class="server-panel">
  <button class="panel-toggle" onclick={() => expanded = !expanded}>
    <span class="toggle-label">Server</span>
    <span class="status-dot" class:running={status.status === "Running"} class:starting={status.status === "Starting"}></span>
    <span class="status-text">{status.status}</span>
    {#if status.uptime_secs !== null}
      <span class="uptime">{formatUptime(status.uptime_secs)}</span>
    {/if}
    <span class="toggle-arrow">{expanded ? "v" : "^"}</span>
  </button>

  {#if expanded}
    <div class="panel-content">
      {#if errorMsg}
        <div class="panel-error">{errorMsg} <button onclick={() => errorMsg = null}>x</button></div>
      {/if}

      {#if !detectedExe && !config.executable_path}
        <!-- Not installed -->
        <div class="not-installed">
          <p>ICARUS Dedicated Server not found.</p>
          <a href="steam://install/2089135" class="install-link">Install via Steam</a>
        </div>
      {:else}
        <!-- Config form -->
        <div class="config-form">
          <div class="form-row">
            <label>Server Name</label>
            <input type="text" bind:value={config.server_name} />
          </div>
          <div class="form-row">
            <label>Port</label>
            <input type="number" bind:value={config.port} min="1024" max="65535" style="width:80px" />
          </div>
          <div class="form-row">
            <label>Max Players</label>
            <input type="number" bind:value={config.max_players} min="1" max="8" style="width:60px" />
          </div>
          <div class="form-row">
            <label>Password</label>
            <input type="password" bind:value={config.password} placeholder="(none)" />
          </div>
          <button class="btn-save-config" onclick={handleSaveConfig} disabled={loading}>
            Save Config
          </button>
        </div>

        <!-- Controls -->
        <div class="controls">
          {#if status.status === "Stopped"}
            <button class="btn-start" onclick={handleStart} disabled={loading}>
              Start Server
            </button>
          {:else}
            <button class="btn-stop" onclick={handleStop} disabled={loading}>
              Stop Server
            </button>
            {#if status.pid}
              <span class="pid-info">PID: {status.pid}</span>
            {/if}
          {/if}
        </div>

        <!-- Log panel -->
        <div class="log-panel">
          <div class="log-toolbar">
            <span class="log-label">Log</span>
            <label class="errors-toggle">
              <input type="checkbox" bind:checked={errorsOnly} />
              Errors only
            </label>
            <button class="btn-clear" onclick={() => logLines = []}>Clear</button>
          </div>
          <div class="log-output">
            {#each displayedLines as entry}
              <div class="log-line" class:error={entry.level === "error"}>{entry.line}</div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .server-panel {
    border-top: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .panel-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 16px;
    text-align: left;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .panel-toggle:hover { background: var(--bg-hover); color: var(--text-primary); }

  .toggle-label {
    font-weight: 600;
    font-size: 12px;
    color: var(--text-primary);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-muted);
    flex-shrink: 0;
  }

  .status-dot.running { background: var(--accent-green); }
  .status-dot.starting { background: var(--accent-orange); }

  .status-text {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
  }

  .uptime {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--accent-green);
  }

  .toggle-arrow {
    margin-left: auto;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
  }

  .panel-content {
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-height: 400px;
    overflow-y: auto;
  }

  .panel-error {
    padding: 6px 10px;
    background: rgba(231, 76, 60, 0.15);
    color: var(--accent-red);
    border-radius: 3px;
    font-size: 12px;
    display: flex;
    justify-content: space-between;
  }

  .not-installed {
    text-align: center;
    padding: 12px;
    color: var(--text-muted);
    font-size: 13px;
  }

  .install-link {
    display: inline-block;
    margin-top: 8px;
    padding: 6px 16px;
    background: var(--accent-blue);
    color: white;
    border-radius: 3px;
    font-size: 12px;
    text-decoration: none;
  }

  .install-link:hover { background: var(--accent-blue-hover); }

  .config-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .form-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .form-row label {
    font-size: 11px;
    color: var(--text-muted);
    width: 90px;
    flex-shrink: 0;
  }

  .form-row input {
    flex: 1;
    padding: 4px 8px;
    font-size: 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
  }

  .btn-save-config {
    align-self: flex-end;
    padding: 4px 12px;
    font-size: 11px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-secondary);
  }

  .btn-save-config:hover { color: var(--text-primary); background: var(--bg-hover); }
  .btn-save-config:disabled { opacity: 0.4; }

  .controls {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .btn-start {
    padding: 6px 16px;
    background: var(--accent-green);
    color: white;
    border-radius: 3px;
    font-size: 12px;
  }

  .btn-start:disabled { opacity: 0.4; }
  .btn-start:not(:disabled):hover { filter: brightness(1.1); }

  .btn-stop {
    padding: 6px 16px;
    background: var(--accent-red);
    color: white;
    border-radius: 3px;
    font-size: 12px;
  }

  .btn-stop:disabled { opacity: 0.4; }
  .btn-stop:not(:disabled):hover { filter: brightness(1.1); }

  .pid-info {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
  }

  .log-panel {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .log-toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .log-label {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 600;
  }

  .errors-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-muted);
    cursor: pointer;
  }

  .btn-clear {
    margin-left: auto;
    padding: 2px 8px;
    font-size: 11px;
    color: var(--text-muted);
  }

  .btn-clear:hover { color: var(--text-primary); }

  .log-output {
    max-height: 150px;
    overflow-y: auto;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    padding: 4px 8px;
    font-family: var(--font-mono);
    font-size: 10px;
  }

  .log-line {
    color: var(--text-secondary);
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .log-line.error { color: var(--accent-red); }
</style>
