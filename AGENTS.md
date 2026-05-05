# AGENTS.md

## Build & Test

```bash
cargo build --release          # builds CLI binary + NAPI shared library
cargo test                     # runs all Rust integration tests
cargo test test_org_to_ast     # single test (use test function name)
```

No `npm test` — `package.json` references `test.js` but it doesn't exist.
No CI configs, Makefile, or lint/format commands in this repo.

## Commit Conventions

- **Before committing**, run `cargo fmt` to format all Rust sources.
- Use **Conventional Commits**: `feat:`, `fix:`, `test:`, `docs:`, `style:`, `refactor:`, `chore:`.

## Coding Principles

- **State assumptions** before implementing. If something is unclear, ask — don't guess.
- **Minimal code**: solve with least code needed. No speculative features, no abstractions for one-off code.
- **Surgical edits**: only change lines directly related to the task. Match existing style. Don't refactor unrelated code.
- **Verify with tests**: `cargo test` before completion. Prove correctness, don't assume it.
- **Define success criteria upfront** for multi-step tasks: what test must pass, what output must appear.

## Architecture

**Dual Rust + Node.js project:**
- `Cargo.toml` — Rust CLI (`src/main.rs`) + library (`src/lib.rs`) + NAPI addon (`src/napi.rs`)
- `package.json` / `index.js` — Node.js wrapper that `require()`s the native `.node` binary from `target/release/`
- NAPI binary names per platform:
  - Linux: `liborg2mdx_napi.so`
  - macOS: `liborg2mdx_napi.dylib`
  - Windows: `org2mdx_napi.dll`

**Module layout:**
- `src/lib.rs` — public submodule entrypoints: `org_to_mdx`, `mdx_to_org`, `org_to_ast`, `mdx_to_ast`
- `src/parser/org/` — Org parser (heading, block, list)
- `src/parser/mdx/` — MDX parser
- `src/renderer/` — mdx_renderer, org_renderer, html_jsx
- `src/ast/` — unist-compliant AST (`Node`, `Position`, `Point`)
- `src/util/` — date conversion, YAML helpers, URL escaping

## Test Architecture (non-obvious)

Tests live in `tests/integration_tests.rs`. Two fixture systems:

1. **Triplet fixtures** (`tests/ast/`): Each stem has `.org`, `.ast` (JSON), `.mdx`
   - `test_org_to_ast_fixtures` — org → AST matches fixture
   - `test_ast_to_mdx_fixtures` — AST → MDX matches fixture
   - `test_mdx_to_ast_fixtures` — MDX → AST matches fixture

2. **Pairwise fixtures** (`tests/org/` + `tests/mdx/`):
   - `test_standard_org_to_mdx_fixtures` — org→mdx, then round-trip AST comparison
   - `test_standard_mdx_to_org_fixtures` — mdx→org→mdx, verifies semantic equivalence

**AST normalization** before comparison strips:
- `blankLine` nodes
- Adjacent text nodes merged
- `tags`, `date`, `updated` from `data` objects
- Links autolinked if children match URL (removes children)
- Links to image URLs with text matching URL → converted to `image` type
- Adjacent lists of same ordered/unordered type merged
- `category` normalized to single-element array

## Gotchas

- **`Error::InvalidOrgFile`** is a catch-all also used for MDX errors — check context, don't rename.
- **Date conversions** are hardcoded to UTC+8 in `src/util/mod.rs` (`org_date_to_iso`, `iso_to_org_date`).
- **YAML frontmatter** is expected in MDX fixtures — `split_yaml_frontmatter` expects `---\n...\n---\n`.
- Tests expect to run from repo root (fixture paths are `tests/ast/`, `tests/org/`, etc.).
- The NAPI `convert` function only supports org→mdx — no mdx→org or AST output via Node.

## Docs Worth Knowing

- `.opencode/ast-test-plan.md` — fixture authoring guide and normalization rationale
- `README.md` — CLI usage and Library API (kept in sync)
