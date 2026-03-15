use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    pub id: u32,
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub content_hash: u64,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    pub prompt_name: String,
    pub versions: Vec<VersionEntry>,
    pub next_id: u32,
}

impl VersionHistory {
    pub fn new(prompt_name: &str) -> Self {
        Self {
            prompt_name: prompt_name.to_string(),
            versions: Vec::new(),
            next_id: 1,
        }
    }
}

fn hash_content(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
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

pub fn load_history(project_dir: &Path, prompt_name: &str) -> VersionHistory {
    let file = history_file(project_dir, prompt_name);
    if !file.exists() {
        return VersionHistory::new(prompt_name);
    }

    fs::read_to_string(&file)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| VersionHistory::new(prompt_name))
}

fn save_history_file(project_dir: &Path, history: &VersionHistory) -> Result<(), String> {
    let dir = history_dir(project_dir, &history.prompt_name);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create history dir: {}", e))?;

    let file = history_file(project_dir, &history.prompt_name);
    let json =
        serde_json::to_string_pretty(history).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&file, json).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

/// Save a new version snapshot. Returns None if the content is unchanged from the latest version.
pub fn save_version(
    project_dir: &Path,
    prompt_name: &str,
    content: &str,
    summary: Option<String>,
) -> Result<Option<VersionEntry>, String> {
    let mut history = load_history(project_dir, prompt_name);
    let content_hash = hash_content(content);

    // Skip if content unchanged
    if let Some(last) = history.versions.last() {
        if last.content_hash == content_hash {
            return Ok(None);
        }
    }

    let entry = VersionEntry {
        id: history.next_id,
        timestamp: Utc::now(),
        content: content.to_string(),
        content_hash,
        summary,
    };

    history.next_id += 1;
    history.versions.push(entry.clone());
    save_history_file(project_dir, &history)?;

    Ok(Some(entry))
}

/// Add or update an annotation on a specific version.
pub fn annotate_version(
    project_dir: &Path,
    prompt_name: &str,
    version_id: u32,
    annotation: &str,
) -> Result<(), String> {
    let mut history = load_history(project_dir, prompt_name);
    let entry = history
        .versions
        .iter_mut()
        .find(|v| v.id == version_id)
        .ok_or_else(|| format!("Version {} not found", version_id))?;
    entry.summary = Some(annotation.to_string());
    save_history_file(project_dir, &history)
}

pub fn get_version(project_dir: &Path, prompt_name: &str, version_id: u32) -> Option<VersionEntry> {
    let history = load_history(project_dir, prompt_name);
    history.versions.into_iter().find(|v| v.id == version_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn save_and_load() {
        let tmp = TempDir::new().unwrap();
        save_version(tmp.path(), "test-prompt", "Hello world", None).unwrap();

        let history = load_history(tmp.path(), "test-prompt");
        assert_eq!(history.versions.len(), 1);
        assert_eq!(history.versions[0].content, "Hello world");
        assert_eq!(history.versions[0].id, 1);
    }

    #[test]
    fn skip_duplicate() {
        let tmp = TempDir::new().unwrap();
        save_version(tmp.path(), "test-prompt", "Same content", None).unwrap();
        let result = save_version(tmp.path(), "test-prompt", "Same content", None).unwrap();
        assert!(result.is_none());

        let history = load_history(tmp.path(), "test-prompt");
        assert_eq!(history.versions.len(), 1);
    }

    #[test]
    fn multiple_versions() {
        let tmp = TempDir::new().unwrap();
        save_version(tmp.path(), "test-prompt", "Version 1", None).unwrap();
        save_version(tmp.path(), "test-prompt", "Version 2", None).unwrap();
        save_version(tmp.path(), "test-prompt", "Version 3", None).unwrap();

        let history = load_history(tmp.path(), "test-prompt");
        assert_eq!(history.versions.len(), 3);
        assert_eq!(history.versions[0].id, 1);
        assert_eq!(history.versions[2].id, 3);
        assert_eq!(history.versions[2].content, "Version 3");
    }

    #[test]
    fn get_specific_version() {
        let tmp = TempDir::new().unwrap();
        save_version(tmp.path(), "test-prompt", "V1", None).unwrap();
        save_version(tmp.path(), "test-prompt", "V2", None).unwrap();

        let v = get_version(tmp.path(), "test-prompt", 1);
        assert!(v.is_some());
        assert_eq!(v.unwrap().content, "V1");

        let v2 = get_version(tmp.path(), "test-prompt", 2);
        assert!(v2.is_some());
        assert_eq!(v2.unwrap().content, "V2");

        let v_missing = get_version(tmp.path(), "test-prompt", 99);
        assert!(v_missing.is_none());
    }

    #[test]
    fn annotate_version_updates_summary() {
        let tmp = TempDir::new().unwrap();
        save_version(tmp.path(), "test-prompt", "Content v1", None).unwrap();

        annotate_version(tmp.path(), "test-prompt", 1, "Initial version").unwrap();

        let v = get_version(tmp.path(), "test-prompt", 1).unwrap();
        assert_eq!(v.summary, Some("Initial version".to_string()));
    }

    #[test]
    fn annotate_version_not_found() {
        let tmp = TempDir::new().unwrap();
        let result = annotate_version(tmp.path(), "test-prompt", 99, "note");
        assert!(result.is_err());
    }
}
