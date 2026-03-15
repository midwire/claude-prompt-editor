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
model: claude-sonnet-4-20250514
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
model: claude-sonnet-4-20250514
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
model: claude-sonnet-4-20250514
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
model: claude-sonnet-4-20250514
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
        Template {
            id: "usecase-classification".into(),
            name: "Classification".into(),
            category: TemplateCategory::UseCase,
            description: "For categorizing inputs into predefined classes.".into(),
            content: r#"---
name: {{name}}
model: claude-sonnet-4-20250514
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
        assert!(templates.len() >= 5);
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
