# org2mdx

Convert between Org and MDX with a [unist](https://github.com/syntax-tree/unist)-compliant AST.

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Org → MDX (default)
org2mdx input.org
org2mdx input.org -o output.mdx

# MDX → Org
org2mdx input.mdx --from mdx --to org
org2mdx input.mdx --from mdx --to org -o output.org

# Org → AST (JSON)
org2mdx input.org --to ast
org2mdx input.org --to ast -o ast.json

# MDX → AST (JSON)
org2mdx input.mdx --from mdx --to ast
org2mdx input.mdx --from mdx --to ast -o ast.json
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--from` | `org` | Input format: `org` or `mdx` |
| `--to` | `mdx` | Output format: `mdx`, `org`, or `ast` |
| `-o` | stdout | Output file path |

### AST output

The `--to ast` flag outputs the parsed syntax tree as pretty-printed JSON following
the unist specification. Each node has a `type` field, optional `children`, `value`,
`data`, and `position` properties.

## Library API

```rust
// Org → MDX
let mdx = org2mdx::org_to_mdx::convert(org_str)?;

// MDX → Org
let org = org2mdx::mdx_to_org::convert(mdx_str)?;

// Parse to AST
let root = org2mdx::org_to_ast::parse(org_str)?;
let root = org2mdx::mdx_to_ast::parse(mdx_str)?;
```
