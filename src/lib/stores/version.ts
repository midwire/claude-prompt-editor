import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface VersionEntry {
  id: number;
  timestamp: string;
  content: string;
  content_hash: number;
  summary: string | null;
}

export interface VersionHistory {
  prompt_name: string;
  versions: VersionEntry[];
  next_id: number;
}

export interface DiffResult {
  unified: string;
  additions: number;
  deletions: number;
}

export const versionHistory = writable<VersionHistory | null>(null);
export const currentDiff = writable<DiffResult | null>(null);

export async function refreshHistory(
  projectDir: string,
  promptName: string,
): Promise<void> {
  try {
    const history: VersionHistory = await invoke("get_version_history", {
      projectDir,
      promptName,
    });
    versionHistory.set(history);
  } catch (e) {
    console.error("Failed to load version history:", e);
  }
}

export async function saveVersion(
  projectDir: string,
  promptName: string,
  content: string,
  summary?: string,
): Promise<void> {
  try {
    await invoke("save_prompt_version", {
      projectDir,
      promptName,
      content,
      summary: summary ?? null,
    });
    await refreshHistory(projectDir, promptName);
  } catch (e) {
    console.error("Failed to save version:", e);
  }
}

export async function computeDiff(
  projectDir: string,
  promptName: string,
  oldVersionId: number,
  newVersionId: number,
): Promise<void> {
  try {
    const diff: DiffResult = await invoke("diff_versions", {
      projectDir,
      promptName,
      oldVersionId,
      newVersionId,
    });
    currentDiff.set(diff);
  } catch (e) {
    console.error("Failed to compute diff:", e);
    currentDiff.set(null);
  }
}
