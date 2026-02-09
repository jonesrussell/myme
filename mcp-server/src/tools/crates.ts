import { getConfig } from "../config.js";
import { readFileUtf8Sync } from "../utils/readFile.js";
import { okResult, failResult, type ToolResult } from "../utils/exec.js";
import path from "path";

export function listCrates(): ToolResult {
  try {
    const { workspaceRoot } = getConfig();
    const cargoPath = path.join(workspaceRoot, "Cargo.toml");
    const content = readFileUtf8Sync(cargoPath);
    const match = content.match(/\[workspace\]\s*\n\s*members\s*=\s*\[([\s\S]*?)\]/);
    if (!match) {
      return okResult("[]", "No workspace.members found in Cargo.toml");
    }
    const membersStr = match[1];
    const members = membersStr
      .split(",")
      .map((s) => s.replace(/^\s*["']|["']\s*$/g, "").trim())
      .filter(Boolean);
    const stdout = JSON.stringify(members, null, 2);
    return okResult(stdout, `Found ${members.length} crates`);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    return failResult(1, "", "", message);
  }
}

export function getWorkspaceCrates(): string[] {
  const { workspaceRoot } = getConfig();
  const content = readFileUtf8Sync(path.join(workspaceRoot, "Cargo.toml"));
  const match = content.match(/\[workspace\]\s*\n\s*members\s*=\s*\[([\s\S]*?)\]/);
  if (!match) return [];
  const membersStr = match[1];
  return membersStr
    .split(",")
    .map((s) => s.replace(/^\s*["']|["']\s*$/g, "").trim())
    .filter(Boolean);
}
