use super::ast::{Block, BlockKind};

/// Represents a found XML tag in the input.
#[derive(Debug, Clone, PartialEq)]
pub struct TagSpan {
    pub name: String,
    pub is_closing: bool,
    pub line_start: usize,
    pub line_end: usize,
    /// Byte offset in the original input where this tag's line begins
    pub offset: usize,
    /// Byte offset where this tag's line ends
    pub end_offset: usize,
}

/// Parse a tag from a line. Handles lines like `<role>`, `</role>`, `<input>some text</input>`.
/// Returns (tag_name, is_closing) for the first tag found.
pub fn parse_tag_from_line(line: &str) -> Option<(String, bool)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('<') {
        return None;
    }

    let after_lt = &trimmed[1..];
    let is_closing = after_lt.starts_with('/');
    let tag_content = if is_closing { &after_lt[1..] } else { after_lt };

    // Find the end of the tag name (first '>' or whitespace)
    let end = tag_content
        .find(|c: char| c == '>' || c.is_whitespace())
        .unwrap_or(tag_content.len());

    if end == 0 {
        return None;
    }

    let name = tag_content[..end].to_string();

    // Validate: tag name should be alphabetic/hyphen/underscore
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return None;
    }

    Some((name, is_closing))
}

/// Map a tag name to a BlockKind.
pub fn tag_to_block_kind(tag: &str) -> BlockKind {
    match tag.to_lowercase().as_str() {
        "role" => BlockKind::Role,
        "instructions" => BlockKind::Instructions,
        "examples" => BlockKind::Examples,
        "example" => BlockKind::Example,
        "context" => BlockKind::Context,
        "documents" => BlockKind::Documents,
        _ => BlockKind::Custom(tag.to_string()),
    }
}

/// Scan input for XML tags, skipping tags inside code blocks (``` fences).
pub fn find_tags(input: &str) -> Vec<TagSpan> {
    let mut tags = Vec::new();
    let mut in_code_block = false;
    let mut offset = 0;

    for (line_num, line) in input.lines().enumerate() {
        let line_len = line.len();
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            offset += line_len + 1; // +1 for newline
            continue;
        }

        if !in_code_block {
            if let Some((name, is_closing)) = parse_tag_from_line(line) {
                tags.push(TagSpan {
                    name,
                    is_closing,
                    line_start: line_num,
                    line_end: line_num,
                    offset,
                    end_offset: offset + line_len,
                });
            }
        }

        offset += line_len + 1; // +1 for newline
    }

    tags
}

/// Find the matching closing tag for the open tag at `open_idx`, handling nesting.
pub fn find_closing_tag(tags: &[TagSpan], open_idx: usize) -> Option<usize> {
    let open_tag = &tags[open_idx];
    if open_tag.is_closing {
        return None;
    }

    let target_name = &open_tag.name;
    let mut depth = 0;

    for (i, tag) in tags.iter().enumerate().skip(open_idx + 1) {
        if tag.name == *target_name {
            if tag.is_closing {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            } else {
                depth += 1;
            }
        }
    }

    None
}

