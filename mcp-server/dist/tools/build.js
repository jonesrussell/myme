import { getConfig } from "../config.js";
import { runCommand } from "../utils/exec.js";
import { resolveAppPath } from "../utils/paths.js";
import path from "path";
const isWindows = process.platform === "win32";
export async function buildQt() {
    const { workspaceRoot } = getConfig();
    const scriptPath = path.join(workspaceRoot, "build-qt.ps1");
    if (isWindows) {
        const r = await runCommand("powershell", ["-ExecutionPolicy", "Bypass", "-File", scriptPath], { cwd: workspaceRoot });
        return r;
    }
    return {
        success: false,
        exitCode: 1,
        stdout: "",
        stderr: "build_qt is only supported on Windows (build-qt.ps1). On other platforms run cargo build and CMake manually.",
        message: "Unsupported platform",
    };
}
export async function cargoBuild(args) {
    const { workspaceRoot } = getConfig();
    const cmdArgs = ["build", "--release"];
    if (args.package) {
        cmdArgs.push("--package", args.package);
    }
    return runCommand("cargo", cmdArgs, { cwd: workspaceRoot });
}
export async function cargoTest(args) {
    const { workspaceRoot } = getConfig();
    const { getWorkspaceCrates } = await import("./crates.js");
    const members = getWorkspaceCrates();
    const testable = members.filter((m) => m !== "crates/myme-ui");
    if (args.package) {
        const pkg = args.package.startsWith("crates/") ? args.package : `crates/${args.package}`;
        if (!testable.includes(pkg)) {
            return {
                success: false,
                exitCode: 1,
                stdout: "",
                stderr: `Package ${args.package} not in testable crates (myme-ui is excluded).`,
                message: "Invalid package",
            };
        }
        return runCommand("cargo", ["test", "-p", pkg], { cwd: workspaceRoot });
    }
    const flat = testable.flatMap((c) => ["-p", c]);
    return runCommand("cargo", ["test", ...flat], { cwd: workspaceRoot });
}
export async function runApp(args) {
    const mode = args.mode ?? "release";
    const appPath = resolveAppPath(mode);
    const { workspaceRoot } = getConfig();
    const env = { ...process.env, ...(args.env ?? {}) };
    const spawnArgs = args.args ?? [];
    const r = await runCommand(appPath, spawnArgs, { cwd: workspaceRoot, env: env });
    return r;
}
