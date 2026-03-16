use super::ast::{Block, BlockKind, PromptAst, PromptMetadata};

/// Serialize a `PromptAst` back to markdown (with YAML frontmatter from metadata).
pub fn serialize(ast: &PromptAst) -> String {
    let mut out = String::new();

    let fm = serialize_frontmatter(&ast.metadata);
    if !fm.is_empty() {
        out.push_str("---\n");
        out.push_str(&fm);
        out.push_str("---\n\n");
    }

    for block in &ast.blocks {
        out.push_str(&serialize_block(block));
    }

    out
}

/// Serialize metadata to YAML frontmatter content (without the --- delimiters).
fn serialize_frontmatter(meta: &PromptMetadata) -> String {
    let mut lines = Vec::new();

    if !meta.name.is_empty() {
        lines.push(format!("name: \"{}\"", meta.name));
    }
    if !meta.model.is_empty() {
        lines.push(format!("model: {}", meta.model));
    }
    if meta.version > 0 {
        lines.push(format!("version: {}", meta.version));
    }
    if !meta.tags.is_empty() {
        let tags: Vec<String> = meta.tags.iter().map(|t| t.to_string()).collect();
        lines.push(format!("tags: [{}]", tags.join(", ")));
    }
    if let Some(ref thinking) = meta.thinking {
        lines.push("thinking:".to_string());
        lines.push(format!("  type: {}", thinking.kind));
    }
    if let Some(ref effort) = meta.effort {
        if !effort.is_empty() {
            lines.push(format!("effort: {}", effort));
        }
    }
    // Serialize any extra fields
    for (key, value) in &meta.extra {
        if let Ok(yaml_val) = serde_yaml::to_string(value) {
            let trimmed = yaml_val.trim();
            lines.push(format!("{}: {}", key, trimmed));
        }
    }

    if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n") + "\n"
    }
}

/// Serialize AST to markdown, but skip disabled blocks.
/// Used by MCP load_prompt to produce the "rendered" prompt.
pub fn serialize_enabled_only(ast: &PromptAst) -> String {
    let mut out = String::new();

    let fm = serialize_frontmatter(&ast.metadata);
    if !fm.is_empty() {
        out.push_str("---\n");
        out.push_str(&fm);
        out.push_str("---\n\n");
    }

    for block in &ast.blocks {
        if block.enabled {
            out.push_str(&serialize_block(block));
        }
    }

    out
}

