<script lang="ts">
  import { currentAst } from "../../stores/prompt";
  import { blockKindLabel, blockKindIcon, type Block, type PromptAst } from "../../types";

  let ast = $state<PromptAst | null>(null);
  currentAst.subscribe((a) => (ast = a));

  interface OutlineEntry {
    label: string;
    icon: string;
    depth: number;
    blockIndex: number;
  }

  function buildOutline(): OutlineEntry[] {
    if (!ast) return [];
    const entries: OutlineEntry[] = [];
    for (let i = 0; i < ast.blocks.length; i++) {
      const block = ast.blocks[i];
      entries.push({
        label: blockKindLabel(block.kind),
        icon: blockKindIcon(block.kind),
        depth: 0,
        blockIndex: i,
      });
      if (block.children) {
        for (const child of block.children) {
          entries.push({
            label: blockKindLabel(child.kind),
            icon: blockKindIcon(child.kind),
            depth: 1,
            blockIndex: i,
          });
        }
      }
    }
    return entries;
  }

  const iconMap: Record<string, string> = {
    user: "&#9899;",     // role
    list: "&#9776;",     // instructions
    book: "&#128218;",   // examples
    file: "&#128196;",   // example
    paperclip: "&#128206;", // context
    folder: "&#128193;", // documents
    edit: "&#9998;",     // freeform
    tag: "&#127991;",    // custom
  };

  function getIcon(icon: string): string {
    return iconMap[icon] ?? "&#8226;";
  }
</script>

<div class="structure-outline">
  <div class="header">
    <h3>Structure</h3>
  </div>

  {#if !ast || ast.blocks.length === 0}
    <div class="empty">No blocks parsed yet.</div>
  {:else}
    {#if ast.metadata.name}
      <div class="meta-row">
        <span class="meta-label">Name:</span>
        <span class="meta-value">{ast.metadata.name}</span>
      </div>
    {/if}
    {#if ast.metadata.model}
      <div class="meta-row">
        <span class="meta-label">Model:</span>
        <span class="meta-value">{ast.metadata.model}</span>
      </div>
    {/if}

    <ul class="outline-list">
      {#each buildOutline() as entry}
        <li class="outline-entry" style="padding-left: {12 + entry.depth * 16}px">
          <span class="outline-icon">{@html getIcon(entry.icon)}</span>
          <span class="outline-label">{entry.label}</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .structure-outline {
    height: 100%;
    overflow-y: auto;
    padding: 12px;
    background-color: #1e1e2e;
    color: #cdd6f4;
    font-size: 13px;
  }

  .header {
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid #313244;
  }

  .header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: #cdd6f4;
  }

  .empty {
    color: #a6adc8;
    font-style: italic;
    text-align: center;
    padding: 24px 8px;
  }

  .meta-row {
    display: flex;
    gap: 6px;
    font-size: 11px;
    margin-bottom: 4px;
  }

  .meta-label {
    color: #6c7086;
  }

  .meta-value {
    color: #a6adc8;
  }

  .outline-list {
    list-style: none;
    padding: 0;
    margin: 8px 0 0 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .outline-entry {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: 3px;
    font-size: 12px;
    color: #cdd6f4;
  }

  .outline-entry:hover {
    background-color: #313244;
  }

  .outline-icon {
    font-size: 12px;
    min-width: 16px;
    text-align: center;
  }

  .outline-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
