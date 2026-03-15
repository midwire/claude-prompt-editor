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

export function parseFromContent(content: string) {
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
  const content = await serializeAst(ast);
  currentContent.set(content);
  currentAst.set(ast);
  fileState.markDirty();
}