/// Parse body text into blocks using XML tag matching.
pub fn parse_blocks(body: &str) -> Vec<Block> {
    let tags = find_tags(body);

    if tags.is_empty() {
        if !body.trim().is_empty() {
            return vec![Block::new(
                BlockKind::Freeform,
                body.to_string(),
                0,
                body.len(),
            )];
        }
        return Vec::new();
    }

    let mut blocks = Vec::new();
    let mut used = vec![false; tags.len()];
    let mut covered_ranges: Vec<(usize, usize)> = Vec::new();

    // First pass: match open/close tag pairs at top level
    let mut i = 0;
    while i < tags.len() {
        if used[i] || tags[i].is_closing {
            i += 1;
            continue;
        }

        if let Some(close_idx) = find_closing_tag(&tags, i) {
            let open_tag = &tags[i];
            let close_tag = &tags[close_idx];
            let tag_name = &open_tag.name;

            // Extract content between open and close tags
            let content_start = open_tag.end_offset + 1; // after newline
            let content_end = close_tag.offset;
            let content = if content_start <= content_end && content_end <= body.len() {
                body[content_start..content_end]
                    .trim_end_matches('\n')
                    .to_string()
            } else {
                String::new()
            };

            let kind = tag_to_block_kind(tag_name);
            let mut block =
                Block::new(kind, content.clone(), open_tag.offset, close_tag.end_offset)
                    .with_tag(tag_name);

            // Check for nested tags (e.g., <example> inside <examples>)
            if tag_name.to_lowercase() == "examples" {
                let inner_body = &content;
                let children = parse_blocks(inner_body);
                block.children = children;
            }

            blocks.push(block);
            covered_ranges.push((open_tag.offset, close_tag.end_offset));

            // Mark all tags between open and close as used
            for used_flag in used.iter_mut().take(close_idx + 1).skip(i) {
                *used_flag = true;
            }

            i = close_idx + 1;
        } else {
            // Unmatched open tag - skip it, will become part of freeform
            i += 1;
        }
    }

    // Sort blocks by start offset
    blocks.sort_by_key(|b| b.start_offset);

    // Second pass: fill in freeform blocks for text not covered by tag blocks
    let mut final_blocks = Vec::new();
    let mut cursor = 0;

    for block in &blocks {
        if cursor < block.start_offset {
            let freeform_text = &body[cursor..block.start_offset];
            let trimmed = freeform_text.trim();
            if !trimmed.is_empty() {
                final_blocks.push(Block::new(
                    BlockKind::Freeform,
                    freeform_text.trim().to_string(),
                    cursor,
                    block.start_offset,
                ));
            }
        }
        final_blocks.push(block.clone());
        cursor = block.end_offset;
        // Skip past newline after closing tag
        if cursor < body.len() && body.as_bytes().get(cursor) == Some(&b'\n') {
            cursor += 1;
        }
    }

    // Trailing freeform text
    if cursor < body.len() {
        let trailing = &body[cursor..];
        let trimmed = trailing.trim();
        if !trimmed.is_empty() {
            final_blocks.push(Block::new(
                BlockKind::Freeform,
                trimmed.to_string(),
                cursor,
                body.len(),
            ));
        }
    }

    final_blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_tags_basic() {
        let input = "<role>\nYou are helpful.\n</role>";
        let tags = find_tags(input);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "role");
        assert!(!tags[0].is_closing);
        assert_eq!(tags[1].name, "role");
        assert!(tags[1].is_closing);
    }

    #[test]
    fn test_find_tags_nested() {
        let input = "<examples>\n<example>\nHello\n</example>\n</examples>";
        let tags = find_tags(input);
        assert_eq!(tags.len(), 4);
        assert_eq!(tags[0].name, "examples");
        assert_eq!(tags[1].name, "example");
        assert_eq!(tags[2].name, "example");
        assert!(tags[2].is_closing);
        assert_eq!(tags[3].name, "examples");
        assert!(tags[3].is_closing);
    }

    #[test]
    fn test_find_tags_ignores_code_blocks() {
        let input = "Before\n```\n<role>\ncode here\n</role>\n```\n<context>\nReal tag\n</context>";
        let tags = find_tags(input);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "context");
        assert_eq!(tags[1].name, "context");
    }

    #[test]
    fn test_parse_blocks_simple() {
        let input = "<role>\nYou are a helpful assistant.\n</role>";
        let blocks = parse_blocks(input);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].kind, BlockKind::Role);
        assert_eq!(blocks[0].tag_name.as_deref(), Some("role"));
        assert_eq!(blocks[0].content, "You are a helpful assistant.");
    }

    #[test]
    fn test_parse_blocks_with_freeform() {
        let input = "Some intro text.\n<role>\nAssistant\n</role>\nSome trailing text.";
        let blocks = parse_blocks(input);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].kind, BlockKind::Freeform);
        assert!(blocks[0].content.contains("Some intro text."));
        assert_eq!(blocks[1].kind, BlockKind::Role);
        assert_eq!(blocks[2].kind, BlockKind::Freeform);
        assert!(blocks[2].content.contains("Some trailing text."));
    }

    #[test]
    fn test_parse_blocks_nested_examples() {
        let input = "<examples>\n<example>\nFirst example\n</example>\n<example>\nSecond example\n</example>\n</examples>";
        let blocks = parse_blocks(input);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].kind, BlockKind::Examples);
        assert_eq!(blocks[0].children.len(), 2);
        assert_eq!(blocks[0].children[0].kind, BlockKind::Example);
        assert_eq!(blocks[0].children[0].content, "First example");
        assert_eq!(blocks[0].children[1].content, "Second example");
    }

    #[test]
    fn test_parse_blocks_unmatched_tag_becomes_freeform() {
        let input = "<role>\nSome content without closing tag\nMore text here.";
        let blocks = parse_blocks(input);
        // Unmatched tag should result in the whole thing becoming freeform
        assert!(!blocks.is_empty());
        // The content should be preserved
        let all_content: String = blocks.iter().map(|b| b.content.clone()).collect();
        assert!(all_content.contains("Some content"));
    }

    #[test]
    fn test_tag_to_block_kind() {
        assert_eq!(tag_to_block_kind("role"), BlockKind::Role);
        assert_eq!(tag_to_block_kind("Role"), BlockKind::Role);
        assert_eq!(tag_to_block_kind("instructions"), BlockKind::Instructions);
        assert_eq!(tag_to_block_kind("examples"), BlockKind::Examples);
        assert_eq!(tag_to_block_kind("example"), BlockKind::Example);
        assert_eq!(tag_to_block_kind("context"), BlockKind::Context);
        assert_eq!(tag_to_block_kind("documents"), BlockKind::Documents);
        assert_eq!(
            tag_to_block_kind("custom-tag"),
            BlockKind::Custom("custom-tag".to_string())
        );
    }

    #[test]
    fn test_parse_tag_from_line_inline() {
        let result = parse_tag_from_line("<input>some text</input>");
        assert_eq!(result, Some(("input".to_string(), false)));
    }

    #[test]
    fn test_parse_tag_from_line_closing() {
        let result = parse_tag_from_line("</role>");
        assert_eq!(result, Some(("role".to_string(), true)));
    }

    #[test]
    fn test_parse_tag_from_line_not_a_tag() {
        assert_eq!(parse_tag_from_line("just text"), None);
        assert_eq!(parse_tag_from_line("< not a tag>"), None);
    }
}
