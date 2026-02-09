export declare function readFileUtf8(filePath: string): Promise<string>;
export declare function readFileUtf8Sync(filePath: string): string;
/**
 * Read a curated resource file from mcp-server/resources/ (relative to this package).
 */
export declare function readResourceFile(relativePath: string): Promise<string>;
export declare function readResourceFileSync(relativePath: string): string;
