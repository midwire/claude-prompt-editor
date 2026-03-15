pub mod antipatterns;
pub mod config;
pub mod rules;
pub mod structural;

use crate::parser::ast::PromptAst;
use rules::{LintResult, LintRule};

/// Run all enabled lint rules against the given AST.
pub fn lint(ast: &PromptAst, disabled_rules: &[String]) -> Vec<LintResult> {
    let all_rules: Vec<Box<dyn LintRule>> = vec![
        // Structural rules
        Box::new(structural::MissingRoleRule),
        Box::new(structural::SparseExamplesRule),
        Box::new(structural::UnstructuredLongPromptRule),
        Box::new(structural::UnbalancedXmlRule),
        Box::new(structural::LongContextLayoutRule),
        // Anti-pattern rules
        Box::new(antipatterns::NegativeFramingRule),
        Box::new(antipatterns::OverPromptingRule),
        Box::new(antipatterns::VagueInstructionsRule),
        Box::new(antipatterns::DeprecatedPatternsRule),
        Box::new(antipatterns::MissingContextRule),
    ];

    all_rules
        .iter()
        .filter(|r| !disabled_rules.contains(&r.id().to_string()))
        .flat_map(|r| r.check(ast))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Block, BlockKind, PromptAst, PromptMetadata};

    fn make_ast(blocks: Vec<Block>) -> PromptAst {
        PromptAst::new(PromptMetadata::default(), blocks, String::new())
    }

    #[test]
    fn lint_runs_all_rules() {
        let ast = make_ast(vec![
            Block::new(BlockKind::Freeform, "Just some text.".into(), 0, 15),
        ]);
        let results = lint(&ast, &[]);
        // Should at least flag missing-role
        assert!(results.iter().any(|r| r.rule_id == "missing-role"));
    }

    #[test]
    fn lint_respects_disabled_rules() {
        let ast = make_ast(vec![
            Block::new(BlockKind::Freeform, "Just some text.".into(), 0, 15),
        ]);
        let results = lint(&ast, &["missing-role".to_string()]);
        assert!(!results.iter().any(|r| r.rule_id == "missing-role"));
    }

    #[test]
    fn lint_clean_prompt_has_no_errors() {
        let mut examples = Block::new(BlockKind::Examples, String::new(), 0, 100);
        for i in 0..3 {
            examples.children.push(Block::new(BlockKind::Example, format!("Example {}", i), 0, 10));
        }
        let ast = make_ast(vec![
            Block::new(BlockKind::Role, "You are a coding assistant.".into(), 0, 30),
            Block::new(BlockKind::Instructions, "Respond in JSON format with keys: result, explanation.\nValidate the input first.\nReturn errors as a separate field.".into(), 30, 120),
            examples,
        ]);
        let results = lint(&ast, &[]);
        assert!(results.is_empty(), "Expected no lint results but got: {:?}", results);
    }
}
