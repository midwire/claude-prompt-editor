import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface MetadataDefaults {
  model?: string;
  tags?: string[];
}

export interface Preset {
  id: string;
  name: string;
  category: "Role" | "Instructions" | "Constraints" | "OutputFormat" | "ExampleSkeleton";
  content: string;
  tag_name: string | null;
  metadata_defaults: MetadataDefaults | null;
}

export interface Template {
  id: string;
  name: string;
  category: "Blank" | "UseCase";
  description: string;
  content: string;
}

export const presets = writable<Preset[]>([]);
export const templates = writable<Template[]>([]);

export async function loadPresets(): Promise<void> {
  try {
    const result: Preset[] = await invoke("list_presets");
    presets.set(result);
  } catch (e) {
    console.error("Failed to load presets:", e);
  }
}

export async function loadTemplates(): Promise<void> {
  try {
    const result: Template[] = await invoke("list_templates");
    templates.set(result);
  } catch (e) {
    console.error("Failed to load templates:", e);
  }
}
