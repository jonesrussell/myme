import { spawn } from "child_process";
export function toolResult(success, exitCode, stdout, stderr, message) {
    return { success, exitCode, stdout, stderr, message };
}
export function okResult(stdout, message) {
    return toolResult(true, 0, stdout, "", message);
}
export function failResult(exitCode, stdout, stderr, message) {
    return toolResult(false, exitCode, stdout, stderr, message);
}
export function formatToolResult(r) {
    const parts = [];
    if (r.message)
        parts.push(r.message);
    parts.push(`success: ${r.success}, exitCode: ${r.exitCode}`);
    if (r.stdout)
        parts.push(`stdout:\n${r.stdout}`);
    if (r.stderr)
        parts.push(`stderr:\n${r.stderr}`);
    return parts.join("\n");
}
export function runCommand(command, args, options = { cwd: process.cwd() }) {
    return new Promise((resolve) => {
        const proc = spawn(command, args, {
            cwd: options.cwd,
            env: { ...process.env, ...options.env },
            shell: process.platform === "win32",
            stdio: ["ignore", "pipe", "pipe"],
        });
        let stdout = "";
        let stderr = "";
        proc.stdout?.on("data", (chunk) => {
            stdout += chunk.toString("utf8");
        });
        proc.stderr?.on("data", (chunk) => {
            stderr += chunk.toString("utf8");
        });
        proc.on("close", (code, signal) => {
            const exitCode = code ?? (signal ? 1 : 0);
            const success = exitCode === 0;
            resolve(success
                ? toolResult(true, exitCode, stdout.trim(), stderr.trim())
                : toolResult(false, exitCode, stdout.trim(), stderr.trim()));
        });
        proc.on("error", (err) => {
            resolve(failResult(1, "", "", `Failed to spawn: ${err.message}`));
        });
    });
}
