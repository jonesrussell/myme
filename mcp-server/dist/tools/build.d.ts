import { type ToolResult } from "../utils/exec.js";
export declare function buildQt(): Promise<ToolResult>;
export declare function cargoBuild(args: {
    package?: string;
}): Promise<ToolResult>;
export declare function cargoTest(args: {
    package?: string;
}): Promise<ToolResult>;
export declare function runApp(args: {
    mode?: "debug" | "release";
    env?: Record<string, string>;
    args?: string[];
}): Promise<ToolResult>;
