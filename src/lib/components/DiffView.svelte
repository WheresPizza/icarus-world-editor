<script lang="ts">
  import type { ProspectDiff, ComponentDiff } from "../types";

  let { diffResult, nameA, nameB, onBack }: {
    diffResult: ProspectDiff;
    nameA: string;
    nameB: string;
    onBack: () => void;
  } = $props();

  let expandedComponents = $state<Set<string>>(new Set());

  function toggleComponent(key: string) {
    const next = new Set(expandedComponents);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    expandedComponents = next;
  }

  const totalChanges = $derived(
    diffResult.metadata_changes.length +
    diffResult.added_components.length +
    diffResult.removed_components.length +
    diffResult.modified_components.reduce((sum, c) => sum + c.property_changes.length, 0)
  );
</script>

<div class="diff-view">
  <div class="diff-header">
    <button class="back-btn" onclick={onBack}>&lt; Back</button>
    <div class="diff-title">
      <h2>Prospect Diff</h2>
      <div class="diff-names">
        <span class="name-a">{nameA}</span>
        <span class="arrow">↔</span>
        <span class="name-b">{nameB}</span>
      </div>
    </div>
    <span class="change-count">{totalChanges} change{totalChanges !== 1 ? "s" : ""}</span>
  </div>

  <div class="diff-content">
    {#if diffResult.metadata_changes.length > 0}
      <section class="diff-section">
        <h3>Metadata Changes ({diffResult.metadata_changes.length})</h3>
        <table class="diff-table">
          <thead>
            <tr><th>Field</th><th class="old">A (Old)</th><th class="new">B (New)</th></tr>
          </thead>
          <tbody>
            {#each diffResult.metadata_changes as change}
              <tr>
                <td class="field-name">{change.field}</td>
                <td class="old-val">{change.old_value}</td>
                <td class="new-val">{change.new_value}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </section>
    {/if}

    {#if diffResult.removed_components.length > 0}
      <section class="diff-section">
        <h3>Removed Components ({diffResult.removed_components.length})</h3>
        {#each diffResult.removed_components as cls}
          <div class="removed-item">- {cls}</div>
        {/each}
      </section>
    {/if}

    {#if diffResult.added_components.length > 0}
      <section class="diff-section">
        <h3>Added Components ({diffResult.added_components.length})</h3>
        {#each diffResult.added_components as cls}
          <div class="added-item">+ {cls}</div>
        {/each}
      </section>
    {/if}

    {#if diffResult.modified_components.length > 0}
      <section class="diff-section">
        <h3>Modified Components ({diffResult.modified_components.length})</h3>
        {#each diffResult.modified_components as comp, i}
          {@const key = `${comp.component_class}-${i}`}
          <div class="comp-diff">
            <button class="comp-header" onclick={() => toggleComponent(key)}>
              <span class="expand-icon">{expandedComponents.has(key) ? "v" : ">"}</span>
              <span class="comp-name">{comp.component_name}</span>
              <span class="change-badge">{comp.property_changes.length} change{comp.property_changes.length !== 1 ? "s" : ""}</span>
            </button>
            {#if expandedComponents.has(key)}
              <table class="diff-table inner">
                <thead>
                  <tr><th>Property</th><th class="old">A</th><th class="new">B</th></tr>
                </thead>
                <tbody>
                  {#each comp.property_changes as change}
                    <tr>
                      <td class="field-name">{change.path}</td>
                      <td class="old-val">{change.old_value}</td>
                      <td class="new-val">{change.new_value}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        {/each}
      </section>
    {/if}

    {#if totalChanges === 0}
      <div class="no-changes">No differences found between these two prospects.</div>
    {/if}
  </div>
</div>

<style>
  .diff-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .diff-header {
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

  .diff-title {
    flex: 1;
  }

  .diff-title h2 {
    font-size: 18px;
    color: var(--text-primary);
  }

  .diff-names {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
    font-size: 12px;
  }

  .name-a {
    color: var(--accent-red);
    font-family: var(--font-mono);
  }

  .name-b {
    color: var(--accent-green);
    font-family: var(--font-mono);
  }

  .arrow {
    color: var(--text-muted);
  }

  .change-count {
    font-size: 12px;
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 4px 10px;
    border-radius: 12px;
  }

  .diff-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px 24px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .diff-section h3 {
    font-size: 14px;
    color: var(--text-secondary);
    margin-bottom: 10px;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border-color);
  }

  .diff-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }

  .diff-table th {
    text-align: left;
    padding: 6px 8px;
    font-size: 11px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border-color);
  }

  .diff-table td {
    padding: 5px 8px;
    border-bottom: 1px solid rgba(255,255,255,0.04);
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .diff-table.inner {
    margin-left: 24px;
    width: calc(100% - 24px);
  }

  .field-name {
    color: var(--text-primary);
    font-weight: 500;
    width: 200px;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .old-val {
    color: var(--accent-red);
    opacity: 0.8;
  }

  .new-val {
    color: var(--accent-green);
    opacity: 0.8;
  }

  th.old { color: var(--accent-red); }
  th.new { color: var(--accent-green); }

  .removed-item {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--accent-red);
    padding: 3px 8px;
    opacity: 0.8;
  }

  .added-item {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--accent-green);
    padding: 3px 8px;
    opacity: 0.8;
  }

  .comp-diff {
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    margin-bottom: 8px;
    overflow: hidden;
  }

  .comp-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    text-align: left;
    background: var(--bg-secondary);
  }

  .comp-header:hover {
    background: var(--bg-hover);
  }

  .expand-icon {
    width: 16px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
  }

  .comp-name {
    flex: 1;
    font-size: 13px;
    color: var(--text-primary);
  }

  .change-badge {
    font-size: 11px;
    color: var(--accent-orange);
    background: rgba(230, 126, 34, 0.1);
    padding: 1px 8px;
    border-radius: 8px;
  }

  .no-changes {
    text-align: center;
    color: var(--text-muted);
    font-size: 14px;
    padding: 40px 0;
  }
</style>
