<script lang="ts">
  import type { Block as BlockType, BlockKind } from "../../types";
  import { blockKindLabel, blockKindIcon } from "../../types";

  interface Props {
    block: BlockType;
    onupdate: (block: BlockType) => void;
    children?: import("svelte").Snippet;
  }

  let { block, onupdate, children }: Props = $props();

  let collapsed = $state(false);

  let label = $derived(blockKindLabel(block.kind));
  let icon = $derived(blockKindIcon(block.kind));

  function toggleCollapse() {
    collapsed = !collapsed;
  }

  function toggleEnabled() {
    onupdate({ ...block, enabled: !block.enabled });
  }

  function iconSvg(name: string): string {
    const svgs: Record<string, string> = {
      user: '<svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M8 8a3 3 0 1 0 0-6 3 3 0 0 0 0 6zm5 6s1 0 1-1-1-4-6-4-6 3-6 4 1 1 1 1h10z"/></svg>',
      list: '<svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M2 3h12v1H2V3zm0 3h12v1H2V6zm0 3h12v1H2V9zm0 3h12v1H2v-1z"/></svg>',
      book: '<svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M1 2.5A1.5 1.5 0 0 1 2.5 1h3A1.5 1.5 0 0 1 7 2.5v11A1.5 1.5 0 0 1 5.5 15h-3A1.5 1.5 0 0 1 1 13.5v-11zm8 0A1.5 1.5 0 0 1 10.5 1h3A1.5 1.5 0 0 1 15 2.5v11a1.5 1.5 0 0 1-1.5 1.5h-3A1.5 1.5 0 0 1 9 13.5v-11z"/></svg>',
      file: '<svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M4 0h5l5 5v9a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2zm5 1v4h4L9 1z"/></svg>',
      paperclip: '<svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 3a2.5 2.5 0 0 1 5 0v8a1.5 1.5 0 0 1-3 0V4.5a.5.5 0 0 1 1 0V11a.5.5 0 0 0 1 0V3a1.5 1.5 0 0 0-3 0v8.5a2.5 2.5 0 0 0 5 0V4a.5.5 0 0 1 1 0v7.5a3.5 3.5 0 0 1-7 0V3z"/></svg>',
      folder: '<svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M1 3.5A1.5 1.5 0 0 1 2.5 2h2.764a1.5 1.5 0 0 1 1.118.498L7.618 4H13.5A1.5 1.5 0 0 1 15 5.5v7a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 1 12.5v-9z"/></svg>',
      edit: '<svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M12.146.854a.5.5 0 0 1 .708 0l2.292 2.292a.5.5 0 0 1 0 .708l-9.5 9.5a.5.5 0 0 1-.168.11l-5 2a.5.5 0 0 1-.65-.65l2-5a.5.5 0 0 1 .11-.168l9.5-9.5z"/></svg>',
      tag: '<svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M2 1a1 1 0 0 0-1 1v4.586a1 1 0 0 0 .293.707l7 7a1 1 0 0 0 1.414 0l4.586-4.586a1 1 0 0 0 0-1.414l-7-7A1 1 0 0 0 6.586 1H2zm4 3.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0z"/></svg>',
    };
    return svgs[name] ?? svgs["tag"];
  }
</script>

<div class="block" class:disabled={!block.enabled}>
  <div class="block-header">
    <button class="drag-handle" title="Drag to reorder">&#9776;</button>
    <button class="collapse-toggle" onclick={toggleCollapse} title={collapsed ? "Expand" : "Collapse"}>
      {collapsed ? "\u25B6" : "\u25BC"}
    </button>
    <span class="block-icon" title={icon}>{@html iconSvg(icon)}</span>
    <span class="block-label">{label}</span>
    {#if block.tag_name}
      <span class="block-tag">&lt;{block.tag_name}&gt;</span>
    {/if}
    <span class="spacer"></span>
    <label class="enable-toggle" title={block.enabled ? "Disable block" : "Enable block"}>
      <input type="checkbox" checked={block.enabled} onchange={toggleEnabled} />
    </label>
  </div>
  {#if !collapsed}
    <div class="block-body">
      {#if children}
        {@render children()}
      {/if}
    </div>
  {/if}
</div>

<style>
  .block {
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    margin-bottom: 8px;
    background-color: #252526;
    overflow: hidden;
  }

  .block.disabled {
    opacity: 0.5;
  }

  .block-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    background-color: #2d2d2d;
    border-bottom: 1px solid #3c3c3c;
    font-size: 13px;
    cursor: default;
  }

  .drag-handle {
    cursor: grab;
    background: none;
    border: none;
    color: #888;
    font-size: 14px;
    padding: 0 2px;
    line-height: 1;
  }

  .drag-handle:hover {
    color: #ccc;
  }

  .collapse-toggle {
    background: none;
    border: none;
    color: #888;
    font-size: 10px;
    padding: 0 2px;
    cursor: pointer;
    line-height: 1;
  }

  .collapse-toggle:hover {
    color: #ccc;
  }

  .block-icon {
    color: #569cd6;
    display: flex;
    align-items: center;
  }

  .block-label {
    font-weight: 500;
    color: #cccccc;
  }

  .block-tag {
    color: #808080;
    font-size: 12px;
    font-family: monospace;
  }

  .spacer {
    flex: 1;
  }

  .enable-toggle {
    display: flex;
    align-items: center;
    cursor: pointer;
  }

  .enable-toggle input {
    cursor: pointer;
  }

  .block-body {
    padding: 8px;
  }
</style>
