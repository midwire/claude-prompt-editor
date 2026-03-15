use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PresetCategory {
    Role,
    Instructions,
    Constraints,
    OutputFormat,
    ExampleSkeleton,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataDefaults {
    pub model: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub category: PresetCategory,
    pub content: String,
    pub tag_name: Option<String>,
    pub metadata_defaults: Option<MetadataDefaults>,
}

pub fn builtin_presets() -> Vec<Preset> {
    vec![
        // --- Roles ---
        Preset {
            id: "role-code-reviewer".into(),
            name: "Code Reviewer".into(),
            category: PresetCategory::Role,
            content: "You are a senior code reviewer. Examine the provided code for bugs, \
                security vulnerabilities, performance issues, and adherence to best practices. \
                Provide clear, actionable feedback with specific line references."
                .into(),
            tag_name: Some("role".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "role-data-analyst".into(),
            name: "Data Analyst".into(),
            category: PresetCategory::Role,
            content: "You are an expert data analyst. Analyze datasets, identify trends and \
                patterns, compute summary statistics, and present findings clearly. \
                Use tables and structured output when appropriate."
                .into(),
            tag_name: Some("role".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "role-tech-writer".into(),
            name: "Technical Writer".into(),
            category: PresetCategory::Role,
            content: "You are a technical writer specializing in developer documentation. \
                Write clear, concise documentation with proper formatting, code examples, \
                and logical structure. Target an audience of intermediate developers."
                .into(),
            tag_name: Some("role".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "role-qa-engineer".into(),
            name: "QA Engineer".into(),
            category: PresetCategory::Role,
            content: "You are a QA engineer. Design comprehensive test plans, identify edge \
                cases, write test scenarios, and evaluate software for correctness. \
                Prioritize tests by risk and impact."
                .into(),
            tag_name: Some("role".into()),
            metadata_defaults: None,
        },
        // --- Constraints ---
        Preset {
            id: "constraint-investigate-first".into(),
            name: "Investigate Before Acting".into(),
            category: PresetCategory::Constraints,
            content: "IMPORTANT: Before making any changes or providing solutions, first \
                investigate the full context. Read all relevant files, understand the \
                existing patterns, and only then propose changes. Never guess—verify."
                .into(),
            tag_name: Some("constraints".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "constraint-default-action".into(),
            name: "Default to Action".into(),
            category: PresetCategory::Constraints,
            content: "When the intent is clear, proceed directly with implementation rather \
                than asking for confirmation. Only ask clarifying questions when the request \
                is genuinely ambiguous or when a wrong choice would be costly to reverse."
                .into(),
            tag_name: Some("constraints".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "constraint-minimize-overengineering".into(),
            name: "Minimize Overengineering".into(),
            category: PresetCategory::Constraints,
            content: "Prefer simple, direct solutions over complex abstractions. Do not \
                introduce patterns, frameworks, or indirection unless there is a concrete, \
                present need. Avoid premature optimization and speculative generality."
                .into(),
            tag_name: Some("constraints".into()),
            metadata_defaults: None,
        },
        // --- Output Formats ---
        Preset {
            id: "format-json-only".into(),
            name: "JSON Only".into(),
            category: PresetCategory::OutputFormat,
            content: "Respond ONLY with valid JSON. No markdown, no explanation, no preamble. \
                The response must be parseable by a standard JSON parser. Use the following \
                schema:\n\n```json\n{\n  \"result\": \"...\"\n}\n```"
                .into(),
            tag_name: Some("output-format".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "format-prose".into(),
            name: "Prose Response".into(),
            category: PresetCategory::OutputFormat,
            content: "Respond in well-structured prose. Use paragraphs to organize your \
                thoughts logically. Avoid bullet points or numbered lists unless they \
                genuinely improve clarity. Write in a professional but approachable tone."
                .into(),
            tag_name: Some("output-format".into()),
            metadata_defaults: None,
        },
        // --- Example Skeleton ---
        // --- Example Skeletons ---
        Preset {
            id: "example-skeleton-classification".into(),
            name: "Classification Example".into(),
            category: PresetCategory::ExampleSkeleton,
            content: "<example>\n<input>[Text to classify]</input>\n<output>\nCategory: [category name]\nConfidence: [high/medium/low]\n</output>\n</example>"
                .into(),
            tag_name: Some("examples".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "example-skeleton-extraction".into(),
            name: "Extraction Example".into(),
            category: PresetCategory::ExampleSkeleton,
            content: "<example>\n<input>[Source document or text]</input>\n<output>\n{\n  \"field_1\": \"extracted value\",\n  \"field_2\": \"extracted value\"\n}\n</output>\n</example>"
                .into(),
            tag_name: Some("examples".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "example-skeleton-qa".into(),
            name: "Q&A Example".into(),
            category: PresetCategory::ExampleSkeleton,
            content: "<example>\n<input>[User question]</input>\n<output>\n[Concise answer with source citation]\n\nSource: [reference]\n</output>\n</example>"
                .into(),
            tag_name: Some("examples".into()),
            metadata_defaults: None,
        },
        Preset {
            id: "example-skeleton-io".into(),
            name: "Input/Output Example".into(),
            category: PresetCategory::ExampleSkeleton,
            content: "<example>\n<input>\n[Your example input here]\n</input>\n<output>\n\
                [Expected output here]\n</output>\n</example>"
                .into(),
            tag_name: Some("examples".into()),
            metadata_defaults: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_presets_not_empty() {
        let presets = builtin_presets();
        assert!(presets.len() >= 13);
    }

    #[test]
    fn builtin_presets_have_unique_ids() {
        let presets = builtin_presets();
        let mut ids: Vec<&str> = presets.iter().map(|p| p.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), presets.len());
    }

    #[test]
    fn builtin_presets_cover_all_categories() {
        let presets = builtin_presets();
        let cats: Vec<&PresetCategory> = presets.iter().map(|p| &p.category).collect();
        assert!(cats.contains(&&PresetCategory::Role));
        assert!(cats.contains(&&PresetCategory::Constraints));
        assert!(cats.contains(&&PresetCategory::OutputFormat));
        assert!(cats.contains(&&PresetCategory::ExampleSkeleton));
    }
}
