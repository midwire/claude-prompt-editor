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
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_prompt,
            save_prompt,
            list_prompts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
