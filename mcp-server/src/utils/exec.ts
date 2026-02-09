import { spawn } from "child_process";

export type ToolResult = {
  success: boolean;
  exitCode: number;
  stdout: string;
  stderr: string;
  message?: string;
};

export function toolResult(
  success: boolean,
  exitCode: number,
  stdout: string,
  stderr: string,
  message?: string
): ToolResult {
  return { success, exitCode, stdout, stderr, message };
}

export function okResult(stdout: string, message?: string): ToolResult {
  return toolResult(true, 0, stdout, "", message);
}

export function failResult(
  exitCode: number,
  stdout: string,
  stderr: string,
  message?: string
): ToolResult {
  return toolResult(false, exitCode, stdout, stderr, message);
}

export function formatToolResult(r: ToolResult): string {
  const parts: string[] = [];
  if (r.message) parts.push(r.message);
  parts.push(`success: ${r.success}, exitCode: ${r.exitCode}`);
  if (r.stdout) parts.push(`stdout:\n${r.stdout}`);
  if (r.stderr) parts.push(`stderr:\n${r.stderr}`);
  return parts.join("\n");
}

export function runCommand(
  command: string,
  args: string[],
  options: { cwd: string; env?: NodeJS.ProcessEnv } = { cwd: process.cwd() }
): Promise<ToolResult> {
  return new Promise((resolve) => {
    const proc = spawn(command, args, {
      cwd: options.cwd,
      env: { ...process.env, ...options.env },
      shell: process.platform === "win32",
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    proc.stdout?.on("data", (chunk: Buffer) => {
      stdout += chunk.toString("utf8");
    });
    proc.stderr?.on("data", (chunk: Buffer) => {
      stderr += chunk.toString("utf8");
    });
    proc.on("close", (code, signal) => {
      const exitCode = code ?? (signal ? 1 : 0);
      const success = exitCode === 0;
      resolve(
        success
          ? toolResult(true, exitCode, stdout.trim(), stderr.trim())
          : toolResult(false, exitCode, stdout.trim(), stderr.trim())
      );
    });
    proc.on("error", (err) => {
      resolve(
        failResult(1, "", "", `Failed to spawn: ${err.message}`)
      );
    });
  });
}
