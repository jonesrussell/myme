import { runCommand } from "./utils/exec.js";
import { getConfig } from "./config.js";
import { readResourceFile } from "./utils/readFile.js";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const RESOURCE_URI_PREFIX = "myme://";
export async function readResource(uri) {
    if (!uri.startsWith(RESOURCE_URI_PREFIX)) {
        throw new Error(`Unknown resource URI: ${uri}`);
    }
    const pathPart = uri.slice(RESOURCE_URI_PREFIX.length);
    let text;
    switch (pathPart) {
        case "theme":
            text = await readResourceFile("theme-reference.md");
            break;
        case "pages":
            text = await readResourceFile("pages-checklist.md");
            break;
        case "project-context":
            text = await readResourceFile("project-context.md");
            break;
        case "version":
            text = await getVersionContent();
            break;
        default:
            throw new Error(`Unknown resource: ${pathPart}`);
    }
    return {
        contents: [{ uri, text }],
    };
}
async function getVersionContent() {
    const pkgPath = path.resolve(__dirname, "..", "package.json");
    const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf-8"));
    const serverVersion = pkg.version ?? "0.0.0";
    const nodeVersion = process.version;
    let gitHash = "unknown";
    try {
        const config = getConfig();
        const r = await runCommand("git", ["rev-parse", "HEAD"], { cwd: config.workspaceRoot });
        if (r.success && r.stdout)
            gitHash = r.stdout.trim();
    }
    catch {
        // ignore
    }
    return [
        "# MyMe MCP Server – Version",
        "",
        `- **MCP server version**: ${serverVersion}`,
        `- **Node version**: ${nodeVersion}`,
        `- **MyMe repo git hash**: ${gitHash}`,
    ].join("\n");
}
export function listResourceUris() {
    return [
        { uri: "myme://theme", name: "theme", description: "Theme colors and typography reference" },
        { uri: "myme://pages", name: "pages", description: "QML pages and models checklist" },
        { uri: "myme://project-context", name: "project-context", description: "Short project summary" },
        { uri: "myme://version", name: "version", description: "Server version, git hash, Node version" },
    ];
}
