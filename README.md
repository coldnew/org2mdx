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

### CLI

```bash
# Convert a single file
org2mdx input.org output.mdx

# Convert with frontmatter extraction
org2mdx input.org output.mdx --frontmatter
```

## Features

- Convert Org headings to MDX headings
- Convert Org markup (*bold*, /italic/, ~code~, etc.) to MDX
- Convert Org code blocks to MDX code blocks
- Convert Org links to MDX links
- Optional frontmatter extraction from Org properties
- Custom component mapping support

## API

### `orgToMdx(orgContent: string, options?: ConvertOptions): string`

Converts Org mode content to MDX.

#### Options

- `frontmatter?: boolean` - Extract Org properties as MDX frontmatter
- `components?: Record<string, string>` - Map Org elements to custom MDX components

## Development

```bash
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