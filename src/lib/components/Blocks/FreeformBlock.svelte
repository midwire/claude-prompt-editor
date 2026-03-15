<script lang="ts">
  import type { Block as BlockType } from "../../types";
  import Block from "./Block.svelte";

  interface Props {
    block: BlockType;
    onupdate: (block: BlockType) => void;
    ondelete?: () => void;
  }

  let { block, onupdate, ondelete }: Props = $props();

  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    onupdate({ ...block, content: target.value });
  }
</script>

<Block {block} {onupdate} {ondelete}>
  <textarea
    class="freeform-textarea"
    value={block.content}
    oninput={handleInput}
    rows={Math.max(3, block.content.split("\n").length + 1)}
    disabled={!block.enabled}
    placeholder="Freeform text..."
    spellcheck={false}
  ></textarea>
</Block>

<style>
  .freeform-textarea {
    width: 100%;
    min-height: 60px;
    background-color: #1e1e1e;
    color: #d4d4d4;
    border: 2px dashed #4a4a4a;
    border-radius: 3px;
    padding: 8px;
    font-family: "Cascadia Code", "Fira Code", "Consolas", monospace;
    font-size: 13px;
    line-height: 1.5;
    resize: vertical;
    box-sizing: border-box;
  }

  .freeform-textarea:focus {
    outline: none;
    border-color: #007acc;
  }

  .freeform-textarea:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
