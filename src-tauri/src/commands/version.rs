use crate::version::diff::{self, DiffResult};
use crate::version::store::{self, VersionEntry, VersionHistory};
use std::path::PathBuf;

#[tauri::command]
pub fn save_prompt_version(
    project_dir: String,
    prompt_name: String,
    content: String,
    summary: Option<String>,
) -> Result<Option<VersionEntry>, String> {
    store::save_version(&PathBuf::from(project_dir), &prompt_name, &content, summary)
}

#[tauri::command]
pub fn get_version_history(project_dir: String, prompt_name: String) -> VersionHistory {
    store::load_history(&PathBuf::from(project_dir), &prompt_name)
}

#[tauri::command]
pub fn diff_versions(
    project_dir: String,
    prompt_name: String,
    old_version_id: u32,
    new_version_id: u32,
) -> Result<DiffResult, String> {
    let old = store::get_version(&PathBuf::from(&project_dir), &prompt_name, old_version_id)
        .ok_or_else(|| format!("Version {} not found", old_version_id))?;
    let new = store::get_version(&PathBuf::from(&project_dir), &prompt_name, new_version_id)
        .ok_or_else(|| format!("Version {} not found", new_version_id))?;

    Ok(diff::compute_diff(&old.content, &new.content))
}
