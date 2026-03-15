use regex::Regex;
use std::collections::HashMap;

/// Extract all unique variable names from `{{name}}` patterns in the input.
pub fn extract_variable_names(input: &str) -> Vec<String> {
    let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
    let mut seen = std::collections::HashSet::new();
    let mut names = Vec::new();

    for cap in re.captures_iter(input) {
        let name = cap[1].to_string();
        if seen.insert(name.clone()) {
            names.push(name);
        }
    }

    names
}

/// Replace `{{key}}` patterns in input with values from vars.
/// Keys not found in vars are left as-is.
pub fn interpolate(input: &str, vars: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    for (key, value) in vars {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_variable_names() {
        let input = "Hello {{name}}, you are a {{role}} using {{tool}}.";
        let names = extract_variable_names(input);
        assert_eq!(names, vec!["name", "role", "tool"]);
    }

    #[test]
    fn test_extract_no_variables() {
        let input = "No variables here, just plain text.";
        let names = extract_variable_names(input);
        assert!(names.is_empty());
    }

    #[test]
    fn test_extract_duplicate_variables() {
        let input = "{{name}} and {{name}} again, also {{role}}";
        let names = extract_variable_names(input);
        assert_eq!(names, vec!["name", "role"]);
    }

    #[test]
    fn test_interpolate_basic() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("role".to_string(), "engineer".to_string());
        let result = interpolate("Hello {{name}}, you are a {{role}}.", &vars);
        assert_eq!(result, "Hello Alice, you are a engineer.");
    }

    #[test]
    fn test_interpolate_missing_preserved() {
        let vars = HashMap::new();
        let result = interpolate("Hello {{name}}, welcome.", &vars);
        assert_eq!(result, "Hello {{name}}, welcome.");
    }

    #[test]
    fn test_interpolate_multiple() {
        let mut vars = HashMap::new();
        vars.insert("a".to_string(), "1".to_string());
        vars.insert("b".to_string(), "2".to_string());
        vars.insert("c".to_string(), "3".to_string());
        let result = interpolate("{{a}}-{{b}}-{{c}}", &vars);
        assert_eq!(result, "1-2-3");
    }
}
