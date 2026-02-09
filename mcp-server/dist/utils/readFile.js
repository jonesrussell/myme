import fs from "fs/promises";
import fsSync from "fs";
import path from "path";
import { fileURLToPath } from "url";
const __dirname = path.dirname(fileURLToPath(import.meta.url));
export async function readFileUtf8(filePath) {
    const content = await fs.readFile(filePath, "utf-8");
    return content;
}
export function readFileUtf8Sync(filePath) {
    return fsSync.readFileSync(filePath, "utf-8");
}
/**
 * Read a curated resource file from mcp-server/resources/ (relative to this package).
 */
export async function readResourceFile(relativePath) {
    const resourcesDir = path.resolve(__dirname, "..", "..", "resources");
    const fullPath = path.join(resourcesDir, relativePath);
    return readFileUtf8(fullPath);
}
export function readResourceFileSync(relativePath) {
    const resourcesDir = path.resolve(__dirname, "..", "..", "resources");
    const fullPath = path.join(resourcesDir, relativePath);
    return readFileUtf8Sync(fullPath);
}
