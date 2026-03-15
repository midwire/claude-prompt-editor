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

pub fn read_prompt_file(path: &Path) -> Result<PromptFile, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("untitled").to_string();
    Ok(PromptFile { path: path.to_path_buf(), name, content })
}

pub fn write_prompt_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    std::fs::write(path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
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
            let modified = metadata.modified().map_err(|e| format!("Failed to read modified time: {}", e))?
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("untitled").to_string();
            entries.push(PromptListEntry { path, name, modified });
        }
    }
    entries.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(entries)
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
