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
    vars.insert((prompt_name.to_string(), key.to_string()), value.to_string());
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
}
