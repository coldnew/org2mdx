import { unified } from 'unified';
import parse from 'uniorg-parse';
import uniorg2rehype from 'uniorg-rehype';
import rehype2remark from 'rehype-remark';
import remarkGfm from 'remark-gfm';
import { default as remarkStringify } from 'remark-stringify';
import { visit } from 'unist-util-visit';
import { readFileSync, writeFileSync, readdirSync, statSync, mkdirSync } from 'fs';
import { join, dirname, extname } from 'path';

// Global variable to pass alignment data between plugins
let globalTableAlignments: any[] = [];

// Global variable to pass caption data
let globalCaptions: any[] = [];

// Function to convert Org AST to HTML text
function astToHtml(ast: any[]): string {
  return ast
    .map((node) => {
      if (node.type === 'text') return node.value;
      if (node.type === 'bold') return '<strong>' + astToHtml(node.children) + '</strong>';
      if (node.type === 'italic') return '<em>' + astToHtml(node.children) + '</em>';
      if (node.type === 'code') return '<code>' + node.value + '</code>';
      if (node.type === 'verbatim') return '<code>' + node.value + '</code>';
      // Add more types as needed
      return '';
    })
    .join('');
}

// Plugin to handle Org captions and other affiliated keywords
function orgCaptions() {
  return (tree: any) => {
    globalCaptions = [];
    let captionIndex = 0;
    visit(tree, ['paragraph', 'link'], (node: any) => {
      if (node.affiliated && node.affiliated.CAPTION) {
        // This node has a caption
        const caption = astToHtml(node.affiliated.CAPTION[0]).trim();

        // Store caption info
        globalCaptions.push({ index: captionIndex++, caption });

        // Remove the affiliated data
        delete node.affiliated;
      }
    });
  };
}

// Plugin to convert figure elements to HTML strings
function convertFiguresToHtml() {
  return (tree: any) => {
    visit(tree, 'element', (element: any, index?: number, parent?: any) => {
      if (element.tagName === 'figure' && index !== undefined && parent) {
        const img = element.children[0];
        const figcaption = element.children[1];
        const imgSrc = img.properties.src || '';
        const imgAlt = img.properties.alt || '';
        const imgHtml = `<img src="${imgSrc}" alt="${imgAlt}" />`;
        const figcaptionText = figcaption.children[0].value;
        const figcaptionHtml = `<figcaption>${figcaptionText}</figcaption>`;
        const html = `<figure>${imgHtml}${figcaptionHtml}</figure>`;
        const htmlNode = {
          type: 'html',
          value: html,
        };
        parent.children[index] = htmlNode;
      }
    });
  };
}

// Plugin to handle Org table alignment
function orgTableAlignment() {
  return (tree: any) => {
    globalTableAlignments = [];
    let tableIndex = 0;
    visit(tree, 'table', (table: any) => {
      const rows = table.children || [];

      // Check if we have at least 3 rows (header, separator, potential alignment)
      if (rows.length < 3) return;

      // Check if the third row contains alignment markers
      const alignmentRow = rows[2];
      if (!alignmentRow || alignmentRow.rowType !== 'standard') return;

      const alignmentCells = alignmentRow.children || [];
      const alignments: (string | null)[] = [];

      // Extract alignment from each cell
      for (const cell of alignmentCells) {
        const text = cell.children?.[0]?.value?.trim();
        if (text === '<l>') {
          alignments.push('left');
        } else if (text === '<c>') {
          alignments.push('center');
        } else if (text === '<r>') {
          alignments.push('right');
        } else {
          alignments.push(null); // default (no alignment marker)
        }
      }

      // Check if this row actually contains alignment markers
      const hasAlignmentMarkers = alignments.some((align) => align !== null);
      if (!hasAlignmentMarkers) return;

      // Store alignment info with table index
      globalTableAlignments.push({ index: tableIndex++, alignments });

      // Check if the header row is empty - if so, use the data row as header
      const headerRow = rows[0];
      const dataRow = rows[3];

      if (headerRow && headerRow.rowType === 'standard') {
        const headerCells = headerRow.children || [];
        const isEmptyHeader = headerCells.every(
          (cell: any) => !cell.children || !cell.children[0] || !cell.children[0].value.trim(),
        );

        if (isEmptyHeader && dataRow) {
          // Replace empty header with data row content
          headerRow.children = dataRow.children;
          // Remove the data row (now duplicated)
          table.children.splice(3, 1);
        }
      }

      // Remove the alignment row from the table
      table.children.splice(2, 1);
    });
  };
}

