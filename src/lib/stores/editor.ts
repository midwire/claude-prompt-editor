import { writable } from "svelte/store";

export const currentContent = writable<string>("");

export interface EditorState {
  cursorLine: number;
  cursorColumn: number;
  language: string;
}

export const editorState = writable<EditorState>({
  cursorLine: 1,
  cursorColumn: 1,
  language: "prompt-md",
});
