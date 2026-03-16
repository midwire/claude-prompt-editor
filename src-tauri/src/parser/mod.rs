pub mod ast;
pub mod frontmatter;
pub mod serializer;
pub mod variables;
pub mod xml_tags;

use ast::PromptAst;

/// Parse a full prompt string (optional YAML frontmatter + tagged body) into a `PromptAst`.
pub fn parse(input: &str) -> Result<PromptAst, String> {
    let (metadata, body, raw_frontmatter) = frontmatter::parse_frontmatter(input);
    let blocks = xml_tags::parse_blocks(&body);
    Ok(PromptAst::new(metadata, blocks, raw_frontmatter))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::BlockKind;
    use crate::parser::serializer;

    #[test]
    fn test_parse_full_prompt() {
        let input = r#"---
name: my-prompt
model: claude-sonnet
version: 1
---

<role>
You are a coding assistant.
</role>
<instructions>
Write clean code.
</instructions>
<examples>
<example>
Print hello world.
</example>
</examples>"#;

        let ast = parse(input).unwrap();
        assert_eq!(ast.metadata.name, "my-prompt");
        assert_eq!(ast.metadata.model, "claude-sonnet");
        assert_eq!(ast.metadata.version, 1);

        // Should have role, instructions, examples blocks
        let kinds: Vec<&BlockKind> = ast.blocks.iter().map(|b| &b.kind).collect();
        assert!(kinds.contains(&&BlockKind::Role));
        assert!(kinds.contains(&&BlockKind::Instructions));
        assert!(kinds.contains(&&BlockKind::Examples));

        // Examples block should have one child
        let examples = ast
            .blocks
            .iter()
            .find(|b| b.kind == BlockKind::Examples)
            .unwrap();
        assert_eq!(examples.children.len(), 1);
        assert_eq!(examples.children[0].kind, BlockKind::Example);
    }

    #[test]
    fn test_parse_roundtrip() {
        let input = r#"---
name: roundtrip
model: claude-opus
---

<role>
You are helpful.
</role>
<instructions>
Be concise.
</instructions>"#;

        let ast1 = parse(input).unwrap();
        let serialized = serializer::serialize(&ast1);
        let ast2 = parse(&serialized).unwrap();

        assert_eq!(ast1.metadata.name, ast2.metadata.name);
        assert_eq!(ast1.metadata.model, ast2.metadata.model);
        assert_eq!(ast1.blocks.len(), ast2.blocks.len());

        for (b1, b2) in ast1.blocks.iter().zip(ast2.blocks.iter()) {
            assert_eq!(b1.kind, b2.kind);
            assert_eq!(b1.content, b2.content);
        }
    }
}
