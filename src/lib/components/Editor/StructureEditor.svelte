<script lang="ts">
  import type { Block as BlockType, BlockKind, PromptAst } from "../../types";
  import { currentAst, parseError, syncAstToContent } from "../../stores/prompt";
  import MetadataBlock from "../Blocks/MetadataBlock.svelte";
  import TaggedBlock from "../Blocks/TaggedBlock.svelte";
  import FreeformBlock from "../Blocks/FreeformBlock.svelte";
  import ExamplesBlock from "../Blocks/ExamplesBlock.svelte";

  let ast = $state<PromptAst | null>(null);
  let error = $state<string | null>(null);
  let showAddMenu = $state(false);
  let dragSourceIndex = $state<number | null>(null);

  const blockTypes: { kind: BlockKind; tag: string; label: string }[] = [
    { kind: "Role", tag: "role", label: "Role" },
    { kind: "Instructions", tag: "instructions", label: "Instructions" },
    { kind: "Context", tag: "context", label: "Context" },
    { kind: "Examples", tag: "examples", label: "Examples" },
    { kind: { Custom: "constraints" }, tag: "constraints", label: "Constraints" },
    { kind: { Custom: "output_format" }, tag: "output_format", label: "Output Format" },
    { kind: { Custom: "documents" }, tag: "documents", label: "Documents" },
    { kind: "Freeform", tag: "", label: "Freeform Text" },
  ];

  currentAst.subscribe((a) => {
    ast = a;
  });

  parseError.subscribe((e) => {
    error = e;
  });

  function handleMetadataUpdate(metadata: PromptAst["metadata"]) {
    if (!ast) return;
    const updated = { ...ast, metadata };
    syncAstToContent(updated);
  }

  function handleBlockUpdate(index: number, block: BlockType) {
    if (!ast) return;
    const newBlocks = [...ast.blocks];
    newBlocks[index] = block;
    const updated = { ...ast, blocks: newBlocks };
    syncAstToContent(updated);
  }

  function addBlock(type: typeof blockTypes[number]) {
    if (!ast) return;
    const newBlock: BlockType = {
      kind: type.kind,
      tag_name: type.tag || null,
      content: "\n",
      children: type.kind === "Examples" ? [] : [],
      enabled: true,
      start_offset: 0,
      end_offset: 0,
    };
    const updated = { ...ast, blocks: [...ast.blocks, newBlock] };
    syncAstToContent(updated);
    showAddMenu = false;
  }

  function deleteBlock(index: number) {
    if (!ast) return;
    const newBlocks = ast.blocks.filter((_, i) => i !== index);
    const updated = { ...ast, blocks: newBlocks };
    syncAstToContent(updated);
  }

  function moveBlock(fromIndex: number, toIndex: number) {
    if (!ast || fromIndex === toIndex) return;
    const newBlocks = [...ast.blocks];
    const [moved] = newBlocks.splice(fromIndex, 1);
    newBlocks.splice(toIndex, 0, moved);
    syncAstToContent({ ...ast, blocks: newBlocks });
  }

  function handleDragStart(index: number, e: DragEvent) {
    dragSourceIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", String(index));
    }
  }

  function handleDragOver(e: DragEvent) {
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = "move";
    }
  }

  function handleDrop(index: number) {
    if (dragSourceIndex !== null && dragSourceIndex !== index) {
      moveBlock(dragSourceIndex, index);
    }
    dragSourceIndex = null;
  }

  function handleDragEnd() {
    dragSourceIndex = null;
  }

  function isFreeform(block: BlockType): boolean {
    return block.kind === "Freeform";
  }

  function isExamples(block: BlockType): boolean {
    return block.kind === "Examples";
  }
</script>