// Custom rehype plugin to handle captions and table alignment
function rehypeCaptionsAndTableAlignment() {
  return (tree: any) => {
    let tableIndex = 0;
    let imgIndex = 0;

    visit(tree, 'element', (element: any, index?: number, parent?: any) => {
      // Handle captions for images
      if (element.tagName === 'img') {
        const captionInfo = globalCaptions[imgIndex++];
        if (captionInfo && index !== undefined && parent) {
          // Wrap the img in a figure
          const figure = {
            type: 'element',
            tagName: 'figure',
            properties: {},
            children: [
              element,
              {
                type: 'element',
                tagName: 'figcaption',
                properties: {},
                children: [
                  {
                    type: 'text',
                    value: captionInfo.caption,
                  },
                ],
              },
            ],
          };

          // Replace the element with figure
          parent.children[index] = figure;
        }
      }

      // Handle table alignment
      if (element.tagName === 'table') {
        const alignmentInfo = globalTableAlignments[tableIndex++];
        if (!alignmentInfo) return;

        const alignments = alignmentInfo.alignments;

        // Check if we need to create thead/tbody structure
        const tbody = element.children?.find((child: any) => child.tagName === 'tbody');
        if (tbody && tbody.children && tbody.children.length >= 2) {
          // Assume first row is header, second is separator
          const headerRow = tbody.children[0];
          const separatorRow = tbody.children[1];

          // Check if separator row contains dashes (indicating it's a separator)
          const isSeparator = separatorRow.children?.every((cell: any) =>
            cell.children?.[0]?.value?.trim().match(/^[-]+$/),
          );

          if (isSeparator) {
            // Create thead with header row
            const thead = {
              type: 'element',
              tagName: 'thead',
              properties: {},
              children: [headerRow],
            };

            // Convert header cells to th and apply alignment
            headerRow.children?.forEach((cell: any, index: number) => {
              cell.tagName = 'th';
              if (alignments[index]) {
                cell.properties = cell.properties || {};
                cell.properties.align = alignments[index];
              }
            });

            // Remove header and separator from tbody
            tbody.children.splice(0, 2);

            // Add thead to table
            element.children.unshift(thead);
          }
        }
      }
    });
  };
}

export interface FrontmatterConfig {
  /** Array of Org property names to extract (e.g., ['TITLE', 'AUTHOR']) */
  include?: string[];
  /** Custom mapping from Org property to frontmatter key */
  mapping?: Record<string, string>;
  /** Custom formatting for specific properties */
  formatters?: Record<string, (value: string) => string>;
}

export interface ConvertOptions {
  /** Preserve Org mode metadata as frontmatter */
  frontmatter?: boolean | FrontmatterConfig;
  /** Custom MDX components mapping */
  components?: Record<string, string>;
  /** Additional unified plugins to apply */
  plugins?: Array<any>;
}

/**
 * Convert Org mode content to MDX
 */
