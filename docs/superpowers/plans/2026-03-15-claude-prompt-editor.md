# Claude Prompt Editor/IDE Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri desktop IDE for crafting, validating, and testing Claude API prompts with dual-mode editing, best-practices linting, and Claude Code integration via MCP.

**Architecture:** Tauri 2.0 app with Rust backend (file I/O, markdown/XML parser, AST, linter, version store, MCP server) and Svelte frontend (Monaco source editor, block-based structure editor, linting panel, version history). Single binary distribution for Linux and Mac.

**Tech Stack:** Rust, Tauri 2.0, Svelte, Monaco Editor, Vite, pnpm, pulldown-cmark, serde/serde_yaml, similar, rmcp

---

## File Structure

### Rust Backend (`src-tauri/src/`)

```
src-tauri/
  Cargo.toml
  tauri.conf.json
  src/
    main.rs                    # Tauri entry point
    lib.rs                     # Module declarations
    commands/
      mod.rs                   # Command module exports
      file.rs                  # Open/save/list prompt files
      prompt.rs                # Parse, render, AST operations
      lint.rs                  # Run linter, get results
      version.rs               # Version history commands
      preset.rs                # Preset/template CRUD
      mcp.rs                   # MCP server start/stop/status
    parser/
      mod.rs                   # Parser module exports
      frontmatter.rs           # YAML frontmatter parsing
      xml_tags.rs              # Fault-tolerant XML tag extraction
      ast.rs                   # PromptAst, Block, BlockKind types
      serializer.rs            # AST → markdown string
      variables.rs             # {{variable}} interpolation
    linter/
      mod.rs                   # Linter entry point, run all rules
      rules.rs                 # Rule trait, RuleResult, Severity
      structural.rs            # Structural rules (missing role, sparse examples, etc.)
      antipatterns.rs          # Anti-pattern rules (negative framing, over-prompting, etc.)
      config.rs                # Load/merge lintrc.yaml
    version/
      mod.rs                   # Version history entry point
      store.rs                 # Save/load version snapshots
      diff.rs                  # Generate/apply text diffs via `similar`
    mcp/
      mod.rs                   # MCP module exports
      server.rs                # SSE transport, server lifecycle
      tools.rs                 # load_prompt, list_prompts, get_prompt_health, set_variable
    preset/
      mod.rs                   # Preset module exports
      builtin.rs               # Ship built-in role/section/variable presets
      custom.rs                # User-defined preset CRUD
      templates.rs             # Template management (blank + use-case)
```

### Svelte Frontend (`src/`)

```
src/
  App.svelte                   # Main layout: sidebar, editor area, panels
  app.css                      # Global styles, design system variables
  main.ts                      # Svelte mount point
  lib/
    tauri.ts                   # Tauri invoke wrappers (typed)
    types.ts                   # TypeScript types mirroring Rust AST
    stores/
      prompt.ts                # Svelte store: current prompt AST
      editor.ts                # Svelte store: active mode, editor state
      lint.ts                  # Svelte store: lint results
      version.ts               # Svelte store: version history
      files.ts                 # Svelte store: file list, active file
      presets.ts               # Svelte store: available presets
    components/
      Editor/
        SourceEditor.svelte    # Monaco wrapper with prompt highlighting
        StructureEditor.svelte # Block editor rendering AST blocks
        EditorTabs.svelte      # Source/Structure mode toggle
      Blocks/
        Block.svelte           # Base block wrapper (collapse, drag, toggle)
        MetadataBlock.svelte   # Frontmatter form fields
        RoleBlock.svelte       # Role content editor
        InstructionsBlock.svelte
        ExamplesBlock.svelte   # Container with sub-example blocks
        ExampleItem.svelte     # Single example with input/output
        CustomBlock.svelte     # Generic XML-tagged block
        FreeformBlock.svelte   # Untagged prose
      Sidebar/
        FileExplorer.svelte    # File tree for prompts directory
        StructureOutline.svelte # Minimap of block hierarchy
      Panels/
        PromptHealth.svelte    # Lint results display
        VersionHistory.svelte  # Timeline, diff viewer
      Dialogs/
        NewPromptWizard.svelte # Template selection dialog
        PresetPalette.svelte   # Searchable preset picker
      StatusBar.svelte         # Token count, MCP status, version
```

### Config & Project Files

```
.claude-prompts/
  history/                     # Version snapshots (auto-managed)
  presets/                     # User presets (markdown snippets)
  templates/                   # Project-specific templates
  variables.yaml               # Persisted variable values
  lintrc.yaml                  # Lint rule configuration
```

---

## Chunk 1: Project Scaffolding & Walking Skeleton

This chunk delivers the walking skeleton: a Tauri app that can open/save markdown prompt files, edit them in a Monaco editor, and serve the current prompt via MCP.

### Task 1: Scaffold Tauri + Svelte Project

**Files:**
- Create: `package.json`, `pnpm-lock.yaml`, `vite.config.ts`, `svelte.config.js`, `tsconfig.json`
- Create: `src/main.ts`, `src/App.svelte`, `src/app.css`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Create Tauri + Svelte project**

Run:
```bash
cd /home/midwire/Code/claude-prompt-editor
pnpm create tauri-app . --template svelte-ts --manager pnpm
```

If the directory isn't empty (due to docs/), initialize in a temp dir and move files:
```bash
pnpm create tauri-app /tmp/cpe-scaffold --template svelte-ts --manager pnpm
# Copy only the scaffold files (not .git) into our project
rsync -a --exclude='.git' /tmp/cpe-scaffold/ .
rm -rf /tmp/cpe-scaffold
```

- [ ] **Step 2: Verify project builds**

Run:
```bash
pnpm install
pnpm tauri build --debug 2>&1 | tail -20
```
Expected: Build completes, produces a binary

- [ ] **Step 3: Configure Tauri app metadata**

Edit `src-tauri/tauri.conf.json`:
- Set `productName` to `"Claude Prompt Editor"`
- Set `identifier` to `"com.claude-prompt-editor.app"`
- Set window title to `"Claude Prompt Editor"`
- Set default window size to `1200x800`

- [ ] **Step 4: Verify app launches**

Run:
```bash
pnpm tauri dev
```
Expected: Window opens with default Svelte template content

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri + Svelte project"
```

---

### Task 2: Rust File I/O Commands

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/file.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/commands/file.rs` (inline tests)

- [ ] **Step 1: Write failing tests for file operations**

Create `src-tauri/src/commands/mod.rs`:
```rust
pub mod file;
```

Create `src-tauri/src/commands/file.rs`:
```rust
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromptFile {
    pub path: PathBuf,
    pub name: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptListEntry {
    pub path: PathBuf,
    pub name: String,
    pub modified: u64,
}

/// Read a prompt file from disk
pub fn read_prompt_file(path: &Path) -> Result<PromptFile, String> {
    todo!()
}

/// Write a prompt file to disk
pub fn write_prompt_file(path: &Path, content: &str) -> Result<(), String> {
    todo!()
}

/// List all .md files in a directory
pub fn list_prompt_files(dir: &Path) -> Result<Vec<PromptListEntry>, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn sample_prompt() -> &'static str {
        "---\nname: \"Test Prompt\"\nmodel: claude-opus-4-6\nversion: 1\n---\n\n<role>\nYou are a test assistant.\n</role>\n"
    }

    #[test]
    fn test_read_prompt_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.md");
        fs::write(&path, sample_prompt()).unwrap();

        let result = read_prompt_file(&path).unwrap();
        assert_eq!(result.name, "test");
        assert_eq!(result.content, sample_prompt());
        assert_eq!(result.path, path);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_prompt_file(Path::new("/nonexistent/file.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_write_prompt_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("output.md");

        write_prompt_file(&path, sample_prompt()).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, sample_prompt());
    }

    #[test]
    fn test_list_prompt_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.md"), "prompt a").unwrap();
        fs::write(dir.path().join("b.md"), "prompt b").unwrap();
        fs::write(dir.path().join("c.txt"), "not a prompt").unwrap();

        let entries = list_prompt_files(dir.path()).unwrap();
        assert_eq!(entries.len(), 2);

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }
}
```

- [ ] **Step 2: Add tempfile dev dependency**

Add to `src-tauri/Cargo.toml` under `[dev-dependencies]`:
```toml
tempfile = "3"
```

- [ ] **Step 3: Run tests to verify they fail**

Run:
```bash
cd src-tauri && cargo test commands::file -- --nocapture
```
Expected: FAIL - `todo!()` panics

- [ ] **Step 4: Implement file operations**

Replace the `todo!()` bodies in `src-tauri/src/commands/file.rs`:

```rust
pub fn read_prompt_file(path: &Path) -> Result<PromptFile, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();
    Ok(PromptFile {
        path: path.to_path_buf(),
        name,
        content,
    })
}

pub fn write_prompt_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    std::fs::write(path, content)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

pub fn list_prompt_files(dir: &Path) -> Result<Vec<PromptListEntry>, String> {
    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let metadata = entry.metadata().map_err(|e| format!("Failed to read metadata: {}", e))?;
            let modified = metadata
                .modified()
                .map_err(|e| format!("Failed to read modified time: {}", e))?
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string();
            entries.push(PromptListEntry { path, name, modified });
        }
    }
    entries.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(entries)
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run:
```bash
cd src-tauri && cargo test commands::file -- --nocapture
```
Expected: All 4 tests pass

- [ ] **Step 6: Wire up Tauri commands**

Update `src-tauri/src/lib.rs`:
```rust
mod commands;

use commands::file;
use std::path::PathBuf;

#[tauri::command]
fn open_prompt(path: String) -> Result<file::PromptFile, String> {
    file::read_prompt_file(&PathBuf::from(path))
}

#[tauri::command]
fn save_prompt(path: String, content: String) -> Result<(), String> {
    file::write_prompt_file(&PathBuf::from(path), &content)
}

#[tauri::command]
fn list_prompts(dir: String) -> Result<Vec<file::PromptListEntry>, String> {
    file::list_prompt_files(&PathBuf::from(dir))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_prompt,
            save_prompt,
            list_prompts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Verify build succeeds**

Run:
```bash
cd src-tauri && cargo build
```
Expected: Compiles without errors

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/commands/ src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: add file I/O commands (open, save, list prompts)"
```

---

### Task 3: Monaco Editor Integration (Source Mode)

**Files:**
- Modify: `package.json` (add monaco-editor dependency)
- Create: `src/lib/tauri.ts` (includes PromptFile/PromptListEntry types for now; `types.ts` with full AST types will be added in Chunk 3 when the structure editor needs them)
- Create: `src/lib/stores/files.ts`
- Create: `src/lib/stores/editor.ts`
- Create: `src/lib/components/Editor/SourceEditor.svelte`
- Create: `src/lib/components/StatusBar.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Install Monaco Editor**

Run:
```bash
pnpm add monaco-editor
```

- [ ] **Step 2: Create Tauri IPC wrapper**

Create `src/lib/tauri.ts`:
```typescript
import { invoke } from "@tauri-apps/api/core";

export interface PromptFile {
  path: string;
  name: string;
  content: string;
}

export interface PromptListEntry {
  path: string;
  name: string;
  modified: number;
}

export async function openPrompt(path: string): Promise<PromptFile> {
  return invoke("open_prompt", { path });
}

export async function savePrompt(path: string, content: string): Promise<void> {
  return invoke("save_prompt", { path, content });
}

export async function listPrompts(dir: string): Promise<PromptListEntry[]> {
  return invoke("list_prompts", { dir });
}
```

- [ ] **Step 3: Create editor store**

Create `src/lib/stores/editor.ts`:
```typescript
import { writable } from "svelte/store";

export type EditorMode = "source" | "structure";

export const editorMode = writable<EditorMode>("source");
export const currentContent = writable<string>("");
export const isDirty = writable<boolean>(false);
```

Create `src/lib/stores/files.ts`:
```typescript
import { writable } from "svelte/store";
import type { PromptFile } from "../tauri";

export const currentFile = writable<PromptFile | null>(null);
export const fileList = writable<string[]>([]);
```

- [ ] **Step 4: Create SourceEditor component**

Create `src/lib/components/Editor/SourceEditor.svelte`:
```svelte
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as monaco from "monaco-editor";
  import { currentContent, isDirty } from "../../stores/editor";

  let editorContainer: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor;

  // Register a custom language for prompt files
  function registerPromptLanguage() {
    monaco.languages.register({ id: "prompt-md" });
    monaco.languages.setMonarchTokensProvider("prompt-md", {
      tokenizer: {
        root: [
          [/^---$/, { token: "meta.frontmatter", next: "@frontmatter" }],
          [/<\/?[\w-]+>/, "tag"],
          [/\{\{[\w_]+\}\}/, "variable"],
          [/^#+\s.*$/, "keyword"],
          [/`.+?`/, "string"],
        ],
        frontmatter: [
          [/^---$/, { token: "meta.frontmatter", next: "@root" }],
          [/[\w_]+(?=:)/, "attribute.name"],
          [/:.*$/, "attribute.value"],
        ],
      },
    });
  }

  onMount(() => {
    registerPromptLanguage();
    editor = monaco.editor.create(editorContainer, {
      value: "",
      language: "prompt-md",
      theme: "vs-dark",
      minimap: { enabled: true },
      wordWrap: "on",
      fontSize: 14,
      lineNumbers: "on",
      renderWhitespace: "selection",
      automaticLayout: true,
    });

    editor.onDidChangeModelContent(() => {
      const value = editor.getValue();
      currentContent.set(value);
      isDirty.set(true);
    });

    // Subscribe to external content changes (e.g., file open)
    const unsubscribe = currentContent.subscribe((value) => {
      if (editor && editor.getValue() !== value) {
        editor.setValue(value);
      }
    });

    return () => {
      unsubscribe();
    };
  });

  onDestroy(() => {
    editor?.dispose();
  });
</script>

<div class="source-editor" bind:this={editorContainer}></div>

<style>
  .source-editor {
    width: 100%;
    height: 100%;
  }
</style>
```

- [ ] **Step 5: Create StatusBar component**

Create `src/lib/components/StatusBar.svelte`:
```svelte
<script lang="ts">
  import { currentContent, isDirty } from "../stores/editor";
  import { currentFile } from "../stores/files";

  $: charCount = $currentContent.length;
  $: tokenEstimate = Math.ceil(charCount / 4);
</script>

<div class="status-bar">
  <span class="file-name">
    {$currentFile?.name ?? "No file open"}
    {#if $isDirty}*{/if}
  </span>
  <span class="spacer"></span>
  <span class="token-count">~{tokenEstimate} tokens</span>
  <span class="char-count">{charCount} chars</span>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    padding: 4px 12px;
    background: var(--bg-darker, #1a1a2e);
    color: var(--text-secondary, #888);
    font-size: 12px;
    border-top: 1px solid var(--border, #333);
    gap: 16px;
  }
  .spacer {
    flex: 1;
  }
</style>
```

- [ ] **Step 6: Wire up App.svelte with file open/save**

Replace `src/App.svelte`:
```svelte
<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import SourceEditor from "./lib/components/Editor/SourceEditor.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import { currentContent, isDirty } from "./lib/stores/editor";
  import { currentFile } from "./lib/stores/files";
  import { openPrompt, savePrompt } from "./lib/tauri";

  async function handleOpen() {
    const selected = await open({
      filters: [{ name: "Prompt", extensions: ["md"] }],
    });
    if (selected) {
      const file = await openPrompt(selected as string);
      currentFile.set(file);
      currentContent.set(file.content);
      isDirty.set(false);
    }
  }

  async function handleSave() {
    const file = $currentFile;
    if (!file) return;
    await savePrompt(file.path, $currentContent);
    isDirty.set(false);
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

<svelte:window on:keydown={handleKeydown} />

<div class="app">
  <div class="toolbar">
    <button on:click={handleOpen}>Open (Ctrl+O)</button>
    <button on:click={handleSave} disabled={!$currentFile || !$isDirty}>
      Save (Ctrl+S)
    </button>
  </div>
  <div class="editor-area">
    <SourceEditor />
  </div>
  <StatusBar />
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: #1e1e2e;
    color: #cdd6f4;
    font-family: system-ui, -apple-system, sans-serif;
  }
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .toolbar {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
    background: #181825;
    border-bottom: 1px solid #313244;
  }
  .toolbar button {
    padding: 4px 12px;
    background: #313244;
    color: #cdd6f4;
    border: 1px solid #45475a;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }
  .toolbar button:hover {
    background: #45475a;
  }
  .toolbar button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .editor-area {
    flex: 1;
    overflow: hidden;
  }
</style>
```

- [ ] **Step 7: Add Tauri dialog plugin**

Run:
```bash
pnpm add @tauri-apps/plugin-dialog
```

Add to `src-tauri/Cargo.toml` dependencies:
```toml
tauri-plugin-dialog = "2"
```

Add plugin in `src-tauri/src/lib.rs` in the builder:
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(...)
```

- [ ] **Step 8: Configure Tauri capabilities for dialog plugin**

Tauri 2.0 requires explicit capability permissions. Add dialog permissions to `src-tauri/capabilities/default.json` (create the file if it doesn't exist):
```json
{
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:allow-open",
    "dialog:allow-save"
  ]
}
```

- [ ] **Step 9: Verify the app opens, edits, and saves a prompt file**

Run:
```bash
pnpm tauri dev
```
Manual test:
1. Create a test file: `echo '---\nname: test\n---\n<role>Test</role>' > /tmp/test-prompt.md`
2. Click "Open", select the file
3. Verify content appears in Monaco editor
4. Edit the content
5. Click "Save"
6. Verify the file on disk matches edits

Expected: Full open → edit → save cycle works

- [ ] **Step 10: Commit**

```bash
git add src/ package.json pnpm-lock.yaml
git commit -m "feat: add Monaco source editor with file open/save"
```

---

### Task 4: Basic MCP Server (load_prompt)

**Files:**
- Create: `src-tauri/src/mcp/mod.rs`
- Create: `src-tauri/src/mcp/server.rs`
- Create: `src-tauri/src/mcp/tools.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/mcp.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Test: `src-tauri/src/mcp/tools.rs` (inline tests)

