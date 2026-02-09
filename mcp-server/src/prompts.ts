import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PROMPTS_DIR = path.resolve(__dirname, "..", "prompts");

function toPascalCase(s: string): string {
  return s.replace(/(?:^|\s|[-_])(\w)/g, (_, c) => c.toUpperCase());
}

function toSnakeCase(s: string): string {
  return s
    .replace(/([A-Z])/g, "_$1")
    .toLowerCase()
    .replace(/^_/, "");
}

export function loadPromptTemplate(
  name: string,
  args: Record<string, string | undefined>
): string {
  const filePath = path.join(PROMPTS_DIR, `${name}.md`);
  const raw = fs.readFileSync(filePath, "utf-8");
  const substitutions: Record<string, string> = {};
  for (const [k, v] of Object.entries(args)) {
    if (v !== undefined) substitutions[k] = v;
  }

  if (args.pageName) {
    const p = args.pageName.trim();
    const pascal = toPascalCase(p) || "NewPage";
    const snake = toSnakeCase(pascal);
    substitutions.pageName = pascal;
    substitutions.modelName = `${pascal}Model`;
    substitutions.modelSnake = snake;
    substitutions.pageTitle = pascal;
  }
  if (args.providerName) {
    const p = args.providerName;
    substitutions.providerSnake = toSnakeCase(toPascalCase(p));
  }
  if (args.crateName) {
    substitutions.crateName = args.crateName;
  }

  let out = raw;
  for (const [key, value] of Object.entries(substitutions)) {
    if (value !== undefined) {
      out = out.replace(new RegExp(`\\{\\{${key}\\}\\}`, "g"), value);
    }
  }
  return out;
}
