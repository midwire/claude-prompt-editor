use crate::parser::ast::{BlockKind, PromptAst};
use super::rules::{LintResult, LintRule, Severity};

/// Flags when no `<role>` block is present.
pub struct MissingRoleRule;

impl LintRule for MissingRoleRule {
    fn id(&self) -> &str { "missing-role" }
    fn description(&self) -> &str { "Prompt should have a <role> block to define the assistant persona" }

    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        let has_role = ast.blocks.iter().any(|b| b.kind == BlockKind::Role);
        if has_role {
            vec![]
        } else {
            vec![LintResult {
                rule_id: self.id().to_string(),
                severity: Severity::Warning,
                message: "No <role> block found".to_string(),
                detail: "Adding a <role> block helps define the assistant's persona and improves response consistency.".to_string(),
                block_index: None,
                fix_suggestion: Some("Add a <role> block at the beginning of your prompt.".to_string()),
            }]
        }
    }
}

/// Flags when an `<examples>` block has fewer than 3 examples.
pub struct SparseExamplesRule;

impl LintRule for SparseExamplesRule {
    fn id(&self) -> &str { "sparse-examples" }
    fn description(&self) -> &str { "Examples block should have at least 3 examples for reliable few-shot learning" }

    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        let mut results = vec![];
        for (i, block) in ast.blocks.iter().enumerate() {
            if block.kind == BlockKind::Examples {
                let example_count = block.children.iter()
                    .filter(|c| c.kind == BlockKind::Example)
                    .count();
                if example_count > 0 && example_count < 3 {
                    results.push(LintResult {
                        rule_id: self.id().to_string(),
                        severity: Severity::Suggestion,
                        message: format!("Examples block has only {} example(s)", example_count),
                        detail: "Few-shot prompting works best with at least 3 diverse examples. Consider adding more.".to_string(),
                        block_index: Some(i),
                        fix_suggestion: Some("Add more <example> blocks inside <examples>.".to_string()),
                    });
                }
            }
        }
        results
    }
}

/// Flags long prompts (>500 estimated tokens) with no XML tags.
pub struct UnstructuredLongPromptRule;

impl UnstructuredLongPromptRule {
    /// Rough token estimate: ~4 chars per token.
    fn estimate_tokens(text: &str) -> usize {
        text.len() / 4
    }
}

impl LintRule for UnstructuredLongPromptRule {
    fn id(&self) -> &str { "unstructured-long-prompt" }
    fn description(&self) -> &str { "Long prompts should use XML tags for structure" }

    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        // Check if all blocks are Freeform (no XML structure)
        let has_tagged = ast.blocks.iter().any(|b| b.kind != BlockKind::Freeform);
        if has_tagged {
            return vec![];
        }

        let total_content: String = ast.blocks.iter()
            .map(|b| b.content.as_str())
            .collect::<Vec<_>>()
            .join("");

        let tokens = Self::estimate_tokens(&total_content);
        if tokens > 500 {
            vec![LintResult {
                rule_id: self.id().to_string(),
                severity: Severity::Warning,
                message: format!("Long prompt (~{} tokens) has no XML structure", tokens),
                detail: "Large prompts benefit from XML tags like <role>, <instructions>, <examples> to organize content and improve model comprehension.".to_string(),
                block_index: None,
                fix_suggestion: Some("Wrap sections in XML tags: <role>, <instructions>, <examples>, <context>.".to_string()),
            }]
        } else {
            vec![]
        }
    }
}

/// Flags freeform blocks that contain unclosed XML-like tags.
pub struct UnbalancedXmlRule;

impl UnbalancedXmlRule {
    fn find_unbalanced(content: &str) -> Vec<String> {
        let open_re = regex::Regex::new(r"<([a-zA-Z][a-zA-Z0-9_-]*)(?:\s[^>]*)?>").unwrap();
        let close_re = regex::Regex::new(r"</([a-zA-Z][a-zA-Z0-9_-]*)>").unwrap();

        let mut open_tags: Vec<String> = vec![];
        for cap in open_re.captures_iter(content) {
            let tag = cap[1].to_lowercase();
            // Skip self-closing tags (ending with />)
            let full = cap.get(0).unwrap().as_str();
            if full.ends_with("/>") {
                continue;
            }
            open_tags.push(tag);
        }

        for cap in close_re.captures_iter(content) {
            let tag = cap[1].to_lowercase();
            if let Some(pos) = open_tags.iter().rposition(|t| t == &tag) {
                open_tags.remove(pos);
            }
        }

        open_tags.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect()
    }
}

impl LintRule for UnbalancedXmlRule {
    fn id(&self) -> &str { "unbalanced-xml" }
    fn description(&self) -> &str { "Freeform blocks should not contain unclosed XML tags" }