fn serialize_block(block: &Block) -> String {
    // Serialize all blocks normally — the enabled flag is tracked in the AST,
    // not encoded in the markdown text. This avoids roundtrip issues where
    // comment-wrapped blocks get re-parsed as freeform text.
    match &block.kind {
        BlockKind::Freeform => {
            format!("{}\n", block.content)
        }
        _ => {
            let tag = block.tag_name.as_deref().unwrap_or("block");

            if !block.children.is_empty() {
                let children_str: String = block.children.iter().map(serialize_block).collect();
                format!("<{}>\n{}</{}>\n", tag, children_str, tag)
            } else {
                format!("<{}>\n{}\n</{}>\n", tag, block.content, tag)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Block, BlockKind, PromptAst, PromptMetadata};

    fn make_block(kind: BlockKind, tag: &str, content: &str) -> Block {
        Block::new(kind, content.to_string(), 0, 0).with_tag(tag)
    }

    fn make_disabled_block(kind: BlockKind, tag: &str, content: &str) -> Block {
        let mut b = Block::new(kind, content.to_string(), 0, 0).with_tag(tag);
        b.enabled = false;
        b
    }

    #[test]
    fn test_serialize_simple_prompt() {
        let metadata = PromptMetadata {
            name: "test-prompt".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            ..Default::default()
        };
        let blocks = vec![make_block(
            BlockKind::Role,
            "role",
            "You are a helpful assistant.",
        )];
        let ast = PromptAst::new(metadata, blocks, String::new());

        let result = serialize(&ast);
        assert!(result.contains("name: \"test-prompt\""));
        assert!(result.contains("model: claude-sonnet-4-6"));
        assert!(result.contains("---\n\n"));
        assert!(result.contains("<role>"));
        assert!(result.contains("You are a helpful assistant."));
        assert!(result.contains("</role>"));
    }

    #[test]
    fn test_serialize_preserves_metadata_changes() {
        let metadata = PromptMetadata {
            name: "my-prompt".to_string(),
            model: "claude-opus-4-6".to_string(),
            version: 2,
            effort: Some("high".to_string()),
            tags: vec!["coding".to_string(), "review".to_string()],
            ..Default::default()
        };
        let blocks = vec![make_block(BlockKind::Role, "role", "Helper.")];
        let ast = PromptAst::new(metadata, blocks, String::new());

        let result = serialize(&ast);
        assert!(result.contains("model: claude-opus-4-6"));
        assert!(result.contains("effort: high"));
        assert!(result.contains("version: 2"));
        assert!(result.contains("tags: [coding, review]"));
    }

    #[test]
    fn test_serialize_freeform_block() {
        let ast = PromptAst::new(
            PromptMetadata::default(),
            vec![Block::new(
                BlockKind::Freeform,
                "Just some free text.".to_string(),
                0,
                20,
            )],
            String::new(),
        );

        let result = serialize(&ast);
        assert_eq!(result, "Just some free text.\n");
        // Freeform must NOT be wrapped in XML tags
        assert!(!result.contains('<'));
    }

    #[test]
    fn test_serialize_disabled_block_still_outputs_normally() {
        // Disabled blocks are serialized the same as enabled blocks.
        // The enabled flag is tracked in the AST only, not in the markdown.
        // This avoids roundtrip issues where comment-wrapped blocks get
        // re-parsed as freeform text.
        let ast = PromptAst::new(
            PromptMetadata::default(),
            vec![make_disabled_block(
                BlockKind::Instructions,
                "instructions",
                "Do the thing.",
            )],
            String::new(),
        );

        let result = serialize(&ast);
        assert!(result.contains("<instructions>"));
        assert!(result.contains("Do the thing."));
        assert!(result.contains("</instructions>"));
        // Should NOT be wrapped in a comment
        assert!(!result.contains("<!--"));
    }

    #[test]
    fn test_serialize_enabled_only_skips_disabled() {
        let ast = PromptAst::new(
            PromptMetadata::default(),
            vec![
                make_block(BlockKind::Role, "role", "I am active."),
                make_disabled_block(BlockKind::Instructions, "instructions", "I am disabled."),
            ],
            String::new(),
        );

        let result = serialize_enabled_only(&ast);
        assert!(result.contains("I am active."));
        assert!(!result.contains("I am disabled."));
    }

    #[test]
    fn test_serialize_nested_examples() {
        let mut examples_block = make_block(BlockKind::Examples, "examples", "");
        let child1 = make_block(BlockKind::Example, "example", "First example");
        let child2 = make_block(BlockKind::Example, "example", "Second example");
        examples_block.children = vec![child1, child2];

        let ast = PromptAst::new(
            PromptMetadata::default(),
            vec![examples_block],
            String::new(),
        );

        let result = serialize(&ast);
        assert!(result.contains("<examples>"));
        assert!(result.contains("<example>"));
        assert!(result.contains("First example"));
        assert!(result.contains("Second example"));
        assert!(result.contains("</example>"));
        assert!(result.contains("</examples>"));
        // examples tag should wrap example tags
        let examples_start = result.find("<examples>").unwrap();
        let examples_end = result.find("</examples>").unwrap();
        let example_start = result.find("<example>").unwrap();
        assert!(example_start > examples_start && example_start < examples_end);
    }

    #[test]
    fn test_serialize_empty_frontmatter() {
        let ast = PromptAst::new(
            PromptMetadata::default(),
            vec![make_block(BlockKind::Role, "role", "You are helpful.")],
            String::new(), // empty raw_frontmatter
        );

        let result = serialize(&ast);
        assert!(!result.starts_with("---"));
        assert!(result.starts_with("<role>"));
    }
}
