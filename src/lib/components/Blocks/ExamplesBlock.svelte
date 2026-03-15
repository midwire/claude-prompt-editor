<script lang="ts">
  import type { Block as BlockType } from "../../types";
  import Block from "./Block.svelte";
  import TaggedBlock from "./TaggedBlock.svelte";

  interface Props {
    block: BlockType;
    onupdate: (block: BlockType) => void;
  }

  let { block, onupdate }: Props = $props();

  function handleChildUpdate(index: number, child: BlockType) {
    const newChildren = [...block.children];
    newChildren[index] = child;
    onupdate({ ...block, children: newChildren });
  }
</script>

<Block {block} {onupdate}>
  {#if block.children.length === 0}
    <p class="empty-hint">No examples yet.</p>
  {:else}
    {#each block.children as child, i}
      <TaggedBlock
        block={child}
        onupdate={(updated) => handleChildUpdate(i, updated)}
      />
    {/each}
  {/if}
</Block>

<style>
  .empty-hint {
    color: #808080;
    font-size: 13px;
    font-style: italic;
    padding: 4px 0;
  }
</style>
