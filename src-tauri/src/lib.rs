mod commands;
pub mod linter;
mod mcp;
pub mod parser;
pub mod preset;
pub mod version;

use commands::file;
use commands::mcp::McpServerState;
use std::path::PathBuf;
use tauri::Manager;

/// Default MCP server port. Can be overridden via MCP_PORT env var.
const DEFAULT_MCP_PORT: u16 = 9780;

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
    state.port.lock().ok().and_then(|guard| *guard)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(McpServerState::new())
        .setup(|app| {
            let mcp_server_state = app.state::<McpServerState>().inner().clone();

            // Prompts directory: use PROMPTS_DIR env var, or find prompts/ relative
            // to the project root. During `tauri dev`, cwd is src-tauri/, so we
            // walk up looking for a directory containing prompts/.
            let prompts_dir = std::env::var("PROMPTS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    // Check cwd, then parent, then grandparent for a prompts/ dir
                    let mut dir = cwd.as_path();
                    loop {
                        let candidate = dir.join("prompts");
                        if candidate.is_dir() {
                            break candidate;
                        }
                        match dir.parent() {
                            Some(parent) => dir = parent,
                            None => break cwd.join("prompts"), // fallback
                        }
                    }
                });

            // MCP port: use MCP_PORT env var, or default to 9780
            let mcp_port = std::env::var("MCP_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(DEFAULT_MCP_PORT);

            eprintln!("Prompts directory: {}", prompts_dir.display());

            tauri::async_runtime::spawn(async move {
                match commands::mcp::start_mcp(prompts_dir, mcp_port).await {
                    Ok(port) => {
                        eprintln!("MCP server started on http://localhost:{}/mcp", port);
                        if let Ok(mut p) = mcp_server_state.port.lock() {
                            *p = Some(port);
                        }
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
            commands::prompt::parse_prompt,
            commands::prompt::parse_content,
            commands::prompt::serialize_ast,
            commands::lint::lint_prompt,
            commands::preset::list_presets,
            commands::preset::list_templates,
            commands::version::save_prompt_version,
            commands::version::get_version_history,
            commands::version::diff_versions,
            commands::version::annotate_version,
            commands::version::restore_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
