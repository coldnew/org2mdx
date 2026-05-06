/**
 * Convert org-mode content to MDX
 * @param input - Org-mode formatted string
 * @returns MDX formatted string
 */
export declare function convert(input: string): string;

/**
 * Convert MDX content to Org-mode
 * @param input - MDX formatted string
 * @returns Org-mode formatted string
 */
export declare function convertMdxToOrg(input: string): string;

/**
 * Parse org-mode content to AST (JSON string)
 * @param input - Org-mode formatted string
 * @returns Pretty-printed JSON AST
 */
export declare function parseOrgToAst(input: string): string;

/**
 * Parse MDX content to AST (JSON string)
 * @param input - MDX formatted string
 * @returns Pretty-printed JSON AST
 */
export declare function parseMdxToAst(input: string): string;
