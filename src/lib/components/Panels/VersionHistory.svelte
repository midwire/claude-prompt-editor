<script lang="ts">
  import {
    versionHistory,
    currentDiff,
    computeDiff,
    type VersionEntry,
    type DiffResult,
    type VersionHistory as VH,
  } from "../../stores/version";

  let { projectDir, promptName }: { projectDir: string; promptName: string } = $props();

  let history = $state<VH | null>(null);
  let diff = $state<DiffResult | null>(null);
  let selectedOld = $state<number | null>(null);
  let selectedNew = $state<number | null>(null);

  versionHistory.subscribe((h) => (history = h));
  currentDiff.subscribe((d) => (diff = d));

  function versions(): VersionEntry[] {
    if (!history) return [];
    return [...history.versions].reverse();
  }

  function handleVersionClick(id: number) {
    if (selectedOld === null) {
      selectedOld = id;
    } else if (selectedNew === null && id !== selectedOld) {
      selectedNew = id;
      // Ensure old < new for consistent diff direction
      const oldId = Math.min(selectedOld, selectedNew);
      const newId = Math.max(selectedOld, selectedNew);
      computeDiff(projectDir, promptName, oldId, newId);
    } else {
      // Reset selection
      selectedOld = id;
      selectedNew = null;
      currentDiff.set(null);
    }
  }

  function isSelected(id: number): boolean {
    return id === selectedOld || id === selectedNew;
  }

  function formatDate(timestamp: string): string {
    try {
      const d = new Date(timestamp);
      return d.toLocaleString(undefined, {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return timestamp;
    }
  }

  function parseDiffLines(unified: string): Array<{ type: "add" | "del" | "ctx" | "header"; text: string }> {
    return unified.split("\n").map((line) => {
      if (line.startsWith("+++") || line.startsWith("---") || line.startsWith("@@")) {
        return { type: "header" as const, text: line };
      }
      if (line.startsWith("+")) {
        return { type: "add" as const, text: line };
      }
      if (line.startsWith("-")) {
        return { type: "del" as const, text: line };
      }
      return { type: "ctx" as const, text: line };
    });
  }
</script>

<div class="version-history">
  <div class="header">
    <h3>Version History</h3>
  </div>

  {#if !history || versions().length === 0}
    <div class="empty">No versions saved yet. Versions are saved automatically when you save the file.</div>
  {:else}
    <div class="instructions">
      {#if selectedOld !== null && selectedNew === null}
        Click another version to compare
      {:else if selectedOld === null}
        Click two versions to compare
      {:else}
        Click a version to start new comparison
      {/if}
    </div>

    <ul class="timeline">
      {#each versions() as version}
        <li>
          <button
            class="version-item"
            class:selected={isSelected(version.id)}
            onclick={() => handleVersionClick(version.id)}
          >
            <span class="version-id">v{version.id}</span>
            <span class="version-date">{formatDate(version.timestamp)}</span>
            {#if version.summary}
              <span class="version-summary">{version.summary}</span>
            {/if}
          </button>
        </li>
      {/each}
    </ul>

    {#if diff}
      <div class="diff-section">
        <div class="diff-header">
          <span class="diff-stat">
            <span class="additions">+{diff.additions}</span>
            <span class="deletions">-{diff.deletions}</span>
          </span>
        </div>
        <pre class="diff-content">{#each parseDiffLines(diff.unified) as line}<span class="diff-line {line.type}">{line.text}
</span>{/each}</pre>
      </div>
    {/if}
  {/if}
</div>

<style>
  .version-history {
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

  .instructions {
    font-size: 11px;
    color: #6c7086;
    margin-bottom: 8px;
    font-style: italic;
  }

  .empty {
    color: #a6adc8;
    font-style: italic;
    text-align: center;
    padding: 24px 8px;
  }

  .timeline {
    list-style: none;
    padding: 0;
    margin: 0 0 12px 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .version-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 10px;
    border: 1px solid #313244;
    border-radius: 4px;
    background-color: #181825;
    color: #cdd6f4;
    cursor: pointer;
    text-align: left;
    font-size: 12px;
  }

  .version-item:hover {
    background-color: #11111b;
  }

  .version-item.selected {
    border-color: #89b4fa;
    background-color: #1e1e3e;
  }

  .version-id {
    font-weight: 700;
    color: #89b4fa;
    min-width: 28px;
  }

  .version-date {
    color: #a6adc8;
    font-size: 11px;
  }

  .version-summary {
    color: #6c7086;
    font-size: 11px;
    margin-left: auto;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .diff-section {
    border-top: 1px solid #313244;
    padding-top: 12px;
  }

  .diff-header {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 8px;
  }

  .diff-stat {
    display: flex;
    gap: 8px;
    font-size: 12px;
    font-weight: 600;
  }

  .additions {
    color: #a6e3a1;
  }

  .deletions {
    color: #f38ba8;
  }

  .diff-content {
    margin: 0;
    padding: 8px;
    background-color: #181825;
    border-radius: 4px;
    font-family: "Fira Code", "Cascadia Code", monospace;
    font-size: 11px;
    line-height: 1.5;
    overflow-x: auto;
    white-space: pre;
  }

  .diff-line {
    display: block;
  }

  .diff-line.add {
    background-color: rgba(166, 227, 161, 0.1);
    color: #a6e3a1;
  }

  .diff-line.del {
    background-color: rgba(243, 139, 168, 0.1);
    color: #f38ba8;
  }

  .diff-line.header {
    color: #89b4fa;
    font-weight: 600;
  }

  .diff-line.ctx {
    color: #a6adc8;
  }
</style>
