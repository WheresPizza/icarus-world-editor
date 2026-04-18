<script lang="ts">
  import PropertyNode from "./PropertyNode.svelte";

  let { properties, prospectId, componentIndex, onPropertyChange }: {
    properties: any[];
    prospectId: string;
    componentIndex: number;
    onPropertyChange?: (path: string, value: unknown) => void;
  } = $props();

  let viewMode = $state<"tree" | "json">("tree");
</script>

<div class="property-tree">
  <div class="tree-toolbar">
    <button
      class="mode-btn"
      class:active={viewMode === "tree"}
      onclick={() => viewMode = "tree"}
    >
      Tree
    </button>
    <button
      class="mode-btn"
      class:active={viewMode === "json"}
      onclick={() => viewMode = "json"}
    >
      JSON
    </button>
  </div>

  {#if viewMode === "tree"}
    <div class="tree-content">
      {#each properties as prop}
        <PropertyNode
          property={prop}
          path={prop.name}
          depth={0}
          {onPropertyChange}
        />
      {/each}
    </div>
  {:else}
    <pre class="json-view">{JSON.stringify(properties, null, 2)}</pre>
  {/if}
</div>

<style>
  .property-tree {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .tree-toolbar {
    display: flex;
    gap: 4px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-color);
  }

  .mode-btn {
    padding: 4px 12px;
    font-size: 12px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }

  .mode-btn:hover {
    background: var(--bg-hover);
  }

  .mode-btn.active {
    background: var(--accent-blue);
    color: white;
  }

  .tree-content {
    flex: 1;
    overflow: auto;
    padding: 8px 0;
  }

  .json-view {
    flex: 1;
    overflow: auto;
    padding: 12px;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.6;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-all;
  }
</style>
