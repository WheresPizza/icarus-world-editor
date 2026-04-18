<script lang="ts">
  import PropertyNode from "./PropertyNode.svelte";

  let { property, path, depth, onPropertyChange }: {
    property: any;
    path: string;
    depth: number;
    onPropertyChange?: (path: string, value: unknown) => void;
  } = $props();

  let expanded = $state(depth < 1);

  const propType = $derived(property.type || "unknown");
  const isExpandable = $derived(
    propType === "Struct" ||
    propType === "Array" ||
    (property.value && typeof property.value === "object" && !Array.isArray(property.value) && property.value.properties) ||
    (property.value && typeof property.value === "object" && property.value.items)
  );

  function getChildren(): any[] {
    if (property.value?.properties) return property.value.properties;
    if (property.value?.items?.items) return property.value.items.items.map((item: any, i: number) => ({
      name: `[${i}]`,
      ...inferType(item),
    }));
    if (property.value?.items?.kind === "Structs" && property.value?.items?.items) {
      return property.value.items.items.map((props: any[], i: number) => ({
        name: `[${i}]`,
        type: "Struct",
        value: { struct_type: property.value.items.struct_type, properties: props },
      }));
    }
    return [];
  }

  function inferType(val: any): any {
    if (Array.isArray(val)) {
      return { type: "Struct", value: { properties: val } };
    }
    return { type: typeof val, value: val };
  }

  function getDisplayValue(): string {
    const v = property.value;
    if (v === null || v === undefined) return "null";
    if (typeof v === "boolean") return v ? "true" : "false";
    if (typeof v === "number") return String(v);
    if (typeof v === "string") return `"${v}"`;
    if (typeof v === "object") {
      if (v.properties) return `{${v.struct_type || "Struct"}}`;
      if (v.items) {
        const count = v.items.items?.length ?? v.items.length ?? "?";
        return `[${count} items]`;
      }
      return JSON.stringify(v).slice(0, 60);
    }
    return String(v);
  }

  function isEditable(): boolean {
    const t = propType;
    return ["Int", "Int64", "Float", "Double", "Bool", "Str", "Name"].includes(t);
  }

  function handleEdit(newValue: string) {
    let parsed: unknown;
    if (propType === "Bool") {
      parsed = newValue === "true";
    } else if (["Int", "Int64"].includes(propType)) {
      parsed = parseInt(newValue, 10);
    } else if (["Float", "Double"].includes(propType)) {
      parsed = parseFloat(newValue);
    } else {
      parsed = newValue;
    }
    onPropertyChange?.(path, parsed);
  }

  const typeColors: Record<string, string> = {
    Int: "#3498db",
    Int64: "#3498db",
    UInt32: "#3498db",
    UInt64: "#3498db",
    Float: "#9b59b6",
    Double: "#9b59b6",
    Bool: "#e67e22",
    Str: "#2ecc71",
    Name: "#27ae60",
    Enum: "#f1c40f",
    Struct: "#e74c3c",
    Array: "#1abc9c",
    Map: "#e91e63",
    Byte: "#95a5a6",
    Raw: "#7f8c8d",
  };
</script>

<div class="node" style="padding-left: {depth * 16 + 8}px">
  <div class="node-header">
    {#if isExpandable}
      <button class="expand-btn" onclick={() => expanded = !expanded}>
        {expanded ? "v" : ">"}
      </button>
    {:else}
      <span class="expand-spacer"></span>
    {/if}

    <span class="node-name">{property.name}</span>
    <span class="type-badge" style="color: {typeColors[propType] || 'var(--text-muted)'}">
      {propType}
    </span>

    {#if isEditable() && !isExpandable}
      {#if propType === "Bool"}
        <button
          class="bool-toggle"
          class:active={property.value === true}
          onclick={() => handleEdit(property.value ? "false" : "true")}
        >
          {property.value ? "true" : "false"}
        </button>
      {:else}
        <input
          class="inline-edit"
          type={["Int", "Int64", "Float", "Double"].includes(propType) ? "number" : "text"}
          value={typeof property.value === "object" ? getDisplayValue() : property.value}
          onchange={(e) => handleEdit((e.target as HTMLInputElement).value)}
        />
      {/if}
    {:else if !isExpandable}
      <span class="node-value">{getDisplayValue()}</span>
    {:else}
      <span class="node-value muted">{getDisplayValue()}</span>
    {/if}
  </div>

  {#if expanded && isExpandable}
    <div class="node-children">
      {#each getChildren() as child}
        <PropertyNode
          property={child}
          path="{path}.{child.name}"
          depth={depth + 1}
          {onPropertyChange}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .node {
    font-size: 13px;
  }

  .node-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    min-height: 26px;
  }

  .node-header:hover {
    background: var(--bg-hover);
  }

  .expand-btn {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .expand-spacer {
    width: 16px;
    flex-shrink: 0;
  }

  .node-name {
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
  }

  .type-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    white-space: nowrap;
    opacity: 0.8;
  }

  .node-value {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .node-value.muted {
    color: var(--text-muted);
  }

  .inline-edit {
    flex: 1;
    max-width: 200px;
    padding: 2px 6px;
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
  }

  .inline-edit:focus {
    border-color: var(--accent-blue);
  }

  .bool-toggle {
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 2px 8px;
    border-radius: 3px;
    background: var(--bg-tertiary);
    color: var(--accent-red);
  }

  .bool-toggle.active {
    background: rgba(46, 204, 113, 0.15);
    color: var(--accent-green);
  }

  .node-children {
    border-left: 1px solid var(--border-color);
    margin-left: 16px;
  }
</style>
