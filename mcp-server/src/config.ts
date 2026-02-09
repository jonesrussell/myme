import { z } from "zod";
import path from "path";
import fs from "fs";

const EnvSchema = z.object({
  MYME_REPO: z.string().optional(),
  QMLFORMAT_PATH: z.string().optional(),
  MYME_APP_PATH: z.string().optional(),
});

function findWorkspaceRoot(cwd: string): string {
  const root = path.resolve(cwd);
  const cargoPath = path.join(root, "Cargo.toml");
  const qmlPath = path.join(root, "qml.qrc");
  if (fs.existsSync(cargoPath) && fs.existsSync(qmlPath)) {
    return root;
  }
  const parent = path.dirname(root);
  if (parent === root) {
    throw new Error("Workspace root not found: no directory containing both Cargo.toml and qml.qrc");
  }
  return findWorkspaceRoot(parent);
}

export type Config = {
  workspaceRoot: string;
  qmlformatPath: string | null;
  appPath: string | null;
};

let cached: Config | null = null;

export function loadConfig(): Config {
  if (cached) return cached;
  const parsed = EnvSchema.safeParse(process.env);
  const env = parsed.success ? parsed.data : {};
  const cwd = env.MYME_REPO
    ? path.resolve(env.MYME_REPO)
    : process.cwd();
  const workspaceRoot = findWorkspaceRoot(cwd);
  cached = {
    workspaceRoot,
    qmlformatPath: env.QMLFORMAT_PATH ?? null,
    appPath: env.MYME_APP_PATH ?? null,
  };
  return cached;
}

export function getConfig(): Config {
  if (!cached) throw new Error("Config not loaded; call loadConfig() first");
  return cached;
}
