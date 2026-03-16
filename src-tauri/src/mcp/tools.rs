use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::commands::file;

#[derive(Clone)]
pub struct McpState {
    pub prompts_dir: PathBuf,
    /// Variables keyed by (prompt_name, variable_key) -> value
    pub variables: Arc<RwLock<HashMap<(String, String), String>>>,
}

impl McpState {
    pub fn new(prompts_dir: PathBuf) -> Self {
        Self {
            prompts_dir,
            variables: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Replace `{{key}}` patterns in content with values from vars.
/// Keys not found in vars are left as-is.
pub fn interpolate_variables(content: &str, vars: &HashMap<String, String>) -> String {
    crate::parser::variables::interpolate(content, vars)
}

/// Strip YAML frontmatter (lines between opening and closing `---`) from content.
pub fn strip_frontmatter(content: &str) -> String {
    let (_metadata, body, _raw) = crate::parser::frontmatter::parse_frontmatter(content);
    body
}

/// Load a prompt by name, strip frontmatter, and interpolate variables.
pub async fn load_prompt(state: &McpState, name: &str) -> Result<String, String> {
    let path = state.prompts_dir.join(format!("{}.md", name));
    let pf = file::read_prompt_file(&path)?;
    let content = strip_frontmatter(&pf.content);

    // Collect variables for this prompt
    let vars_lock = state.variables.read().await;
    let vars: HashMap<String, String> = vars_lock
        .iter()
        .filter(|((pname, _), _)| pname == name)
        .map(|((_, key), val)| (key.clone(), val.clone()))
        .collect();

    Ok(interpolate_variables(&content, &vars))
}

/// List all prompts in the prompts directory.
pub async fn list_prompts(state: &McpState) -> Result<Vec<file::PromptListEntry>, String> {
    file::list_prompt_files(&state.prompts_dir)
}

/// Set a variable for a specific prompt.
pub async fn set_variable(state: &McpState, prompt_name: &str, key: &str, value: &str) {
    let mut vars = state.variables.write().await;
    vars.insert(
        (prompt_name.to_string(), key.to_string()),
        value.to_string(),
    );
}

/// Run the linter on a prompt and return health results as JSON.
pub fn get_prompt_health(state: &McpState, name: &str) -> Result<String, String> {
    let path = state.prompts_dir.join(format!("{}.md", name));
    let file = crate::commands::file::read_prompt_file(&path)?;
    let ast = crate::parser::parse(&file.content)?;
    let results = crate::linter::lint(&ast, &[]);
    Ok(serde_json::to_string_pretty(&results).unwrap())
}

/// Persist variables to `<project_dir>/.claude-prompts/variables.yaml`.
#[allow(dead_code)]
pub fn persist_variables(state: &McpState, project_dir: &std::path::Path) -> Result<(), String> {
    let dir = project_dir.join(".claude-prompts");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create .claude-prompts dir: {}", e))?;
    let path = dir.join("variables.yaml");

    // Build a nested map: { prompt_name -> { key -> value } }
    let vars = state
        .variables
        .try_read()
        .map_err(|e| format!("Lock error: {}", e))?;
    let mut nested: std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>> =
        std::collections::BTreeMap::new();
    for ((pname, key), value) in vars.iter() {
        nested
            .entry(pname.clone())
            .or_default()
            .insert(key.clone(), value.clone());
    }

    let yaml = serde_yaml::to_string(&nested)
        .map_err(|e| format!("Failed to serialize variables: {}", e))?;
    std::fs::write(&path, yaml).map_err(|e| format!("Failed to write variables file: {}", e))?;
    Ok(())
}

/// Load persisted variables from `<project_dir>/.claude-prompts/variables.yaml`.
#[allow(dead_code)]
pub fn load_persisted_variables(
    state: &McpState,
    project_dir: &std::path::Path,
) -> Result<(), String> {
    let path = project_dir.join(".claude-prompts").join("variables.yaml");
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read variables file: {}", e))?;
    let nested: std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>> =
        serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse variables YAML: {}", e))?;

    let mut vars = state
        .variables
        .try_write()
        .map_err(|e| format!("Lock error: {}", e))?;
    for (pname, keys) in nested {
        for (key, value) in keys {
            vars.insert((pname.clone(), key), value);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_interpolate_variables_basic() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("role".to_string(), "engineer".to_string());
        let result = interpolate_variables("Hello {{name}}, you are a {{role}}.", &vars);
        assert_eq!(result, "Hello Alice, you are a engineer.");
    }

    #[test]
    fn test_interpolate_variables_missing_kept() {
        let vars = HashMap::new();
        let result = interpolate_variables("Hello {{name}}, welcome.", &vars);
        assert_eq!(result, "Hello {{name}}, welcome.");
    }

    #[tokio::test]
    async fn test_load_prompt_basic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("greeting.md");
        std::fs::write(
            &path,
            "---\nname: greeting\nmodel: claude\n---\nHello {{user}}, welcome to {{place}}.",
        )
        .unwrap();

        let state = McpState::new(dir.path().to_path_buf());
        set_variable(&state, "greeting", "user", "Bob").await;
        set_variable(&state, "greeting", "place", "the lab").await;

        let result = load_prompt(&state, "greeting").await.unwrap();
        assert_eq!(result, "Hello Bob, welcome to the lab.");
    }

    #[tokio::test]
    async fn test_load_prompt_not_found() {
        let dir = TempDir::new().unwrap();
        let state = McpState::new(dir.path().to_path_buf());
        let result = load_prompt(&state, "nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_and_get_variable() {
        let dir = TempDir::new().unwrap();
        let state = McpState::new(dir.path().to_path_buf());
        set_variable(&state, "my_prompt", "key1", "value1").await;
        set_variable(&state, "my_prompt", "key2", "value2").await;

        let vars = state.variables.read().await;
        assert_eq!(
            vars.get(&("my_prompt".to_string(), "key1".to_string())),
            Some(&"value1".to_string())
        );
        assert_eq!(
            vars.get(&("my_prompt".to_string(), "key2".to_string())),
            Some(&"value2".to_string())
        );
    }

    #[test]
    fn test_get_prompt_health_no_role() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("norole.md");
        // A prompt with no <role> block should trigger missing-role lint finding
        std::fs::write(
            &path,
            "---\nname: norole\n---\nJust some instructions here without a role.",
        )
        .unwrap();

        let state = McpState::new(dir.path().to_path_buf());
        let result = get_prompt_health(&state, "norole").unwrap();
        // Result should be valid JSON array
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_array());
        // Should contain a missing-role finding
        let findings = parsed.as_array().unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.get("rule_id").and_then(|v| v.as_str()) == Some("missing-role")),
            "Expected missing-role finding in: {}",
            result
        );
    }

