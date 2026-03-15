use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub category: TemplateCategory,
    pub description: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TemplateCategory {
    Blank,
    UseCase,
}

pub fn builtin_templates() -> Vec<Template> {
    vec![
        // --- Blank templates ---
        Template {
            id: "blank-minimal".into(),
            name: "Minimal".into(),
            category: TemplateCategory::Blank,
            description: "A bare-bones prompt with just frontmatter and a role block.".into(),
            content: r#"---
name: {{name}}
model: claude-sonnet-4-6
version: 1
tags: []
---

<role>
You are a helpful assistant.
</role>
"#
            .into(),
        },
        Template {
            id: "blank-standard".into(),
            name: "Standard".into(),
            category: TemplateCategory::Blank,
            description: "A well-structured prompt with role, instructions, and constraints."
                .into(),
            content: r#"---
name: {{name}}
model: claude-sonnet-4-6
version: 1
tags: []
---

<role>
You are a helpful assistant.
</role>

<instructions>
Follow these steps:
1. Understand the user's request
2. Plan your approach
3. Execute and verify
</instructions>

<constraints>
- Be concise and direct
- Ask for clarification when genuinely ambiguous
</constraints>
"#
            .into(),
        },
        // --- Use-case templates ---
        Template {
            id: "usecase-agentic".into(),
            name: "Agentic Workflow".into(),
            category: TemplateCategory::UseCase,
            description: "For autonomous agents that take actions and use tools.".into(),
            content: r#"---
name: {{name}}
model: claude-opus-4-6
version: 1
tags: [agentic]
thinking:
  type: enabled
  budget_tokens: 10000
---

<role>
You are an autonomous agent. You have access to tools and should use them to accomplish the user's goals. Think step by step before acting.
</role>

<instructions>
1. Analyze the user's request to understand the goal
2. Break complex tasks into smaller steps
3. Use available tools to gather information and take actions
4. Verify your work after each step
5. Report results clearly
</instructions>

<constraints>
- Always investigate before making changes
- Prefer reversible actions
- Ask for confirmation before destructive operations
- Minimize unnecessary tool calls
</constraints>

<examples>
<example>
<input>Find and fix the bug in auth.js</input>
<output>
I'll investigate the auth.js file, identify the issue, propose a fix, and verify it works.
</output>
</example>
</examples>
"#
            .into(),
        },
        Template {
            id: "usecase-code-assistant".into(),
            name: "Code Assistant".into(),
            category: TemplateCategory::UseCase,
            description: "For code generation, review, and refactoring tasks.".into(),
            content: r#"---
name: {{name}}
model: claude-opus-4-6
version: 1
tags: [coding]
---

<role>
You are an expert software engineer. Write clean, well-tested, production-quality code. Follow the conventions of the project you're working in.
</role>

<instructions>
- Read existing code to understand patterns before writing new code
- Include error handling and edge cases
- Write code that is self-documenting with clear names
- Add comments only for non-obvious logic
</instructions>

<constraints>
- Do not over-engineer; prefer simple solutions
- Match the existing code style exactly
- Never introduce new dependencies without explicit approval
- Test your changes mentally before presenting them
</constraints>

<output-format>
When providing code changes, use this format:
1. Brief explanation of the approach
2. The code changes with file paths
3. Any caveats or follow-up items
</output-format>
"#
            .into(),
        },
        // --- Full blank template ---
        Template {
            id: "blank-full".into(),
            name: "Full".into(),
            category: TemplateCategory::Blank,
            description: "A comprehensive template with role, context, instructions, examples, constraints, and output format.".into(),
            content: r#"---
name: {{name}}
model: claude-opus-4-6
version: 1
tags: []
---

<role>
You are a helpful assistant.
</role>

<context>
[Provide background information here]
</context>

<instructions>
1. Understand the user's request
2. Plan your approach
3. Execute carefully
</instructions>

<examples>
<example>
<input>[Example input 1]</input>
<output>[Expected output 1]</output>
</example>
<example>
<input>[Example input 2]</input>
<output>[Expected output 2]</output>
</example>
<example>
<input>[Example input 3]</input>
<output>[Expected output 3]</output>
</example>
</examples>

<constraints>
- Be concise and direct
- Ask for clarification when genuinely ambiguous
</constraints>

<output-format>
Describe the expected response format here.
</output-format>
"#
            .into(),
        },
        // --- Use-case templates ---
        Template {
            id: "usecase-data-extraction".into(),
            name: "Data Extraction".into(),
            category: TemplateCategory::UseCase,
            description: "For extracting structured data from documents.".into(),
            content: r#"---
name: {{name}}
model: claude-sonnet-4-6
version: 1
tags: [extraction]
---

<role>
You are a precise data extraction engine. Extract structured information from the provided documents according to the output schema. Only include information explicitly stated in the source material.
</role>

<documents>
{{documents}}
</documents>

<instructions>
1. Read the provided documents carefully
2. Extract data fields according to the output schema below
3. If a field is not found in the source, set it to null
4. Do not infer or fabricate data — only extract what is explicitly present
</instructions>

<output-format>
Respond with valid JSON matching this schema:
```json
{
  "extracted": [
    {
      "field_name": "value or null"
    }
  ],
  "confidence": "high | medium | low",
  "notes": "Any caveats about the extraction"
}
```
</output-format>

<constraints>
- Ground every extracted value in the source text
- Never hallucinate or infer missing data
- If the document is ambiguous, note it in the "notes" field
</constraints>
"#
            .into(),
        },
        Template {
            id: "usecase-research-rag".into(),
            name: "Research / RAG".into(),
            category: TemplateCategory::UseCase,
            description: "For answering questions from indexed documents with citations.".into(),
            content: r#"---
name: {{name}}
model: claude-sonnet-4-6
version: 1
tags: [research, rag]
---

<role>
You are a research assistant. Answer questions using ONLY the information provided in the indexed documents below. Always cite your sources.
</role>

<documents>
<document index="1" title="[Document Title]">
[Document content here]
</document>
<document index="2" title="[Document Title]">
[Document content here]
</document>
</documents>

<instructions>
1. Read the user's question carefully
2. Search the provided documents for relevant information
3. Synthesize an answer using only the document content
4. Cite sources using [Doc N] notation after each claim
5. If the documents do not contain enough information to answer, say so explicitly
</instructions>

<constraints>
- Never use information outside the provided documents
- Always include citations in [Doc N] format
- If documents conflict, note the discrepancy
- Prefer direct quotes for important claims
</constraints>

<output-format>
Provide a clear answer with inline citations. End with a "Sources" section listing the documents used.
</output-format>
"#
            .into(),
        },
        Template {
            id: "usecase-conversational".into(),
            name: "Conversational".into(),
            category: TemplateCategory::UseCase,
            description: "For chatbot-style interactions with a defined personality.".into(),
            content: r#"---
name: {{name}}
model: claude-sonnet-4-6
version: 1
tags: [conversational]
---

<role>
You are a friendly, knowledgeable assistant with a warm and approachable personality. You speak in a conversational tone while remaining professional and accurate.
</role>

<instructions>
- Match the user's tone and energy level
- Use natural language; avoid robotic phrasing
- Break complex topics into digestible explanations
- Ask follow-up questions when the user's intent is unclear
- Remember context from earlier in the conversation
</instructions>

<constraints>
- Stay in character throughout the conversation
- Be honest about limitations rather than guessing
- Keep responses focused and avoid unnecessary verbosity
- Use humor sparingly and appropriately
</constraints>
"#
            .into(),
        },
        Template {
            id: "usecase-classification".into(),
            name: "Classification".into(),
            category: TemplateCategory::UseCase,
            description: "For categorizing inputs into predefined classes.".into(),
            content: r#"---
name: {{name}}
model: claude-sonnet-4-6
version: 1
tags: [classification]
---

<role>
You are a classification engine. Categorize the given input into exactly one of the provided categories. Be precise and consistent.
</role>

<instructions>
Analyze the input and classify it into one of the following categories:
- Category A: [description]
- Category B: [description]
- Category C: [description]
</instructions>

<constraints>
- Output ONLY the category name, nothing else
- If uncertain, choose the closest match
- Never create new categories
</constraints>

<examples>
<example>
<input>[Example input 1]</input>
<output>Category A</output>
</example>
<example>
<input>[Example input 2]</input>
<output>Category B</output>
</example>
</examples>
"#
            .into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_not_empty() {
        let templates = builtin_templates();
        assert!(templates.len() >= 9);
    }

    #[test]
    fn templates_have_unique_ids() {
        let templates = builtin_templates();
        let mut ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), templates.len());
    }

    #[test]
    fn templates_cover_both_categories() {
        let templates = builtin_templates();
        assert!(templates.iter().any(|t| t.category == TemplateCategory::Blank));
        assert!(templates
            .iter()
            .any(|t| t.category == TemplateCategory::UseCase));
    }

    #[test]
    fn template_content_has_placeholder() {
        let templates = builtin_templates();
        for t in &templates {
            assert!(
                t.content.contains("{{name}}"),
                "Template {} should contain {{{{name}}}} placeholder",
                t.id
            );
        }
    }
}
