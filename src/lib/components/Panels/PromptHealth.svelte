<script lang="ts">
  import {
    lintResults,
    errorCount,
    warningCount,
    suggestionCount,
    type LintResult,
  } from "../../stores/lint";

  let results = $state<LintResult[]>([]);
  let errors = $state(0);
  let warnings = $state(0);
  let suggestions = $state(0);
  let expandedIndex = $state<number | null>(null);

  lintResults.subscribe((r) => (results = r));
  errorCount.subscribe((n) => (errors = n));
  warningCount.subscribe((n) => (warnings = n));
  suggestionCount.subscribe((n) => (suggestions = n));

  function healthScore(): { label: string; color: string } {
    if (errors > 0) return { label: "Needs Work", color: "#f38ba8" };
    if (warnings > 0) return { label: "Fair", color: "#fab387" };
    if (suggestions > 0) return { label: "Good", color: "#a6e3a1" };
    return { label: "Excellent", color: "#a6e3a1" };
  }

  function severityIcon(severity: LintResult["severity"]): string {
    switch (severity) {
      case "Error":
        return "!!";
      case "Warning":
        return "!";
      case "Suggestion":
        return "i";
    }
  }

  function severityColor(severity: LintResult["severity"]): string {
    switch (severity) {
      case "Error":
        return "#f38ba8";
      case "Warning":
        return "#fab387";
      case "Suggestion":
        return "#89b4fa";
    }
  }

  function toggleExpand(index: number) {
    expandedIndex = expandedIndex === index ? null : index;
  }
</script>

<div class="prompt-health">
  <div class="header">
    <h3>Prompt Health</h3>
    <div class="score" style="color: {healthScore().color}">
      {healthScore().label}
    </div>
  </div>

  <div class="counts">
    <span class="count error-count" title="Errors">
      {errors} error{errors !== 1 ? "s" : ""}
    </span>
    <span class="count warning-count" title="Warnings">
      {warnings} warning{warnings !== 1 ? "s" : ""}
    </span>
    <span class="count suggestion-count" title="Suggestions">
      {suggestions} suggestion{suggestions !== 1 ? "s" : ""}
    </span>
  </div>

  {#if results.length === 0}
    <div class="empty">No issues found. Your prompt looks great.</div>
  {:else}
    <ul class="findings">
      {#each results as result, i}
        <li class="finding">
          <button
            class="finding-header"
            onclick={() => toggleExpand(i)}
          >
            <span
              class="severity-badge"
              style="background-color: {severityColor(result.severity)}"
            >
              {severityIcon(result.severity)}
            </span>
            <span class="finding-message">{result.message}</span>
            <span class="expand-icon">{expandedIndex === i ? "-" : "+"}</span>
          </button>
          {#if expandedIndex === i}
            <div class="finding-detail">
              <p class="detail-text">{result.detail}</p>
              {#if result.fix_suggestion}
                <p class="fix-suggestion">
                  <strong>Suggestion:</strong>
                  {result.fix_suggestion}
                </p>
              {/if}
              <span class="rule-id">{result.rule_id}</span>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .prompt-health {
    height: 100%;
    overflow-y: auto;
    padding: 12px;
    background-color: #1e1e2e;
    color: #cdd6f4;
    font-size: 13px;
    border-left: 1px solid #313244;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid #313244;
  }

  .header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: #cdd6f4;
  }

  .score {
    font-weight: 700;
    font-size: 13px;
  }

  .counts {
    display: flex;
    gap: 12px;
    margin-bottom: 12px;
    font-size: 12px;
  }

  .count {
    font-weight: 500;
  }

  .error-count {
    color: #f38ba8;
  }

  .warning-count {
    color: #fab387;
  }

  .suggestion-count {
    color: #89b4fa;
  }

  .empty {
    color: #a6adc8;
    font-style: italic;
    text-align: center;
    padding: 24px 8px;
  }

  .findings {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .finding {
    border-radius: 4px;
    background-color: #181825;
    overflow: hidden;
  }

  .finding-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 10px;
    border: none;
    background: none;
    color: #cdd6f4;
    cursor: pointer;
    text-align: left;
    font-size: 12px;
  }

  .finding-header:hover {
    background-color: #11111b;
  }

  .severity-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
    color: #1e1e2e;
    flex-shrink: 0;
  }

  .finding-message {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .expand-icon {
    color: #6c7086;
    font-size: 14px;
    flex-shrink: 0;
  }

  .finding-detail {
    padding: 8px 10px 10px 36px;
    border-top: 1px solid #313244;
    font-size: 12px;
    line-height: 1.5;
  }

  .detail-text {
    margin: 0 0 6px 0;
    color: #a6adc8;
  }

  .fix-suggestion {
    margin: 0 0 6px 0;
    color: #a6e3a1;
    font-size: 11px;
  }

  .rule-id {
    display: inline-block;
    font-size: 10px;
    color: #6c7086;
    background-color: #313244;
    padding: 1px 6px;
    border-radius: 3px;
  }
</style>
