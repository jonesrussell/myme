export type ResourceReadResult = {
    contents: Array<{
        uri: string;
        text: string;
    }>;
};
export declare function readResource(uri: string): Promise<ResourceReadResult>;
export declare function listResourceUris(): Array<{
    uri: string;
    name: string;
    description?: string;
}>;
