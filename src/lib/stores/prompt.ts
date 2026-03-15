import { writable } from "svelte/store";
import type { PromptAst } from "../types";
import { parseContent, serializeAst } from "../tauri";
import { currentContent } from "./editor";
import { fileState } from "./files";

export const currentAst = writable<PromptAst | null>(null);
export const parseError = writable<string | null>(null);

/** Current editor mode: "source" or "structure" */
export const editorMode = writable<"source" | "structure">("source");

let debounceTimer: ReturnType<typeof setTimeout>;

/**
 * Guard to skip re-parsing when content was set by syncAstToContent.
 * This prevents losing AST-only state (like enabled flags) during
 * structure mode edits.
 */
let skipNextParse = false;

export function parseFromContent(content: string) {
  if (skipNextParse) {
    skipNextParse = false;
    return;
  }
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(async () => {
    try {
      const ast = await parseContent(content);
      currentAst.set(ast);
      parseError.set(null);
    } catch (e) {
      parseError.set(String(e));
    }
  }, 300);
}

export async function syncAstToContent(ast: PromptAst) {
  // Serialize all blocks (including disabled ones) to keep source view complete.
  // The enabled flag is tracked only in the AST, not encoded in the markdown.
  const content = await serializeAst(ast);
  // Skip the re-parse that would be triggered by currentContent changing,
  // since we already have the correct AST with enabled flags preserved.
  skipNextParse = true;
  currentContent.set(content);
  currentAst.set(ast);
  fileState.markDirty();
}

/** Update only the AST without serializing (for flag-only changes like enable/disable) */
export function updateAstOnly(ast: PromptAst) {
  currentAst.set(ast);
  fileState.markDirty();
}
