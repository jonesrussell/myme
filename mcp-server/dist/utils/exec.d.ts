export type ToolResult = {
    success: boolean;
    exitCode: number;
    stdout: string;
    stderr: string;
    message?: string;
};
export declare function toolResult(success: boolean, exitCode: number, stdout: string, stderr: string, message?: string): ToolResult;
export declare function okResult(stdout: string, message?: string): ToolResult;
export declare function failResult(exitCode: number, stdout: string, stderr: string, message?: string): ToolResult;
export declare function formatToolResult(r: ToolResult): string;
export declare function runCommand(command: string, args: string[], options?: {
    cwd: string;
    env?: NodeJS.ProcessEnv;
}): Promise<ToolResult>;
