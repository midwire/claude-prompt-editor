<script lang="ts">
  import { fileState } from "../stores/files";
  import { currentContent, editorState } from "../stores/editor";

  let fileName = $state("Untitled");
  let dirty = $state(false);
  let content = $state("");
  let cursorLine = $state(1);
  let cursorColumn = $state(1);

  // Subscribe to stores
  fileState.subscribe((s) => {
    fileName = s.name;
    dirty = s.dirty;
  });

  currentContent.subscribe((c) => {
    content = c;
  });

  editorState.subscribe((s) => {
    cursorLine = s.cursorLine;
    cursorColumn = s.cursorColumn;
  });

  let charCount = $derived(content.length);
  let tokenEstimate = $derived(Math.ceil(content.length / 4));
</script>

<div class="status-bar">
  <span class="file-name">
    {fileName}{dirty ? " *" : ""}
  </span>
  <span class="spacer"></span>
  <span class="info">Ln {cursorLine}, Col {cursorColumn}</span>
  <span class="info">{charCount} chars</span>
  <span class="info">~{tokenEstimate} tokens</span>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    height: 24px;
    padding: 0 12px;
    background-color: #007acc;
    color: #ffffff;
    font-size: 12px;
    font-family: "Segoe UI", sans-serif;
    gap: 16px;
    flex-shrink: 0;
  }

  .file-name {
    font-weight: 500;
  }

  .spacer {
    flex: 1;
  }

  .info {
    opacity: 0.9;
  }
</style>
