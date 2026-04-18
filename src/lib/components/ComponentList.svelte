<script lang="ts">
  import type { ComponentSummary } from "../types";
  import { shortClassName, formatBytes, groupByClass } from "../helpers";

  let { components, onSelect }: {
    components: ComponentSummary[];
    onSelect: (index: number) => void;
  } = $props();

  let searchQuery = $state("");
  let expandedGroups = $state<Set<string>>(new Set());

  let grouped = $derived.by(() => {
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
</script>

<div class="component-list">
  <div class="list-header">
    <h3>Components ({components.length})</h3>
    <input
      type="text"
      class="search"
      placeholder="Filter components..."
      bind:value={searchQuery}
    />
  </div>

  <div class="groups">
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

  .search {
    width: 100%;
    font-size: 13px;
  }

  .groups {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
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
