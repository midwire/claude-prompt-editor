use super::ast::PromptMetadata;

/// Parse YAML frontmatter from a prompt string.
/// Returns (metadata, body, raw_frontmatter).
/// If no frontmatter is found, returns default metadata, the full input as body, and empty raw.
pub fn parse_frontmatter(input: &str) -> (PromptMetadata, String, String) {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return (PromptMetadata::default(), input.to_string(), String::new());
    }

    // Find the end of the opening ---
    let after_opening = match trimmed.strip_prefix("---") {
        Some(rest) => rest,
        None => return (PromptMetadata::default(), input.to_string(), String::new()),
    };

    // Find closing ---
    if let Some(end_idx) = after_opening.find("\n---") {
        let yaml_str = &after_opening[..end_idx];
        let raw_frontmatter = format!("---{}\n---", yaml_str);

        let metadata: PromptMetadata = serde_yaml::from_str(yaml_str).unwrap_or_default();

        let after_closing = &after_opening[end_idx + 4..]; // skip "\n---"
                                                           // Skip optional newline after closing ---
        let body = after_closing.strip_prefix('\n').unwrap_or(after_closing);

        return (metadata, body.to_string(), raw_frontmatter);
    }

    // No closing ---, treat as no frontmatter
    (PromptMetadata::default(), input.to_string(), String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_frontmatter() {
        let input = "---\nname: test-prompt\nmodel: claude-sonnet\nversion: 2\ntags:\n  - coding\n  - review\n---\nHello world";
        let (meta, body, _raw) = parse_frontmatter(input);
        assert_eq!(meta.name, "test-prompt");
        assert_eq!(meta.model, "claude-sonnet");
        assert_eq!(meta.version, 2);
        assert_eq!(meta.tags, vec!["coding", "review"]);
        assert_eq!(body, "Hello world");
    }

    #[test]
    fn test_parse_without_frontmatter() {
        let input = "Just some prompt text\nwith multiple lines";
        let (meta, body, raw) = parse_frontmatter(input);
        assert_eq!(meta, PromptMetadata::default());
        assert_eq!(body, input);
        assert_eq!(raw, "");
    }

    #[test]
    fn test_parse_with_thinking_config() {
        let input = "---\nname: thinker\nthinking:\n  type: enabled\n---\nDo some thinking.";
        let (meta, body, _raw) = parse_frontmatter(input);
        assert_eq!(meta.name, "thinker");
        let thinking = meta.thinking.unwrap();
        assert_eq!(thinking.kind, "enabled");
        assert_eq!(body, "Do some thinking.");
    }

    #[test]
    fn test_parse_with_extra_fields() {
        let input = "---\nname: extra\ncustom_field: hello\nanother: 42\n---\nBody here.";
        let (meta, body, _raw) = parse_frontmatter(input);
        assert_eq!(meta.name, "extra");
        assert_eq!(
            meta.extra.get("custom_field"),
            Some(&serde_yaml::Value::String("hello".to_string()))
        );
        assert_eq!(body, "Body here.");
    }

    #[test]
    fn test_roundtrip_raw_frontmatter() {
        let input = "---\nname: roundtrip\nmodel: opus\n---\nContent after.";
        let (meta, body, raw) = parse_frontmatter(input);
        assert_eq!(meta.name, "roundtrip");
        assert_eq!(body, "Content after.");
        // Raw frontmatter should contain the original YAML
        assert!(raw.starts_with("---"));
        assert!(raw.ends_with("---"));
        assert!(raw.contains("name: roundtrip"));
        assert!(raw.contains("model: opus"));
    }
}
