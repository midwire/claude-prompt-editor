<script lang="ts">
  import type { PromptMetadata } from "../../types";

  interface Props {
    metadata: PromptMetadata;
    onupdate: (metadata: PromptMetadata) => void;
  }

  let { metadata, onupdate }: Props = $props();

  let tagsText = $derived(metadata.tags.join(", "));

  function updateField(field: string, value: string) {
    if (field === "tags") {
      const tags = value
        .split(",")
        .map((t) => t.trim())
        .filter((t) => t.length > 0);
      onupdate({ ...metadata, tags });
    } else {
      onupdate({ ...metadata, [field]: value });
    }
  }
</script>

<div class="metadata-block">
  <div class="metadata-header">
    <span class="metadata-icon">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
        <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319z"/>
      </svg>
    </span>
    <span class="metadata-label">Metadata</span>
  </div>
  <div class="metadata-body">
    <div class="field">
      <label for="meta-name">Name</label>
      <input
        id="meta-name"
        type="text"
        value={metadata.name}
        oninput={(e) => updateField("name", (e.target as HTMLInputElement).value)}
      />
    </div>
    <div class="field">
      <label for="meta-model">Model</label>
      <select
        id="meta-model"
        onchange={(e) => updateField("model", (e.target as HTMLSelectElement).value)}
      >
        <option value="claude-opus-4-6" selected={metadata.model === "claude-opus-4-6"}>Claude Opus 4.6</option>
        <option value="claude-sonnet-4-6" selected={metadata.model === "claude-sonnet-4-6"}>Claude Sonnet 4.6</option>
        <option value="claude-haiku-4-5" selected={metadata.model === "claude-haiku-4-5"}>Claude Haiku 4.5</option>
      </select>
    </div>
    <div class="field">
      <label for="meta-effort">Effort</label>
      <select
        id="meta-effort"
        onchange={(e) => {
          const val = (e.target as HTMLSelectElement).value;
          updateField("effort", val);
        }}
      >
        {#each [
          { value: "", label: "None" },
          { value: "low", label: "Low" },
          { value: "medium", label: "Medium" },
          { value: "high", label: "High" },
          { value: "max", label: "Max" },
        ] as opt}
          <option value={opt.value} selected={(metadata.effort ?? "") === opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="meta-tags">Tags</label>
      <input
        id="meta-tags"
        type="text"
        value={tagsText}
        oninput={(e) => updateField("tags", (e.target as HTMLInputElement).value)}
        placeholder="tag1, tag2, ..."
      />
    </div>
  </div>
</div>

<style>
  .metadata-block {
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    margin-bottom: 8px;
    background-color: #252526;
    overflow: hidden;
  }

  .metadata-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    background-color: #2d2d2d;
    border-bottom: 1px solid #3c3c3c;
    font-size: 13px;
  }

  .metadata-icon {
    color: #569cd6;
    display: flex;
    align-items: center;
  }

  .metadata-label {
    font-weight: 500;
    color: #cccccc;
  }

  .metadata-body {
    padding: 8px;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .field label {
    font-size: 11px;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .field input,
  .field select {
    background-color: #1e1e1e;
    color: #d4d4d4;
    border: 1px solid #3c3c3c;
    border-radius: 3px;
    padding: 4px 8px;
    font-size: 13px;
    font-family: inherit;
  }

  .field input:focus,
  .field select:focus {
    outline: none;
    border-color: #007acc;
  }

  .field select {
    cursor: pointer;
    -webkit-appearance: none;
    -moz-appearance: none;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23888' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
    padding-right: 28px;
  }

  .field select option {
    background-color: #1e1e1e;
    color: #d4d4d4;
  }
</style>
