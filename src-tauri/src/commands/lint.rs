use crate::linter;
use crate::linter::rules::LintResult;
use std::path::PathBuf;

#[tauri::command]
pub fn lint_prompt(
    content: String,
    project_dir: Option<String>,
) -> Result<Vec<LintResult>, String> {
    let ast = crate::parser::parse(&content)?;
    let config = project_dir
        .map(|d| linter::config::load_config(&PathBuf::from(d)))
        .unwrap_or_default();
    Ok(linter::lint(&ast, &config.disabled_rules))
}
