## What This Is

A Tauri 2.0 desktop IDE for crafting Claude API prompts. Rust backend, Svelte 5 frontend, Monaco editor. Includes a built-in MCP server so Claude Code can load prompts directly.

## Build & Dev Commands

```bash
# Run the app (starts both frontend dev server and Tauri backend)
pnpm tauri dev

# Frontend only (no Tauri shell)
pnpm build

# Rust tests (run from src-tauri/)
cd src-tauri && cargo test --lib

# Single test
cd src-tauri && cargo test parser::xml_tags::tests::test_find_tags_basic -- --nocapture

# Rust build only
cd src-tauri && cargo build
```

**Note:** During `pnpm tauri dev`, the Rust binary's cwd is `src-tauri/`, not the project root.

## Architecture

Three-layer Tauri app:

**Rust Backend (`src-tauri/src/`)** — all business logic:
- `parser/` — Prompt file → AST. Frontmatter YAML extraction, fault-tolerant XML tag parsing, variable interpolation (`{{var}}`), AST-to-markdown serializer. The `PromptAst` struct is the core data type.
- `linter/` — Runs rules against `PromptAst`. Rules implement the `LintRule` trait. Structural rules (missing role, sparse examples) and anti-pattern rules (negative framing, over-prompting, vague instructions).
- `mcp/` — HTTP JSON-RPC server on port 9780 (configurable via `MCP_PORT` env var). Tools: `load_prompt`, `list_prompts`, `set_variable`, `get_prompt_health`. Runs on axum, starts automatically with the app.
- `commands/` — Tauri IPC commands bridging Rust to the frontend. Each file maps to a domain (file, prompt, lint, preset, version, mcp).
- `preset/` — Built-in presets (roles, constraints, output formats) and templates (blank + use-case starters).
- `version/` — Version history stored as JSON in `.claude-prompts/history/`. Uses `similar` crate for text diffs.

**Svelte 5 Frontend (`src/`)** — UI only:
- `lib/stores/` — Svelte writable stores. `prompt.ts` is central: holds `currentAst`, manages source↔structure sync with `syncAstToContent()` (serializes AST, sets content, skips re-parse via `skipNextParse` guard).
- `lib/components/Editor/` — SourceEditor (Monaco wrapper with `prompt-md` language), StructureEditor (renders AST blocks), EditorTabs (mode toggle).
- `lib/components/Blocks/` — Block.svelte (base wrapper with collapse/enable/delete), TaggedBlock, FreeformBlock, ExamplesBlock, MetadataBlock.
- `lib/tauri.ts` — Typed `invoke()` wrappers for all Tauri commands.
- `lib/types.ts` — TypeScript types mirroring Rust AST types. `BlockKind` is a union matching Rust's serde-serialized enum.

**IPC Bridge** — Tauri `invoke()` calls. Frontend calls `parseContent(string)` → Rust parses → returns `PromptAst` as JSON. Frontend calls `serializeAst(ast)` → Rust serializes → returns markdown string.

## Key Design Decisions

**Dual-mode sync:** Source mode and structure mode share a `PromptAst`. Edits in source mode trigger debounced re-parsing. Edits in structure mode call `syncAstToContent()` which serializes the AST to markdown and sets `skipNextParse = true` to prevent the content change from triggering a redundant re-parse that would lose AST-only state (like the `enabled` flag on blocks).

**Block enable/disable:** The `enabled` flag lives only in the AST, not encoded in the markdown. Disabled blocks are serialized normally to the file. `serialize_enabled_only()` exists for the MCP use case where disabled blocks should be excluded.

**Frontmatter:** The serializer generates YAML from the `PromptMetadata` struct (not from `raw_frontmatter`). This ensures metadata changes in structure mode are persisted.

**Prompts directory:** Resolved at startup by walking up from cwd to find a `prompts/` directory. Override with `PROMPTS_DIR` env var.

## Svelte 5 Syntax

This project uses Svelte 5. Use `$state()`, `$derived()`, `$effect()`, `$props()`, `onclick` (not `on:click`), `{@render children()}` for slots. Stores from `svelte/store` work with `$storeName` auto-subscription in `.svelte` files.

## File Format

Prompts are markdown with YAML frontmatter:
```markdown
---
name: "My Prompt"
model: claude-opus-4-6
version: 1
tags: [coding]
thinking:
  type: adaptive
effort: high
---

<role>
You are a {{language}} expert.
</role>

<instructions>
Do the thing.
</instructions>
```

Canonical model IDs: `claude-opus-4-6`, `claude-sonnet-4-6`, `claude-haiku-4-5`.

## Environment Variables

- `MCP_PORT` — MCP server port (default: 9780)
- `PROMPTS_DIR` — Absolute path to prompts directory
