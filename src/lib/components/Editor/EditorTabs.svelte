<script lang="ts">
  import { editorMode } from "../../stores/prompt";

  let mode = $state<"source" | "structure">("source");

  editorMode.subscribe((m) => {
    mode = m;
  });

  function setMode(m: "source" | "structure") {
    editorMode.set(m);
  }
</script>

<div class="editor-tabs">
  <button
    class="tab"
    class:active={mode === "source"}
    onclick={() => setMode("source")}
  >
    Source
  </button>
  <button
    class="tab"
    class:active={mode === "structure"}
    onclick={() => setMode("structure")}
  >
    Structure
  </button>
</div>

<style>
  .editor-tabs {
    display: flex;
    background-color: #252526;
    border-bottom: 1px solid #3c3c3c;
    flex-shrink: 0;
  }

  .tab {
    padding: 6px 16px;
    font-size: 13px;
    font-family: inherit;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: #808080;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
  }

  .tab:hover {
    color: #cccccc;
  }

  .tab.active {
    color: #ffffff;
    border-bottom-color: #007acc;
  }
</style>
