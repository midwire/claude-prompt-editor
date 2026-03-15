use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PromptMetadata {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub thinking: Option<ThinkingConfig>,
    #[serde(default)]
    pub effort: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockKind {
    Role,
    Instructions,
    Examples,
    Example,
    Context,
    Documents,
    Custom(String),
    Freeform,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub kind: BlockKind,
    pub tag_name: Option<String>,
    pub content: String,
    pub children: Vec<Block>,
    pub enabled: bool,
    pub start_offset: usize,
    pub end_offset: usize,
}

impl Block {
    pub fn new(kind: BlockKind, content: String, start: usize, end: usize) -> Self {
        Self {
            kind,
            tag_name: None,
            content,
            children: Vec::new(),
            enabled: true,
            start_offset: start,
            end_offset: end,
        }
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tag_name = Some(tag.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptAst {
    pub metadata: PromptMetadata,
    pub blocks: Vec<Block>,
    pub raw_frontmatter: String,
}

impl PromptAst {
    pub fn new(metadata: PromptMetadata, blocks: Vec<Block>, raw_frontmatter: String) -> Self {
        Self {
            metadata,
            blocks,
            raw_frontmatter,
        }
    }
}
