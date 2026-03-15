<script lang="ts">
  import type { Block as BlockType } from "../../types";
  import Block from "./Block.svelte";
  import TaggedBlock from "./TaggedBlock.svelte";

  interface Props {
    block: BlockType;
    onupdate: (block: BlockType) => void;
    ondelete?: () => void;
  }

  let { block, onupdate, ondelete }: Props = $props();

  function handleChildUpdate(index: number, child: BlockType) {
    const newChildren = [...block.children];
    newChildren[index] = child;
    onupdate({ ...block, children: newChildren });
  }

  function addExample() {
    const newExample: BlockType = {
      kind: "Example",
      tag_name: "example",
      content: "[Your example input here]\n",
      children: [],
      enabled: true,
      start_offset: 0,
      end_offset: 0,
    };
    onupdate({ ...block, children: [...block.children, newExample] });
  }
</script>

<Block {block} {onupdate} {ondelete}>
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
  <button class="add-example-btn" onclick={addExample}>+ Add Example</button>
</Block>

<style>
  .empty-hint {
    color: #808080;
    font-size: 13px;
    font-style: italic;
    padding: 4px 0;
  }

  .add-example-btn {
    margin-top: 8px;
    padding: 4px 12px;
    font-size: 12px;
    background-color: #313244;
    color: #a6adc8;
    border: 1px dashed #45475a;
    border-radius: 4px;
    cursor: pointer;
  }

  .add-example-btn:hover {
    background-color: #45475a;
    color: #cdd6f4;
  }
</style>
