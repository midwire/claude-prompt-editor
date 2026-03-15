<script lang="ts">
  import { fileState } from "../stores/files";
  import { currentContent, editorState } from "../stores/editor";
  import { getMcpPort } from "../tauri";

  let fileName = $state("Untitled");
  let dirty = $state(false);
  let content = $state("");
  let cursorLine = $state(1);
  let cursorColumn = $state(1);
  let mcpPort = $state<number | null>(null);

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

  // Fetch MCP port on mount; poll until available
  async function fetchMcpPort() {
    try {
      const port = await getMcpPort();
      if (port !== null) {
        mcpPort = port;
      } else {
        setTimeout(fetchMcpPort, 1000);
      }
    } catch {
      // Not in Tauri context (e.g., browser dev mode); ignore
    }
  }

  fetchMcpPort();

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
  {#if mcpPort !== null}
    <span class="mcp-indicator" title="MCP server running">MCP :{mcpPort}</span>
  {/if}
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

  .mcp-indicator {
    opacity: 0.9;
    font-weight: 500;
    background-color: rgba(255, 255, 255, 0.15);
    padding: 0 6px;
    border-radius: 3px;
  }
</style>
