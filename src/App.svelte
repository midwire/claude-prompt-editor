<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { openPrompt, savePrompt, getMcpPort } from "./lib/tauri";
  import { currentContent } from "./lib/stores/editor";
  import { fileState } from "./lib/stores/files";
  import { editorMode, parseFromContent } from "./lib/stores/prompt";
  import { runLint } from "./lib/stores/lint";
  import { saveVersion, refreshHistory } from "./lib/stores/version";
  import SourceEditor from "./lib/components/Editor/SourceEditor.svelte";
  import StructureEditor from "./lib/components/Editor/StructureEditor.svelte";
  import EditorTabs from "./lib/components/Editor/EditorTabs.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import PromptHealth from "./lib/components/Panels/PromptHealth.svelte";
  import VersionHistory from "./lib/components/Panels/VersionHistory.svelte";
  import NewPromptWizard from "./lib/components/Dialogs/NewPromptWizard.svelte";
  import StructureOutline from "./lib/components/Sidebar/StructureOutline.svelte";

  let currentPath: string | null = $state(null);
  let currentName: string = $state("Untitled");
  let mode = $state<"source" | "structure">("source");
  let rightPanelTab = $state<"health" | "history" | "structure">("health");
  let showNewPromptWizard = $state(false);
  let mcpCopyFeedback = $state(false);

  fileState.subscribe((s) => {
    currentPath = s.path;
    currentName = s.name;
  });

  editorMode.subscribe((m) => {
    mode = m;
  });

  // When content changes, parse to AST (debounced) and run lint
  currentContent.subscribe((content) => {
    if (content) {
      parseFromContent(content);
      runLint(content);
    }
  });

  function getProjectDir(filePath: string | null): string {
    if (!filePath) return ".";
    const parts = filePath.replace(/\\/g, "/").split("/");
    parts.pop();
    return parts.join("/") || ".";
  }

  function getPromptName(filePath: string | null, fallback: string): string {
    if (!filePath) return fallback;
    const name = filePath.replace(/\\/g, "/").split("/").pop() ?? fallback;
    return name.replace(/\.md$/, "");
  }

  async function handleOpen() {
    const selected = await open({
      multiple: false,
      filters: [
        { name: "Markdown", extensions: ["md"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });
    if (!selected) return;
    const path = typeof selected === "string" ? selected : selected;
    try {
      const file = await openPrompt(path);
      fileState.openFile(file.path, file.name, file.content);
      currentContent.set(file.content);
      // Load version history for the opened file
      const projDir = getProjectDir(file.path);
      const pName = getPromptName(file.path, file.name);
      refreshHistory(projDir, pName);
    } catch (e) {
      console.error("Failed to open file:", e);
    }
  }

  async function handleSave() {
    let content = "";
    currentContent.subscribe((c) => (content = c))();

    let path = currentPath;
    if (!path) {
      const selected = await save({
        filters: [
          { name: "Markdown", extensions: ["md"] },
          { name: "All Files", extensions: ["*"] },
        ],
      });
      if (!selected) return;
      path = selected;
    }
    try {
      await savePrompt(path, content);
      fileState.markClean();
      // Auto-save version on file save
      const projDir = getProjectDir(path);
      const pName = getPromptName(path, currentName);
      await saveVersion(projDir, pName, content);
    } catch (e) {
      console.error("Failed to save file:", e);
    }
  }

  function handleNewPrompt() {
    showNewPromptWizard = true;
  }

  function handleWizardCreate(name: string, content: string) {
    showNewPromptWizard = false;
    fileState.openFile("", name, content);
    currentContent.set(content);
  }

  async function handleCopyMcpConfig() {
    try {
      const port = await getMcpPort();
      if (port === null) {
        console.warn("MCP server port not yet available");
        return;
      }
      const config = JSON.stringify(
        {
          mcpServers: {
            "claude-prompt-editor": {
              url: `http://localhost:${port}/mcp`,
            },
          },
        },
        null,
        2,
      );
      await navigator.clipboard.writeText(config);
      mcpCopyFeedback = true;
      setTimeout(() => {
        mcpCopyFeedback = false;
      }, 2000);
    } catch (e) {
      console.error("Failed to copy MCP config:", e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "o") {
      e.preventDefault();
      handleOpen();
    }
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      handleSave();
    }
    if ((e.ctrlKey || e.metaKey) && e.key === "n") {
      e.preventDefault();
      handleNewPrompt();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app">
  <div class="toolbar">
    <button onclick={handleNewPrompt} title="New prompt (Ctrl+N)">New</button>
    <button onclick={handleOpen} title="Open file (Ctrl+O)">Open</button>
    <button onclick={handleSave} title="Save file (Ctrl+S)">Save</button>
    <button onclick={handleCopyMcpConfig} title="Copy MCP server config to clipboard" class:copied={mcpCopyFeedback}>
      {mcpCopyFeedback ? "Copied!" : "Copy MCP Config"}
    </button>
  </div>
  <EditorTabs />
  <div class="main-content">
    <div class="editor-area">
      {#if mode === "source"}
        <SourceEditor />
      {:else}
        <StructureEditor />
      {/if}
    </div>
    <div class="right-panel">
      <div class="panel-tabs">
        <button
          class="panel-tab"
          class:active={rightPanelTab === "health"}
          onclick={() => (rightPanelTab = "health")}
        >
          Health
        </button>
        <button
          class="panel-tab"
          class:active={rightPanelTab === "history"}
          onclick={() => (rightPanelTab = "history")}
        >
          History
        </button>
        <button
          class="panel-tab"
          class:active={rightPanelTab === "structure"}
          onclick={() => (rightPanelTab = "structure")}
        >
          Structure
        </button>
      </div>
      <div class="panel-content">
        {#if rightPanelTab === "health"}
          <PromptHealth />
        {:else if rightPanelTab === "history"}
          <VersionHistory
            projectDir={getProjectDir(currentPath)}
            promptName={getPromptName(currentPath, currentName)}
          />
        {:else}
          <StructureOutline />
        {/if}
      </div>
    </div>
  </div>
  <StatusBar />
</div>

{#if showNewPromptWizard}
  <NewPromptWizard
    onclose={() => (showNewPromptWizard = false)}
    oncreate={handleWizardCreate}
  />
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background-color: #333333;
    border-bottom: 1px solid #252525;
    flex-shrink: 0;
  }

  .toolbar button {
    padding: 4px 12px;
    font-size: 13px;
    background-color: #0e639c;
    color: #ffffff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    box-shadow: none;
  }

  .toolbar button:hover {
    background-color: #1177bb;
    border-color: transparent;
  }

  .toolbar button.copied {
    background-color: #2ea043;
  }

  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .editor-area {
    flex: 1;
    overflow: hidden;
  }

  .right-panel {
    width: 300px;
    flex-shrink: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    background-color: #1e1e2e;
    border-left: 1px solid #313244;
  }

  .panel-tabs {
    display: flex;
    border-bottom: 1px solid #313244;
    flex-shrink: 0;
  }

  .panel-tab {
    flex: 1;
    padding: 8px 12px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: #6c7086;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
  }

  .panel-tab:hover {
    color: #cdd6f4;
  }

  .panel-tab.active {
    color: #89b4fa;
    border-bottom-color: #89b4fa;
  }

  .panel-content {
    flex: 1;
    overflow: hidden;
  }
</style>