- [ ] **Step 1: Add MCP dependencies**

Add to `src-tauri/Cargo.toml`:
```toml
rmcp = { version = "0.1", features = ["server", "transport-sse-server"] }
tokio = { version = "1", features = ["full"] }
axum = "0.7"
serde_json = "1"
```

Note: The spec requires SSE transport for the MCP server. If `rmcp` supports SSE server transport, use it. Otherwise, implement a minimal MCP-compatible SSE server using `axum` directly. The MCP protocol over SSE uses a `/sse` GET endpoint for the event stream and a `/message` POST endpoint for client-to-server messages. See the MCP specification for the exact SSE transport format. The implementation in Step 6 below uses a simplified HTTP POST approach — during implementation, verify Claude Code's expected transport and adjust accordingly.

- [ ] **Step 2: Write failing test for prompt rendering**

Create `src-tauri/src/mcp/mod.rs`:
```rust
pub mod server;
pub mod tools;
```

Create `src-tauri/src/mcp/tools.rs`:
```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::commands::file;

/// Shared state for the MCP server
#[derive(Debug, Clone)]
pub struct McpState {
    pub prompts_dir: PathBuf,
    pub variables: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
}

impl McpState {
    pub fn new(prompts_dir: PathBuf) -> Self {
        Self {
            prompts_dir,
            variables: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Interpolate {{variables}} in prompt content
pub fn interpolate_variables(content: &str, vars: &HashMap<String, String>) -> String {
    todo!()
}

/// Load and render a prompt by name
pub fn load_prompt(state: &McpState, name: &str) -> Result<String, String> {
    todo!()
}

/// List available prompts
pub fn list_prompts(state: &McpState) -> Result<Vec<file::PromptListEntry>, String> {
    file::list_prompt_files(&state.prompts_dir)
}

/// Set a variable value for a prompt
pub fn set_variable(state: &McpState, prompt_name: &str, key: &str, value: &str) {
    let mut vars = state.variables.write().unwrap();
    vars.entry(prompt_name.to_string())
        .or_default()
        .insert(key.to_string(), value.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_interpolate_variables_basic() {
        let mut vars = HashMap::new();
        vars.insert("language".to_string(), "Rust".to_string());
        vars.insert("level".to_string(), "expert".to_string());

        let input = "You are a {{language}} developer at {{level}} level.";
        let result = interpolate_variables(input, &vars);
        assert_eq!(result, "You are a Rust developer at expert level.");
    }

    #[test]
    fn test_interpolate_variables_missing_kept() {
        let vars = HashMap::new();
        let input = "Hello {{name}}, welcome!";
        let result = interpolate_variables(input, &vars);
        assert_eq!(result, "Hello {{name}}, welcome!");
    }

    #[test]
    fn test_load_prompt_basic() {
        let dir = TempDir::new().unwrap();
        let prompts_dir = dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(
            prompts_dir.join("test.md"),
            "---\nname: test\n---\n\n<role>\nYou are a {{language}} expert.\n</role>\n",
        )
        .unwrap();

        let state = McpState::new(prompts_dir);
        set_variable(&state, "test", "language", "Python");

        let result = load_prompt(&state, "test").unwrap();
        assert!(result.contains("You are a Python expert."));
        // Frontmatter should be stripped from the rendered output
        assert!(!result.contains("---"));
    }

    #[test]
    fn test_load_prompt_not_found() {
        let dir = TempDir::new().unwrap();
        let state = McpState::new(dir.path().to_path_buf());
        let result = load_prompt(&state, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_and_get_variable() {
        let dir = TempDir::new().unwrap();
        let state = McpState::new(dir.path().to_path_buf());

        set_variable(&state, "my-prompt", "lang", "Go");

        let vars = state.variables.read().unwrap();
        assert_eq!(vars["my-prompt"]["lang"], "Go");
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run:
```bash
cd src-tauri && cargo test mcp::tools -- --nocapture
```
Expected: FAIL - `todo!()` panics

- [ ] **Step 4: Implement interpolation and load_prompt**

Replace `todo!()` bodies in `src-tauri/src/mcp/tools.rs`:

```rust
pub fn interpolate_variables(content: &str, vars: &HashMap<String, String>) -> String {
    let mut result = content.to_string();
    for (key, value) in vars {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }
    result
}

pub fn load_prompt(state: &McpState, name: &str) -> Result<String, String> {
    let path = state.prompts_dir.join(format!("{}.md", name));
    let file = file::read_prompt_file(&path)?;

    // Strip frontmatter (everything between first --- and second ---)
    let body = strip_frontmatter(&file.content);

    // Interpolate variables
    let vars = state.variables.read().unwrap();
    let prompt_vars = vars.get(name).cloned().unwrap_or_default();
    Ok(interpolate_variables(&body, &prompt_vars))
}

fn strip_frontmatter(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return content.to_string();
    }
    // Find second ---
    if let Some(end) = trimmed[3..].find("\n---") {
        let after = &trimmed[3 + end + 4..]; // skip past \n---
        after.trim_start_matches('\n').to_string()
    } else {
        content.to_string()
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run:
```bash
cd src-tauri && cargo test mcp::tools -- --nocapture
```
Expected: All 5 tests pass

- [ ] **Step 6: Implement MCP SSE server**

Create `src-tauri/src/mcp/server.rs`:
```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;

use super::tools::McpState;

#[derive(Debug, Serialize)]
struct McpToolDefinition {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Debug, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    id: Option<Value>,
    result: Option<Value>,
    error: Option<McpError>,
}

#[derive(Debug, Serialize)]
struct McpError {
    code: i32,
    message: String,
}

struct AppState {
    mcp: McpState,
}

pub async fn start_server(mcp_state: McpState, port: u16) -> Result<u16, String> {
    let state = Arc::new(AppState { mcp: mcp_state });

    let app = Router::new()
        .route("/mcp", post(handle_mcp_request))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;
    let actual_port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    Ok(actual_port)
}

async fn handle_mcp_request(
    State(state): State<Arc<AppState>>,
    Json(req): Json<McpRequest>,
) -> Json<McpResponse> {
    let result = match req.method.as_str() {
        "initialize" => Ok(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "claude-prompt-editor",
                "version": "0.1.0"
            },
            "capabilities": {
                "tools": {}
            }
        })),
        "tools/list" => Ok(serde_json::json!({
            "tools": [
                {
                    "name": "load_prompt",
                    "description": "Load a prompt by name, fully rendered with variables interpolated",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Prompt name (filename without .md)" }
                        },
                        "required": ["name"]
                    }
                },
                {
                    "name": "list_prompts",
                    "description": "List all available prompts in the project",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "set_variable",
                    "description": "Set a variable value for a prompt",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "prompt_name": { "type": "string" },
                            "key": { "type": "string" },
                            "value": { "type": "string" }
                        },
                        "required": ["prompt_name", "key", "value"]
                    }
                }
            ]
        })),
        "tools/call" => handle_tool_call(&state.mcp, req.params),
        _ => Err(McpError {
            code: -32601,
            message: format!("Method not found: {}", req.method),
        }),
    };

    Json(match result {
        Ok(value) => McpResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: Some(value),
            error: None,
        },
        Err(err) => McpResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: None,
            error: Some(err),
        },
    })
}

fn handle_tool_call(state: &McpState, params: Option<Value>) -> Result<Value, McpError> {
    let params = params.ok_or(McpError {
        code: -32602,
        message: "Missing params".into(),
    })?;
    let tool_name = params["name"]
        .as_str()
        .ok_or(McpError {
            code: -32602,
            message: "Missing tool name".into(),
        })?;

    match tool_name {
        "load_prompt" => {
            let prompt_name = params["arguments"]["name"]
                .as_str()
                .ok_or(McpError {
                    code: -32602,
                    message: "Missing prompt name argument".into(),
                })?;
            let content = super::tools::load_prompt(state, prompt_name)
                .map_err(|e| McpError { code: -1, message: e })?;
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": content }]
            }))
        }
        "list_prompts" => {
            let prompts = super::tools::list_prompts(state)
                .map_err(|e| McpError { code: -1, message: e })?;
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": serde_json::to_string_pretty(&prompts).unwrap() }]
            }))
        }
        "set_variable" => {
            let prompt_name = params["arguments"]["prompt_name"]
                .as_str()
                .ok_or(McpError {
                    code: -32602,
                    message: "Missing prompt_name".into(),
                })?;
            let key = params["arguments"]["key"]
                .as_str()
                .ok_or(McpError {
                    code: -32602,
                    message: "Missing key".into(),
                })?;
            let value = params["arguments"]["value"]
                .as_str()
                .ok_or(McpError {
                    code: -32602,
                    message: "Missing value".into(),
                })?;
            super::tools::set_variable(state, prompt_name, key, value);
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": format!("Set {}={{}} for prompt {}", key, value, prompt_name) }]
            }))
        }
        _ => Err(McpError {
            code: -32602,
            message: format!("Unknown tool: {}", tool_name),
        }),
    }
}
```

- [ ] **Step 7: Add MCP Tauri command to start/stop server**

Create `src-tauri/src/commands/mcp.rs`:
```rust
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::mcp::tools::McpState;

/// Needs Clone + Arc wrapping for Tauri managed state shared across threads
#[derive(Clone)]
pub struct McpServerState {
    pub port: Arc<Mutex<Option<u16>>>,
    pub mcp_state: Arc<Mutex<Option<McpState>>>,
}

impl McpServerState {
    pub fn new() -> Self {
        Self {
            port: Arc::new(Mutex::new(None)),
            mcp_state: Arc::new(Mutex::new(None)),
        }
    }
}

pub async fn start_mcp(prompts_dir: PathBuf, port: u16) -> Result<u16, String> {
    let state = McpState::new(prompts_dir);
    crate::mcp::server::start_server(state, port).await
}
```

Update `src-tauri/src/commands/mod.rs`:
```rust
pub mod file;
pub mod mcp;
```

- [ ] **Step 8: Wire MCP into Tauri app startup**

Update `src-tauri/src/lib.rs` to start MCP server on launch:
```rust
mod commands;
mod mcp;

use commands::file;
use commands::mcp::McpServerState;
use std::path::PathBuf;

#[tauri::command]
fn open_prompt(path: String) -> Result<file::PromptFile, String> {
    file::read_prompt_file(&PathBuf::from(path))
}

#[tauri::command]
fn save_prompt(path: String, content: String) -> Result<(), String> {
    file::write_prompt_file(&PathBuf::from(path), &content)
}

#[tauri::command]
fn list_prompts(dir: String) -> Result<Vec<file::PromptListEntry>, String> {
    file::list_prompt_files(&PathBuf::from(dir))
}

#[tauri::command]
fn get_mcp_port(state: tauri::State<McpServerState>) -> Option<u16> {
    *state.port.lock().unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(McpServerState::new())
        .setup(|app| {
            // Start MCP server on a random available port
            let mcp_state = app.state::<McpServerState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                match commands::mcp::start_mcp(
                    PathBuf::from(".").join("prompts"),
                    0, // 0 = auto-select port
                )
                .await
                {
                    Ok(port) => {
                        *mcp_state.port.lock().unwrap() = Some(port);
                        println!("MCP server started on port {}", port);
                    }
                    Err(e) => {
                        eprintln!("Failed to start MCP server: {}", e);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_prompt,
            save_prompt,
            list_prompts,
            get_mcp_port,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 9: Verify build and MCP server responds**

Run:
```bash
cd src-tauri && cargo build
```
Expected: Compiles without errors

Then test MCP endpoint manually:
```bash
pnpm tauri dev &
sleep 3
# Find the port from stdout, then:
curl -X POST http://localhost:<PORT>/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```
Expected: JSON response listing the 3 tools

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/mcp/ src-tauri/src/commands/mcp.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: add MCP server with load_prompt, list_prompts, set_variable tools"
```

---

## Chunk 2: Prompt Parser & AST

This chunk builds the core prompt parsing engine: YAML frontmatter extraction, fault-tolerant XML tag parsing, AST types, variable interpolation, and AST-to-markdown serialization. This is the foundation for both the structure editor and the linter.

### Task 5: AST Types and Frontmatter Parser

**Files:**
- Create: `src-tauri/src/parser/mod.rs`
- Create: `src-tauri/src/parser/ast.rs`
- Create: `src-tauri/src/parser/frontmatter.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod parser`)
- Modify: `src-tauri/Cargo.toml` (add `serde_yaml`)

- [ ] **Step 1: Add serde_yaml dependency**

Add to `src-tauri/Cargo.toml`:
```toml
serde_yaml = "0.9"
```

- [ ] **Step 2: Define AST types**

Create `src-tauri/src/parser/mod.rs` (only declare modules that exist now; add others as they're created in later tasks):
```rust
pub mod ast;
pub mod frontmatter;
// pub mod serializer;  -- added in Task 8
// pub mod variables;   -- added in Task 7
// pub mod xml_tags;    -- added in Task 6
```

Create `src-tauri/src/parser/ast.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata from YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PromptMetadata {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub thinking: Option<ThinkingConfig>,
    #[serde(default)]
    pub effort: Option<String>,
    /// Catch-all for unknown fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub kind: String,
}

/// The kind of block in a prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockKind {
    Role,
    Instructions,
    Examples,
    Example,
    Context,
    Documents,
    Custom(String),
    Freeform,
}

/// A single block in the prompt AST
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub kind: BlockKind,
    pub tag_name: Option<String>,
    pub content: String,
    pub children: Vec<Block>,
    pub enabled: bool,
    /// Byte offset in source text where this block starts
    pub start_offset: usize,
    /// Byte offset in source text where this block ends
    pub end_offset: usize,
}

impl Block {
    pub fn new(kind: BlockKind, content: String, start: usize, end: usize) -> Self {
        Self {
            kind,
            tag_name: None,
            content,
            children: Vec::new(),
            enabled: true,
            start_offset: start,
            end_offset: end,
        }
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tag_name = Some(tag.to_string());
        self
    }
}

/// The full prompt AST
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptAst {
    pub metadata: PromptMetadata,
    pub blocks: Vec<Block>,
    /// Raw frontmatter string (preserved for round-tripping)
    pub raw_frontmatter: String,
}

impl PromptAst {
    pub fn new(metadata: PromptMetadata, blocks: Vec<Block>, raw_frontmatter: String) -> Self {
        Self {
            metadata,
            blocks,
            raw_frontmatter,
        }
    }

    /// Get all enabled blocks, flattened (including children)
    pub fn enabled_blocks(&self) -> Vec<&Block> {
        let mut result = Vec::new();
        for block in &self.blocks {
            if block.enabled {
                result.push(block);
                for child in &block.children {
                    if child.enabled {
                        result.push(child);
                    }
                }
            }
        }
        result
    }
}
```

- [ ] **Step 3: Write failing tests for frontmatter parser**

Create `src-tauri/src/parser/frontmatter.rs`:
```rust
use super::ast::PromptMetadata;

/// Result of splitting a prompt file into frontmatter and body
#[derive(Debug, PartialEq)]
pub struct SplitResult {
    pub raw_frontmatter: String,
    pub metadata: PromptMetadata,
    pub body: String,
}

