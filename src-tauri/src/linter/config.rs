use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LintConfig {
    #[serde(default)]
    pub disabled_rules: Vec<String>,
    #[serde(default)]
    pub severity_overrides: HashMap<String, String>,
}

/// Load lint config from `.claude-prompts/lintrc.yaml` in the given project directory.
/// Returns default config if the file doesn't exist or can't be parsed.
pub fn load_config(project_dir: &Path) -> LintConfig {
    let config_path = project_dir.join(".claude-prompts").join("lintrc.yaml");
    match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            serde_yaml::from_str(&content).unwrap_or_default()
        }
        Err(_) => LintConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_config_returns_default_when_missing() {
        let dir = std::path::PathBuf::from("/tmp/nonexistent-lint-dir");
        let config = load_config(&dir);
        assert!(config.disabled_rules.is_empty());
        assert!(config.severity_overrides.is_empty());
    }

    #[test]
    fn load_config_reads_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join(".claude-prompts");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("lintrc.yaml"),
            "disabled_rules:\n  - missing-role\n  - sparse-examples\nseverity_overrides:\n  over-prompting: Error\n",
        ).unwrap();

        let config = load_config(dir.path());
        assert_eq!(config.disabled_rules, vec!["missing-role", "sparse-examples"]);
        assert_eq!(config.severity_overrides.get("over-prompting").unwrap(), "Error");
    }
}
