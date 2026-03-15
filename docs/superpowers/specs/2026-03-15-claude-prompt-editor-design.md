# Claude Prompt Editor/IDE - Design Specification

## Overview

A lightweight desktop IDE for Linux and Mac (Windows as a future target) that makes it easy and intuitive to engineer prompts for the Claude API following Anthropic's best practices. Built with Tauri 2.0 (Rust backend, Svelte frontend), distributed as a single self-contained binary.

**Primary users:** Developers building applications with the Claude API (system prompts, tool definitions, structured outputs).

**Core value proposition:** Combines structured prompt authoring, best-practices guidance, and seamless Claude Code integration so developers can craft, validate, and test prompts without leaving their workflow or being forced onto API billing.

## Architecture

### Three-Layer Design

```
┌─────────────────────────────────────┐
│           Svelte Frontend           │
│  (Monaco editor, block editor UI,   │
│   linting panel, version history)   │
├─────────────────────────────────────┤
│          Tauri IPC Bridge           │
│     (commands, events, state)       │
├─────────────────────────────────────┤
│           Rust Backend              │
│  (file I/O, markdown parser, AST,  │
│   linter, version store, MCP srv)  │
└─────────────────────────────────────┘
```

**Rust backend** handles: file I/O, markdown parsing/serialization, prompt AST management, version history storage, best-practices linting engine, and the MCP server.

**Tauri IPC bridge** connects backend to frontend via Tauri's command system.

**Svelte frontend** provides: dual-mode editor UI, structure panel, linting display, version history viewer, preset palette, and template wizard.

### Single Binary Distribution

Tauri compiles everything into one binary. No separate backend/frontend launch. User runs the app and everything starts together: editor UI, file system operations, MCP server.

**Distribution:**
- Linux: `.deb` / `.AppImage`
- Mac: `.dmg`
- Windows (future): `.msi` / `.exe`

### Cross-Platform Considerations

Windows support is a future target. To minimize effort when adding it:
- Use `std::path` consistently in Rust (never hardcode path separators)
- Abstract any shell/terminal integration behind a platform trait
- Test MCP server port binding across platforms
- Account for Claude Code running via WSL on Windows

## File Format

Prompts are stored as Markdown files with YAML frontmatter.

```markdown
---
name: "Code Review Assistant"
model: claude-opus-4-6
version: 3
tags: [coding, review]
thinking:
  type: adaptive
effort: high
---

<role>
You are an expert code reviewer specializing in {{language}}.
</role>

<instructions>
Review the provided code for:
1. Security vulnerabilities
2. Performance issues
3. Readability
</instructions>

<examples>
<example>
<input>def foo(x): return eval(x)</input>
<output>CRITICAL: eval() with user input is a code injection vulnerability...</output>
</example>
</examples>
```

**Frontmatter** captures API parameters and editor metadata (name, model, version, tags, thinking config, effort).

**Body** is the actual prompt text sent to Claude. XML tags structure the content. Variable interpolation via `{{variable_name}}` is supported for reusable prompts.

### Project Structure on Disk

```
my-prompts/
  .claude-prompts/          # editor metadata
    history/                # version history (diffs)
    presets/                # user-defined presets
    templates/              # project-specific templates
    lintrc.yaml             # linting rule configuration
  prompts/
    code-review.md
    data-analyst.md
    agentic-search.md
```

## Dual-Mode Editor

The editor presents two synced views of the same prompt, sharing a common AST (abstract syntax tree) as the source of truth.

### Structure Mode (Block Editor)

The prompt is parsed into typed, collapsible blocks displayed as cards:

- **Metadata block** - rendered from frontmatter as editable form fields (model, effort, thinking config)
- **Role block** - content within `<role>` tags
- **Instructions block** - content within `<instructions>` or similar tags
- **Examples block** - collapsible container of individual `<example>` sub-blocks with add/remove/reorder
- **Context block** - `<context>`, `<documents>`, or custom tagged sections
- **Custom block** - any XML-tagged section (e.g., `<constraints>`, `<output_format>`)
- **Freeform block** - untagged prose between tagged sections

Each block has:
- Type indicator with icon
- Collapse/expand toggle
- Drag handle for reordering
- Enable/disable toggle (disabled blocks excluded from serialized output, preserved as comments in file)
- Contextual best-practice hints based on block type

### Source Mode (Code Editor)

