<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { openPrompt, savePrompt } from "./lib/tauri";
  import { currentContent } from "./lib/stores/editor";
  import { fileState } from "./lib/stores/files";
  import { editorMode, parseFromContent } from "./lib/stores/prompt";
  import { runLint } from "./lib/stores/lint";
  import SourceEditor from "./lib/components/Editor/SourceEditor.svelte";
  import StructureEditor from "./lib/components/Editor/StructureEditor.svelte";
  import EditorTabs from "./lib/components/Editor/EditorTabs.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import PromptHealth from "./lib/components/Panels/PromptHealth.svelte";

  let currentPath: string | null = $state(null);
  let mode = $state<"source" | "structure">("source");

  fileState.subscribe((s) => {
    currentPath = s.path;
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
    } catch (e) {
      console.error("Failed to save file:", e);
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
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app">
  <div class="toolbar">
    <button onclick={handleOpen} title="Open file (Ctrl+O)">Open</button>
    <button onclick={handleSave} title="Save file (Ctrl+S)">Save</button>
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
    <div class="health-panel">
      <PromptHealth />
    </div>
  </div>
  <StatusBar />
</div>

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

  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .editor-area {
    flex: 1;
    overflow: hidden;
  }

  .health-panel {
    width: 300px;
    flex-shrink: 0;
    overflow: hidden;
  }
</style>
