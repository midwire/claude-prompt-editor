use super::ast::{Block, BlockKind, PromptAst};

/// Serialize a `PromptAst` back to markdown (with optional YAML frontmatter).
pub fn serialize(ast: &PromptAst) -> String {
    let mut out = String::new();

    if !ast.raw_frontmatter.is_empty() {
        out.push_str(&ast.raw_frontmatter);
        out.push_str("\n\n");
    }

    for block in &ast.blocks {
        out.push_str(&serialize_block(block));
    }

    out
}

fn serialize_block(block: &Block) -> String {
    // Disabled blocks: wrap entire block in HTML comment
    if !block.enabled {
        let tag = block
            .tag_name
            .as_deref()
            .unwrap_or("block");
        let inner = if block.children.is_empty() {
            format!("<{}>\n{}\n</{}>", tag, block.content, tag)
        } else {
            let children_str: String = block.children.iter().map(serialize_block).collect();
            format!("<{}>\n{}</{}>", tag, children_str, tag)
        };
        return format!("<!-- disabled: {} -->\n", inner);
    }

    match &block.kind {
        BlockKind::Freeform => {
            format!("{}\n", block.content)
        }
        _ => {
            let tag = block
                .tag_name
                .as_deref()
                .unwrap_or("block");

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
            model: "claude-sonnet".to_string(),
            ..Default::default()
        };
        let raw_frontmatter = "---\nname: test-prompt\nmodel: claude-sonnet\n---".to_string();
        let blocks = vec![make_block(
            BlockKind::Role,
            "role",
            "You are a helpful assistant.",
        )];
        let ast = PromptAst::new(metadata, blocks, raw_frontmatter);

        let result = serialize(&ast);
        assert!(result.starts_with("---\nname: test-prompt"));
        assert!(result.contains("---\n\n"));
        assert!(result.contains("<role>"));
        assert!(result.contains("You are a helpful assistant."));
        assert!(result.contains("</role>"));
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
    fn test_serialize_disabled_block_as_comment() {
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
        assert!(result.starts_with("<!-- disabled:"));
        assert!(result.contains("<instructions>"));
        assert!(result.contains("Do the thing."));
        assert!(result.contains("</instructions>"));
        assert!(result.contains("-->"));
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
