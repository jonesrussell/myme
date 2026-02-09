import path from "path";
import { getConfig } from "../config.js";

export function resolveInWorkspace(relativePath: string): string {
  const { workspaceRoot } = getConfig();
  const normalized = path.normalize(relativePath).replace(/^(\.\.(\/|\\))+/, "");
  return path.join(workspaceRoot, normalized);
}

export function resolveAppPath(mode: "debug" | "release"): string {
  const config = getConfig();
  if (config.appPath) return config.appPath;
  const buildDir = "build-qt";
  const configName = mode === "release" ? "Release" : "Debug";
  const exe = process.platform === "win32" ? "myme-qt.exe" : "myme-qt";
  return path.join(config.workspaceRoot, buildDir, configName, exe);
}
