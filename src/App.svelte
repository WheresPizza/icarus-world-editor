<script lang="ts">
  import "./app.css";
  import { onMount } from "svelte";
  import type {
    ProspectSummary,
    ProspectOverview,
    ProspectInfo,
    AppViewState,
  } from "./lib/types";
  import * as api from "./lib/api";
  import { formatElapsedTime, shortClassName } from "./lib/helpers";
  import ProspectCard from "./lib/components/ProspectCard.svelte";
  import DiffView from "./lib/components/DiffView.svelte";
  import MetadataEditor from "./lib/components/MetadataEditor.svelte";
  import ComponentList from "./lib/components/ComponentList.svelte";
  import PropertyTree from "./lib/components/PropertyTree.svelte";
  import type { ProspectDiff } from "./lib/types";

  // State
  let prospectsDir = $state<string | null>(null);
  let prospects = $state<ProspectSummary[]>([]);
  let overview = $state<ProspectOverview | null>(null);
  let componentProps = $state<any[] | null>(null);
  let loading = $state(false);
  let errorMsg = $state<string | null>(null);
  let searchQuery = $state("");
  let navTab = $state<"prospects" | "backups" | "settings">("prospects");

  // Compare / diff state
  let compareMode = $state(false);
  let compareIdA = $state<string | null>(null);
  let compareIdB = $state<string | null>(null);
  let diffResult = $state<ProspectDiff | null>(null);

  let view = $state<AppViewState>({
    mode: "library",
    selectedProspectId: null,
    selectedProspectPath: null,
    selectedComponentIndex: null,
  });

  // Filtered prospects
  let filteredProspects = $derived.by(() => {
    if (!searchQuery) return prospects;
    const q = searchQuery.toLowerCase();
    return prospects.filter(
      (p) =>
        p.prospect_info.ProspectID.toLowerCase().includes(q) ||
        p.prospect_info.ProspectDTKey.toLowerCase().includes(q) ||
        p.prospect_info.Difficulty.toLowerCase().includes(q) ||
        p.prospect_info.AssociatedMembers.some((m) =>
          m.CharacterName.toLowerCase().includes(q)
        )
    );
  });

  onMount(async () => {
    try {
      const detected = await api.autoDetectProspectsDir();
      if (detected) {
        prospectsDir = detected;
        await loadProspects();
      }
    } catch (e) {
      console.warn("Auto-detect failed:", e);
    }
  });

  async function loadProspects() {
    if (!prospectsDir) return;
    loading = true;
    errorMsg = null;
    try {
      prospects = await api.listProspects(prospectsDir);
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      loading = false;
    }
  }

  async function openProspect(p: ProspectSummary) {
    loading = true;
    errorMsg = null;
    try {
      overview = await api.getProspectOverview(p.file_path);
      view = {
        mode: "detail",
        selectedProspectId: p.prospect_info.ProspectID,
        selectedProspectPath: p.file_path,
        selectedComponentIndex: null,
      };
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      loading = false;
    }
  }

  async function openComponent(index: number) {
    if (!view.selectedProspectId) return;
    loading = true;
    errorMsg = null;
    try {
      componentProps = await api.getComponentDetails(
        view.selectedProspectId,
        index
      );
      view = { ...view, mode: "component", selectedComponentIndex: index };
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      loading = false;
    }
  }

  function goBack() {
    if (view.mode === "component") {
      view = { ...view, mode: "detail", selectedComponentIndex: null };
      componentProps = null;
    } else if (view.mode === "detail") {
      view = {
        mode: "library",
        selectedProspectId: null,
        selectedProspectPath: null,
        selectedComponentIndex: null,
      };
      overview = null;
    }
  }

  async function handleMetadataSave(info: ProspectInfo) {
    if (!view.selectedProspectId) return;
    try {
      await api.updateMetadata(view.selectedProspectId, info);
      // Save to file
      await api.saveProspect(view.selectedProspectId);
      // Refresh
      if (overview) {
        overview = { ...overview, prospect_info: info };
      }
      errorMsg = null;
    } catch (e: any) {
      errorMsg = "Save failed: " + e.toString();
    }
  }

  async function handlePropertyChange(path: string, value: unknown) {
    if (!view.selectedProspectId || view.selectedComponentIndex === null) return;
    try {
      await api.updateComponentProperty(
        view.selectedProspectId,
        view.selectedComponentIndex,
        path,
        value
      );
    } catch (e: any) {
      errorMsg = "Update failed: " + e.toString();
    }
  }

  async function handleSave() {
    if (!view.selectedProspectId) return;
    loading = true;
    try {
      await api.saveProspect(view.selectedProspectId);
      errorMsg = null;
    } catch (e: any) {
      errorMsg = "Save failed: " + e.toString();
    } finally {
      loading = false;
    }
  }

  async function handleSetDir() {
    const dir = prompt("Enter prospects directory path:");
    if (dir) {
      prospectsDir = dir;
      await loadProspects();
    }
  }

  function selectForCompare(prospectId: string) {
    if (!compareIdA) {
      compareIdA = prospectId;
    } else if (!compareIdB && prospectId !== compareIdA) {
      compareIdB = prospectId;
    }
  }

  async function runDiff() {
    if (!compareIdA || !compareIdB) return;
    loading = true;
    errorMsg = null;
    try {
      diffResult = await api.diffProspects(compareIdA, compareIdB);
      view = { ...view, mode: "diff" as any };
    } catch (e: any) {
      errorMsg = "Diff failed: " + e.toString();
    } finally {
      loading = false;
    }
  }

  function cancelCompare() {
    compareMode = false;
    compareIdA = null;
    compareIdB = null;
    diffResult = null;
    if ((view.mode as any) === "diff") {
      view = {
        mode: "library",
        selectedProspectId: null,
        selectedProspectPath: null,
        selectedComponentIndex: null,
      };
    }
  }
