import { getConfig } from "../config.js";
import { runCommand, type ToolResult } from "../utils/exec.js";
import { resolveInWorkspace } from "../utils/paths.js";

const DEFAULT_QMLFORMAT_WIN = "C:\\Qt\\6.10.1\\msvc2022_64\\bin\\qmlformat.exe";

export async function qmlFormat(args: { paths: string[] }): Promise<ToolResult> {
  const config = getConfig();
  const qmlformatPath =
    config.qmlformatPath ??
    (process.platform === "win32" ? DEFAULT_QMLFORMAT_WIN : "qmlformat");
  const absolutePaths = args.paths.map((p) => resolveInWorkspace(p));
  if (absolutePaths.length === 0) {
    return {
      success: false,
      exitCode: 1,
      stdout: "",
      stderr: "At least one path is required.",
      message: "Missing paths",
    };
  }
  const r = await runCommand(qmlformatPath, ["-i", ...absolutePaths], {
    cwd: config.workspaceRoot,
  });
  return r;
}
