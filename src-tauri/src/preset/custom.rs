use super::builtin::Preset;
use std::fs;
use std::path::{Path, PathBuf};

fn presets_dir(project_dir: &Path) -> PathBuf {
    project_dir.join(".claude-prompts").join("presets")
}

pub fn save_custom_preset(project_dir: &Path, preset: &Preset) -> Result<(), String> {
    let dir = presets_dir(project_dir);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create presets dir: {}", e))?;

    let file_path = dir.join(format!("{}.json", preset.id));
    let json =
        serde_json::to_string_pretty(preset).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&file_path, json).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

pub fn load_custom_presets(project_dir: &Path) -> Vec<Preset> {
    let dir = presets_dir(project_dir);
    if !dir.exists() {
        return Vec::new();
    }

    let mut presets = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(preset) = serde_json::from_str::<Preset>(&content) {
                        presets.push(preset);
                    }
                }
            }
        }
    }
    presets
}

pub fn delete_custom_preset(project_dir: &Path, preset_id: &str) -> Result<(), String> {
    let file_path = presets_dir(project_dir).join(format!("{}.json", preset_id));
    if file_path.exists() {
        fs::remove_file(&file_path).map_err(|e| format!("Delete error: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::builtin::PresetCategory;
    use tempfile::TempDir;

    fn test_preset() -> Preset {
        Preset {
            id: "custom-test".into(),
            name: "Test Preset".into(),
            category: PresetCategory::Instructions,
            content: "Test content".into(),
            tag_name: None,
            metadata_defaults: None,
        }
    }

    #[test]
    fn save_and_load_custom_preset() {
        let tmp = TempDir::new().unwrap();
        let preset = test_preset();
        save_custom_preset(tmp.path(), &preset).unwrap();

        let loaded = load_custom_presets(tmp.path());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "custom-test");
        assert_eq!(loaded[0].content, "Test content");
    }

    #[test]
    fn delete_custom_preset_removes_file() {
        let tmp = TempDir::new().unwrap();
        let preset = test_preset();
        save_custom_preset(tmp.path(), &preset).unwrap();

        delete_custom_preset(tmp.path(), "custom-test").unwrap();
        let loaded = load_custom_presets(tmp.path());
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn load_from_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let loaded = load_custom_presets(tmp.path());
        assert!(loaded.is_empty());
    }
}
