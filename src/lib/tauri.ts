import { invoke } from "@tauri-apps/api/core";
import type { PromptAst } from "./types";

export interface PromptFile {
  path: string;
  name: string;
  content: string;
}

export interface PromptListEntry {
  path: string;
  name: string;
  modified: number;
}

export async function openPrompt(path: string): Promise<PromptFile> {
  return invoke("open_prompt", { path });
}

export async function savePrompt(path: string, content: string): Promise<void> {
  return invoke("save_prompt", { path, content });
}

export async function listPrompts(dir: string): Promise<PromptListEntry[]> {
  return invoke("list_prompts", { dir });
}

export async function parseContent(content: string): Promise<PromptAst> {
  return invoke("parse_content", { content });
}

export async function serializeAst(ast: PromptAst): Promise<string> {
  return invoke("serialize_ast", { ast });
}
