<script lang="ts">
  import type { Block as BlockType, PromptAst } from "../../types";
  import { currentAst, parseError, syncAstToContent } from "../../stores/prompt";
  import MetadataBlock from "../Blocks/MetadataBlock.svelte";
  import TaggedBlock from "../Blocks/TaggedBlock.svelte";
  import FreeformBlock from "../Blocks/FreeformBlock.svelte";
  import ExamplesBlock from "../Blocks/ExamplesBlock.svelte";

  let ast = $state<PromptAst | null>(null);
  let error = $state<string | null>(null);

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
          />
        {:else if isExamples(block)}
          <ExamplesBlock
            {block}
            onupdate={(updated) => handleBlockUpdate(i, updated)}
          />
        {:else}
          <TaggedBlock
            {block}
            onupdate={(updated) => handleBlockUpdate(i, updated)}
          />
        {/if}
      {/each}

      {#if ast.blocks.length === 0}
        <p class="empty-hint">No blocks found. Switch to Source mode to add content.</p>
      {/if}
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
</style>
