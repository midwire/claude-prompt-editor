<script lang="ts">
  import { presets, loadPresets, type Preset } from "../../stores/presets";

  interface Props {
    onselect: (preset: Preset) => void;
    onclose: () => void;
  }

  let { onselect, onclose }: Props = $props();

  let allPresets = $state<Preset[]>([]);
  let searchQuery = $state("");
  let searchInput: HTMLInputElement | undefined = $state();

  presets.subscribe((p) => (allPresets = p));

  // Load presets on mount
  $effect(() => {
    loadPresets();
  });

  $effect(() => {
    if (searchInput) {
      searchInput.focus();
    }
  });

  function filteredPresets(): Preset[] {
    if (!searchQuery.trim()) return allPresets;
    const q = searchQuery.toLowerCase();
    return allPresets.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        p.category.toLowerCase().includes(q) ||
        p.content.toLowerCase().includes(q),
    );
  }

  function handleSelect(preset: Preset) {
    onselect(preset);
    onclose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains("palette-overlay")) {
      onclose();
    }
  }

  function categoryLabel(category: string): string {
    const labels: Record<string, string> = {
      Role: "Role",
      Instructions: "Instructions",
      Constraints: "Constraints",
      OutputFormat: "Output Format",
      ExampleSkeleton: "Example",
    };
    return labels[category] ?? category;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="palette-overlay" role="dialog" onclick={handleBackdropClick}>
  <div class="palette">
    <div class="palette-header">
      <input
        bind:this={searchInput}
        bind:value={searchQuery}
        type="text"
        class="search-input"
        placeholder="Search presets..."
      />
    </div>
    <ul class="preset-list">
      {#each filteredPresets() as preset}
        <li>
          <button class="preset-item" onclick={() => handleSelect(preset)}>
            <span class="preset-category">{categoryLabel(preset.category)}</span>
            <span class="preset-name">{preset.name}</span>
            <span class="preset-preview">{preset.content.slice(0, 80)}...</span>
          </button>
        </li>
      {:else}
        <li class="no-results">No presets match your search.</li>
      {/each}
    </ul>
  </div>
</div>

<style>
  .palette-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 80px;
    z-index: 100;
  }

  .palette {
    width: 500px;
    max-height: 400px;
    background-color: #1e1e2e;
    border: 1px solid #313244;
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  .palette-header {
    padding: 12px;
    border-bottom: 1px solid #313244;
  }

  .search-input {
    width: 100%;
    padding: 8px 12px;
    font-size: 14px;
    background-color: #181825;
    color: #cdd6f4;
    border: 1px solid #45475a;
    border-radius: 4px;
    outline: none;
    box-sizing: border-box;
  }

  .search-input:focus {
    border-color: #89b4fa;
  }

  .preset-list {
    list-style: none;
    padding: 0;
    margin: 0;
    overflow-y: auto;
    flex: 1;
  }

  .preset-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: 10px 16px;
    background: none;
    border: none;
    border-bottom: 1px solid #313244;
    color: #cdd6f4;
    cursor: pointer;
    text-align: left;
    font-size: 13px;
  }

  .preset-item:hover {
    background-color: #313244;
  }

  .preset-category {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    color: #89b4fa;
    letter-spacing: 0.5px;
  }

  .preset-name {
    font-weight: 500;
    color: #cdd6f4;
  }

  .preset-preview {
    font-size: 11px;
    color: #6c7086;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .no-results {
    padding: 24px 16px;
    text-align: center;
    color: #6c7086;
    font-style: italic;
  }
</style>
