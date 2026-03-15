# Contributing to Claude Prompt Editor

Thanks for your interest in contributing! This project is in early development and contributions are welcome.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (18+)
- [pnpm](https://pnpm.io/) (9+)
- Linux: `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libcairo2-dev`, `libpango1.0-dev`, `libgdk-pixbuf-2.0-dev`, `librsvg2-dev`
- Mac: Xcode Command Line Tools

### Setup

```bash
git clone git@github.com:midwire/claude-prompt-editor.git
cd claude-prompt-editor
pnpm install
pnpm tauri dev
```

### Running Tests

```bash
cd src-tauri
cargo test --lib
```

### Building

```bash
# Frontend only
pnpm build

# Full app binary
pnpm tauri build
```

## What to Work On

- Check [open issues](https://github.com/midwire/claude-prompt-editor/issues) for bugs and feature requests
- Issues labeled `good first issue` are a great starting point
- If you want to work on something not listed, open an issue first to discuss

## Submitting Changes

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Ensure `cargo test --lib` passes (all tests must pass)
4. Ensure `pnpm build` succeeds
5. Open a pull request with a clear description of what you changed and why

## Project Structure

- **Rust backend** (`src-tauri/src/`) — parser, linter, MCP server, version history, presets
- **Svelte frontend** (`src/`) — editor UI, stores, components
- See [CLAUDE.md](CLAUDE.md) for detailed architecture notes

## Guidelines

- The frontend uses **Svelte 5** syntax (`$state`, `$derived`, `$props`, `onclick`)
- Rust tests use inline `#[cfg(test)] mod tests` — keep tests next to the code they test
- Prompt files use canonical model IDs: `claude-opus-4-6`, `claude-sonnet-4-6`, `claude-haiku-4-5`
- The MCP server defaults to port 9780

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
