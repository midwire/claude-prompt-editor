<script lang="ts">
  import { templates, loadTemplates, type Template } from "../../stores/presets";

  let { onclose, oncreate }: { onclose: () => void; oncreate: (name: string, content: string) => void } = $props();

  let promptName = $state("");
  let selectedTemplate = $state<Template | null>(null);
  let templateList = $state<Template[]>([]);

  templates.subscribe((t) => (templateList = t));

  $effect(() => {
    loadTemplates();
  });

  function handleCreate() {
    if (!promptName.trim()) return;
    const tmpl = selectedTemplate ?? templateList[0];
    if (!tmpl) return;

    const content = tmpl.content.replace(/\{\{name\}\}/g, promptName.trim());
    oncreate(promptName.trim(), content);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
    if (e.key === "Enter" && promptName.trim()) handleCreate();
  }

  function categoryLabel(cat: Template["category"]): string {
    return cat === "Blank" ? "Blank" : "Use Case";
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="overlay" role="dialog" aria-modal="true" onkeydown={handleKeydown}>
  <div class="wizard">
    <div class="wizard-header">
      <h2>New Prompt</h2>
      <button class="close-btn" onclick={onclose} aria-label="Close">x</button>
    </div>

    <div class="wizard-body">
      <label class="name-label">
        Prompt Name
        <input
          type="text"
          bind:value={promptName}
          placeholder="my-prompt"
          class="name-input"
        />
      </label>

      <div class="template-section">
        <h3>Choose a template</h3>
        <div class="template-grid">
          {#each templateList as tmpl}
            <button
              class="template-card"
              class:selected={selectedTemplate?.id === tmpl.id}
              onclick={() => (selectedTemplate = tmpl)}
            >
              <span class="template-category">{categoryLabel(tmpl.category)}</span>
              <span class="template-name">{tmpl.name}</span>
              <span class="template-desc">{tmpl.description}</span>
            </button>
          {/each}
        </div>
      </div>
    </div>

    <div class="wizard-footer">
      <button class="btn-cancel" onclick={onclose}>Cancel</button>
      <button
        class="btn-create"
        onclick={handleCreate}
        disabled={!promptName.trim()}
      >
        Create
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .wizard {
    background-color: #1e1e2e;
    border: 1px solid #313244;
    border-radius: 8px;
    width: 560px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    color: #cdd6f4;
  }

  .wizard-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #313244;
  }

  .wizard-header h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: #6c7086;
    font-size: 18px;
    cursor: pointer;
    padding: 4px;
  }

  .close-btn:hover {
    color: #cdd6f4;
  }

  .wizard-body {
    padding: 20px;
    overflow-y: auto;
    flex: 1;
  }

  .name-label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 20px;
  }

  .name-input {
    padding: 8px 12px;
    background-color: #181825;
    border: 1px solid #313244;
    border-radius: 4px;
    color: #cdd6f4;
    font-size: 14px;
    outline: none;
  }

  .name-input:focus {
    border-color: #89b4fa;
  }

  .template-section h3 {
    margin: 0 0 12px 0;
    font-size: 13px;
    font-weight: 500;
  }

  .template-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .template-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px;
    background-color: #181825;
    border: 1px solid #313244;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    color: #cdd6f4;
    transition: border-color 0.15s;
  }

  .template-card:hover {
    border-color: #585b70;
  }

  .template-card.selected {
    border-color: #89b4fa;
    background-color: #1e1e3e;
  }

  .template-category {
    font-size: 10px;
    text-transform: uppercase;
    color: #6c7086;
    font-weight: 600;
    letter-spacing: 0.5px;
  }

  .template-name {
    font-size: 14px;
    font-weight: 600;
  }

  .template-desc {
    font-size: 11px;
    color: #a6adc8;
    line-height: 1.4;
  }

  .wizard-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px 20px;
    border-top: 1px solid #313244;
  }

  .btn-cancel {
    padding: 6px 16px;
    background: none;
    border: 1px solid #313244;
    border-radius: 4px;
    color: #cdd6f4;
    cursor: pointer;
    font-size: 13px;
  }

  .btn-cancel:hover {
    background-color: #313244;
  }

  .btn-create {
    padding: 6px 16px;
    background-color: #0e639c;
    border: none;
    border-radius: 4px;
    color: #ffffff;
    cursor: pointer;
    font-size: 13px;
  }

  .btn-create:hover:not(:disabled) {
    background-color: #1177bb;
  }

  .btn-create:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