Monaco-based editor with:
- Syntax highlighting for XML tags, YAML frontmatter, and `{{variable}}` interpolation
- Tag auto-closing and matching (highlight closing tag when cursor is on opening tag)
- Autocomplete for common tag names (`<instructions>`, `<example>`, `<documents>`, etc.)
- Minimap-style structure outline in sidebar showing block hierarchy

### Sync Behavior

- Edits in either mode update the shared AST
- The AST is the single source of truth; both views render from it
- Parsing is fault-tolerant: malformed XML doesn't break structure mode, those sections render as freeform blocks
- Live token counter in the status bar updates as you edit

## Best Practices Engine (Linting)

The linter runs against the prompt AST and surfaces guidance inline in the editor and in a dedicated "Prompt Health" panel.

### Structural Rules

- **Missing role definition** - no `<role>` or system role text detected
- **Sparse examples** - fewer than 3 examples in an examples block
- **Unstructured long prompt** - exceeds ~500 tokens with no XML tags
- **Long context layout** - documents/data appear below instructions (should be above)
- **Unbalanced XML** - mismatched or unclosed tags
- **Vague instructions** - heuristic detection of non-specific phrases like "do a good job"

### Anti-Pattern Detection

- **Negative framing** - "Don't use markdown" → suggest positive framing
- **Over-prompting for 4.6** - aggressive emphasis like "CRITICAL: You MUST always" → suggest normal prompting
- **Deprecated patterns** - prefilled response patterns, manual CoT when thinking is available
- **Missing context/motivation** - bare instructions without explaining why

### Prompt Health Panel

- Overall score (green/yellow/red)
- Categorized findings: errors, warnings, suggestions
- Each finding links to the relevant prompt section and has expandable Anthropic guidance
- One-click fixes where applicable (e.g., "Wrap in XML tags")

### Configuration

Rules are configurable via `.claude-prompts/lintrc.yaml` - disable rules, set severity levels, share across a team.

## Preset and Snippet Library

### Role Presets

Searchable palette of curated roles:
- "Expert code reviewer specializing in {{language}}"
- "Data analyst with access to {{tools}}"
- "Technical writer for {{audience}}"
- "QA engineer focused on edge cases"

Each role preset includes recommended companion settings (e.g., a coding role defaults to `thinking: adaptive, effort: high`).

### Section Presets

Pre-built blocks for common patterns:
- **Output format blocks** - JSON-only, markdown-free prose, structured XML output
- **Constraint blocks** - "investigate before answering", "default to action", "minimize overengineering"
- **Example skeletons** - classification, extraction, Q&A patterns
- **Tool definition templates** - common tool patterns

### Variable Presets

For `{{variables}}`, a palette of common variable types with sensible defaults and descriptions (language, audience, tone, format).

### Custom Presets

- Save any block or combination of blocks as a reusable preset
- Stored in `.claude-prompts/presets/` as markdown snippets (shareable, version-controllable)

### Interaction

- Structure mode: adding a new block offers "Empty" or "From preset..." opening the searchable palette
- Source mode: keyboard shortcut (Ctrl+Shift+P) opens the palette and inserts at cursor

## Version History

### Auto-Versioning

- Every save creates a version snapshot in `.claude-prompts/history/<prompt-name>/`
- Snapshots are lightweight diffs (not full copies)
- Each version records: timestamp, optional user annotation, content hash

### Version History Panel