export function orgToMdx(orgContent: string, options: ConvertOptions = {}): string {
  try {
    // Create unified processor
    const processor = unified()
      .use(parse)
      .use(orgCaptions)
      .use(orgTableAlignment)
      .use(uniorg2rehype)
      .use(rehypeCaptionsAndTableAlignment)
      .use(convertFiguresToHtml)
      .use(rehype2remark)
      .use(remarkGfm)
      .use(remarkStringify);

    // Process the content
    const file = processor.processSync(orgContent);
    let mdxContent = String(file).trim();

    // Unescape HTML tags in html nodes
    mdxContent = mdxContent.replace(/\\</g, '<').replace(/\\>/g, '>');

    // Add frontmatter if requested
    if (options.frontmatter) {
      const config = typeof options.frontmatter === 'object' ? options.frontmatter : undefined;
      const frontmatter = extractFrontmatter(orgContent, config);
      if (frontmatter) {
        mdxContent = `---\n${frontmatter}---\n\n${mdxContent}`;
      }
    }

    // Apply component mappings
    if (options.components) {
      mdxContent = applyComponentMappings(mdxContent, options.components);
    }

    return mdxContent;
  } catch (error) {
    throw new Error(
      `Failed to convert Org to MDX: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

/**
 * Extract frontmatter from Org content
 */
function extractFrontmatter(orgContent: string, config?: FrontmatterConfig): string | null {
  const properties: string[] = [];
  const lines = orgContent.split('\n');

  // Default properties to extract if no config specified
  const include = config?.include;
  const mapping = config?.mapping || {};
  const formatters = config?.formatters || {};

  // Default mapping
  const defaultMapping: Record<string, string> = {
    TITLE: 'title',
    AUTHOR: 'author',
    DATE: 'date',
    DESCRIPTION: 'description',
    KEYWORDS: 'keywords',
    CATEGORY: 'category',
    TAGS: 'tags',
  };

  // Merge mappings
  const finalMapping = { ...defaultMapping, ...mapping };

  // Properties that should always be formatted as arrays
  const arrayProperties = ['ALIAS', 'ALIASES', 'KEYWORDS', 'TAGS'];

  // Default formatters for array properties
  const defaultFormatters: Record<string, (value: string) => string> = {
    KEYWORDS: (value: string) => {
      const keywords = value
        .trim()
        .split(/\s*,\s*|\s+/)
        .map((k) => `"${k.trim()}"`);
      return `[${keywords.join(', ')}]`;
    },
    TAGS: (value: string) => {
      const tags = value
        .trim()
        .split(/\s*,\s*|\s+/)
        .map((t) => `"${t.trim()}"`);
      return `[${tags.join(', ')}]`;
    },
  };

  // Merge formatters
  const finalFormatters = { ...defaultFormatters, ...formatters };

  // Collect all values for each property
  const propertyValues: Record<string, string[]> = {};

  for (const line of lines) {
    const match = line.match(/^#\+([A-Z_]+):\s*(.+)$/i);
    if (match) {
      const orgKey = match[1];
      const value = match[2].trim();

      // Check if this property should be included
      const shouldInclude = include ? include.includes(orgKey) : true; // If no include specified, include all
      if (shouldInclude) {
        if (!propertyValues[orgKey]) {
          propertyValues[orgKey] = [];
        }
        propertyValues[orgKey].push(value);
      }
    }
  }

  // Format the collected values
  for (const [orgKey, values] of Object.entries(propertyValues)) {
    const frontmatterKey = finalMapping[orgKey] || orgKey.toLowerCase();
    const formatter = finalFormatters[orgKey];

    if (formatter) {
      // Use custom formatter for all values
      const formattedValue = formatter(values.join(', '));
      properties.push(`${frontmatterKey}: ${formattedValue}`);
    } else if (values.length === 1 && !arrayProperties.includes(orgKey)) {
      // Single value
      properties.push(`${frontmatterKey}: "${values[0]}"`);
    } else {
      // Multiple values or array property as YAML array
      const arrayItems = values.map((v) => `  - "${v}"`).join('\n');
      properties.push(`${frontmatterKey}:\n${arrayItems}`);
    }
  }

  return properties.length > 0 ? properties.join('\n') + '\n' : null;
}

/**
 * Apply custom component mappings to MDX content
 */
function applyComponentMappings(content: string, components: Record<string, string>): string {
  let result = content;

  for (const [orgElement, mdxComponent] of Object.entries(components)) {
    // Simple regex replacement for common Org elements
    // This is a basic implementation - could be enhanced for more complex mappings
    const pattern = new RegExp(`<${orgElement}([^>]*)>`, 'g');
    result = result.replace(pattern, `<${mdxComponent}$1>`);
  }

  return result;
}

/**
 * Convert an Org file to MDX
 */
export function orgFileToMdx(filePath: string, options: ConvertOptions = {}): string {
  try {
    const orgContent = readFileSync(filePath, 'utf8');
    return orgToMdx(orgContent, options);
  } catch (error) {
    throw new Error(
      `Failed to convert Org file ${filePath} to MDX: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

/**
 * Convert an Org file to MDX and write to output file
 */
export function convertOrgFile(
  inputPath: string,
  outputPath: string,
  options: ConvertOptions = {},
): void {
  const mdxContent = orgFileToMdx(inputPath, options);
  writeFileSync(outputPath, mdxContent, 'utf8');
}

/**
 * Convert all .org files in a directory to .mdx files
 */
export function convertOrgDirectory(
  inputDir: string,
  outputDir: string,
  options: ConvertOptions = {},
): void {
  const files = readdirSync(inputDir);

  for (const file of files) {
    const inputPath = join(inputDir, file);
    const stat = statSync(inputPath);

    if (stat.isDirectory()) {
      // Recursively process subdirectories
      const subOutputDir = join(outputDir, file);
      mkdirSync(subOutputDir, { recursive: true });
      convertOrgDirectory(inputPath, subOutputDir, options);
    } else if (extname(file) === '.org') {
      const outputPath = join(outputDir, file.replace('.org', '.mdx'));
      mkdirSync(dirname(outputPath), { recursive: true });
      convertOrgFile(inputPath, outputPath, options);
    }
  }
}
