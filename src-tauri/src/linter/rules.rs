use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Suggestion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub detail: String,
    pub block_index: Option<usize>,
    pub fix_suggestion: Option<String>,
}

pub trait LintRule: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn check(&self, ast: &crate::parser::ast::PromptAst) -> Vec<LintResult>;
}
