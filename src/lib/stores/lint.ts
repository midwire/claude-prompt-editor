import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface LintResult {
  rule_id: string;
  severity: "Error" | "Warning" | "Suggestion";
  message: string;
  detail: string;
  block_index: number | null;
  fix_suggestion: string | null;
}

export const lintResults = writable<LintResult[]>([]);

let debounceTimer: ReturnType<typeof setTimeout>;

export function runLint(content: string) {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(async () => {
    try {
      const results: LintResult[] = await invoke("lint_prompt", {
        content,
        projectDir: null,
      });
      lintResults.set(results);
    } catch {
      lintResults.set([]);
    }
  }, 500);
}

export const errorCount = derived(lintResults, ($r) =>
  $r.filter((r) => r.severity === "Error").length,
);

export const warningCount = derived(lintResults, ($r) =>
  $r.filter((r) => r.severity === "Warning").length,
);

export const suggestionCount = derived(lintResults, ($r) =>
  $r.filter((r) => r.severity === "Suggestion").length,
);
