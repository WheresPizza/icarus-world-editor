<script lang="ts">
  import type { InventoryView, InventoryComponent, ItemSlot } from "../types";
  import * as api from "../api";

  let { prospectId, onSave }: {
    prospectId: string;
    onSave?: () => void;
  } = $props();

  let inventoryView = $state<InventoryView | null>(null);
  let selectedComponentIdx = $state<number | null>(null);
  let loading = $state(false);
  let errorMsg = $state<string | null>(null);
  let newItemKey = $state("");
  let newItemQuantity = $state(1);
  let editingSlot = $state<number | null>(null);
  let editQuantity = $state(1);

  const selectedComponent = $derived(
    inventoryView?.components.find(c => c.component_idx === selectedComponentIdx) ?? null
  );

  const dtKeyValid = $derived(
    !newItemKey || newItemKey.startsWith("Item_") && newItemKey.endsWith("_C")
  );

  async function loadInventory() {
    loading = true;
    errorMsg = null;
    try {
      inventoryView = await api.getInventoryView(prospectId);
      if (inventoryView.components.length > 0 && selectedComponentIdx === null) {
        selectedComponentIdx = inventoryView.components[0].component_idx;
      }
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      loading = false;
    }
  }

  async function handleDelete(slot: ItemSlot) {
    if (!selectedComponent) return;
    loading = true;
    try {
      await api.deleteInventorySlot(prospectId, selectedComponent.component_idx, slot.slot_index);
      await loadInventory();
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      loading = false;
    }
  }

  async function handleUpdateQuantity(slot: ItemSlot) {
    if (!selectedComponent) return;
    loading = true;
    try {
      await api.updateInventorySlot(
        prospectId,
        selectedComponent.component_idx,
        slot.slot_index,
        slot.item_key,
        editQuantity
      );
      editingSlot = null;
      await loadInventory();
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      loading = false;
    }
  }

  async function handleAddItem() {
    if (!selectedComponent || !newItemKey.trim()) return;
    loading = true;
    try {
      await api.addInventoryItem(
        prospectId,
        selectedComponent.component_idx,
        newItemKey.trim(),
        newItemQuantity
      );
      newItemKey = "";
      newItemQuantity = 1;
      await loadInventory();
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      loading = false;
    }
  }

  // Load on mount
  $effect(() => {
    if (prospectId) loadInventory();
  });
</script>

