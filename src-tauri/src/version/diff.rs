use serde::{Deserialize, Serialize};
use similar::TextDiff;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub unified: String,
    pub additions: usize,
    pub deletions: usize,
}

pub fn compute_diff(old: &str, new: &str) -> DiffResult {
    let diff = TextDiff::from_lines(old, new);
    let unified = diff
        .unified_diff()
        .context_radius(3)
        .header("old", "new")
        .to_string();

    let mut additions = 0;
    let mut deletions = 0;
    for change in diff.iter_all_changes() {
        match change.tag() {
            similar::ChangeTag::Insert => additions += 1,
            similar::ChangeTag::Delete => deletions += 1,
            similar::ChangeTag::Equal => {}
        }
    }

    DiffResult {
        unified,
        additions,
        deletions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_basic() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nline 2 modified\nline 3\nline 4\n";
        let result = compute_diff(old, new);

        assert!(result.unified.contains("line 2 modified"));
        assert!(result.unified.contains("line 4"));
        assert_eq!(result.additions, 2); // "line 2 modified" and "line 4"
        assert_eq!(result.deletions, 1); // "line 2"
    }

    #[test]
    fn diff_identical() {
        let text = "same\ncontent\nhere\n";
        let result = compute_diff(text, text);
        assert_eq!(result.additions, 0);
        assert_eq!(result.deletions, 0);
        // Unified diff should be empty for identical content
        assert!(result.unified.is_empty() || !result.unified.contains('+'));
    }

    #[test]
    fn diff_empty_to_content() {
        let result = compute_diff("", "new content\n");
        assert_eq!(result.additions, 1);
        assert_eq!(result.deletions, 0);
    }

    #[test]
    fn diff_content_to_empty() {
        let result = compute_diff("old content\n", "");
        assert_eq!(result.additions, 0);
        assert_eq!(result.deletions, 1);
    }
}
