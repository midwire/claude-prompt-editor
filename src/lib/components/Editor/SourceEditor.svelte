<script lang="ts">
  import * as monaco from "monaco-editor";
  import { onMount, onDestroy } from "svelte";
  import { currentContent, editorState } from "../../stores/editor";
  import { fileState } from "../../stores/files";

  import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
  import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
  import cssWorker from "monaco-editor/esm/vs/language/css/css.worker?worker";
  import htmlWorker from "monaco-editor/esm/vs/language/html/html.worker?worker";
  import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";

  // Set up Monaco workers for Vite
  self.MonacoEnvironment = {
    getWorker(_: unknown, label: string) {
      if (label === "json") return new jsonWorker();
      if (label === "css" || label === "scss" || label === "less") return new cssWorker();
      if (label === "html" || label === "handlebars" || label === "razor") return new htmlWorker();
      if (label === "typescript" || label === "javascript") return new tsWorker();
      return new editorWorker();
    },
  };

  let container: HTMLDivElement | undefined = $state(undefined);
  let editor: monaco.editor.IStandaloneCodeEditor | undefined;
  let isUpdatingFromStore = false;

  function registerPromptLanguage() {
    if (monaco.languages.getLanguages().some((lang) => lang.id === "prompt-md")) {
      return;
    }

    monaco.languages.register({ id: "prompt-md" });

    monaco.languages.setMonarchTokensProvider("prompt-md", {
      tokenizer: {
        root: [
          // YAML frontmatter
          [/^---$/, { token: "keyword", next: "@frontmatter" }],
          // XML tags (opening/closing/self-closing)
          [/<\/?[\w-]+[^>]*\/?>/, "tag"],
          // Template variables {{ }}
          [/\{\{[\w.]+\}\}/, "variable"],
          // Markdown headings
          [/^#{1,6}\s.*$/, "keyword"],
          // Bold
          [/\*\*[^*]+\*\*/, "strong"],
          // Italic
          [/\*[^*]+\*/, "emphasis"],
          // Code blocks
          [/```[\s\S]*?```/, "string"],
          // Inline code
          [/`[^`]+`/, "string"],
          // Comments (HTML style)
          [/<!--[\s\S]*?-->/, "comment"],
        ],
        frontmatter: [
          [/^---$/, { token: "keyword", next: "@root" }],
          [/^[\w-]+:/, "attribute.name"],
          [/.*$/, "attribute.value"],
        ],
      },
    });
  }

  onMount(() => {
    if (!container) return;

    registerPromptLanguage();

    editor = monaco.editor.create(container, {
      value: "",
      language: "prompt-md",
      theme: "vs-dark",
      wordWrap: "on",
      lineNumbers: "on",
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 14,
      padding: { top: 8 },
      scrollBeyondLastLine: false,
      renderWhitespace: "selection",
      bracketPairColorization: { enabled: true },
    });

    // Sync editor changes to store
    editor.onDidChangeModelContent(() => {
      if (isUpdatingFromStore) return;
      const value = editor!.getModel()?.getValue() ?? "";
      currentContent.set(value);
      fileState.markDirty();
    });

    // Track cursor position
    editor.onDidChangeCursorPosition((e) => {
      editorState.set({
        cursorLine: e.position.lineNumber,
        cursorColumn: e.position.column,
        language: "prompt-md",
      });
    });

    // Subscribe to store changes (e.g., file open)
    const unsubscribe = currentContent.subscribe((value) => {
      if (!editor) return;
      const currentValue = editor.getModel()?.getValue() ?? "";
      if (currentValue !== value) {
        isUpdatingFromStore = true;
        editor.getModel()?.setValue(value);
        isUpdatingFromStore = false;
      }
    });

    return () => {
      unsubscribe();
    };
  });

  onDestroy(() => {
    editor?.dispose();
  });
</script>

<div class="editor-container" bind:this={container}></div>

<style>
  .editor-container {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
</style>