</script>

<main class="app">
  <!-- Sidebar -->
  <div class="sidebar">
    <div class="logo">
      <h1>ICARUS</h1>
      <span class="subtitle">Prospect Editor</span>
    </div>
    <nav>
      <button
        class="nav-item"
        class:active={navTab === "prospects"}
        onclick={() => { navTab = "prospects"; if (view.mode !== "library") goBack(); }}
      >
        <span class="nav-icon">&#128203;</span>
        Prospects
      </button>
      <button
        class="nav-item"
        class:active={navTab === "settings"}
        onclick={() => navTab = "settings"}
      >
        <span class="nav-icon">&#9881;</span>
        Settings
      </button>
    </nav>

    {#if prospectsDir}
      <div class="sidebar-info">
        <span class="dir-label">Directory</span>
        <span class="dir-path" title={prospectsDir}>
          ...{prospectsDir.slice(-30)}
        </span>
        <span class="prospect-count">{prospects.length} prospect(s)</span>
      </div>
    {/if}
  </div>

  <!-- Main Content -->
  <div class="content">
    <!-- Error bar -->
    {#if errorMsg}
      <div class="error-bar">
        <span>{errorMsg}</span>
        <button onclick={() => errorMsg = null}>x</button>
      </div>
    {/if}

    <!-- Loading overlay -->
    {#if loading}
      <div class="loading-overlay">
        <div class="spinner"></div>
      </div>
    {/if}

    {#if navTab === "settings"}
      <!-- Settings View -->
      <div class="settings-view">
        <h2>Settings</h2>
        <div class="setting-item">
          <label>Prospects Directory</label>
          <div class="setting-row">
            <input type="text" value={prospectsDir || ""} onchange={(e) => {
              prospectsDir = (e.target as HTMLInputElement).value;
            }} placeholder="Path to prospects folder..." />
            <button class="btn btn-primary" onclick={loadProspects}>Load</button>
          </div>
        </div>
      </div>

    {:else if !prospectsDir}
      <!-- First Run -->
      <div class="welcome">
        <h2>Welcome to ICARUS Prospect Editor</h2>
        <p>Set your prospects directory to get started.</p>
        <button class="btn btn-primary" onclick={handleSetDir}>
          Set Prospects Directory
        </button>
      </div>

    {:else if view.mode === "library"}
      <!-- Library View -->
      <div class="library-view">
        <div class="library-header">
          <h2>Prospects</h2>
          <input
            type="text"
            class="search-bar"
            placeholder="Search by name, map, difficulty, player..."
            bind:value={searchQuery}
          />
          <div class="compare-controls">
            <button
              class="btn"
              class:active={compareMode}
              onclick={() => { compareMode = !compareMode; if (!compareMode) cancelCompare(); }}
            >
              {compareMode ? "Cancel Compare" : "Compare"}
            </button>
            {#if compareMode}
              <span class="compare-status">
                {#if !compareIdA}Select prospect A{:else if !compareIdB}Select prospect B{:else}Ready to compare{/if}
              </span>
              {#if compareIdA && compareIdB}
                <button class="btn btn-primary" onclick={runDiff}>Run Diff</button>
              {/if}
            {/if}
          </div>
        </div>

        {#if filteredProspects.length === 0}
          <div class="empty-state">
            <p>No prospects found.</p>
          </div>
        {:else}
          <div class="prospect-grid">
            {#each filteredProspects as prospect}
              <div class="card-wrapper">
                <ProspectCard {prospect} onOpen={compareMode ? () => {} : openProspect} />
                {#if compareMode}
                  {@const pid = prospect.prospect_info.ProspectID}
                  {@const isA = compareIdA === pid}
                  {@const isB = compareIdB === pid}
                  <button
                    class="compare-overlay"
                    class:selected-a={isA}
                    class:selected-b={isB}
                    onclick={() => selectForCompare(pid)}
                  >
                    {#if isA}A{:else if isB}B{:else}Select{/if}
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>

    {:else if (view.mode as any) === "diff" && diffResult}
      <!-- Diff View -->
      <DiffView
        {diffResult}
        nameA={compareIdA || "A"}
        nameB={compareIdB || "B"}
        onBack={cancelCompare}
      />

    {:else if view.mode === "detail" && overview}
      <!-- Detail View -->
      <div class="detail-view">
        <div class="detail-header">
          <button class="back-btn" onclick={goBack}>&lt; Back</button>
          <div class="detail-title">
            <h2>{overview.prospect_info.ProspectID}</h2>
            <div class="detail-badges">
              <span class="detail-badge map">
                {overview.prospect_map_name || overview.prospect_info.ProspectDTKey}
              </span>
              <span class="detail-badge difficulty">
                {overview.prospect_info.Difficulty}
              </span>
              <span class="detail-badge state">
                {overview.prospect_info.ProspectState}
              </span>
              {#if overview.blob_version}
                <span class="detail-badge version">v{overview.blob_version}</span>
              {/if}
            </div>
          </div>
          <button class="btn btn-primary" onclick={handleSave}>Save All</button>
        </div>

        <div class="detail-panels">
          <div class="detail-left">
            <MetadataEditor info={overview.prospect_info} onSave={handleMetadataSave} />
          </div>
          <div class="detail-right">
            <ComponentList
              components={overview.components}
              onSelect={openComponent}
              prospectId={view.selectedProspectId || ""}
              onSearchHitSelect={openComponent}
            />
          </div>
        </div>
      </div>

    {:else if view.mode === "component" && componentProps !== null}
      <!-- Component Inspector -->
      <div class="component-view">
        <div class="component-header">
          <button class="back-btn" onclick={goBack}>&lt; Back to Prospect</button>
          {#if overview && view.selectedComponentIndex !== null}
            {@const comp = overview.components[view.selectedComponentIndex]}
            {#if comp}
              <h2>
                {shortClassName(comp.class_name)}
                <span class="component-index">#{view.selectedComponentIndex}</span>
              </h2>
              <span class="component-class">{comp.class_name}</span>
            {/if}
          {/if}
          <button class="btn btn-primary" onclick={handleSave}>Save</button>
        </div>

        <PropertyTree
          properties={componentProps}
          prospectId={view.selectedProspectId || ""}
          componentIndex={view.selectedComponentIndex || 0}
          onPropertyChange={handlePropertyChange}
        />
      </div>
    {/if}
  </div>
</main>

<style>
  .app {
    display: flex;
    height: 100vh;
    width: 100vw;
  }

  /* Sidebar */
  .sidebar {
    width: 220px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .logo {
    padding: 20px 16px;
    border-bottom: 1px solid var(--border-color);
  }

  .logo h1 {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: 2px;
    color: var(--accent-blue);
  }

  .logo .subtitle {
    font-size: 11px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  nav {
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    transition: all 0.15s ease;
    text-align: left;
  }

  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--bg-active);
    color: var(--accent-blue);
  }

  .nav-icon {
    font-size: 16px;
    width: 20px;
    text-align: center;
  }

  .sidebar-info {
    margin-top: auto;
    padding: 12px 16px;
    border-top: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .dir-label {
    font-size: 10px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .dir-path {
    font-size: 11px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .prospect-count {
    font-size: 11px;
    color: var(--text-muted);
  }

  /* Content */
  .content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    position: relative;
  }

  /* Error bar */
  .error-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: rgba(231, 76, 60, 0.15);
    border-bottom: 1px solid var(--accent-red);
    color: var(--accent-red);
    font-size: 13px;
  }

  .error-bar button {
    color: var(--accent-red);
    font-weight: bold;
    padding: 2px 6px;
  }

  /* Loading */
  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(10, 14, 20, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border-light);
    border-top-color: var(--accent-blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Welcome */
  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
    color: var(--text-secondary);
  }

  .welcome h2 {
    color: var(--text-primary);
    font-size: 20px;
  }

  .btn {
    padding: 8px 20px;
    border-radius: var(--radius-md);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn-primary {
    background: var(--accent-blue);
    color: white;
  }

  .btn-primary:hover {
    background: var(--accent-blue-hover);
  }

  /* Library View */
  .library-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .library-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px 24px;
    border-bottom: 1px solid var(--border-color);
  }

  .library-header h2 {
    font-size: 18px;
    color: var(--text-primary);
    white-space: nowrap;
  }

  .search-bar {
    flex: 1;
    padding: 8px 12px;
    max-width: 400px;
  }

  .prospect-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 12px;
    padding: 20px 24px;
    overflow-y: auto;
    flex: 1;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
  }

  /* Detail View */
  .detail-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .detail-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 24px;
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .back-btn {
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 13px;
  }

  .back-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .detail-title {
    flex: 1;
  }

  .detail-title h2 {
    font-size: 18px;
    color: var(--text-primary);
  }

  .detail-badges {
    display: flex;
    gap: 6px;
    margin-top: 4px;
  }

  .detail-badge {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 4px;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .detail-badge.map {
    color: var(--accent-blue);
    background: rgba(61, 142, 240, 0.1);
  }

  .detail-badge.difficulty {
    color: var(--accent-orange);
    background: rgba(230, 126, 34, 0.1);
  }

  .detail-badge.state {
    color: var(--accent-green);
    background: rgba(46, 204, 113, 0.1);
  }

  .detail-badge.version {
    color: var(--text-muted);
  }

  .detail-panels {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .detail-left {
    flex: 1;
    overflow-y: auto;
    border-right: 1px solid var(--border-color);
  }

  .detail-right {
    width: 350px;
    flex-shrink: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* Component View */
  .component-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .component-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 24px;
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .component-header h2 {
    font-size: 16px;
    color: var(--text-primary);
  }

  .component-index {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text-muted);
  }

  .component-class {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
    flex: 1;
  }

  /* Settings View */
  .settings-view {
    padding: 24px;
    max-width: 600px;
  }

  .settings-view h2 {
    font-size: 18px;
    color: var(--text-primary);
    margin-bottom: 20px;
  }

  .setting-item {
    margin-bottom: 16px;
  }

  .setting-item label {
    display: block;
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 6px;
  }

  .setting-row {
    display: flex;
    gap: 8px;
  }

  .setting-row input {
    flex: 1;
  }

  /* Compare controls */
  .compare-controls {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  .compare-status {
    font-size: 12px;
    color: var(--text-muted);
  }

  .btn.active {
    background: var(--bg-active);
    color: var(--accent-blue);
  }

  /* Card wrapper for compare overlay */
  .card-wrapper {
    position: relative;
  }

  .compare-overlay {
    position: absolute;
    top: 8px;
    right: 8px;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 600;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
    cursor: pointer;
    z-index: 10;
  }

  .compare-overlay:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .compare-overlay.selected-a {
    background: rgba(231, 76, 60, 0.2);
    color: var(--accent-red);
    border-color: var(--accent-red);
  }

  .compare-overlay.selected-b {
    background: rgba(46, 204, 113, 0.2);
    color: var(--accent-green);
    border-color: var(--accent-green);
  }
</style>
