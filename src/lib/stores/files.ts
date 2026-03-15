import { writable } from "svelte/store";

export interface FileState {
  path: string | null;
  name: string;
  originalContent: string;
  dirty: boolean;
}

function createFileStore() {
  const { subscribe, set, update } = writable<FileState>({
    path: null,
    name: "Untitled",
    originalContent: "",
    dirty: false,
  });

  return {
    subscribe,
    openFile(path: string, name: string, content: string) {
      set({
        path,
        name,
        originalContent: content,
        dirty: false,
      });
    },
    markDirty() {
      update((s) => ({ ...s, dirty: true }));
    },
    markClean() {
      update((s) => ({ ...s, dirty: false, originalContent: s.originalContent }));
    },
    reset() {
      set({
        path: null,
        name: "Untitled",
        originalContent: "",
        dirty: false,
      });
    },
  };
}

export const fileState = createFileStore();
