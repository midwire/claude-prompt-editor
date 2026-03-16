<script lang="ts">
  import type { Block as BlockType } from "../../types";
  import Block from "./Block.svelte";

  interface Props {
    block: BlockType;
    onupdate: (block: BlockType) => void;
    ondelete?: () => void;
    ondragstart?: (e: DragEvent) => void;
    ondragover?: (e: DragEvent) => void;
    ondrop?: (e: DragEvent) => void;
    ondragend?: (e: DragEvent) => void;
  }

  let { block, onupdate, ondelete, ondragstart, ondragover, ondrop, ondragend }: Props = $props();

  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    onupdate({ ...block, content: target.value });
  }
</script>

<Block {block} {onupdate} {ondelete} {ondragstart} {ondragover} {ondrop} {ondragend}>
  <textarea
    class="block-textarea"
    value={block.content}
    oninput={handleInput}
    rows={Math.max(3, block.content.split("\n").length + 1)}
    disabled={!block.enabled}
    spellcheck={false}
  ></textarea>
</Block>

<style>
  .block-textarea {
    width: 100%;
    min-height: 60px;
    background-color: #1e1e1e;
    color: #d4d4d4;
    border: 1px solid #3c3c3c;
    border-radius: 3px;
    padding: 8px;
    font-family: "Cascadia Code", "Fira Code", "Consolas", monospace;
    font-size: 13px;
    line-height: 1.5;
    resize: vertical;
    box-sizing: border-box;
  }

  .block-textarea:focus {
    outline: none;
    border-color: #007acc;
  }

  .block-textarea:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
