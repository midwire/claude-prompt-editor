<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { openPrompt, savePrompt } from "./lib/tauri";
  import { currentContent } from "./lib/stores/editor";
  import { fileState } from "./lib/stores/files";
  import SourceEditor from "./lib/components/Editor/SourceEditor.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";

  let currentPath: string | null = $state(null);

  fileState.subscribe((s) => {
    currentPath = s.path;
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
  <div class="editor-area">
    <SourceEditor />
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

  .editor-area {
    flex: 1;
    overflow: hidden;
  }
</style>