    fn check(&self, ast: &PromptAst) -> Vec<LintResult> {
        let mut results = vec![];
        for (i, block) in ast.blocks.iter().enumerate() {
            if block.kind == BlockKind::Freeform {
                let unbalanced = Self::find_unbalanced(&block.content);
                if !unbalanced.is_empty() {
                    let tags = unbalanced.join(", ");
                    results.push(LintResult {
                        rule_id: self.id().to_string(),
                        severity: Severity::Error,
                        message: format!("Unclosed XML tag(s): {}", tags),
                        detail: "Unclosed tags may cause the model to misinterpret prompt structure.".to_string(),
                        block_index: Some(i),
                        fix_suggestion: Some(format!("Add closing tag(s) for: {}", tags)),
                    });
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Block, PromptAst, PromptMetadata};

    fn make_ast(blocks: Vec<Block>) -> PromptAst {
        PromptAst::new(PromptMetadata::default(), blocks, String::new())
    }

    // MissingRoleRule tests
    #[test]
    fn missing_role_flags_when_no_role() {
        let ast = make_ast(vec![
            Block::new(BlockKind::Instructions, "Do things.".into(), 0, 10),
        ]);
        let results = MissingRoleRule.check(&ast);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_id, "missing-role");
        assert_eq!(results[0].severity, Severity::Warning);
    }

    #[test]
    fn missing_role_passes_when_role_present() {
        let ast = make_ast(vec![
            Block::new(BlockKind::Role, "You are helpful.".into(), 0, 10),
        ]);
        let results = MissingRoleRule.check(&ast);
        assert!(results.is_empty());
    }

    // SparseExamplesRule tests
    #[test]
    fn sparse_examples_flags_one_example() {
        let mut examples = Block::new(BlockKind::Examples, String::new(), 0, 100);
        examples.children.push(Block::new(BlockKind::Example, "ex1".into(), 0, 10));

        let ast = make_ast(vec![examples]);
        let results = SparseExamplesRule.check(&ast);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_id, "sparse-examples");
        assert_eq!(results[0].severity, Severity::Suggestion);
    }

    #[test]
    fn sparse_examples_passes_with_three() {
        let mut examples = Block::new(BlockKind::Examples, String::new(), 0, 100);
        for i in 0..3 {
            examples.children.push(Block::new(BlockKind::Example, format!("ex{}", i), 0, 10));
        }
        let ast = make_ast(vec![examples]);
        let results = SparseExamplesRule.check(&ast);
        assert!(results.is_empty());
    }

    #[test]
    fn sparse_examples_ignores_empty_examples() {
        let examples = Block::new(BlockKind::Examples, String::new(), 0, 100);
        let ast = make_ast(vec![examples]);
        let results = SparseExamplesRule.check(&ast);
        assert!(results.is_empty());
    }

    // UnstructuredLongPromptRule tests
    #[test]
    fn unstructured_long_prompt_flags_long_freeform() {
        let long_text = "a ".repeat(1200); // ~2400 chars => ~600 tokens
        let ast = make_ast(vec![
            Block::new(BlockKind::Freeform, long_text, 0, 2400),
        ]);
        let results = UnstructuredLongPromptRule.check(&ast);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_id, "unstructured-long-prompt");
    }

    #[test]
    fn unstructured_long_prompt_passes_with_tags() {
        let long_text = "a ".repeat(1200);
        let ast = make_ast(vec![
            Block::new(BlockKind::Role, "You are helpful.".into(), 0, 20),
            Block::new(BlockKind::Freeform, long_text, 20, 2420),
        ]);
        let results = UnstructuredLongPromptRule.check(&ast);
        assert!(results.is_empty());
    }

    #[test]
    fn unstructured_long_prompt_passes_when_short() {
        let ast = make_ast(vec![
            Block::new(BlockKind::Freeform, "Hello world.".into(), 0, 12),
        ]);
        let results = UnstructuredLongPromptRule.check(&ast);
        assert!(results.is_empty());
    }

    // UnbalancedXmlRule tests
    #[test]
    fn unbalanced_xml_flags_unclosed_tag() {
        let ast = make_ast(vec![
            Block::new(BlockKind::Freeform, "<note>Some text here".into(), 0, 20),
        ]);
        let results = UnbalancedXmlRule.check(&ast);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_id, "unbalanced-xml");
        assert_eq!(results[0].severity, Severity::Error);
    }

    #[test]
    fn unbalanced_xml_passes_balanced() {
        let ast = make_ast(vec![
            Block::new(BlockKind::Freeform, "<note>text</note>".into(), 0, 20),
        ]);
        let results = UnbalancedXmlRule.check(&ast);
        assert!(results.is_empty());
    }

    #[test]
    fn unbalanced_xml_ignores_non_freeform() {
        let ast = make_ast(vec![
            Block::new(BlockKind::Instructions, "<note>unclosed".into(), 0, 14),
        ]);
        let results = UnbalancedXmlRule.check(&ast);
        assert!(results.is_empty());
    }
}
