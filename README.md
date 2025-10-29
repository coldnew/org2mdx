# org2mdx

Convert Emacs Org mode files to MDX format.

## Installation

```bash
npm install org2mdx
```

## Usage

### Programmatic API

```typescript
import { orgToMdx } from 'org2mdx';

const orgContent = `* Hello World

This is a paragraph with *bold* and /italic/ text.`;

const mdxContent = orgToMdx(orgContent);
console.log(mdxContent);
// Output:
// # Hello World
//
// This is a paragraph with **bold** and *italic* text.
```

### Advanced Options

```typescript
const mdx = orgToMdx(orgContent, {
  frontmatter: true, // Extract default properties as frontmatter
  components: {
    // Map Org elements to custom MDX components
    note: 'Callout',
  },
  plugins: [
    // Add custom unified plugins
    remarkPlugin,
  ],
});

// Custom frontmatter configuration
const mdxCustom = orgToMdx(orgContent, {
  frontmatter: {
    include: ['TITLE', 'AUTHOR', 'DATE'], // Only extract these properties
    mapping: {
      TITLE: 'pageTitle', // Map #+TITLE to pageTitle
      AUTHOR: 'writer', // Map #+AUTHOR to writer
    },
  },
});
```

### File Processing

```typescript
import { orgFileToMdx, convertOrgFile, convertOrgDirectory } from 'org2mdx';

// Convert a single file
const mdx = orgFileToMdx('path/to/file.org');

// Convert and save
convertOrgFile('input.org', 'output.mdx');

// Convert entire directory
convertOrgDirectory('org-posts/', 'mdx-posts/');
```

### Next.js Integration

For Next.js projects, you can use this library to convert Org files to MDX for your content layer:

```typescript
// In your Next.js API route or getStaticProps
import { orgFileToMdx } from 'org2mdx';

export async function getStaticProps({ params }) {
  const mdxContent = orgFileToMdx(`content/${params.slug}.org`, {
    frontmatter: true,
  });

  return {
    props: {
      content: mdxContent,
    },
  };
}
```

### CLI

```bash
# Convert a single file
org2mdx input.org output.mdx

# Convert with frontmatter extraction
org2mdx input.org output.mdx --frontmatter
```

## Features

- Convert Org headings to MDX headings
- Convert Org markup (_bold_, /italic/, ~code~, etc.) to MDX
- Convert Org code blocks to MDX code blocks
- Convert Org links to MDX links
- Convert Org tables to MDX tables with alignment support
- Convert Org image captions to HTML `<figure>` and `<figcaption>` elements
- Comprehensive frontmatter extraction from Org properties (TITLE, AUTHOR, DATE, DESCRIPTION, KEYWORDS, CATEGORY, TAGS, and custom properties)
- Custom component mapping support
- Extensible plugin system
- File and directory processing utilities

## API

### `orgToMdx(orgContent: string, options?: ConvertOptions): string`

Converts Org mode content to MDX.

#### Options

- `frontmatter?: boolean | FrontmatterConfig` - Extract Org properties as MDX frontmatter
  - `boolean`: Enable/disable with default properties
  - `FrontmatterConfig`: Customize extraction
    - `include?: string[]` - Array of Org property names to extract
    - `mapping?: Record<string, string>` - Custom mapping from Org property to frontmatter key
    - `formatters?: Record<string, (value: string) => string>` - Custom formatters for property values
- `components?: Record<string, string>` - Map Org elements to custom MDX components
- `plugins?: Array<any>` - Additional unified plugins to apply

### File Processing Functions

- `orgFileToMdx(filePath: string, options?: ConvertOptions): string` - Convert Org file to MDX string
- `convertOrgFile(inputPath: string, outputPath: string, options?: ConvertOptions): void` - Convert Org file and save as MDX
- `convertOrgDirectory(inputDir: string, outputDir: string, options?: ConvertOptions): void` - Convert all .org files in directory to .mdx files

## Development

````bash
# Install dependencies
npm install

# Run tests
npm test

# Build
npm run build

# Lint
npm run lint

# Type check
npm run typecheck
```</content>
</xai:function_call">

<xai:function_call name="write">
<parameter name="filePath">/tmp/ramdisk/org2mdx/example.org
````
