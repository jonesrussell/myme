import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { loadConfig } from "./config.js";
import { formatToolResult } from "./utils/exec.js";
import { listCrates } from "./tools/crates.js";
import { buildQt, cargoBuild, cargoTest, runApp } from "./tools/build.js";
import { qmlFormat } from "./tools/qml-format.js";
import { readResource, listResourceUris } from "./resources.js";
import { loadPromptTemplate } from "./prompts.js";
import { z } from "zod";
const LOG = (msg) => console.error(`[myme-mcp] ${msg}`);
async function main() {
    loadConfig();
    LOG("Config loaded");
    const server = new McpServer({
        name: "myme-mcp-server",
        version: "0.1.0",
    }, {
        capabilities: {
            tools: {},
            resources: {},
            prompts: {},
        },
    });
    // --- Tools ---
    server.registerTool("list_crates", {
        description: "List workspace crate names from Cargo.toml (no subprocess).",
        inputSchema: z.object({}),
    }, () => {
        LOG("tool list_crates");
        const r = listCrates();
        LOG(`list_crates exitCode=${r.exitCode} success=${r.success}`);
        return {
            content: [{ type: "text", text: formatToolResult(r) }],
            isError: !r.success,
        };
    });
    server.registerTool("cargo_build", {
        description: "Run cargo build --release in workspace root. Optionally build one package.",
        inputSchema: z.object({
            package: z.string().optional().describe("Crate name to build (e.g. myme-core)"),
        }),
    }, async (args) => {
        LOG("tool cargo_build");
        const r = await cargoBuild({ package: args?.package });
        LOG(`cargo_build exitCode=${r.exitCode} success=${r.success}`);
        return {
            content: [{ type: "text", text: formatToolResult(r) }],
            isError: !r.success,
        };
    });
    server.registerTool("cargo_test", {
        description: "Run cargo test for all workspace crates except myme-ui. Optional package filter.",
        inputSchema: z.object({
            package: z.string().optional().describe("Single crate to test (e.g. myme-core)"),
        }),
    }, async (args) => {
        LOG("tool cargo_test");
        const r = await cargoTest({ package: args?.package });
        LOG(`cargo_test exitCode=${r.exitCode} success=${r.success}`);
        return {
            content: [{ type: "text", text: formatToolResult(r) }],
            isError: !r.success,
        };
    });
    server.registerTool("build_qt", {
        description: "Run scripts/build.ps1 for full Qt app build (Windows only).",
        inputSchema: z.object({}),
    }, async () => {
        LOG("tool build_qt");
        const r = await buildQt();
        LOG(`build_qt exitCode=${r.exitCode} success=${r.success}`);
        return {
            content: [{ type: "text", text: formatToolResult(r) }],
            isError: !r.success,
        };
    });
    server.registerTool("run_app", {
        description: "Run the MyMe Qt application. Supports mode (debug|release), env overrides, and optional args.",
        inputSchema: z.object({
            mode: z.enum(["debug", "release"]).optional().describe("Build mode"),
            env: z.record(z.string()).optional().describe("Env overrides (e.g. RUST_LOG, QT_LOGGING_RULES)"),
            args: z.array(z.string()).optional().describe("Arguments to pass to the app"),
        }),
    }, async (args) => {
        LOG("tool run_app");
        const r = await runApp({
            mode: args?.mode,
            env: args?.env,
            args: args?.args,
        });
        LOG(`run_app exitCode=${r.exitCode} success=${r.success}`);
        return {
            content: [{ type: "text", text: formatToolResult(r) }],
            isError: !r.success,
        };
    });
    server.registerTool("qml_format", {
        description: "Format QML files with qmlformat. Paths relative to workspace root.",
        inputSchema: z.object({
            paths: z.array(z.string()).describe("Paths to QML files (relative to workspace)"),
        }),
    }, async (args) => {
        LOG("tool qml_format");
        const paths = args?.paths ?? [];
        const r = await qmlFormat({ paths });
        LOG(`qml_format exitCode=${r.exitCode} success=${r.success}`);
        return {
            content: [{ type: "text", text: formatToolResult(r) }],
            isError: !r.success,
        };
    });
    // --- Resources (fixed URIs) ---
    for (const { uri, name, description } of listResourceUris()) {
        server.registerResource(name, uri, { description }, async (url) => {
            LOG(`resource read ${url.toString()}`);
            const result = await readResource(url.toString());
            return {
                contents: result.contents.map((c) => ({ uri: c.uri, text: c.text })),
            };
        });
    }
    // --- Prompts (stub for scaffold; full prompts in step 3) ---
    server.registerPrompt("add_new_ui_page", {
        description: "Get a step-by-step checklist to add a new QML page and model.",
        argsSchema: {
            pageName: z.string().describe("Human-readable page name (e.g. Settings)"),
        },
    }, async (args) => {
        LOG("prompt add_new_ui_page");
        const pageName = args?.pageName ?? "NewPage";
        const text = loadPromptTemplate("add-new-ui-page", { pageName });
        return {
            messages: [{ role: "user", content: { type: "text", text } }],
        };
    });
    server.registerPrompt("add_new_service_client", {
        description: "Get a step-by-step checklist to add a new service client in myme-services.",
        argsSchema: {
            crateName: z.string().optional().describe("Optional crate/module name"),
        },
    }, async (args) => {
        LOG("prompt add_new_service_client");
        const text = loadPromptTemplate("add-new-service-client", { crateName: args?.crateName ?? "" });
        return {
            messages: [{ role: "user", content: { type: "text", text } }],
        };
    });
    server.registerPrompt("add_oauth_provider", {
        description: "Get a step-by-step checklist to add a new OAuth provider in myme-auth.",
        argsSchema: {
            providerName: z.string().describe("Provider name (e.g. GitHub)"),
        },
    }, async (args) => {
        LOG("prompt add_oauth_provider");
        const providerName = args?.providerName ?? "NewProvider";
        const text = loadPromptTemplate("add-oauth-provider", { providerName });
        return {
            messages: [{ role: "user", content: { type: "text", text } }],
        };
    });
    const transport = new StdioServerTransport();
    await server.connect(transport);
    LOG("Connected to stdio transport");
}
main().catch((err) => {
    console.error("[myme-mcp] Fatal:", err);
    process.exit(1);
});