    #[test]
    fn test_get_prompt_health_not_found() {
        let dir = TempDir::new().unwrap();
        let state = McpState::new(dir.path().to_path_buf());
        let result = get_prompt_health(&state, "nonexistent");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_persist_and_load_variables() {
        let dir = TempDir::new().unwrap();
        let state = McpState::new(dir.path().to_path_buf());

        set_variable(&state, "prompt_a", "lang", "Rust").await;
        set_variable(&state, "prompt_a", "style", "concise").await;
        set_variable(&state, "prompt_b", "topic", "testing").await;

        persist_variables(&state, dir.path()).unwrap();

        // Verify file was created
        let yaml_path = dir.path().join(".claude-prompts").join("variables.yaml");
        assert!(yaml_path.exists());

        // Create a fresh state and load
        let state2 = McpState::new(dir.path().to_path_buf());
        load_persisted_variables(&state2, dir.path()).unwrap();

        let vars = state2.variables.read().await;
        assert_eq!(
            vars.get(&("prompt_a".to_string(), "lang".to_string())),
            Some(&"Rust".to_string())
        );
        assert_eq!(
            vars.get(&("prompt_a".to_string(), "style".to_string())),
            Some(&"concise".to_string())
        );
        assert_eq!(
            vars.get(&("prompt_b".to_string(), "topic".to_string())),
            Some(&"testing".to_string())
        );
    }

    #[test]
    fn test_load_persisted_variables_no_file() {
        let dir = TempDir::new().unwrap();
        let state = McpState::new(dir.path().to_path_buf());
        // Should succeed silently when no file exists
        let result = load_persisted_variables(&state, dir.path());
        assert!(result.is_ok());
    }
}
