export type Config = {
    workspaceRoot: string;
    qmlformatPath: string | null;
    appPath: string | null;
};
export declare function loadConfig(): Config;
export declare function getConfig(): Config;