<div class="structure-editor">
  {#if error}
    <div class="parse-error">
      <strong>Parse Error:</strong> {error}
    </div>
  {/if}

  {#if ast}
    <div class="blocks-list">
      <MetadataBlock
        metadata={ast.metadata}
        onupdate={handleMetadataUpdate}
      />

      {#each ast.blocks as block, i}
        {#if isFreeform(block)}
          <FreeformBlock
            {block}
            onupdate={(updated) => handleBlockUpdate(i, updated)}
            ondelete={() => deleteBlock(i)}
            ondragstart={(e) => handleDragStart(i, e)}
            ondragover={handleDragOver}
            ondrop={() => handleDrop(i)}
            ondragend={handleDragEnd}
          />
        {:else if isExamples(block)}
          <ExamplesBlock
            {block}
            onupdate={(updated) => handleBlockUpdate(i, updated)}
            ondelete={() => deleteBlock(i)}
            ondragstart={(e) => handleDragStart(i, e)}
            ondragover={handleDragOver}
            ondrop={() => handleDrop(i)}
            ondragend={handleDragEnd}
          />
        {:else}
          <TaggedBlock
            {block}
            onupdate={(updated) => handleBlockUpdate(i, updated)}
            ondelete={() => deleteBlock(i)}
            ondragstart={(e) => handleDragStart(i, e)}
            ondragover={handleDragOver}
            ondrop={() => handleDrop(i)}
            ondragend={handleDragEnd}
          />
        {/if}
      {/each}

      {#if ast.blocks.length === 0}
        <p class="empty-hint">No blocks yet. Click "Add Block" below to get started.</p>
      {/if}

      <div class="add-block-container">
        <button class="add-block-btn" onclick={() => showAddMenu = !showAddMenu}>
          + Add Block
        </button>
        {#if showAddMenu}
          <div class="add-menu">
            {#each blockTypes as type}
              <button class="add-menu-item" onclick={() => addBlock(type)}>
                <span class="add-menu-tag">{type.tag ? `<${type.tag}>` : "text"}</span>
                <span class="add-menu-label">{type.label}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {:else if !error}
    <div class="loading">Parsing content...</div>
  {/if}
</div>

<style>
  .structure-editor {
    height: 100%;
    overflow-y: auto;
    padding: 12px;
    background-color: #1e1e1e;
  }

  .blocks-list {
    max-width: 800px;
    margin: 0 auto;
  }

  .parse-error {
    background-color: #5a1d1d;
    border: 1px solid #be1100;
    border-radius: 4px;
    padding: 8px 12px;
    margin-bottom: 12px;
    color: #f48771;
    font-size: 13px;
  }

  .empty-hint {
    color: #808080;
    font-size: 13px;
    font-style: italic;
    text-align: center;
    padding: 24px;
  }

  .loading {
    color: #808080;
    font-size: 13px;
    text-align: center;
    padding: 24px;
  }

  .add-block-container {
    position: relative;
    margin-top: 8px;
  }

  .add-block-btn {
    width: 100%;
    padding: 10px;
    background-color: transparent;
    border: 2px dashed #3c3c3c;
    border-radius: 4px;
    color: #808080;
    font-size: 13px;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .add-block-btn:hover {
    border-color: #007acc;
    color: #007acc;
  }

  .add-menu {
    position: absolute;
    bottom: 100%;
    left: 0;
    right: 0;
    margin-bottom: 4px;
    background-color: #252526;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    overflow: hidden;
    z-index: 10;
    box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.3);
  }

  .add-menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 12px;
    background: none;
    border: none;
    border-bottom: 1px solid #3c3c3c;
    color: #cccccc;
    font-size: 13px;
    cursor: pointer;
    text-align: left;
  }

  .add-menu-item:last-child {
    border-bottom: none;
  }

  .add-menu-item:hover {
    background-color: #04395e;
  }

  .add-menu-tag {
    font-family: "Cascadia Code", "Fira Code", monospace;
    font-size: 12px;
    color: #569cd6;
    min-width: 120px;
  }

  .add-menu-label {
    color: #cccccc;
  }
</style>
