<script lang="ts">
  import type { ComponentSummary, SearchHit } from "../types";
  import * as api from "../api";
  import { shortClassName, formatBytes, groupByClass } from "../helpers";

  let { components, onSelect, prospectId = "", onSearchHitSelect }: {
    components: ComponentSummary[];
    onSelect: (index: number) => void;
    prospectId?: string;
    onSearchHitSelect?: (componentIdx: number) => void;
  } = $props();

  let searchQuery = $state("");
  let expandedGroups = $state<Set<string>>(new Set());
  let searchMode = $state<"components" | "properties">("components");
  let searchHits = $state<SearchHit[]>([]);
  let searching = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  let grouped = $derived.by(() => {
    if (searchMode !== "components") return [];
    const groups = groupByClass(components);
    const sorted = [...groups.entries()].sort((a, b) => b[1].length - a[1].length);
    if (!searchQuery) return sorted;
    const q = searchQuery.toLowerCase();
    return sorted.filter(([className]) =>
      shortClassName(className).toLowerCase().includes(q) ||
      className.toLowerCase().includes(q)
    );
  });

  function toggleGroup(className: string) {
    const next = new Set(expandedGroups);
    if (next.has(className)) {
      next.delete(className);
    } else {
      next.add(className);
    }
    expandedGroups = next;
  }

  function toggleSearchMode() {
    searchMode = searchMode === "components" ? "properties" : "components";
    searchHits = [];
    if (searchMode === "properties" && searchQuery.trim()) {
      doPropertySearch(searchQuery);
    }
  }

  function handleSearchInput(e: Event) {
    searchQuery = (e.target as HTMLInputElement).value;
    if (searchMode === "properties") {
      if (debounceTimer) clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => doPropertySearch(searchQuery), 300);
    }
  }

  async function doPropertySearch(q: string) {
    if (!q.trim() || !prospectId) {
      searchHits = [];
      return;
    }
    searching = true;
    try {
      searchHits = await api.searchComponents(prospectId, q);
    } catch (e) {
      searchHits = [];
    } finally {
      searching = false;
    }
  }
</script>

<div class="component-list">
  <div class="list-header">
    <h3>Components ({components.length})</h3>
    <div class="search-row">
      <input
        type="text"
        class="search"
        placeholder={searchMode === "components" ? "Filter components..." : "Search properties..."}
        value={searchQuery}
        oninput={handleSearchInput}
        onkeydown={(e) => e.key === "Enter" && searchMode === "properties" && doPropertySearch(searchQuery)}
      />
      <button
        class="mode-toggle"
        class:active={searchMode === "properties"}
        onclick={toggleSearchMode}
        title="Toggle property search"
      >
        Props
      </button>
    </div>
  </div>

  <div class="groups">
    {#if searchMode === "properties"}
      {#if searching}
        <div class="search-status">Searching...</div>
      {:else if searchQuery && searchHits.length === 0}
        <div class="search-status">No results</div>
      {:else}
        {#each searchHits as hit}
          <button class="hit-card" onclick={() => onSearchHitSelect?.(hit.component_idx)}>
            <div class="hit-header">
              <span class="hit-name">{hit.component_name}</span>
              <span class="hit-idx">#{hit.component_idx}</span>
            </div>
            <div class="hit-path">{hit.property_path}</div>
            <div class="hit-value">{hit.value_preview}</div>
          </button>
        {/each}
      {/if}
    {:else}
      {#each grouped as [className, items]}
        <div class="group">
          <button class="group-header" onclick={() => toggleGroup(className)}>
            <span class="expand-icon">{expandedGroups.has(className) ? "v" : ">"}</span>
            <span class="group-name">{shortClassName(className)}</span>
            <span class="group-count">{items.length}</span>
          </button>

          {#if expandedGroups.has(className)}
            <div class="group-items">
              {#each items as item}
                <button class="item" onclick={() => onSelect(item.index)}>
                  <span class="item-index">#{item.index}</span>
                  <span class="item-name">{shortClassName(item.class_name)}</span>
                  <span class="item-size">{formatBytes(item.data_size)}</span>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .component-list {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .list-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .list-header h3 {
    font-size: 14px;
    color: var(--text-primary);
  }

  .search-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .search {
    flex: 1;
    font-size: 13px;
  }

  .mode-toggle {
    padding: 4px 8px;
    font-size: 11px;
    border-radius: 3px;
    background: var(--bg-tertiary);
    color: var(--text-muted);
    border: 1px solid var(--border-color);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .mode-toggle.active {
    background: rgba(61, 142, 240, 0.15);
    color: var(--accent-blue);
    border-color: var(--accent-blue);
  }

  .groups {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .search-status {
    padding: 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
  }

  .hit-card {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: 8px 16px;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
  }

  .hit-card:hover {
    background: var(--bg-hover);
  }

  .hit-header {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .hit-name {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .hit-idx {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
  }

  .hit-path {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--accent-blue);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .hit-value {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .group-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 16px;
    text-align: left;
    color: var(--text-primary);
    font-weight: 500;
    font-size: 13px;
  }

  .group-header:hover {
    background: var(--bg-hover);
  }

  .expand-icon {
    width: 16px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
  }

  .group-name {
    flex: 1;
  }

  .group-count {
    font-size: 11px;
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 1px 6px;
    border-radius: 8px;
  }

  .group-items {
    padding-left: 24px;
  }

  .item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 4px 16px;
    text-align: left;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .item-index {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
    width: 40px;
  }

  .item-name {
    flex: 1;
  }

  .item-size {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
  }
</style>