<div class="inventory-editor">
  {#if errorMsg}
    <div class="error-bar">
      {errorMsg}
      <button onclick={() => errorMsg = null}>×</button>
    </div>
  {/if}

  {#if loading}
    <div class="loading-bar">Loading...</div>
  {/if}

  {#if !inventoryView || inventoryView.components.length === 0}
    <div class="empty-state">
      {#if !loading}
        <p>No inventory components found in this prospect.</p>
      {/if}
    </div>
  {:else}
    <div class="editor-layout">
      <!-- Left: component list -->
      <div class="comp-list">
        <div class="comp-list-header">Inventories ({inventoryView.components.length})</div>
        {#each inventoryView.components as comp}
          <button
            class="comp-item"
            class:active={comp.component_idx === selectedComponentIdx}
            onclick={() => selectedComponentIdx = comp.component_idx}
          >
            <span class="comp-name">{comp.component_name}</span>
            <span class="comp-count">{comp.slots.length}</span>
          </button>
        {/each}
      </div>

      <!-- Right: item grid -->
      <div class="item-grid">
        {#if selectedComponent}
          <div class="grid-header">
            <span class="grid-title">{selectedComponent.component_name}</span>
            <span class="grid-class">{selectedComponent.component_class}</span>
          </div>

          <table class="items-table">
            <thead>
              <tr>
                <th>#</th>
                <th>Item Key</th>
                <th>Qty</th>
                <th>Durability</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {#each selectedComponent.slots as slot}
                <tr>
                  <td class="slot-idx">{slot.slot_index}</td>
                  <td class="item-key">{slot.item_key}</td>
                  <td class="qty">
                    {#if editingSlot === slot.slot_index}
                      <input
                        type="number"
                        class="qty-input"
                        min="0"
                        bind:value={editQuantity}
                        onkeydown={(e) => e.key === "Enter" && handleUpdateQuantity(slot)}
                      />
                      <button class="save-btn" onclick={() => handleUpdateQuantity(slot)}>✓</button>
                      <button class="cancel-btn" onclick={() => editingSlot = null}>✕</button>
                    {:else}
                      <button class="qty-display" onclick={() => { editingSlot = slot.slot_index; editQuantity = slot.quantity; }}>
                        {slot.quantity}
                      </button>
                    {/if}
                  </td>
                  <td class="dur">
                    {slot.durability !== null ? slot.durability?.toFixed(1) : "—"}
                  </td>
                  <td>
                    <button class="delete-btn" onclick={() => handleDelete(slot)}>×</button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>

          <!-- Add item form -->
          <div class="add-form">
            <input
              type="text"
              class="key-input"
              class:invalid={!dtKeyValid && !!newItemKey}
              placeholder="Item_ExampleItem_T1_C"
              bind:value={newItemKey}
            />
            <input
              type="number"
              class="qty-input-sm"
              min="1"
              bind:value={newItemQuantity}
            />
            <button
              class="btn-add"
              onclick={handleAddItem}
              disabled={!newItemKey.trim()}
            >
              Add Item
            </button>
            {#if !dtKeyValid && newItemKey}
              <span class="key-warning">Key should match Item_*_C pattern</span>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .inventory-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .error-bar {
    padding: 8px 16px;
    background: rgba(231, 76, 60, 0.15);
    border-bottom: 1px solid var(--accent-red);
    color: var(--accent-red);
    font-size: 13px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .loading-bar {
    padding: 8px 16px;
    font-size: 12px;
    color: var(--text-muted);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: 13px;
  }

  .editor-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .comp-list {
    width: 200px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-color);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .comp-list-header {
    padding: 10px 12px;
    font-size: 12px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border-color);
  }

  .comp-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    text-align: left;
    font-size: 12px;
    color: var(--text-secondary);
    border-bottom: 1px solid rgba(255,255,255,0.03);
  }

  .comp-item:hover { background: var(--bg-hover); color: var(--text-primary); }

  .comp-item.active {
    background: var(--bg-active);
    color: var(--accent-blue);
  }

  .comp-name { flex: 1; }

  .comp-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 1px 5px;
    border-radius: 8px;
  }

  .item-grid {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .grid-header {
    padding: 10px 16px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: baseline;
    gap: 10px;
    flex-shrink: 0;
  }

  .grid-title {
    font-size: 14px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .grid-class {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .items-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
    flex: 1;
    overflow-y: auto;
    display: block;
  }

  .items-table thead {
    display: table;
    width: 100%;
    table-layout: fixed;
  }

  .items-table tbody {
    display: block;
    overflow-y: auto;
    max-height: calc(100% - 120px);
    width: 100%;
  }

  .items-table tr {
    display: table;
    width: 100%;
    table-layout: fixed;
  }

  .items-table th {
    padding: 6px 12px;
    text-align: left;
    font-size: 11px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border-color);
  }

  .items-table td {
    padding: 5px 12px;
    border-bottom: 1px solid rgba(255,255,255,0.04);
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .slot-idx {
    width: 40px;
    color: var(--text-muted);
  }

  .item-key {
    color: var(--accent-blue);
  }

  .qty {
    width: 80px;
  }

  .qty-display {
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 3px;
  }

  .qty-display:hover { background: var(--bg-hover); }

  .qty-input {
    width: 50px;
    padding: 2px 4px;
    font-family: var(--font-mono);
    font-size: 11px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
  }

  .save-btn {
    color: var(--accent-green);
    font-size: 12px;
    padding: 2px 4px;
  }

  .cancel-btn {
    color: var(--accent-red);
    font-size: 12px;
    padding: 2px 4px;
  }

  .dur {
    width: 60px;
    color: var(--text-muted);
  }

  .delete-btn {
    color: var(--accent-red);
    font-size: 14px;
    opacity: 0.5;
    padding: 2px 8px;
  }

  .delete-btn:hover { opacity: 1; }

  .add-form {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-top: 1px solid var(--border-color);
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .key-input {
    flex: 1;
    min-width: 180px;
    padding: 5px 8px;
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
  }

  .key-input.invalid {
    border-color: var(--accent-orange);
  }

  .qty-input-sm {
    width: 60px;
    padding: 5px 6px;
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
  }

  .btn-add {
    padding: 5px 12px;
    background: var(--accent-blue);
    color: white;
    border-radius: 3px;
    font-size: 12px;
  }

  .btn-add:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-add:not(:disabled):hover { background: var(--accent-blue-hover); }

  .key-warning {
    font-size: 11px;
    color: var(--accent-orange);
    width: 100%;
  }
</style>
