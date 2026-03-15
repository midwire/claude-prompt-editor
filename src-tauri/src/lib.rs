mod commands;
pub mod linter;
mod mcp;
pub mod parser;

use commands::file;
use commands::mcp::McpServerState;
use std::path::PathBuf;
use tauri::Manager;

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
        .manage(McpServerState::new())
        .setup(|app| {
            let mcp_server_state = app.state::<McpServerState>().inner().clone();
            let prompts_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("prompts");

            tauri::async_runtime::spawn(async move {
                match commands::mcp::start_mcp(prompts_dir, 0).await {
                    Ok(port) => {
                        eprintln!("MCP server started on port {}", port);
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
