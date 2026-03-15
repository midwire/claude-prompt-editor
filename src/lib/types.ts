// TypeScript AST types mirroring the Rust parser types

export interface ThinkingConfig {
  type: string;
}

export interface PromptMetadata {
  name: string;
  model: string;
  version: number;
  tags: string[];
  thinking?: ThinkingConfig;
  effort?: string;
  extra: Record<string, unknown>;
}

// Rust serde externally-tagged enum: unit variants serialize as strings,
// Custom(String) serializes as { Custom: "value" }
export type BlockKind =
  | "Role"
  | "Instructions"
  | "Examples"
  | "Example"
  | "Context"
  | "Documents"
  | "Freeform"
  | { Custom: string };

export interface Block {
  kind: BlockKind;
  tag_name: string | null;
  content: string;
  children: Block[];
  enabled: boolean;
  start_offset: number;
  end_offset: number;
}

export interface PromptAst {
  metadata: PromptMetadata;
  blocks: Block[];
  raw_frontmatter: string;
}

export function blockKindLabel(kind: BlockKind): string {
  if (typeof kind === "string") return kind;
  return kind.Custom;
}

export function blockKindIcon(kind: BlockKind): string {
  const k = typeof kind === "string" ? kind : "Custom";
  const icons: Record<string, string> = {
    Role: "user",
    Instructions: "list",
    Examples: "book",
    Example: "file",
    Context: "paperclip",
    Documents: "folder",
    Freeform: "edit",
    Custom: "tag",
  };
  return icons[k] ?? "tag";
}