- Timeline view of all versions for the current prompt
- Click any version to preview in read-only editor
- Visual diff between any two versions (side-by-side or inline, additions/deletions highlighted)
- Annotations - add notes to versions (e.g., "reduced hallucinations by adding grounding instructions")
- Restore any previous version (creates a new version, doesn't destroy history)

### Relationship to Git

- Version history is independent of git - works before you commit
- Prompt files are normal markdown files in your repo
- `.claude-prompts/history/` can be gitignored (ephemeral) or committed (shared) - user's choice
- The editor supplements git for the rapid iteration phase, doesn't replace it

## MCP Server Integration

### Built-In Server

The MCP server starts automatically with the editor, running on a local port.

### Tools Exposed to Claude Code

- **`load_prompt(name)`** - Returns current prompt content, fully rendered (variables interpolated, disabled blocks excluded). Primary injection mechanism.
- **`list_prompts()`** - Lists all prompts in current project with name, description, last modified.
- **`get_prompt_health(name)`** - Returns linting results for a prompt, allowing Claude Code to suggest improvements.
- **`set_variable(name, key, value)`** - Sets a variable value for a prompt so the next `load_prompt` returns the interpolated version.

### Claude Code Setup

- Editor generates the MCP config snippet for `~/.claude/claude_code_config.json`
- One-click copy or auto-install on first launch (detects if Claude Code is installed)
- Connection status indicator in editor's status bar

### Workflow

1. Edit prompt in editor
2. In Claude Code: "Load the code-review prompt and use it to review this file"
3. Claude Code calls `load_prompt("code-review")` via MCP
4. Claude receives the prompt and applies it
5. Iterate in editor, call `load_prompt` again for updated version

**No API key needed.** The MCP server only serves prompt content. All LLM interaction happens through the user's existing Claude Code subscription (Max/Pro plan).

## Testing and Validation

Testing works at two levels:

### In-Editor Validation (No API)

- Linting engine validates structure, best practices, and anti-patterns
- XML tag balance checking
- Token count estimation
- Variable interpolation preview (see the fully rendered prompt)

### Live Testing via Claude Code

- Inject prompts via MCP `load_prompt` tool
- Test using existing Claude Code subscription (Max/Pro plan)
- No API billing required

### Future Enhancements (Post-MVP)

- **Test suite** - define multiple test inputs with expected behavior/criteria, run via Claude Code, track results
- **Conversation simulator** - multi-turn testing through Claude Code
- **Version-to-results tracking** - correlate prompt versions with test outcomes

## Templates

### New Prompt Wizard

When creating a prompt, choose a starting point.

**Blank templates (structural):**
- **Minimal** - frontmatter + single instruction block
- **Standard** - role, instructions, output format, 1 example skeleton
- **Full** - role, context, instructions, examples (3), constraints, output format

**Use-case templates:**
- **Agentic system** - role, tool usage guidance, autonomy/safety guardrails, state tracking
- **Code assistant** - role with language variable, coding conventions, example I/O pairs
- **Data extraction** - role, document context, output schema, grounding instructions
- **Classification** - role, label definitions, diverse few-shot examples, structured output
- **Research/RAG** - role, indexed document structure, citation instructions
- **Conversational** - role with personality, conversation style, multi-turn handling

### Template Properties

- Each template is a valid `.md` prompt file (inspectable/editable)
- Ships with annotations explaining why each section exists (shown as hints in structure mode, stripped from output)
- Pre-populates sensible frontmatter defaults for the use case
- Includes placeholder variables ready to fill in

### Custom Templates

- Save any prompt as a template
- Project templates: `.claude-prompts/templates/`
- Global templates: `~/.config/claude-prompt-editor/templates/`
- Community template repository: out of scope for v1

## Technology Stack

### Backend (Rust)

- **Tauri 2.0** - app framework, window management, IPC
- **pulldown-cmark** (or similar) - markdown parsing
- **Custom XML parser** - lightweight, fault-tolerant tag extraction (not full XML compliance)
- **serde + serde_yaml** - frontmatter serialization
- **rmcp** (or custom) - MCP server protocol
- **similar** - diff algorithm for version history

### Frontend

- **Svelte** - UI framework (lightweight, reactive, Tauri-recommended)
- **Monaco Editor** - source mode code editor
- **Custom Svelte components** - block editor, drag-and-drop (HTML5 drag API)
- **Custom CSS** - no heavy UI framework, lean design system

### Why Svelte Over React

- Smaller bundle size (matters for system webview)
- Less boilerplate for reactive state (critical for dual-mode sync)
- Tauri's official templates and docs favor Svelte

### Build/Dev

- Cargo for Rust backend
- Vite for frontend bundling
- pnpm for JS dependencies

## Scope Summary

### MVP (v1)

- Tauri app with single-binary distribution (Linux + Mac)
- Dual-mode editor (structure + source) with AST sync
- Markdown file format with YAML frontmatter
- Best practices linter with prompt health panel
- Curated preset/snippet library (roles, sections, variables)
- Custom preset creation
- Built-in version history with visual diff
- MCP server for Claude Code integration
- Template wizard with blank and use-case starters
- In-editor validation (no API required)

### Future (v2+)

- Windows support
- Test suite with multiple test cases
- Conversation simulator for multi-turn testing
- Version-to-test-results tracking
- Community template repository
- Plugin system for custom linting rules