/// Split a markdown prompt file into frontmatter metadata and body content.
/// Returns default metadata if no frontmatter is present.
pub fn parse_frontmatter(input: &str) -> Result<SplitResult, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_frontmatter() {
        let input = "---\nname: \"Test\"\nmodel: claude-opus-4-6\nversion: 2\ntags: [coding]\n---\n\n<role>\nHello\n</role>\n";
        let result = parse_frontmatter(input).unwrap();
        assert_eq!(result.metadata.name, "Test");
        assert_eq!(result.metadata.model, "claude-opus-4-6");
        assert_eq!(result.metadata.version, 2);
        assert_eq!(result.metadata.tags, vec!["coding"]);
        assert!(result.body.contains("<role>"));
        assert!(!result.body.contains("---"));
    }

    #[test]
    fn test_parse_without_frontmatter() {
        let input = "<role>\nJust a prompt with no frontmatter.\n</role>\n";
        let result = parse_frontmatter(input).unwrap();
        assert_eq!(result.metadata.name, "");
        assert_eq!(result.body, input);
        assert_eq!(result.raw_frontmatter, "");
    }

    #[test]
    fn test_parse_with_thinking_config() {
        let input = "---\nname: test\nthinking:\n  type: adaptive\neffort: high\n---\n\nBody here\n";
        let result = parse_frontmatter(input).unwrap();
        assert_eq!(result.metadata.thinking.as_ref().unwrap().kind, "adaptive");
        assert_eq!(result.metadata.effort.as_deref(), Some("high"));
    }

    #[test]
    fn test_parse_with_extra_fields() {
        let input = "---\nname: test\ncustom_field: value\n---\n\nBody\n";
        let result = parse_frontmatter(input).unwrap();
        assert!(result.metadata.extra.contains_key("custom_field"));
    }

    #[test]
    fn test_roundtrip_raw_frontmatter() {
        let fm = "name: \"Test\"\nmodel: claude-opus-4-6\nversion: 1";
        let input = format!("---\n{}\n---\n\nBody\n", fm);
        let result = parse_frontmatter(&input).unwrap();
        assert_eq!(result.raw_frontmatter, fm);
    }
}
```

- [ ] **Step 4: Run tests to verify they fail**

Run:
```bash
cd src-tauri && cargo test parser::frontmatter -- --nocapture
```
Expected: FAIL - `todo!()` panics

- [ ] **Step 5: Implement frontmatter parser**

Replace `todo!()` in `src-tauri/src/parser/frontmatter.rs`:
```rust
pub fn parse_frontmatter(input: &str) -> Result<SplitResult, String> {
    let trimmed = input.trim_start();

    if !trimmed.starts_with("---") {
        return Ok(SplitResult {
            raw_frontmatter: String::new(),
            metadata: PromptMetadata::default(),
            body: input.to_string(),
        });
    }

    // Find the end of frontmatter (second ---)
    let after_first = &trimmed[3..];
    let after_first = after_first.strip_prefix('\n').unwrap_or(after_first);

    let end_pos = after_first
        .find("\n---")
        .ok_or_else(|| "Unclosed frontmatter: missing closing ---".to_string())?;

    let raw_frontmatter = after_first[..end_pos].to_string();
    let body_start = end_pos + 4; // skip \n---
    let body = after_first[body_start..]
        .strip_prefix('\n')
        .unwrap_or(&after_first[body_start..])
        .to_string();

    let metadata: PromptMetadata = serde_yaml::from_str(&raw_frontmatter)
        .map_err(|e| format!("Invalid frontmatter YAML: {}", e))?;

    Ok(SplitResult {
        raw_frontmatter,
        metadata,
        body,
    })
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run:
```bash
cd src-tauri && cargo test parser::frontmatter -- --nocapture
```
Expected: All 5 tests pass

- [ ] **Step 7: Add `mod parser` to lib.rs**

Add to `src-tauri/src/lib.rs`:
```rust
mod parser;
```

- [ ] **Step 8: Verify full build**

Run:
```bash
cd src-tauri && cargo build
```
Expected: Compiles without errors

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/parser/ src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: add AST types and frontmatter parser"
```

---

### Task 6: Fault-Tolerant XML Tag Parser

**Files:**
- Create: `src-tauri/src/parser/xml_tags.rs`
- Test: inline in `xml_tags.rs`

- [ ] **Step 1: Write failing tests for XML tag extraction**

Create `src-tauri/src/parser/xml_tags.rs`:
```rust
use super::ast::{Block, BlockKind};

/// A raw tag found in the text
#[derive(Debug, PartialEq)]
pub struct TagSpan {
    pub name: String,
    pub is_closing: bool,
    pub byte_start: usize,
    pub byte_end: usize,
}

/// Find all XML-style tags in the input text.
/// This is NOT a full XML parser — it finds <tagname> and </tagname> patterns
/// at the start of lines (fault-tolerant, ignores tags inside code blocks).
pub fn find_tags(input: &str) -> Vec<TagSpan> {
    todo!()
}

/// Map a tag name to a BlockKind
pub fn tag_to_block_kind(tag: &str) -> BlockKind {
    match tag {
        "role" => BlockKind::Role,
        "instructions" => BlockKind::Instructions,
        "examples" => BlockKind::Examples,
        "example" => BlockKind::Example,
        "context" => BlockKind::Context,
        "documents" => BlockKind::Documents,
        _ => BlockKind::Custom(tag.to_string()),
    }
}

/// Parse body text into a list of Blocks by extracting XML-tagged sections.
/// Text between tagged sections becomes Freeform blocks.
/// Unmatched tags are treated as freeform text (fault-tolerant).
pub fn parse_blocks(body: &str) -> Vec<Block> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_tags_basic() {
        let input = "<role>\nYou are helpful.\n</role>\n";
        let tags = find_tags(input);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "role");
        assert!(!tags[0].is_closing);
        assert_eq!(tags[1].name, "role");
        assert!(tags[1].is_closing);
    }

    #[test]
    fn test_find_tags_nested() {
        let input = "<examples>\n<example>\nfoo\n</example>\n</examples>\n";
        let tags = find_tags(input);
        assert_eq!(tags.len(), 4);
        assert_eq!(tags[0].name, "examples");
        assert_eq!(tags[1].name, "example");
        assert_eq!(tags[2].name, "example");
        assert!(tags[2].is_closing);
        assert_eq!(tags[3].name, "examples");
        assert!(tags[3].is_closing);
    }

    #[test]
    fn test_find_tags_ignores_code_blocks() {
        let input = "```\n<fake_tag>\nnot a real tag\n</fake_tag>\n```\n<role>\nreal\n</role>\n";
        let tags = find_tags(input);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "role");
    }

    #[test]
    fn test_parse_blocks_simple() {
        let input = "<role>\nYou are a helper.\n</role>\n";
        let blocks = parse_blocks(input);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].kind, BlockKind::Role);
        assert!(blocks[0].content.contains("You are a helper."));
    }

    #[test]
    fn test_parse_blocks_with_freeform() {
        let input = "Some intro text.\n\n<role>\nHelper.\n</role>\n\nSome outro.\n";
        let blocks = parse_blocks(input);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].kind, BlockKind::Freeform);
        assert_eq!(blocks[1].kind, BlockKind::Role);
        assert_eq!(blocks[2].kind, BlockKind::Freeform);
    }

    #[test]
    fn test_parse_blocks_nested_examples() {
        let input = "<examples>\n<example>\nEx 1\n</example>\n<example>\nEx 2\n</example>\n</examples>\n";
        let blocks = parse_blocks(input);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].kind, BlockKind::Examples);
        assert_eq!(blocks[0].children.len(), 2);
        assert_eq!(blocks[0].children[0].kind, BlockKind::Example);
        assert_eq!(blocks[0].children[1].kind, BlockKind::Example);
    }

    #[test]
    fn test_parse_blocks_unmatched_tag_becomes_freeform() {
        let input = "<role>\nNo closing tag here.\n";
        let blocks = parse_blocks(input);
        // Should not crash; unmatched tag becomes freeform
        assert!(!blocks.is_empty());
        assert_eq!(blocks[0].kind, BlockKind::Freeform);
    }

    #[test]
    fn test_tag_to_block_kind() {
        assert_eq!(tag_to_block_kind("role"), BlockKind::Role);
        assert_eq!(tag_to_block_kind("instructions"), BlockKind::Instructions);
        assert_eq!(
            tag_to_block_kind("my_custom"),
            BlockKind::Custom("my_custom".to_string())
        );
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```bash
cd src-tauri && cargo test parser::xml_tags -- --nocapture
```
Expected: FAIL - `todo!()` panics

- [ ] **Step 3: Implement find_tags**

Replace `find_tags` `todo!()`:
```rust
pub fn find_tags(input: &str) -> Vec<TagSpan> {
    let mut tags = Vec::new();
    let mut in_code_block = false;
    let mut pos = 0;

    for line in input.lines() {
        let line_start = pos;
        let trimmed = line.trim();

        // Track code blocks
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            pos += line.len() + 1; // +1 for newline
            continue;
        }

        if !in_code_block {
            // Match opening tag: <tagname> or <tagname attr="...">
            if let Some(tag) = parse_tag_from_line(trimmed) {
                let byte_start = line_start + line.find('<').unwrap_or(0);
                let byte_end = line_start + line.rfind('>').map(|p| p + 1).unwrap_or(line.len());
                tags.push(TagSpan {
                    name: tag.0,
                    is_closing: tag.1,
                    byte_start,
                    byte_end,
                });
            }
        }

        pos += line.len() + 1;
    }

    tags
}

fn parse_tag_from_line(line: &str) -> Option<(String, bool)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('<') {
        return None;
    }

    // Find the first '>' to extract the tag (handles inline content like <input>text</input>)
    let close_bracket = trimmed.find('>')?;
    let inner = &trimmed[1..close_bracket];
    if inner.is_empty() {
        return None;
    }

    let is_closing = inner.starts_with('/');
    let name_part = if is_closing { &inner[1..] } else { inner };

    // Extract tag name (stop at space for attributes)
    let name: String = name_part
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();

    if name.is_empty() {
        return None;
    }

    Some((name, is_closing))
}
```

- [ ] **Step 4: Implement parse_blocks**

Replace `parse_blocks` `todo!()`:
```rust
pub fn parse_blocks(body: &str) -> Vec<Block> {
    let tags = find_tags(body);
    let mut blocks = Vec::new();
    let mut cursor = 0;

    let mut i = 0;
    while i < tags.len() {
        let open_tag = &tags[i];

        // If there's text before this tag, add as freeform
        if open_tag.byte_start > cursor {
            let text = body[cursor..open_tag.byte_start].trim().to_string();
            if !text.is_empty() {
                blocks.push(Block::new(
                    BlockKind::Freeform,
                    text,
                    cursor,
                    open_tag.byte_start,
                ));
            }
        }

        if open_tag.is_closing {
            // Stray closing tag — treat as freeform
            let text = body[open_tag.byte_start..open_tag.byte_end].to_string();
            blocks.push(Block::new(
                BlockKind::Freeform,
                text,
                open_tag.byte_start,
                open_tag.byte_end,
            ));
            cursor = open_tag.byte_end;
            if cursor < body.len() && body.as_bytes().get(cursor) == Some(&b'\n') {
                cursor += 1;
            }
            i += 1;
            continue;
        }

        // Find matching closing tag
        if let Some(close_idx) = find_closing_tag(&tags, i) {
            let close_tag = &tags[close_idx];
            let content_start = open_tag.byte_end;
            let content_start = if content_start < body.len()
                && body.as_bytes().get(content_start) == Some(&b'\n')
            {
                content_start + 1
            } else {
                content_start
            };
            let content_end = close_tag.byte_start;
            let content = body[content_start..content_end].trim_end().to_string();

            let kind = tag_to_block_kind(&open_tag.name);
            let mut block =
                Block::new(kind, content.clone(), open_tag.byte_start, close_tag.byte_end)
                    .with_tag(&open_tag.name);

            // Parse children for container tags (examples, documents)
            if open_tag.name == "examples" || open_tag.name == "documents" {
                let inner = &body[content_start..content_end];
                block.children = parse_blocks(inner);
                // Adjust child offsets relative to parent
                for child in &mut block.children {
                    child.start_offset += content_start;
                    child.end_offset += content_start;
                }
            }

            blocks.push(block);
            cursor = close_tag.byte_end;
            if cursor < body.len() && body.as_bytes().get(cursor) == Some(&b'\n') {
                cursor += 1;
            }
            i = close_idx + 1;
        } else {
            // No matching close tag — treat entire rest as freeform
            let text = body[open_tag.byte_start..].trim().to_string();
            blocks.push(Block::new(
                BlockKind::Freeform,
                text,
                open_tag.byte_start,
                body.len(),
            ));
            cursor = body.len();
            break;
        }
    }

    // Remaining text after last tag
    if cursor < body.len() {
        let text = body[cursor..].trim().to_string();
        if !text.is_empty() {
            blocks.push(Block::new(BlockKind::Freeform, text, cursor, body.len()));
        }
    }

    blocks
}

/// Find the index of the matching closing tag for the opening tag at `open_idx`.
/// Handles nesting of same-named tags.
fn find_closing_tag(tags: &[TagSpan], open_idx: usize) -> Option<usize> {
    let open_name = &tags[open_idx].name;
    let mut depth = 1;
    for j in (open_idx + 1)..tags.len() {
        if tags[j].name == *open_name {
            if tags[j].is_closing {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            } else {
                depth += 1;
            }
        }
    }
    None
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run:
```bash
cd src-tauri && cargo test parser::xml_tags -- --nocapture
```
Expected: All 7 tests pass

- [ ] **Step 6: Add module declaration**

Add `pub mod xml_tags;` to `src-tauri/src/parser/mod.rs` (uncomment it from the placeholder).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/parser/xml_tags.rs src-tauri/src/parser/mod.rs
git commit -m "feat: add fault-tolerant XML tag parser with block extraction"
```

---

### Task 7: Variable Interpolation (Refactored)

**Files:**
- Create: `src-tauri/src/parser/variables.rs`
- Modify: `src-tauri/src/mcp/tools.rs` (reuse parser's interpolation)
- Test: inline in `variables.rs`

- [ ] **Step 1: Write failing tests**

Create `src-tauri/src/parser/variables.rs`:
```rust
use std::collections::HashMap;

/// Extract all variable names ({{name}}) from prompt text
pub fn extract_variable_names(input: &str) -> Vec<String> {
    todo!()
}

/// Interpolate {{variables}} with provided values.
/// Variables not in the map are left as-is.
pub fn interpolate(input: &str, vars: &HashMap<String, String>) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_variable_names() {
        let input = "Hello {{name}}, you are a {{role}} using {{tool}}.";
        let vars = extract_variable_names(input);
        assert_eq!(vars, vec!["name", "role", "tool"]);
    }

    #[test]
    fn test_extract_no_variables() {
        let vars = extract_variable_names("No variables here.");
        assert!(vars.is_empty());
    }

    #[test]
    fn test_extract_duplicate_variables() {
        let input = "{{x}} and {{x}} again";
        let vars = extract_variable_names(input);
        assert_eq!(vars, vec!["x"]);
    }

    #[test]
    fn test_interpolate_basic() {
        let mut vars = HashMap::new();
        vars.insert("lang".to_string(), "Rust".to_string());
        let result = interpolate("Write {{lang}} code.", &vars);
        assert_eq!(result, "Write Rust code.");
    }

    #[test]
    fn test_interpolate_missing_preserved() {
        let vars = HashMap::new();
        let result = interpolate("Hello {{name}}!", &vars);
        assert_eq!(result, "Hello {{name}}!");
    }

    #[test]
    fn test_interpolate_multiple() {
        let mut vars = HashMap::new();
        vars.insert("a".to_string(), "1".to_string());
        vars.insert("b".to_string(), "2".to_string());
        let result = interpolate("{{a}} + {{b}} = 3", &vars);
        assert_eq!(result, "1 + 2 = 3");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```bash
cd src-tauri && cargo test parser::variables -- --nocapture
```
Expected: FAIL

- [ ] **Step 3: Implement variable functions**

Replace `todo!()` bodies:
```rust
pub fn extract_variable_names(input: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut rest = input;

    while let Some(start) = rest.find("{{") {
        let after_open = &rest[start + 2..];
        if let Some(end) = after_open.find("}}") {
            let name = after_open[..end].trim().to_string();
            if !name.is_empty() && seen.insert(name.clone()) {
                names.push(name);
            }
            rest = &after_open[end + 2..];
        } else {
            break;
        }
    }

    names
}

pub fn interpolate(input: &str, vars: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    for (key, value) in vars {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }
    result
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```bash
cd src-tauri && cargo test parser::variables -- --nocapture
```
Expected: All 6 tests pass

- [ ] **Step 5: Refactor MCP tools to use parser's interpolation**

Update `src-tauri/src/mcp/tools.rs` to replace its local `interpolate_variables` with a call to the parser module:

Replace the `interpolate_variables` function body:
```rust
pub fn interpolate_variables(content: &str, vars: &HashMap<String, String>) -> String {
    crate::parser::variables::interpolate(content, vars)
}
```

- [ ] **Step 6: Run all tests to verify nothing broke**

Run:
```bash
cd src-tauri && cargo test -- --nocapture
```
Expected: All tests pass

- [ ] **Step 7: Add module declaration**

Add `pub mod variables;` to `src-tauri/src/parser/mod.rs` (uncomment it).

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/parser/variables.rs src-tauri/src/parser/mod.rs src-tauri/src/mcp/tools.rs
git commit -m "feat: add variable extraction and interpolation, refactor MCP to reuse"
```

---

### Task 8: AST Serializer (AST → Markdown)

**Files:**
- Create: `src-tauri/src/parser/serializer.rs`
- Test: inline in `serializer.rs`

- [ ] **Step 1: Write failing tests**

Create `src-tauri/src/parser/serializer.rs`:
```rust
use super::ast::{Block, BlockKind, PromptAst};

/// Serialize a PromptAst back to a markdown string.
/// This is the inverse of parsing — used when structure mode edits need
/// to be reflected back to the source text.
pub fn serialize(ast: &PromptAst) -> String {
    todo!()
}

/// Serialize a single block to markdown text
fn serialize_block(block: &Block) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::PromptMetadata;

    fn make_block(kind: BlockKind, tag: Option<&str>, content: &str) -> Block {
        let mut block = Block::new(kind, content.to_string(), 0, 0);
        if let Some(t) = tag {
            block = block.with_tag(t);
        }
        block
    }

    #[test]
    fn test_serialize_simple_prompt() {
        let ast = PromptAst::new(
            PromptMetadata {
                name: "test".into(),
                model: "claude-opus-4-6".into(),
                version: 1,
                ..Default::default()
            },
            vec![make_block(BlockKind::Role, Some("role"), "You are helpful.")],
            "name: \"test\"\nmodel: claude-opus-4-6\nversion: 1".into(),
        );

        let result = serialize(&ast);
        assert!(result.starts_with("---\n"));
        assert!(result.contains("name: \"test\""));
        assert!(result.contains("<role>\nYou are helpful.\n</role>"));
    }

    #[test]
    fn test_serialize_freeform_block() {
        let ast = PromptAst::new(
            PromptMetadata::default(),
            vec![make_block(BlockKind::Freeform, None, "Just some text.")],
            "".into(),
        );

        let result = serialize(&ast);
        assert!(result.contains("Just some text."));
        // Freeform blocks should NOT be wrapped in tags
        assert!(!result.contains('<'));
    }

    #[test]
    fn test_serialize_disabled_block_as_comment() {
        let mut block = make_block(BlockKind::Role, Some("role"), "I am hidden.");
        block.enabled = false;

        let ast = PromptAst::new(PromptMetadata::default(), vec![block], "".into());

        let result = serialize(&ast);
        // Disabled blocks should be preserved as HTML comments
        assert!(result.contains("<!-- disabled"));
        assert!(result.contains("I am hidden."));
    }

    #[test]
    fn test_serialize_nested_examples() {
        let mut examples = make_block(BlockKind::Examples, Some("examples"), "");
        examples.children = vec![
            make_block(BlockKind::Example, Some("example"), "Example 1"),
            make_block(BlockKind::Example, Some("example"), "Example 2"),
        ];

        let ast = PromptAst::new(PromptMetadata::default(), vec![examples], "".into());

        let result = serialize(&ast);
        assert!(result.contains("<examples>"));
        assert!(result.contains("<example>\nExample 1\n</example>"));
        assert!(result.contains("<example>\nExample 2\n</example>"));
        assert!(result.contains("</examples>"));
    }

    #[test]
    fn test_serialize_empty_frontmatter() {
        let ast = PromptAst::new(
            PromptMetadata::default(),
            vec![make_block(BlockKind::Freeform, None, "Hello")],
            "".into(),
        );

        let result = serialize(&ast);
        // Empty frontmatter should not produce ---\n---
        assert!(!result.starts_with("---"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```bash
cd src-tauri && cargo test parser::serializer -- --nocapture
```
Expected: FAIL

- [ ] **Step 3: Implement serializer**

Replace `todo!()` bodies:
```rust
pub fn serialize(ast: &PromptAst) -> String {
    let mut output = String::new();

    // Frontmatter
    if !ast.raw_frontmatter.is_empty() {
        output.push_str("---\n");
        output.push_str(&ast.raw_frontmatter);
        output.push_str("\n---\n\n");
    }

    // Blocks
    for (i, block) in ast.blocks.iter().enumerate() {
        output.push_str(&serialize_block(block));
        if i < ast.blocks.len() - 1 {
            output.push('\n');
        }
    }

    output
}

fn serialize_block(block: &Block) -> String {
    if !block.enabled {
        // Disabled blocks are preserved as HTML comments
        return if let Some(ref tag) = block.tag_name {
            format!(
                "<!-- disabled: <{}>\n{}\n</{}> -->\n",
                tag, block.content, tag
            )
        } else {
            format!("<!-- disabled\n{}\n-->\n", block.content)
        };
    }

    match (&block.kind, &block.tag_name) {
        (BlockKind::Freeform, _) => {
            format!("{}\n", block.content)
        }
        (_, Some(tag)) => {
            if block.children.is_empty() {
                format!("<{}>\n{}\n</{}>\n", tag, block.content, tag)
            } else {
                let mut inner = String::new();
                for child in &block.children {
                    inner.push_str(&serialize_block(child));
                }
                format!("<{}>\n{}</{}>\n", tag, inner, tag)
            }
        }
        (_, None) => {
            format!("{}\n", block.content)
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```bash
cd src-tauri && cargo test parser::serializer -- --nocapture
```
Expected: All 5 tests pass

- [ ] **Step 5: Add module declaration**

Add `pub mod serializer;` to `src-tauri/src/parser/mod.rs` (uncomment it).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/parser/serializer.rs src-tauri/src/parser/mod.rs
git commit -m "feat: add AST-to-markdown serializer with disabled block support"
```

---

### Task 9: Full Parse Pipeline & Tauri Command

**Files:**
- Modify: `src-tauri/src/parser/mod.rs` (add top-level `parse` function)
- Create: `src-tauri/src/commands/prompt.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs` (register new commands)

- [ ] **Step 1: Write failing test for full parse pipeline**

Add to `src-tauri/src/parser/mod.rs`:
```rust
pub mod ast;
pub mod frontmatter;
pub mod serializer;
pub mod variables;
pub mod xml_tags;

use ast::PromptAst;

/// Parse a complete prompt file (frontmatter + body) into a PromptAst
pub fn parse(input: &str) -> Result<PromptAst, String> {
    let split = frontmatter::parse_frontmatter(input)?;
    let blocks = xml_tags::parse_blocks(&split.body);
    Ok(PromptAst::new(split.metadata, blocks, split.raw_frontmatter))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_prompt() {
        let input = r#"---
name: "Code Review"
model: claude-opus-4-6
version: 1
tags: [coding]
---

<role>
You are an expert code reviewer specializing in {{language}}.
</role>

<instructions>
Review the provided code for:
1. Security vulnerabilities
2. Performance issues
</instructions>

<examples>
<example>
Input: eval(user_input)
Output: CRITICAL security issue
</example>
</examples>
"#;
        let ast = parse(input).unwrap();
        assert_eq!(ast.metadata.name, "Code Review");
        assert_eq!(ast.metadata.model, "claude-opus-4-6");
        assert_eq!(ast.blocks.len(), 3); // role, instructions, examples
        assert_eq!(ast.blocks[0].kind, ast::BlockKind::Role);
        assert_eq!(ast.blocks[1].kind, ast::BlockKind::Instructions);
        assert_eq!(ast.blocks[2].kind, ast::BlockKind::Examples);
        assert_eq!(ast.blocks[2].children.len(), 1);
    }

    #[test]
    fn test_parse_roundtrip() {
        let input = "---\nname: test\nversion: 1\n---\n\n<role>\nHelper.\n</role>\n";
        let ast = parse(input).unwrap();
        let output = serializer::serialize(&ast);
        let ast2 = parse(&output).unwrap();
        assert_eq!(ast.metadata.name, ast2.metadata.name);
        assert_eq!(ast.blocks.len(), ast2.blocks.len());
        assert_eq!(ast.blocks[0].kind, ast2.blocks[0].kind);
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run:
```bash
cd src-tauri && cargo test parser::tests -- --nocapture
```
Expected: Both tests pass (implementation is already in place)

- [ ] **Step 3: Create prompt command for Tauri IPC**

Create `src-tauri/src/commands/prompt.rs`:
```rust
use crate::parser;
use crate::parser::ast::PromptAst;
use crate::parser::serializer;
use std::path::PathBuf;

/// Parse a prompt file and return its AST
#[tauri::command]
pub fn parse_prompt(path: String) -> Result<PromptAst, String> {
    let file = super::file::read_prompt_file(&PathBuf::from(&path))?;
    parser::parse(&file.content)
}

/// Parse raw content string into AST (for live editing)
#[tauri::command]
pub fn parse_content(content: String) -> Result<PromptAst, String> {
    parser::parse(&content)
}

/// Serialize an AST back to markdown string
#[tauri::command]
pub fn serialize_ast(ast: PromptAst) -> String {
    serializer::serialize(&ast)
}
```

- [ ] **Step 4: Register new commands**

Update `src-tauri/src/commands/mod.rs`:
```rust
pub mod file;
pub mod mcp;
pub mod prompt;
```

Update `src-tauri/src/lib.rs` invoke_handler to add:
```rust
.invoke_handler(tauri::generate_handler![
    open_prompt,
    save_prompt,
    list_prompts,
    get_mcp_port,
    commands::prompt::parse_prompt,
    commands::prompt::parse_content,
    commands::prompt::serialize_ast,
])
```

- [ ] **Step 5: Verify build**

Run:
```bash
cd src-tauri && cargo build
```
Expected: Compiles without errors

- [ ] **Step 6: Run all tests**

Run:
```bash
cd src-tauri && cargo test -- --nocapture
```
Expected: All tests pass

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/parser/mod.rs src-tauri/src/commands/prompt.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add full parse pipeline and Tauri IPC commands for AST operations"
```

---

## Chunk 3: Dual-Mode Editor

This chunk builds the structure mode (block editor), the mode toggle, TypeScript AST types, and the sync mechanism between source and structure modes.

### Task 10: TypeScript AST Types and Tauri IPC Extensions

**Files:**
- Create: `src/lib/types.ts`
- Modify: `src/lib/tauri.ts` (add AST-related IPC calls)
- Create: `src/lib/stores/prompt.ts`

- [ ] **Step 1: Create TypeScript types mirroring Rust AST**

Create `src/lib/types.ts`:
```typescript
export interface ThinkingConfig {
  type: string;
}

export interface PromptMetadata {
  name: string;
  model: string;
  version: number;
  tags: string[];
  thinking?: ThinkingConfig;
  effort?: string;
  extra: Record<string, unknown>;
}

export type BlockKind =
  | "Role"
  | "Instructions"
  | "Examples"
  | "Example"
  | "Context"
  | "Documents"
  | "Freeform"
  | { Custom: string };

export interface Block {
  kind: BlockKind;
  tag_name: string | null;
  content: string;
  children: Block[];
  enabled: boolean;
  start_offset: number;
  end_offset: number;
}

export interface PromptAst {
  metadata: PromptMetadata;
  blocks: Block[];
  raw_frontmatter: string;
}

/** Get a human-readable label for a block kind */
export function blockKindLabel(kind: BlockKind): string {
  if (typeof kind === "string") return kind;
  return kind.Custom;
}

/** Get the icon for a block kind */
export function blockKindIcon(kind: BlockKind): string {
  const k = typeof kind === "string" ? kind : "Custom";
  const icons: Record<string, string> = {
    Role: "👤",
    Instructions: "📋",
    Examples: "📝",
    Example: "📄",
    Context: "📎",
    Documents: "📂",
    Freeform: "✏️",
    Custom: "🏷️",
  };
  return icons[k] ?? "🏷️";
}
```

- [ ] **Step 2: Add AST IPC calls to tauri.ts**

Add to `src/lib/tauri.ts`:
```typescript
import type { PromptAst } from "./types";

export async function parseContent(content: string): Promise<PromptAst> {
  return invoke("parse_content", { content });
}

export async function serializeAst(ast: PromptAst): Promise<string> {
  return invoke("serialize_ast", { ast });
}
```

- [ ] **Step 3: Create prompt store**

Create `src/lib/stores/prompt.ts`:
```typescript
import { writable, derived, get } from "svelte/store";
import type { PromptAst, Block } from "../types";
import { parseContent, serializeAst } from "../tauri";
import { currentContent, isDirty } from "./editor";

export const currentAst = writable<PromptAst | null>(null);
export const parseError = writable<string | null>(null);

let debounceTimer: ReturnType<typeof setTimeout>;

/** Parse content string into AST (debounced) */
export function parseFromContent(content: string) {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(async () => {
    try {
      const ast = await parseContent(content);
      currentAst.set(ast);
      parseError.set(null);
    } catch (e) {
      parseError.set(String(e));
    }
  }, 300);
}

/** Serialize AST back to content string (for structure mode edits) */
export async function syncAstToContent(ast: PromptAst) {
  const content = await serializeAst(ast);
  currentContent.set(content);
  currentAst.set(ast);
  isDirty.set(true);
}
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/types.ts src/lib/tauri.ts src/lib/stores/prompt.ts
git commit -m "feat: add TypeScript AST types, IPC extensions, and prompt store"
```

---

### Task 11: Structure Mode Block Editor

**Files:**
- Create: `src/lib/components/Blocks/Block.svelte`
- Create: `src/lib/components/Blocks/MetadataBlock.svelte`
- Create: `src/lib/components/Blocks/TaggedBlock.svelte`
- Create: `src/lib/components/Blocks/FreeformBlock.svelte`
- Create: `src/lib/components/Blocks/ExamplesBlock.svelte`
- Create: `src/lib/components/Editor/StructureEditor.svelte`

- [ ] **Step 1: Create base Block wrapper component**

Create `src/lib/components/Blocks/Block.svelte`:
```svelte
<script lang="ts">
  import type { Block as BlockType } from "../../types";
  import { blockKindLabel, blockKindIcon } from "../../types";
  import { createEventDispatcher } from "svelte";

  export let block: BlockType;
  export let index: number;

  const dispatch = createEventDispatcher();
  let collapsed = false;

  function toggleEnabled() {
    dispatch("toggle", { index, enabled: !block.enabled });
  }

  function toggleCollapse() {
    collapsed = !collapsed;
  }

  function handleDragStart(e: DragEvent) {
    e.dataTransfer?.setData("text/plain", String(index));
  }
</script>

<div
  class="block"
  class:disabled={!block.enabled}
  draggable="true"
  on:dragstart={handleDragStart}
>
  <div class="block-header">
    <span class="drag-handle" title="Drag to reorder">⠿</span>
    <span class="block-icon">{blockKindIcon(block.kind)}</span>
    <span class="block-label">{block.tag_name ?? blockKindLabel(block.kind)}</span>
    <span class="spacer"></span>
    <button class="toggle-btn" on:click={toggleCollapse} title={collapsed ? "Expand" : "Collapse"}>
      {collapsed ? "▸" : "▾"}
    </button>
    <button class="toggle-btn" on:click={toggleEnabled} title={block.enabled ? "Disable" : "Enable"}>
      {block.enabled ? "●" : "○"}
    </button>
  </div>
  {#if !collapsed}
    <div class="block-body">
      <slot />
    </div>
  {/if}
</div>

<style>
  .block {
    border: 1px solid var(--border, #313244);
    border-radius: 6px;
    margin-bottom: 8px;
    background: var(--bg-block, #1e1e2e);
    overflow: hidden;
  }
  .block.disabled {
    opacity: 0.5;
  }
  .block-header {
    display: flex;
    align-items: center;
    padding: 6px 10px;
    background: var(--bg-block-header, #181825);
    gap: 8px;
    font-size: 13px;
    cursor: pointer;
  }
  .drag-handle {
    cursor: grab;
    opacity: 0.5;
  }
  .block-icon {
    font-size: 14px;
  }
  .block-label {
    font-weight: 600;
    text-transform: capitalize;
  }
  .spacer { flex: 1; }
  .toggle-btn {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 2px 4px;
    opacity: 0.6;
  }
  .toggle-btn:hover { opacity: 1; }
  .block-body {
    padding: 10px;
  }
</style>
```

- [ ] **Step 2: Create TaggedBlock for role, instructions, context, custom blocks**

Create `src/lib/components/Blocks/TaggedBlock.svelte`:
```svelte
<script lang="ts">
  import type { Block as BlockType } from "../../types";
  import { createEventDispatcher } from "svelte";

  export let block: BlockType;
  export let index: number;

  const dispatch = createEventDispatcher();

  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    dispatch("update", { index, content: target.value });
  }
</script>

<textarea
  class="block-textarea"
  value={block.content}
  on:input={handleInput}
  disabled={!block.enabled}
  rows={Math.max(3, block.content.split("\n").length + 1)}
></textarea>

<style>
  .block-textarea {
    width: 100%;
    background: var(--bg-input, #11111b);
    color: var(--text-primary, #cdd6f4);
    border: 1px solid var(--border, #313244);
    border-radius: 4px;
    padding: 8px;
    font-family: "JetBrains Mono", monospace;
    font-size: 13px;
    resize: vertical;
    box-sizing: border-box;
  }
  .block-textarea:disabled {
    opacity: 0.5;
  }
</style>
```

- [ ] **Step 3: Create FreeformBlock**

Create `src/lib/components/Blocks/FreeformBlock.svelte`:
```svelte
<script lang="ts">
  import type { Block as BlockType } from "../../types";
  import { createEventDispatcher } from "svelte";

  export let block: BlockType;
  export let index: number;

  const dispatch = createEventDispatcher();

  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    dispatch("update", { index, content: target.value });
  }
</script>

<textarea
  class="freeform-textarea"
  value={block.content}
  on:input={handleInput}
  rows={Math.max(2, block.content.split("\n").length + 1)}
></textarea>

<style>
  .freeform-textarea {
    width: 100%;
    background: transparent;
    color: var(--text-secondary, #a6adc8);
    border: 1px dashed var(--border, #313244);
    border-radius: 4px;
    padding: 8px;
    font-family: "JetBrains Mono", monospace;
    font-size: 13px;
    resize: vertical;
    box-sizing: border-box;
  }
</style>
```

- [ ] **Step 4: Create ExamplesBlock (handles nested example children)**

Create `src/lib/components/Blocks/ExamplesBlock.svelte`:
```svelte
<script lang="ts">
  import type { Block as BlockType } from "../../types";
  import TaggedBlock from "./TaggedBlock.svelte";
  import Block from "./Block.svelte";
  import { createEventDispatcher } from "svelte";

  export let block: BlockType;
  export let index: number;

  const dispatch = createEventDispatcher();

  function handleChildUpdate(e: CustomEvent, childIdx: number) {
    dispatch("child-update", {
      parentIndex: index,
      childIndex: childIdx,
      content: e.detail.content,
    });
  }

  function handleChildToggle(e: CustomEvent, childIdx: number) {
    dispatch("child-toggle", {
      parentIndex: index,
      childIndex: childIdx,
      enabled: e.detail.enabled,
    });
  }
</script>

{#each block.children as child, childIdx}
  <Block
    block={child}
    index={childIdx}
    on:toggle={(e) => handleChildToggle(e, childIdx)}
  >
    <TaggedBlock
      block={child}
      index={childIdx}
      on:update={(e) => handleChildUpdate(e, childIdx)}
    />
  </Block>
{/each}

{#if block.children.length === 0}
  <p class="empty-hint">No examples yet. Add one to get started.</p>
{/if}

<style>
  .empty-hint {
    color: var(--text-secondary, #888);
    font-style: italic;
    font-size: 13px;
  }
</style>
```

- [ ] **Step 5: Create MetadataBlock**

Create `src/lib/components/Blocks/MetadataBlock.svelte`:
```svelte
<script lang="ts">
  import type { PromptMetadata } from "../../types";
  import { createEventDispatcher } from "svelte";

  export let metadata: PromptMetadata;

  const dispatch = createEventDispatcher();

  function handleChange(field: string, value: string) {
    dispatch("metadata-update", { field, value });
  }
</script>

<div class="metadata-form">
  <label>
    <span class="label">Name</span>
    <input type="text" value={metadata.name} on:input={(e) => handleChange("name", e.currentTarget.value)} />
  </label>
  <label>
    <span class="label">Model</span>
    <select value={metadata.model} on:change={(e) => handleChange("model", e.currentTarget.value)}>
      <option value="claude-opus-4-6">Claude Opus 4.6</option>
      <option value="claude-sonnet-4-6">Claude Sonnet 4.6</option>
      <option value="claude-haiku-4-5">Claude Haiku 4.5</option>
    </select>
  </label>
  <label>
    <span class="label">Effort</span>
    <select value={metadata.effort ?? ""} on:change={(e) => handleChange("effort", e.currentTarget.value)}>
      <option value="">Default</option>
      <option value="low">Low</option>
      <option value="medium">Medium</option>
      <option value="high">High</option>
      <option value="max">Max</option>
    </select>
  </label>
  <label>
    <span class="label">Tags</span>
    <input type="text" value={metadata.tags.join(", ")} on:input={(e) => handleChange("tags", e.currentTarget.value)} />
  </label>
</div>

<style>
  .metadata-form {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .label {
    font-size: 11px;
    text-transform: uppercase;
    color: var(--text-secondary, #888);
    letter-spacing: 0.5px;
  }
  input, select {
    background: var(--bg-input, #11111b);
    color: var(--text-primary, #cdd6f4);
    border: 1px solid var(--border, #313244);
    border-radius: 4px;
    padding: 6px 8px;
    font-size: 13px;
  }
</style>
```

- [ ] **Step 6: Create StructureEditor that renders the AST as blocks**

Create `src/lib/components/Editor/StructureEditor.svelte`:
```svelte
<script lang="ts">
  import { currentAst } from "../../stores/prompt";
  import { syncAstToContent } from "../../stores/prompt";
  import type { PromptAst, Block as BlockType } from "../../types";
  import Block from "../Blocks/Block.svelte";
  import MetadataBlock from "../Blocks/MetadataBlock.svelte";
  import TaggedBlock from "../Blocks/TaggedBlock.svelte";
  import FreeformBlock from "../Blocks/FreeformBlock.svelte";
  import ExamplesBlock from "../Blocks/ExamplesBlock.svelte";

  function handleBlockUpdate(e: CustomEvent) {
    if (!$currentAst) return;
    const { index, content } = e.detail;
    const updated = structuredClone($currentAst);
    updated.blocks[index].content = content;
    syncAstToContent(updated);
  }

  function handleBlockToggle(e: CustomEvent) {
    if (!$currentAst) return;
    const { index, enabled } = e.detail;
    const updated = structuredClone($currentAst);
    updated.blocks[index].enabled = enabled;
    syncAstToContent(updated);
  }

  function handleMetadataUpdate(e: CustomEvent) {
    if (!$currentAst) return;
    const { field, value } = e.detail;
    const updated = structuredClone($currentAst);
    if (field === "tags") {
      updated.metadata.tags = value.split(",").map((t: string) => t.trim()).filter(Boolean);
    } else {
      (updated.metadata as any)[field] = value;
    }
    syncAstToContent(updated);
  }

  function handleChildUpdate(e: CustomEvent) {
    if (!$currentAst) return;
    const { parentIndex, childIndex, content } = e.detail;
    const updated = structuredClone($currentAst);
    updated.blocks[parentIndex].children[childIndex].content = content;
    syncAstToContent(updated);
  }

  function handleChildToggle(e: CustomEvent) {
    if (!$currentAst) return;
    const { parentIndex, childIndex, enabled } = e.detail;
    const updated = structuredClone($currentAst);
    updated.blocks[parentIndex].children[childIndex].enabled = enabled;
    syncAstToContent(updated);
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    if (!$currentAst) return;
    const fromStr = e.dataTransfer?.getData("text/plain");
    if (fromStr === undefined) return;
    const from = parseInt(fromStr);
    const target = (e.target as HTMLElement).closest("[data-block-index]");
    if (!target) return;
    const to = parseInt((target as HTMLElement).dataset.blockIndex!);
    if (from === to) return;

    const updated = structuredClone($currentAst);
    const [moved] = updated.blocks.splice(from, 1);
    updated.blocks.splice(to, 0, moved);
    syncAstToContent(updated);
  }

  function isExamplesBlock(block: BlockType): boolean {
    return block.kind === "Examples" || block.kind === "Documents";
  }
</script>

<div class="structure-editor" on:dragover|preventDefault on:drop={handleDrop}>
  {#if $currentAst}
    <!-- Metadata block (always first) -->
    <div class="block-wrapper metadata-wrapper">
      <div class="block-header-simple">⚙️ Metadata</div>
      <div class="block-body-simple">
        <MetadataBlock metadata={$currentAst.metadata} on:metadata-update={handleMetadataUpdate} />
      </div>
    </div>

    <!-- Content blocks -->
    {#each $currentAst.blocks as block, index (block.start_offset)}
      <div data-block-index={index}>
        <Block {block} {index} on:toggle={handleBlockToggle}>
          {#if block.kind === "Freeform"}
            <FreeformBlock {block} {index} on:update={handleBlockUpdate} />
          {:else if isExamplesBlock(block)}
            <ExamplesBlock
              {block}
              {index}
              on:child-update={handleChildUpdate}
              on:child-toggle={handleChildToggle}
            />
          {:else}
            <TaggedBlock {block} {index} on:update={handleBlockUpdate} />
          {/if}
        </Block>
      </div>
    {/each}
  {:else}
    <p class="no-content">Open a prompt file to begin editing.</p>
  {/if}
</div>

<style>
  .structure-editor {
    padding: 16px;
    overflow-y: auto;
    height: 100%;
  }
  .block-wrapper {
    border: 1px solid var(--border, #313244);
    border-radius: 6px;
    margin-bottom: 8px;
    background: var(--bg-block, #1e1e2e);
    overflow: hidden;
  }
  .block-header-simple {
    padding: 6px 10px;
    background: var(--bg-block-header, #181825);
    font-size: 13px;
    font-weight: 600;
  }
  .block-body-simple {
    padding: 10px;
  }
  .no-content {
    color: var(--text-secondary, #888);
    text-align: center;
    margin-top: 40px;
  }
</style>
```

- [ ] **Step 7: Commit**

```bash
git add src/lib/components/Blocks/ src/lib/components/Editor/StructureEditor.svelte
git commit -m "feat: add structure mode block editor with all block types"
```

---

### Task 12: Mode Toggle and Dual-Mode Sync

**Files:**
- Create: `src/lib/components/Editor/EditorTabs.svelte`
- Modify: `src/App.svelte` (integrate both modes)
- Modify: `src/lib/stores/editor.ts` (add mode switching logic)

- [ ] **Step 1: Create EditorTabs toggle component**

Create `src/lib/components/Editor/EditorTabs.svelte`:
```svelte
<script lang="ts">
  import { editorMode } from "../../stores/editor";
  import type { EditorMode } from "../../stores/editor";

  function setMode(mode: EditorMode) {
    editorMode.set(mode);
  }
</script>

<div class="editor-tabs">
  <button
    class="tab"
    class:active={$editorMode === "source"}
    on:click={() => setMode("source")}
  >
    Source
  </button>
  <button
    class="tab"
    class:active={$editorMode === "structure"}
    on:click={() => setMode("structure")}
  >
    Structure
  </button>
</div>

<style>
  .editor-tabs {
    display: flex;
    background: var(--bg-darker, #11111b);
    border-bottom: 1px solid var(--border, #313244);
  }
  .tab {
    padding: 6px 16px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary, #888);
    cursor: pointer;
    font-size: 13px;
  }
  .tab.active {
    color: var(--text-primary, #cdd6f4);
    border-bottom-color: var(--accent, #89b4fa);
  }
  .tab:hover {
    color: var(--text-primary, #cdd6f4);
  }
</style>
```

- [ ] **Step 2: Update App.svelte to integrate both modes with sync**

Replace `src/App.svelte` with updated version that includes mode toggle and sync:
```svelte
<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import SourceEditor from "./lib/components/Editor/SourceEditor.svelte";
  import StructureEditor from "./lib/components/Editor/StructureEditor.svelte";
  import EditorTabs from "./lib/components/Editor/EditorTabs.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import { currentContent, isDirty, editorMode } from "./lib/stores/editor";
  import { currentFile } from "./lib/stores/files";
  import { currentAst, parseFromContent } from "./lib/stores/prompt";
  import { openPrompt, savePrompt } from "./lib/tauri";

  // Sync: when content changes in source mode, re-parse to AST
  $: if ($editorMode === "source") {
    parseFromContent($currentContent);
  }

  async function handleOpen() {
    const selected = await open({
      filters: [{ name: "Prompt", extensions: ["md"] }],
    });
    if (selected) {
      const file = await openPrompt(selected as string);
      currentFile.set(file);
      currentContent.set(file.content);
      isDirty.set(false);
      parseFromContent(file.content);
    }
  }

  async function handleSave() {
    const file = $currentFile;
    if (!file) return;
    await savePrompt(file.path, $currentContent);
    isDirty.set(false);
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

<svelte:window on:keydown={handleKeydown} />

<div class="app">
  <div class="toolbar">
    <button on:click={handleOpen}>Open (Ctrl+O)</button>
    <button on:click={handleSave} disabled={!$currentFile || !$isDirty}>
      Save (Ctrl+S)
    </button>
  </div>
  <EditorTabs />
  <div class="editor-area">
    {#if $editorMode === "source"}
      <SourceEditor />
    {:else}
      <StructureEditor />
    {/if}
  </div>
  <StatusBar />
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: #1e1e2e;
    color: #cdd6f4;
    font-family: system-ui, -apple-system, sans-serif;
  }
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .toolbar {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
    background: #181825;
    border-bottom: 1px solid #313244;
  }
  .toolbar button {
    padding: 4px 12px;
    background: #313244;
    color: #cdd6f4;
    border: 1px solid #45475a;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }
  .toolbar button:hover { background: #45475a; }
  .toolbar button:disabled { opacity: 0.5; cursor: default; }
  .editor-area { flex: 1; overflow: hidden; }
</style>
```

- [ ] **Step 3: Verify dual-mode works**

Run:
```bash
pnpm tauri dev
```
Manual test:
1. Open a prompt file
2. Verify content appears in Source mode
3. Switch to Structure mode — blocks should appear
4. Edit a block in Structure mode
5. Switch back to Source mode — changes should be reflected
6. Edit in Source mode, switch to Structure — AST should update

Expected: Edits in either mode appear in the other

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/Editor/EditorTabs.svelte src/App.svelte src/lib/stores/
git commit -m "feat: add dual-mode editor with source/structure sync"
```

---

## Chunk 4: Linting Engine

This chunk builds the best-practices linting engine: rule framework, structural rules, anti-pattern detection, lint configuration, Tauri commands, and the Prompt Health panel UI.

### Task 13: Linting Framework and Structural Rules

**Files:**
- Create: `src-tauri/src/linter/mod.rs`
- Create: `src-tauri/src/linter/rules.rs`
- Create: `src-tauri/src/linter/structural.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod linter`)
- Test: inline in each file

- [ ] **Step 1: Define rule trait and result types**

Create `src-tauri/src/linter/rules.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Suggestion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub detail: String,
    pub block_index: Option<usize>,
    pub fix_suggestion: Option<String>,
}

/// Trait for lint rules
pub trait LintRule: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn check(&self, ast: &crate::parser::ast::PromptAst) -> Vec<LintResult>;
}
```

- [ ] **Step 2: Write failing tests for structural rules**

Create `src-tauri/src/linter/structural.rs`:
```rust
use super::rules::{LintResult, LintRule, Severity};
use crate::parser::ast::{BlockKind, PromptAst};

pub struct MissingRoleRule;

impl LintRule for MissingRoleRule {
    fn id(&self) -> &str { "missing-role" }
    fn description(&self) -> &str { "No <role> block found" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        todo!()
    }
}

pub struct SparseExamplesRule;

impl LintRule for SparseExamplesRule {
    fn id(&self) -> &str { "sparse-examples" }
    fn description(&self) -> &str { "Fewer than 3 examples" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        todo!()
    }
}

pub struct UnstructuredLongPromptRule;

impl LintRule for UnstructuredLongPromptRule {
    fn id(&self) -> &str { "unstructured-long-prompt" }
    fn description(&self) -> &str { "Long prompt with no XML structure" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        todo!()
    }
}

pub struct UnbalancedXmlRule;

impl LintRule for UnbalancedXmlRule {
    fn id(&self) -> &str { "unbalanced-xml" }
    fn description(&self) -> &str { "Mismatched XML tags" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_missing_role_triggers() {
        let ast = parser::parse("---\nname: test\n---\n\n<instructions>\nDo stuff.\n</instructions>\n").unwrap();
        let results = MissingRoleRule.check(&ast);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_id, "missing-role");
    }

    #[test]
    fn test_missing_role_no_trigger_when_present() {
        let ast = parser::parse("---\nname: test\n---\n\n<role>\nHelper.\n</role>\n").unwrap();
        let results = MissingRoleRule.check(&ast);
        assert!(results.is_empty());
    }

    #[test]
    fn test_sparse_examples_triggers() {
        let ast = parser::parse("---\nname: test\n---\n\n<examples>\n<example>\nOne.\n</example>\n</examples>\n").unwrap();
        let results = SparseExamplesRule.check(&ast);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].severity, Severity::Suggestion);
    }

    #[test]
    fn test_unstructured_long_prompt_triggers() {
        let long_text = "word ".repeat(200); // ~1000 chars, ~250 tokens
        let input = format!("---\nname: test\n---\n\n{}", long_text);
        let ast = parser::parse(&input).unwrap();
        let results = UnstructuredLongPromptRule.check(&ast);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_unstructured_short_prompt_no_trigger() {
        let ast = parser::parse("---\nname: test\n---\n\nShort prompt.").unwrap();
        let results = UnstructuredLongPromptRule.check(&ast);
        assert!(results.is_empty());
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run:
```bash
cd src-tauri && cargo test linter::structural -- --nocapture
```
Expected: FAIL

- [ ] **Step 4: Implement structural rules**

Replace `todo!()` bodies:
```rust
impl LintRule for MissingRoleRule {
    fn id(&self) -> &str { "missing-role" }
    fn description(&self) -> &str { "No <role> block found" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        let has_role = ast.blocks.iter().any(|b| b.kind == BlockKind::Role);
        if has_role {
            vec![]
        } else {
            vec![LintResult {
                rule_id: self.id().to_string(),
                severity: Severity::Suggestion,
                message: "No <role> definition found.".into(),
                detail: "Adding a role helps Claude understand context and produce better responses.".into(),
                block_index: None,
                fix_suggestion: Some("Add a <role> block describing who Claude should be.".into()),
            }]
        }
    }
}

impl LintRule for SparseExamplesRule {
    fn id(&self) -> &str { "sparse-examples" }
    fn description(&self) -> &str { "Fewer than 3 examples" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        let mut results = vec![];
        for (i, block) in ast.blocks.iter().enumerate() {
            if block.kind == BlockKind::Examples && block.children.len() < 3 {
                results.push(LintResult {
                    rule_id: self.id().to_string(),
                    severity: Severity::Suggestion,
                    message: format!(
                        "Examples block has {} example(s). Anthropic recommends 3-5 diverse examples.",
                        block.children.len()
                    ),
                    detail: "More diverse examples improve accuracy and consistency.".into(),
                    block_index: Some(i),
                    fix_suggestion: None,
                });
            }
        }
        results
    }
}

impl LintRule for UnstructuredLongPromptRule {
    fn id(&self) -> &str { "unstructured-long-prompt" }
    fn description(&self) -> &str { "Long prompt with no XML structure" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        let has_tagged_blocks = ast.blocks.iter().any(|b| b.tag_name.is_some());
        if has_tagged_blocks {
            return vec![];
        }
        let total_chars: usize = ast.blocks.iter().map(|b| b.content.len()).sum();
        let estimated_tokens = total_chars / 4;
        if estimated_tokens > 500 {
            vec![LintResult {
                rule_id: self.id().to_string(),
                severity: Severity::Warning,
                message: format!("Prompt is ~{} tokens with no XML structure.", estimated_tokens),
                detail: "XML tags help Claude parse complex prompts unambiguously.".into(),
                block_index: None,
                fix_suggestion: Some("Consider wrapping sections in XML tags like <role>, <instructions>, <examples>.".into()),
            }]
        } else {
            vec![]
        }
    }
}

impl LintRule for UnbalancedXmlRule {
    fn id(&self) -> &str { "unbalanced-xml" }
    fn description(&self) -> &str { "Mismatched XML tags" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        // Freeform blocks that contain unmatched tags indicate parse failures
        let mut results = vec![];
        for (i, block) in ast.blocks.iter().enumerate() {
            if block.kind == BlockKind::Freeform && block.content.contains('<') {
                let opens: Vec<&str> = block.content.lines()
                    .filter(|l| {
                        let t = l.trim();
                        t.starts_with('<') && !t.starts_with("</") && t.ends_with('>')
                    })
                    .collect();
                if !opens.is_empty() {
                    results.push(LintResult {
                        rule_id: self.id().to_string(),
                        severity: Severity::Error,
                        message: "Possible unmatched XML tag detected.".into(),
                        detail: "This text contains what looks like an XML tag without a matching close tag.".into(),
                        block_index: Some(i),
                        fix_suggestion: Some("Add the matching closing tag or remove the stray tag.".into()),
                    });
                }
            }
        }
        results
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run:
```bash
cd src-tauri && cargo test linter::structural -- --nocapture
```
Expected: All 5 tests pass

- [ ] **Step 6: Create linter entry point**

Create `src-tauri/src/linter/mod.rs`:
```rust
pub mod rules;
pub mod structural;
pub mod antipatterns;
pub mod config;

use rules::{LintResult, LintRule};
use crate::parser::ast::PromptAst;

/// Run all enabled lint rules against a prompt AST
pub fn lint(ast: &PromptAst, disabled_rules: &[String]) -> Vec<LintResult> {
    let all_rules: Vec<Box<dyn LintRule>> = vec![
        Box::new(structural::MissingRoleRule),
        Box::new(structural::SparseExamplesRule),
        Box::new(structural::UnstructuredLongPromptRule),
        Box::new(structural::UnbalancedXmlRule),
    ];

    all_rules
        .iter()
        .filter(|r| !disabled_rules.contains(&r.id().to_string()))
        .flat_map(|r| r.check(ast))
        .collect()
}
```

Add `mod linter;` to `src-tauri/src/lib.rs`.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/linter/ src-tauri/src/lib.rs
git commit -m "feat: add linting framework with structural rules"
```

---

### Task 14: Anti-Pattern Detection Rules

**Files:**
- Create: `src-tauri/src/linter/antipatterns.rs`
- Modify: `src-tauri/src/linter/mod.rs` (register new rules)
- Test: inline

- [ ] **Step 1: Write failing tests for anti-pattern rules**

Create `src-tauri/src/linter/antipatterns.rs`:
```rust
use super::rules::{LintResult, LintRule, Severity};
use crate::parser::ast::{BlockKind, PromptAst};

pub struct NegativeFramingRule;

impl LintRule for NegativeFramingRule {
    fn id(&self) -> &str { "negative-framing" }
    fn description(&self) -> &str { "Instructions use 'don't' instead of positive framing" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        todo!()
    }
}

pub struct OverPromptingRule;

impl LintRule for OverPromptingRule {
    fn id(&self) -> &str { "over-prompting" }
    fn description(&self) -> &str { "Aggressive emphasis that may cause overtriggering" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        todo!()
    }
}

pub struct VagueInstructionsRule;

impl LintRule for VagueInstructionsRule {
    fn id(&self) -> &str { "vague-instructions" }
    fn description(&self) -> &str { "Non-specific instructions detected" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_negative_framing_triggers() {
        let ast = parser::parse("---\nname: t\n---\n\n<instructions>\nDon't use markdown in your response.\n</instructions>\n").unwrap();
        let results = NegativeFramingRule.check(&ast);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_negative_framing_no_trigger() {
        let ast = parser::parse("---\nname: t\n---\n\n<instructions>\nUse flowing prose paragraphs.\n</instructions>\n").unwrap();
        let results = NegativeFramingRule.check(&ast);
        assert!(results.is_empty());
    }

    #[test]
    fn test_over_prompting_triggers() {
        let ast = parser::parse("---\nname: t\n---\n\n<instructions>\nCRITICAL: You MUST ALWAYS use this tool.\n</instructions>\n").unwrap();
        let results = OverPromptingRule.check(&ast);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_vague_instructions_triggers() {
        let ast = parser::parse("---\nname: t\n---\n\n<instructions>\nDo a good job and be helpful.\n</instructions>\n").unwrap();
        let results = VagueInstructionsRule.check(&ast);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_vague_instructions_no_trigger() {
        let ast = parser::parse("---\nname: t\n---\n\n<instructions>\nReview the code for SQL injection vulnerabilities. Return findings as JSON.\n</instructions>\n").unwrap();
        let results = VagueInstructionsRule.check(&ast);
        assert!(results.is_empty());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
```bash
cd src-tauri && cargo test linter::antipatterns -- --nocapture
```
Expected: FAIL

- [ ] **Step 3: Implement anti-pattern rules**

Replace `todo!()` bodies:
```rust
impl LintRule for NegativeFramingRule {
    fn id(&self) -> &str { "negative-framing" }
    fn description(&self) -> &str { "Instructions use 'don't' instead of positive framing" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        let patterns = ["don't", "do not", "never", "avoid", "don't"];
        let mut results = vec![];
        for (i, block) in ast.blocks.iter().enumerate() {
            if block.kind == BlockKind::Freeform || block.tag_name.is_some() {
                let lower = block.content.to_lowercase();
                for pattern in &patterns {
                    if lower.contains(pattern) {
                        results.push(LintResult {
                            rule_id: self.id().to_string(),
                            severity: Severity::Suggestion,
                            message: format!("Consider positive framing instead of '{}'.", pattern),
                            detail: "Tell Claude what to do instead of what not to do for better results.".into(),
                            block_index: Some(i),
                            fix_suggestion: Some("Reframe as a positive instruction.".into()),
                        });
                        break; // One finding per block
                    }
                }
            }
        }
        results
    }
}

impl LintRule for OverPromptingRule {
    fn id(&self) -> &str { "over-prompting" }
    fn description(&self) -> &str { "Aggressive emphasis that may cause overtriggering" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        let patterns = [
            "CRITICAL:", "MUST ALWAYS", "EXTREMELY IMPORTANT",
            "NEVER EVER", "ABSOLUTELY MUST", "YOU MUST",
        ];
        let mut results = vec![];
        for (i, block) in ast.blocks.iter().enumerate() {
            for pattern in &patterns {
                if block.content.contains(pattern) {
                    results.push(LintResult {
                        rule_id: self.id().to_string(),
                        severity: Severity::Warning,
                        message: format!("Aggressive emphasis detected: '{}'.", pattern),
                        detail: "Claude 4.6 responds well to normal prompting. Aggressive emphasis may cause overtriggering.".into(),
                        block_index: Some(i),
                        fix_suggestion: Some("Use normal instructional language instead.".into()),
                    });
                    break;
                }
            }
        }
        results
    }
}

impl LintRule for VagueInstructionsRule {
    fn id(&self) -> &str { "vague-instructions" }
    fn description(&self) -> &str { "Non-specific instructions detected" }
    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        let vague_phrases = [
            "do a good job", "be helpful", "try your best",
            "be thorough", "be careful", "do well",
            "be smart", "think hard", "be creative",
        ];
        let mut results = vec![];
        for (i, block) in ast.blocks.iter().enumerate() {
            let lower = block.content.to_lowercase();
            for phrase in &vague_phrases {
                if lower.contains(phrase) {
                    results.push(LintResult {
                        rule_id: self.id().to_string(),
                        severity: Severity::Suggestion,
                        message: format!("Vague instruction detected: '{}'.", phrase),
                        detail: "Be specific about desired output format and constraints for better results.".into(),
                        block_index: Some(i),
                        fix_suggestion: None,
                    });
                    break;
                }
            }
        }
        results
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```bash
cd src-tauri && cargo test linter::antipatterns -- --nocapture
```
Expected: All 5 tests pass

- [ ] **Step 5: Register anti-pattern rules in linter/mod.rs**

Update the `lint()` function to include:
```rust
Box::new(antipatterns::NegativeFramingRule),
Box::new(antipatterns::OverPromptingRule),
Box::new(antipatterns::VagueInstructionsRule),
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/linter/
git commit -m "feat: add anti-pattern detection rules (negative framing, over-prompting, vague)"
```

---

### Task 15: Lint Configuration and Tauri Commands

**Files:**
- Create: `src-tauri/src/linter/config.rs`
- Create: `src-tauri/src/commands/lint.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs` (register lint command)

- [ ] **Step 1: Create lint config loader**

Create `src-tauri/src/linter/config.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LintConfig {
    #[serde(default)]
    pub disabled_rules: Vec<String>,
    #[serde(default)]
    pub severity_overrides: HashMap<String, String>,
}

pub fn load_config(project_dir: &Path) -> LintConfig {
    let config_path = project_dir.join(".claude-prompts").join("lintrc.yaml");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).unwrap_or_default();
        serde_yaml::from_str(&content).unwrap_or_default()
    } else {
        LintConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_missing_config_returns_default() {
        let dir = TempDir::new().unwrap();
        let config = load_config(dir.path());
        assert!(config.disabled_rules.is_empty());
    }

    #[test]
    fn test_load_existing_config() {
        let dir = TempDir::new().unwrap();
        let config_dir = dir.path().join(".claude-prompts");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("lintrc.yaml"),
            "disabled_rules:\n  - missing-role\n",
        )
        .unwrap();
        let config = load_config(dir.path());
        assert_eq!(config.disabled_rules, vec!["missing-role"]);
    }
}
```

- [ ] **Step 2: Create lint Tauri command**

Create `src-tauri/src/commands/lint.rs`:
```rust
use crate::linter;
use crate::linter::rules::LintResult;
use crate::parser;
use std::path::PathBuf;

#[tauri::command]
pub fn lint_prompt(content: String, project_dir: Option<String>) -> Result<Vec<LintResult>, String> {
    let ast = parser::parse(&content)?;
    let config = project_dir
        .map(|d| linter::config::load_config(&PathBuf::from(d)))
        .unwrap_or_default();
    Ok(linter::lint(&ast, &config.disabled_rules))
}
```

- [ ] **Step 3: Register lint command**

Update `src-tauri/src/commands/mod.rs`:
```rust
pub mod file;
pub mod lint;
pub mod mcp;
pub mod prompt;
```

Add to invoke_handler in `src-tauri/src/lib.rs`:
```rust
commands::lint::lint_prompt,
```

- [ ] **Step 4: Run all tests**

Run:
```bash
cd src-tauri && cargo test -- --nocapture
```
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/linter/config.rs src-tauri/src/commands/lint.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add lint configuration and Tauri lint command"
```

---

### Task 16: Prompt Health Panel UI

**Files:**
- Create: `src/lib/stores/lint.ts`
- Create: `src/lib/components/Panels/PromptHealth.svelte`
- Modify: `src/lib/tauri.ts` (add lint IPC)
- Modify: `src/App.svelte` (add panel)

- [ ] **Step 1: Add lint IPC and store**

Add to `src/lib/tauri.ts`:
```typescript
import type { LintResult } from "./types";

export async function lintPrompt(content: string, projectDir?: string): Promise<LintResult[]> {
  return invoke("lint_prompt", { content, projectDir: projectDir ?? null });
}
```

Add to `src/lib/types.ts`:
```typescript
export type LintSeverity = "Error" | "Warning" | "Suggestion";

export interface LintResult {
  rule_id: string;
  severity: LintSeverity;
  message: string;
  detail: string;
  block_index: number | null;
  fix_suggestion: string | null;
}
```

Create `src/lib/stores/lint.ts`:
```typescript
import { writable, derived } from "svelte/store";
import type { LintResult } from "../types";
import { lintPrompt } from "../tauri";
import { currentContent } from "./editor";

export const lintResults = writable<LintResult[]>([]);

let debounceTimer: ReturnType<typeof setTimeout>;

export function runLint(content: string) {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(async () => {
    try {
      const results = await lintPrompt(content);
      lintResults.set(results);
    } catch {
      lintResults.set([]);
    }
  }, 500);
}

export const errorCount = derived(lintResults, ($r) => $r.filter((r) => r.severity === "Error").length);
export const warningCount = derived(lintResults, ($r) => $r.filter((r) => r.severity === "Warning").length);
export const suggestionCount = derived(lintResults, ($r) => $r.filter((r) => r.severity === "Suggestion").length);
```

- [ ] **Step 2: Create PromptHealth panel component**

Create `src/lib/components/Panels/PromptHealth.svelte`:
```svelte
<script lang="ts">
  import { lintResults, errorCount, warningCount, suggestionCount } from "../../stores/lint";
  import type { LintResult } from "../../types";

  let expandedRule: string | null = null;

  function toggleDetail(ruleId: string) {
    expandedRule = expandedRule === ruleId ? null : ruleId;
  }

  function severityColor(severity: string): string {
    switch (severity) {
      case "Error": return "#f38ba8";
      case "Warning": return "#fab387";
      case "Suggestion": return "#89b4fa";
      default: return "#cdd6f4";
    }
  }

  function overallHealth(): { label: string; color: string } {
    if ($errorCount > 0) return { label: "Issues Found", color: "#f38ba8" };
    if ($warningCount > 0) return { label: "Could Improve", color: "#fab387" };
    if ($suggestionCount > 0) return { label: "Good", color: "#89b4fa" };
    return { label: "Excellent", color: "#a6e3a1" };
  }

  $: health = overallHealth();
</script>

<div class="prompt-health">
  <div class="health-header">
    <span class="health-label">Prompt Health</span>
    <span class="health-badge" style="background: {health.color}20; color: {health.color}">
      {health.label}
    </span>
  </div>

  <div class="health-counts">
    <span style="color: #f38ba8">{$errorCount} errors</span>
    <span style="color: #fab387">{$warningCount} warnings</span>
    <span style="color: #89b4fa">{$suggestionCount} suggestions</span>
  </div>

  {#if $lintResults.length === 0}
    <p class="all-clear">No issues found.</p>
  {:else}
    <div class="results-list">
      {#each $lintResults as result}
        <div class="result-item" on:click={() => toggleDetail(result.rule_id)}>
          <span class="severity-dot" style="background: {severityColor(result.severity)}"></span>
          <span class="result-message">{result.message}</span>
        </div>
        {#if expandedRule === result.rule_id}
          <div class="result-detail">
            <p>{result.detail}</p>
            {#if result.fix_suggestion}
              <p class="fix-suggestion">Suggestion: {result.fix_suggestion}</p>
            {/if}
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .prompt-health {
    padding: 12px;
    height: 100%;
    overflow-y: auto;
  }
  .health-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .health-label { font-weight: 600; font-size: 14px; }
  .health-badge {
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 600;
  }
  .health-counts {
    display: flex;
    gap: 12px;
    font-size: 12px;
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border, #313244);
  }
  .all-clear { color: #a6e3a1; font-size: 13px; }
  .results-list { display: flex; flex-direction: column; gap: 4px; }
  .result-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }
  .result-item:hover { background: var(--bg-block-header, #181825); }
  .severity-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .result-detail {
    padding: 8px 8px 8px 24px;
    font-size: 12px;
    color: var(--text-secondary, #a6adc8);
  }
  .fix-suggestion {
    color: #89b4fa;
    font-style: italic;
  }
</style>
```

- [ ] **Step 3: Integrate lint panel and auto-lint into App.svelte**

Add to App.svelte imports:
```typescript
import PromptHealth from "./lib/components/Panels/PromptHealth.svelte";
import { runLint } from "./lib/stores/lint";
```

Add reactive lint trigger:
```typescript
$: runLint($currentContent);
```

Add the panel to the layout (right sidebar):
```svelte
<div class="main-layout">
  <div class="editor-area">
    {#if $editorMode === "source"}
      <SourceEditor />
    {:else}
      <StructureEditor />
    {/if}
  </div>
  <div class="right-panel">
    <PromptHealth />
  </div>
</div>
```

Update CSS for the side-by-side layout:
```css
.main-layout {
  flex: 1;
  display: flex;
  overflow: hidden;
}
.editor-area {
  flex: 1;
  overflow: hidden;
}
.right-panel {
  width: 300px;
  border-left: 1px solid var(--border, #313244);
  background: var(--bg-block, #1e1e2e);
}
```

- [ ] **Step 4: Verify lint panel works**

Run:
```bash
pnpm tauri dev
```
Manual test:
1. Open a prompt file with no role block
2. Verify "No <role> definition found" appears in Prompt Health panel
3. Add a `<role>` block, verify the lint finding disappears

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/lint.ts src/lib/components/Panels/ src/lib/tauri.ts src/lib/types.ts src/App.svelte
git commit -m "feat: add Prompt Health panel with live linting"
```

---

## Chunk 5: Presets, Templates & Version History

### Task 17: Built-in Presets

**Files:**
- Create: `src-tauri/src/preset/mod.rs`
- Create: `src-tauri/src/preset/builtin.rs`
- Create: `src-tauri/src/preset/custom.rs`
- Create: `src-tauri/src/commands/preset.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Define preset types and built-in roles**

Create `src-tauri/src/preset/mod.rs`:
```rust
pub mod builtin;
pub mod custom;
pub mod templates;
```

Create `src-tauri/src/preset/builtin.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub category: PresetCategory,
    pub content: String,
    pub tag_name: Option<String>,
    pub metadata_defaults: Option<MetadataDefaults>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresetCategory {
    Role,
    Instructions,
    Constraints,
    OutputFormat,
    ExampleSkeleton,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataDefaults {
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking_type: Option<String>,
}

pub fn builtin_presets() -> Vec<Preset> {
    vec![
        // Roles
        Preset {
            id: "role-code-reviewer".into(),
            name: "Code Reviewer".into(),
            category: PresetCategory::Role,
            content: "You are an expert code reviewer specializing in {{language}}. You focus on security vulnerabilities, performance issues, and code readability.".into(),
            tag_name: Some("role".into()),
            metadata_defaults: Some(MetadataDefaults {
                model: None,
                effort: Some("high".into()),
                thinking_type: Some("adaptive".into()),
            }),
        },
        Preset {
            id: "role-data-analyst".into(),
            name: "Data Analyst".into(),
            category: PresetCategory::Role,
            content: "You are a data analyst with access to {{tools}}. You analyze datasets, identify trends, and provide actionable insights.".into(),
            tag_name: Some("role".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "role-tech-writer".into(),
            name: "Technical Writer".into(),
            category: PresetCategory::Role,
            content: "You are a technical writer creating documentation for {{audience}}. You write clearly, use consistent terminology, and structure content for easy navigation.".into(),
            tag_name: Some("role".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "role-qa-engineer".into(),
            name: "QA Engineer".into(),
            category: PresetCategory::Role,
            content: "You are a QA engineer focused on edge cases, error handling, and regression testing. You think adversarially about inputs and states.".into(),
            tag_name: Some("role".into()),
            metadata_defaults: None,
        },
        // Constraints
        Preset {
            id: "constraint-investigate-first".into(),
            name: "Investigate Before Answering".into(),
            category: PresetCategory::Constraints,
            content: "Never speculate about code you have not opened. If the user references a specific file, read the file before answering. Investigate and read relevant files BEFORE answering questions about the codebase.".into(),
            tag_name: Some("investigate_before_answering".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "constraint-default-to-action".into(),
            name: "Default to Action".into(),
            category: PresetCategory::Constraints,
            content: "By default, implement changes rather than only suggesting them. If the user's intent is unclear, infer the most useful likely action and proceed, using tools to discover any missing details instead of guessing.".into(),
            tag_name: Some("default_to_action".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "constraint-minimize-overengineering".into(),
            name: "Minimize Overengineering".into(),
            category: PresetCategory::Constraints,
            content: "Only make changes that are directly requested or clearly necessary. Keep solutions simple and focused. Don't add features, refactor code, or make improvements beyond what was asked.".into(),
            tag_name: Some("constraints".into()),
            metadata_defaults: None,
        },
        // Output formats
        Preset {
            id: "output-json-only".into(),
            name: "JSON Output Only".into(),
            category: PresetCategory::OutputFormat,
            content: "Respond with valid JSON only. No markdown, no explanation, no preamble. The response must be parseable by JSON.parse().".into(),
            tag_name: Some("output_format".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "output-prose".into(),
            name: "Markdown-Free Prose".into(),
            category: PresetCategory::OutputFormat,
            content: "Your response should be composed of smoothly flowing prose paragraphs. Do not use markdown formatting, bullet points, or numbered lists. Write in clear, natural language.".into(),
            tag_name: Some("output_format".into()),
            metadata_defaults: None,
        },
    ]
}
```

- [ ] **Step 2: Create preset Tauri command**

Create `src-tauri/src/commands/preset.rs`:
```rust
use crate::preset::builtin::{self, Preset};

#[tauri::command]
pub fn list_presets() -> Vec<Preset> {
    builtin::builtin_presets()
}
```

Add `pub mod preset;` to `src-tauri/src/commands/mod.rs`.
Add `mod preset;` to `src-tauri/src/lib.rs`.
Add `commands::preset::list_presets` to invoke_handler.

- [ ] **Step 3: Verify build and test**

Run:
```bash
cd src-tauri && cargo build && cargo test -- --nocapture
```
Expected: All pass

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/preset/ src-tauri/src/commands/preset.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add built-in preset library with roles, constraints, output formats"
```

---

### Task 18: Template Wizard

**Files:**
- Create: `src-tauri/src/preset/templates.rs`
- Modify: `src-tauri/src/commands/preset.rs` (add template commands)
- Create: `src/lib/components/Dialogs/NewPromptWizard.svelte`
- Create: `src/lib/stores/presets.ts`

- [ ] **Step 1: Define built-in templates**

Create `src-tauri/src/preset/templates.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateCategory {
    Blank,
    UseCase,
}

pub fn builtin_templates() -> Vec<Template> {
    vec![
        Template {
            id: "blank-minimal".into(),
            name: "Minimal".into(),
            description: "Frontmatter + single instruction block".into(),
            category: TemplateCategory::Blank,
            content: "---\nname: \"{{name}}\"\nmodel: claude-opus-4-6\nversion: 1\ntags: []\n---\n\n<instructions>\n\n</instructions>\n".into(),
        },
        Template {
            id: "blank-standard".into(),
            name: "Standard".into(),
            description: "Role, instructions, output format, 1 example".into(),
            category: TemplateCategory::Blank,
            content: "---\nname: \"{{name}}\"\nmodel: claude-opus-4-6\nversion: 1\ntags: []\nthinking:\n  type: adaptive\n---\n\n<role>\nYou are a {{role_description}}.\n</role>\n\n<instructions>\n\n</instructions>\n\n<output_format>\n\n</output_format>\n\n<examples>\n<example>\n<input>\n\n</input>\n<output>\n\n</output>\n</example>\n</examples>\n".into(),
        },
        Template {
            id: "usecase-agentic".into(),
            name: "Agentic System".into(),
            description: "Role, tool guidance, safety guardrails, state tracking".into(),
            category: TemplateCategory::UseCase,
            content: "---\nname: \"{{name}}\"\nmodel: claude-opus-4-6\nversion: 1\ntags: [agentic]\nthinking:\n  type: adaptive\neffort: high\n---\n\n<role>\nYou are an autonomous agent that {{agent_purpose}}.\n</role>\n\n<instructions>\n\n</instructions>\n\n<tool_usage>\nUse tools when they would enhance your understanding of the problem.\n</tool_usage>\n\n<safety>\nConsider the reversibility and potential impact of your actions. For actions that are hard to reverse or affect shared systems, ask before proceeding.\n</safety>\n\n<state_tracking>\nKeep track of your progress. Save state to structured files so work can continue across sessions.\n</state_tracking>\n".into(),
        },
        Template {
            id: "usecase-code-assistant".into(),
            name: "Code Assistant".into(),
            description: "Role with language variable, coding conventions, examples".into(),
            category: TemplateCategory::UseCase,
            content: "---\nname: \"{{name}}\"\nmodel: claude-opus-4-6\nversion: 1\ntags: [coding]\nthinking:\n  type: adaptive\neffort: high\n---\n\n<role>\nYou are an expert {{language}} developer.\n</role>\n\n<instructions>\n\n</instructions>\n\n<conventions>\nFollow the project's existing patterns and coding style.\n</conventions>\n\n<examples>\n<example>\n<input>\n\n</input>\n<output>\n\n</output>\n</example>\n</examples>\n".into(),
        },
        Template {
            id: "usecase-classification".into(),
            name: "Classification".into(),
            description: "Role, label definitions, diverse few-shot examples".into(),
            category: TemplateCategory::UseCase,
            content: "---\nname: \"{{name}}\"\nmodel: claude-sonnet-4-6\nversion: 1\ntags: [classification]\n---\n\n<role>\nYou are a classifier that categorizes {{input_type}} into the following labels.\n</role>\n\n<labels>\n- Label A: Description\n- Label B: Description\n- Label C: Description\n</labels>\n\n<output_format>\nRespond with only the label name. No explanation.\n</output_format>\n\n<examples>\n<example>\n<input>Example input 1</input>\n<output>Label A</output>\n</example>\n<example>\n<input>Example input 2</input>\n<output>Label B</output>\n</example>\n<example>\n<input>Edge case input</input>\n<output>Label C</output>\n</example>\n</examples>\n".into(),
        },
    ]
}
```

- [ ] **Step 2: Add template list command**

Add to `src-tauri/src/commands/preset.rs`:
```rust
use crate::preset::templates::{self, Template};

#[tauri::command]
pub fn list_templates() -> Vec<Template> {
    templates::builtin_templates()
}
```

Register `commands::preset::list_templates` in invoke_handler.

- [ ] **Step 3: Create frontend preset/template stores and wizard**

Create `src/lib/stores/presets.ts`:
```typescript
import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface Preset {
  id: string;
  name: string;
  category: string;
  content: string;
  tag_name: string | null;
}

export interface Template {
  id: string;
  name: string;
  description: string;
  category: string;
  content: string;
}

export const presets = writable<Preset[]>([]);
export const templates = writable<Template[]>([]);

export async function loadPresets() {
  const p: Preset[] = await invoke("list_presets");
  presets.set(p);
}

export async function loadTemplates() {
  const t: Template[] = await invoke("list_templates");
  templates.set(t);
}
```

Create `src/lib/components/Dialogs/NewPromptWizard.svelte`:
```svelte
<script lang="ts">
  import { templates, loadTemplates } from "../../stores/presets";
  import { onMount, createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();
  let promptName = "";
  let selectedTemplate: string | null = null;

  onMount(() => {
    loadTemplates();
  });

  function handleCreate() {
    const template = $templates.find((t) => t.id === selectedTemplate);
    const content = template
      ? template.content.replace("{{name}}", promptName)
      : `---\nname: "${promptName}"\nmodel: claude-opus-4-6\nversion: 1\n---\n\n`;
    dispatch("create", { name: promptName, content });
  }

  function handleCancel() {
    dispatch("cancel");
  }
</script>

<div class="wizard-overlay" on:click|self={handleCancel}>
  <div class="wizard">
    <h2>New Prompt</h2>

    <label>
      <span class="label">Prompt Name</span>
      <input type="text" bind:value={promptName} placeholder="my-prompt" autofocus />
    </label>

    <h3>Choose a Template</h3>

    <div class="template-grid">
      {#each $templates as template}
        <button
          class="template-card"
          class:selected={selectedTemplate === template.id}
          on:click={() => (selectedTemplate = template.id)}
        >
          <span class="template-name">{template.name}</span>
          <span class="template-desc">{template.description}</span>
        </button>
      {/each}
    </div>

    <div class="wizard-actions">
      <button class="cancel" on:click={handleCancel}>Cancel</button>
      <button class="create" disabled={!promptName} on:click={handleCreate}>Create</button>
    </div>
  </div>
</div>

<style>
  .wizard-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .wizard {
    background: #1e1e2e;
    border: 1px solid #313244;
    border-radius: 8px;
    padding: 24px;
    width: 500px;
    max-height: 80vh;
    overflow-y: auto;
  }
  h2 { margin: 0 0 16px; font-size: 18px; }
  h3 { margin: 16px 0 8px; font-size: 14px; color: #a6adc8; }
  .label { font-size: 12px; color: #888; }
  input {
    width: 100%;
    padding: 8px;
    background: #11111b;
    border: 1px solid #313244;
    border-radius: 4px;
    color: #cdd6f4;
    font-size: 14px;
    margin-top: 4px;
    box-sizing: border-box;
  }
  .template-grid { display: flex; flex-direction: column; gap: 6px; }
  .template-card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 10px;
    background: #181825;
    border: 1px solid #313244;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    color: #cdd6f4;
  }
  .template-card.selected { border-color: #89b4fa; background: #1e1e3e; }
  .template-name { font-weight: 600; font-size: 13px; }
  .template-desc { font-size: 12px; color: #a6adc8; }
  .wizard-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }
  .cancel, .create {
    padding: 6px 16px;
    border-radius: 4px;
    border: 1px solid #45475a;
    cursor: pointer;
    font-size: 13px;
  }
  .cancel { background: #313244; color: #cdd6f4; }
  .create { background: #89b4fa; color: #11111b; font-weight: 600; }
  .create:disabled { opacity: 0.5; cursor: default; }
</style>
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/preset/templates.rs src-tauri/src/commands/preset.rs src/lib/stores/presets.ts src/lib/components/Dialogs/
git commit -m "feat: add template wizard with built-in blank and use-case templates"
```

---

### Task 19: Version History Backend

**Files:**
- Create: `src-tauri/src/version/mod.rs`
- Create: `src-tauri/src/version/store.rs`
- Create: `src-tauri/src/version/diff.rs`
- Create: `src-tauri/src/commands/version.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml` (add `similar`)

- [ ] **Step 1: Add similar dependency**

Add to `src-tauri/Cargo.toml`:
```toml
similar = "2"
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: Write failing tests for diff and store**

Create `src-tauri/src/version/mod.rs`:
```rust
pub mod diff;
pub mod store;
```

Create `src-tauri/src/version/diff.rs`:
```rust
use similar::{ChangeTag, TextDiff};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDiff {
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub new_start: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub tag: String, // "equal", "insert", "delete"
    pub content: String,
}

pub fn compute_diff(old: &str, new: &str) -> UnifiedDiff {
    let diff = TextDiff::from_lines(old, new);
    let mut hunks = Vec::new();

    for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
        let mut lines = Vec::new();
        for change in hunk.iter_changes() {
            let tag = match change.tag() {
                ChangeTag::Equal => "equal",
                ChangeTag::Insert => "insert",
                ChangeTag::Delete => "delete",
            };
            lines.push(DiffLine {
                tag: tag.to_string(),
                content: change.value().to_string(),
            });
        }
        hunks.push(DiffHunk {
            old_start: hunk.header().old_range().start,
            new_start: hunk.header().new_range().start,
            lines,
        });
    }

    UnifiedDiff { hunks }
}

/// Apply a sequence of inserts/deletes to reconstruct new text from old text
pub fn apply_diff(old: &str, diff: &UnifiedDiff) -> String {
    // For simplicity, we store full content in snapshots and use diffs for display only.
    // This is fine for MVP — prompts are small.
    old.to_string() // placeholder — actual reconstruction not needed for MVP
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_diff_basic() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nline2 modified\nline3\n";
        let diff = compute_diff(old, new);
        assert!(!diff.hunks.is_empty());
        let has_insert = diff.hunks.iter().any(|h| h.lines.iter().any(|l| l.tag == "insert"));
        let has_delete = diff.hunks.iter().any(|h| h.lines.iter().any(|l| l.tag == "delete"));
        assert!(has_insert);
        assert!(has_delete);
    }

    #[test]
    fn test_compute_diff_identical() {
        let text = "same\ncontent\n";
        let diff = compute_diff(text, text);
        assert!(diff.hunks.is_empty());
    }
}
```

Create `src-tauri/src/version/store.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    pub version: u32,
    pub timestamp: i64,
    pub content_hash: String,
    pub annotation: Option<String>,
    pub content: String, // Full content for MVP (small files)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    pub entries: Vec<VersionEntry>,
}

fn history_dir(project_dir: &Path, prompt_name: &str) -> PathBuf {
    project_dir
        .join(".claude-prompts")
        .join("history")
        .join(prompt_name)
}

fn history_file(project_dir: &Path, prompt_name: &str) -> PathBuf {
    history_dir(project_dir, prompt_name).join("history.json")
}

pub fn save_version(
    project_dir: &Path,
    prompt_name: &str,
    content: &str,
    annotation: Option<&str>,
) -> Result<u32, String> {
    let mut history = load_history(project_dir, prompt_name);

    let hash = format!("{:x}", md5_hash(content));

    // Skip if content unchanged
    if let Some(last) = history.entries.last() {
        if last.content_hash == hash {
            return Ok(last.version);
        }
    }

    let version = history.entries.last().map(|e| e.version + 1).unwrap_or(1);
    let entry = VersionEntry {
        version,
        timestamp: chrono::Utc::now().timestamp(),
        content_hash: hash,
        annotation: annotation.map(|s| s.to_string()),
        content: content.to_string(),
    };

    history.entries.push(entry);

    let dir = history_dir(project_dir, prompt_name);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create history dir: {}", e))?;
    let json = serde_json::to_string_pretty(&history)
        .map_err(|e| format!("Failed to serialize history: {}", e))?;
    std::fs::write(history_file(project_dir, prompt_name), json)
        .map_err(|e| format!("Failed to write history: {}", e))?;

    Ok(version)
}

pub fn load_history(project_dir: &Path, prompt_name: &str) -> VersionHistory {
    let path = history_file(project_dir, prompt_name);
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or(VersionHistory { entries: vec![] })
    } else {
        VersionHistory { entries: vec![] }
    }
}

pub fn get_version(project_dir: &Path, prompt_name: &str, version: u32) -> Option<VersionEntry> {
    let history = load_history(project_dir, prompt_name);
    history.entries.into_iter().find(|e| e.version == version)
}

fn md5_hash(content: &str) -> u64 {
    // Simple hash for deduplication — not cryptographic
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_version() {
        let dir = TempDir::new().unwrap();
        let v = save_version(dir.path(), "test", "content v1", None).unwrap();
        assert_eq!(v, 1);

        let history = load_history(dir.path(), "test");
        assert_eq!(history.entries.len(), 1);
        assert_eq!(history.entries[0].content, "content v1");
    }

    #[test]
    fn test_skip_duplicate_content() {
        let dir = TempDir::new().unwrap();
        save_version(dir.path(), "test", "same", None).unwrap();
        save_version(dir.path(), "test", "same", None).unwrap();

        let history = load_history(dir.path(), "test");
        assert_eq!(history.entries.len(), 1);
    }

    #[test]
    fn test_multiple_versions() {
        let dir = TempDir::new().unwrap();
        save_version(dir.path(), "test", "v1", None).unwrap();
        save_version(dir.path(), "test", "v2", Some("improved")).unwrap();
        save_version(dir.path(), "test", "v3", None).unwrap();

        let history = load_history(dir.path(), "test");
        assert_eq!(history.entries.len(), 3);
        assert_eq!(history.entries[1].annotation.as_deref(), Some("improved"));
    }

    #[test]
    fn test_get_specific_version() {
        let dir = TempDir::new().unwrap();
        save_version(dir.path(), "test", "v1", None).unwrap();
        save_version(dir.path(), "test", "v2", None).unwrap();

        let entry = get_version(dir.path(), "test", 1).unwrap();
        assert_eq!(entry.content, "v1");
    }
}
```

- [ ] **Step 3: Run tests**

Run:
```bash
cd src-tauri && cargo test version -- --nocapture
```
Expected: All tests pass

- [ ] **Step 4: Create version Tauri commands**

Create `src-tauri/src/commands/version.rs`:
```rust
use crate::version::{diff, store};
use std::path::PathBuf;

#[tauri::command]
pub fn save_prompt_version(
    project_dir: String,
    prompt_name: String,
    content: String,
    annotation: Option<String>,
) -> Result<u32, String> {
    store::save_version(
        &PathBuf::from(&project_dir),
        &prompt_name,
        &content,
        annotation.as_deref(),
    )
}

#[tauri::command]
pub fn get_version_history(
    project_dir: String,
    prompt_name: String,
) -> store::VersionHistory {
    store::load_history(&PathBuf::from(&project_dir), &prompt_name)
}

#[tauri::command]
pub fn diff_versions(old_content: String, new_content: String) -> diff::UnifiedDiff {
    diff::compute_diff(&old_content, &new_content)
}
```

Add `pub mod version;` to commands/mod.rs. Add `mod version;` to lib.rs. Register commands.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/version/ src-tauri/src/commands/version.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: add version history backend with diff support"
```

---

### Task 20: Version History Panel UI

**Files:**
- Create: `src/lib/stores/version.ts`
- Create: `src/lib/components/Panels/VersionHistory.svelte`
- Modify: `src/App.svelte` (integrate version panel, auto-save versions on save)

- [ ] **Step 1: Create version store and IPC**

Add to `src/lib/tauri.ts`:
```typescript
export async function savePromptVersion(
  projectDir: string, promptName: string, content: string, annotation?: string
): Promise<number> {
  return invoke("save_prompt_version", { projectDir, promptName, content, annotation: annotation ?? null });
}

export async function getVersionHistory(projectDir: string, promptName: string): Promise<VersionHistory> {
  return invoke("get_version_history", { projectDir, promptName });
}

export async function diffVersions(oldContent: string, newContent: string): Promise<UnifiedDiff> {
  return invoke("diff_versions", { oldContent, newContent });
}
```

Add types to `src/lib/types.ts`:
```typescript
export interface VersionEntry {
  version: number;
  timestamp: number;
  content_hash: string;
  annotation: string | null;
  content: string;
}

export interface VersionHistory {
  entries: VersionEntry[];
}

export interface UnifiedDiff {
  hunks: DiffHunk[];
}

export interface DiffHunk {
  old_start: number;
  new_start: number;
  lines: DiffLine[];
}

export interface DiffLine {
  tag: string;
  content: string;
}
```

Create `src/lib/stores/version.ts`:
```typescript
import { writable } from "svelte/store";
import type { VersionHistory, VersionEntry, UnifiedDiff } from "../types";
import { getVersionHistory, diffVersions } from "../tauri";

export const versionHistory = writable<VersionHistory>({ entries: [] });
export const selectedVersions = writable<[number | null, number | null]>([null, null]);
export const currentDiff = writable<UnifiedDiff | null>(null);

export async function refreshHistory(projectDir: string, promptName: string) {
  const history = await getVersionHistory(projectDir, promptName);
  versionHistory.set(history);
}

export async function computeDiff(oldContent: string, newContent: string) {
  const diff = await diffVersions(oldContent, newContent);
  currentDiff.set(diff);
}
```

- [ ] **Step 2: Create VersionHistory panel**

Create `src/lib/components/Panels/VersionHistory.svelte`:
```svelte
<script lang="ts">
  import { versionHistory, currentDiff, computeDiff } from "../../stores/version";
  import type { VersionEntry } from "../../types";

  let selectedA: VersionEntry | null = null;
  let selectedB: VersionEntry | null = null;

  function formatTime(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  function selectForDiff(entry: VersionEntry) {
    if (!selectedA) {
      selectedA = entry;
    } else if (!selectedB) {
      selectedB = entry;
      computeDiff(selectedA.content, selectedB.content);
    } else {
      selectedA = entry;
      selectedB = null;
      currentDiff.set(null);
    }
  }
</script>

<div class="version-history">
  <h3>Version History</h3>

  {#if $versionHistory.entries.length === 0}
    <p class="empty">No versions saved yet. Save the file to create the first version.</p>
  {:else}
    <p class="hint">Click two versions to compare.</p>
    <div class="timeline">
      {#each [...$versionHistory.entries].reverse() as entry}
        <div
          class="version-entry"
          class:selected-a={selectedA?.version === entry.version}
          class:selected-b={selectedB?.version === entry.version}
          on:click={() => selectForDiff(entry)}
        >
          <span class="version-num">v{entry.version}</span>
          <span class="version-time">{formatTime(entry.timestamp)}</span>
          {#if entry.annotation}
            <span class="version-note">{entry.annotation}</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if $currentDiff && $currentDiff.hunks.length > 0}
    <div class="diff-view">
      <h4>Diff: v{selectedA?.version} → v{selectedB?.version}</h4>
      {#each $currentDiff.hunks as hunk}
        <div class="diff-hunk">
          {#each hunk.lines as line}
            <div class="diff-line {line.tag}">
              <span class="diff-marker">
                {line.tag === "insert" ? "+" : line.tag === "delete" ? "-" : " "}
              </span>
              <span>{line.content}</span>
            </div>
          {/each}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .version-history { padding: 12px; overflow-y: auto; height: 100%; }
  h3 { margin: 0 0 8px; font-size: 14px; }
  h4 { margin: 12px 0 6px; font-size: 13px; }
  .empty, .hint { font-size: 12px; color: #888; }
  .timeline { display: flex; flex-direction: column; gap: 4px; }
  .version-entry {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 6px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }
  .version-entry:hover { background: #181825; }
  .version-entry.selected-a { border-left: 3px solid #f38ba8; }
  .version-entry.selected-b { border-left: 3px solid #89b4fa; }
  .version-num { font-weight: 600; color: #89b4fa; min-width: 30px; }
  .version-time { color: #888; }
  .version-note { color: #a6e3a1; font-style: italic; }
  .diff-view { margin-top: 8px; font-family: monospace; font-size: 12px; }
  .diff-hunk { border: 1px solid #313244; border-radius: 4px; margin-bottom: 4px; overflow: hidden; }
  .diff-line { display: flex; padding: 1px 8px; }
  .diff-line.insert { background: rgba(166, 227, 161, 0.1); color: #a6e3a1; }
  .diff-line.delete { background: rgba(243, 139, 168, 0.1); color: #f38ba8; }
  .diff-marker { width: 16px; flex-shrink: 0; }
</style>
```

- [ ] **Step 3: Integrate version history into App.svelte**

Update the save handler to auto-version:
```typescript
import { savePromptVersion } from "./lib/tauri";
import { refreshHistory } from "./lib/stores/version";

async function handleSave() {
  const file = $currentFile;
  if (!file) return;
  await savePrompt(file.path, $currentContent);
  // Auto-save version
  const projectDir = file.path.substring(0, file.path.lastIndexOf("/"));
  await savePromptVersion(projectDir, file.name, $currentContent);
  await refreshHistory(projectDir, file.name);
  isDirty.set(false);
}
```

Add a tab system in the right panel to toggle between Prompt Health and Version History.

- [ ] **Step 4: Verify version history works**

Run:
```bash
pnpm tauri dev
```
Manual test:
1. Open a prompt, make changes, save
2. Check version history panel shows v1
3. Edit again, save — v2 appears
4. Click v1 then v2 — diff view appears

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/version.ts src/lib/components/Panels/VersionHistory.svelte src/lib/tauri.ts src/lib/types.ts src/App.svelte
git commit -m "feat: add version history panel with visual diff"
```

---

## Chunk 6: MCP Polish & Final Integration

### Task 21: Full MCP Server with get_prompt_health Tool

**Files:**
- Modify: `src-tauri/src/mcp/tools.rs` (add get_prompt_health)
- Modify: `src-tauri/src/mcp/server.rs` (register new tool)
- Test: inline

- [ ] **Step 1: Write failing test for get_prompt_health**

Add to `src-tauri/src/mcp/tools.rs` tests:
```rust
#[test]
fn test_get_prompt_health() {
    let dir = TempDir::new().unwrap();
    let prompts_dir = dir.path().join("prompts");
    fs::create_dir_all(&prompts_dir).unwrap();
    fs::write(
        prompts_dir.join("bad.md"),
        "---\nname: bad\n---\n\nJust a long unstructured prompt without any XML tags at all. ".repeat(50).as_str(),
    ).unwrap();

    let state = McpState::new(prompts_dir);
    let health = get_prompt_health(&state, "bad").unwrap();
    assert!(health.contains("unstructured"));
}
```

Add function signature:
```rust
pub fn get_prompt_health(state: &McpState, name: &str) -> Result<String, String> {
    todo!()
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd src-tauri && cargo test mcp::tools::tests::test_get_prompt_health -- --nocapture
```

- [ ] **Step 3: Implement get_prompt_health**

```rust
pub fn get_prompt_health(state: &McpState, name: &str) -> Result<String, String> {
    let path = state.prompts_dir.join(format!("{}.md", name));
    let file = crate::commands::file::read_prompt_file(&path)?;
    let ast = crate::parser::parse(&file.content)?;
    let results = crate::linter::lint(&ast, &[]);
    Ok(serde_json::to_string_pretty(&results).unwrap())
}
```

- [ ] **Step 4: Register in MCP server tool list and handler**

Add to tools/list in `server.rs`:
```json
{
    "name": "get_prompt_health",
    "description": "Get linting results for a prompt",
    "inputSchema": {
        "type": "object",
        "properties": {
            "name": { "type": "string", "description": "Prompt name" }
        },
        "required": ["name"]
    }
}
```

Add handler in `handle_tool_call`:
```rust
"get_prompt_health" => {
    let name = params["arguments"]["name"].as_str()
        .ok_or(McpError { code: -32602, message: "Missing name".into() })?;
    let health = super::tools::get_prompt_health(state, name)
        .map_err(|e| McpError { code: -1, message: e })?;
    Ok(serde_json::json!({
        "content": [{ "type": "text", "text": health }]
    }))
}
```

- [ ] **Step 5: Run all tests**

Run:
```bash
cd src-tauri && cargo test -- --nocapture
```
Expected: All pass

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/mcp/
git commit -m "feat: add get_prompt_health MCP tool"
```

---

### Task 22: Claude Code Setup Helper and Status Bar MCP Indicator

**Files:**
- Modify: `src/lib/components/StatusBar.svelte` (add MCP port display)
- Modify: `src/lib/tauri.ts` (add get_mcp_port IPC)
- Modify: `src/App.svelte` (add MCP config copy button in toolbar)

- [ ] **Step 1: Add MCP port to status bar**

Add to `src/lib/tauri.ts`:
```typescript
export async function getMcpPort(): Promise<number | null> {
  return invoke("get_mcp_port");
}
```

Update `StatusBar.svelte` to show MCP status:
```svelte
<script lang="ts">
  import { currentContent, isDirty } from "../stores/editor";
  import { currentFile } from "../stores/files";
  import { getMcpPort } from "../tauri";
  import { onMount } from "svelte";

  let mcpPort: number | null = null;

  onMount(async () => {
    mcpPort = await getMcpPort();
  });

  $: charCount = $currentContent.length;
  $: tokenEstimate = Math.ceil(charCount / 4);
</script>

<div class="status-bar">
  <span class="file-name">
    {$currentFile?.name ?? "No file open"}
    {#if $isDirty}*{/if}
  </span>
  <span class="spacer"></span>
  <span class="mcp-status" class:connected={mcpPort !== null}>
    MCP: {mcpPort ? `localhost:${mcpPort}` : "not running"}
  </span>
  <span class="token-count">~{tokenEstimate} tokens</span>
  <span class="char-count">{charCount} chars</span>
</div>
```

- [ ] **Step 2: Add MCP config copy button**

Add to App.svelte toolbar a button that copies the Claude Code MCP config:
```typescript
async function copyMcpConfig() {
  const port = await getMcpPort();
  if (!port) return;
  const config = JSON.stringify({
    "mcpServers": {
      "claude-prompt-editor": {
        "url": `http://localhost:${port}/mcp`
      }
    }
  }, null, 2);
  await navigator.clipboard.writeText(config);
}
```

Add button to toolbar:
```svelte
<button on:click={copyMcpConfig} title="Copy MCP config for Claude Code">
  Copy MCP Config
</button>
```

- [ ] **Step 3: Verify MCP indicator and config copy**

Run:
```bash
pnpm tauri dev
```
Manual test:
1. Check status bar shows MCP port
2. Click "Copy MCP Config"
3. Paste into a text editor — should be valid JSON config

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/StatusBar.svelte src/lib/tauri.ts src/App.svelte
git commit -m "feat: add MCP status indicator and config copy button"
```

---

### Task 23: Variable Persistence and Final Polish

**Files:**
- Modify: `src-tauri/src/mcp/tools.rs` (persist variables to yaml)
- Modify: `src-tauri/Cargo.toml` (if not already added)

- [ ] **Step 1: Write test for variable persistence**

Add test to `src-tauri/src/mcp/tools.rs`:
```rust
#[test]
fn test_variable_persistence() {
    let dir = TempDir::new().unwrap();
    let state = McpState::new(dir.path().to_path_buf());

    set_variable(&state, "test", "lang", "Python");
    persist_variables(&state, dir.path()).unwrap();

    // Create new state and load
    let state2 = McpState::new(dir.path().to_path_buf());
    load_persisted_variables(&state2, dir.path()).unwrap();

    let vars = state2.variables.read().unwrap();
    assert_eq!(vars["test"]["lang"], "Python");
}
```

- [ ] **Step 2: Implement persistence functions**

Add to `src-tauri/src/mcp/tools.rs`:
```rust
pub fn persist_variables(state: &McpState, project_dir: &Path) -> Result<(), String> {
    let vars = state.variables.read().unwrap();
    let path = project_dir.join(".claude-prompts").join("variables.yaml");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let yaml = serde_yaml::to_string(&*vars).map_err(|e| e.to_string())?;
    std::fs::write(&path, yaml).map_err(|e| e.to_string())
}

pub fn load_persisted_variables(state: &McpState, project_dir: &Path) -> Result<(), String> {
    let path = project_dir.join(".claude-prompts").join("variables.yaml");
    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let loaded: HashMap<String, HashMap<String, String>> =
            serde_yaml::from_str(&content).map_err(|e| e.to_string())?;
        let mut vars = state.variables.write().unwrap();
        *vars = loaded;
    }
    Ok(())
}
```

- [ ] **Step 3: Run all tests**

Run:
```bash
cd src-tauri && cargo test -- --nocapture
```
Expected: All pass

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/mcp/tools.rs
git commit -m "feat: add variable persistence to .claude-prompts/variables.yaml"
```

- [ ] **Step 5: Final integration verification**

Run:
```bash
pnpm tauri dev
```
End-to-end test:
1. Create new prompt via wizard (select a template)
2. Edit in source mode — verify syntax highlighting
3. Switch to structure mode — verify blocks render
4. Edit a block in structure mode, switch back — verify sync
5. Check Prompt Health panel shows relevant lint results
6. Save — verify version history records v1
7. Edit and save again — v2 appears, diff works
8. Copy MCP config, verify port is shown
9. In a separate terminal, curl the MCP endpoint to verify it responds

- [ ] **Step 6: Final commit**

```bash
git add -A
git commit -m "feat: complete MVP of Claude Prompt Editor"
```

---
