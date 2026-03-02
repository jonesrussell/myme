#!/usr/bin/env node

const { McpServer } = require("@modelcontextprotocol/sdk/server/mcp.js");
const { StdioServerTransport } = require("@modelcontextprotocol/sdk/server/stdio.js");
const fs = require("fs");
const path = require("path");

const SPECS_DIR = path.resolve(__dirname, "../../docs/specs");

function loadSpecs() {
  if (!fs.existsSync(SPECS_DIR)) return [];
  return fs
    .readdirSync(SPECS_DIR)
    .filter((f) => f.endsWith(".md"))
    .map((f) => {
      const filePath = path.join(SPECS_DIR, f);
      const content = fs.readFileSync(filePath, "utf-8");
      const name = f.replace(".md", "");
      const firstLine = content.split("\n").find((l) => l.startsWith("# "));
      const description = firstLine ? firstLine.replace("# ", "").trim() : name;
      return { name, description, file: f, content };
    });
}

const server = new McpServer({
  name: "myme-specs",
  version: "1.0.0",
});

server.tool("list_specs", "List all available subsystem specs", {}, () => {
  const specs = loadSpecs();
  const listing = specs.map((s) => ({
    name: s.name,
    description: s.description,
    file: s.file,
    lines: s.content.split("\n").length,
  }));
  return { content: [{ type: "text", text: JSON.stringify(listing, null, 2) }] };
});

server.tool(
  "get_spec",
  "Get full content of a subsystem spec by name",
  { name: { type: "string", description: "Spec name (e.g. 'core-auth', 'ui-bridge')" } },
  ({ name }) => {
    const specs = loadSpecs();
    const spec = specs.find((s) => s.name === name);
    if (!spec) {
      const available = specs.map((s) => s.name).join(", ");
      return {
        content: [{ type: "text", text: `Spec '${name}' not found. Available: ${available}` }],
        isError: true,
      };
    }
    return { content: [{ type: "text", text: spec.content }] };
  }
);

server.tool(
  "search_specs",
  "Search specs by keyword (case-insensitive substring match)",
  { query: { type: "string", description: "Search query" } },
  ({ query }) => {
    const specs = loadSpecs();
    const q = query.toLowerCase();
    const results = [];

    for (const spec of specs) {
      const lines = spec.content.split("\n");
      const matches = [];

      for (let i = 0; i < lines.length; i++) {
        if (lines[i].toLowerCase().includes(q)) {
          const start = Math.max(0, i - 2);
          const end = Math.min(lines.length - 1, i + 2);
          const context = lines.slice(start, end + 1).join("\n");
          matches.push({ line: i + 1, context });
        }
      }

      if (matches.length > 0) {
        results.push({ spec: spec.name, file: spec.file, matches });
      }
    }

    if (results.length === 0) {
      return { content: [{ type: "text", text: `No matches for '${query}'` }] };
    }
    return { content: [{ type: "text", text: JSON.stringify(results, null, 2) }] };
  }
);

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch(console.error);
