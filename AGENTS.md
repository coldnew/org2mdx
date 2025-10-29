# AGENTS.md

## Build/Lint/Test Commands

- **Build**: `npm run build` or `tsc`
- **Lint**: `npm run lint` or `eslint src/ --ext .ts`
- **Test all**: `npm test` or `jest`
- **Test single**: `npm test -- <test-file>` or `jest <test-file>`
- **Type check**: `npm run typecheck` or `tsc --noEmit`

## Code Style Guidelines

### Imports

- Use ES6 imports with named exports preferred
- Group imports: React, third-party libs, local modules
- No wildcard imports (`import * as`)

### Formatting

- Use Prettier with default settings
- 2-space indentation
- Semicolons required
- Single quotes for strings

### Types

- Strict TypeScript with `strict: true`
- Use interfaces for object shapes
- Avoid `any` type; use `unknown` when necessary

### Naming Conventions

- Functions: camelCase
- Classes: PascalCase
- Constants: UPPER_SNAKE_CASE
- Files: kebab-case.ts

### Error Handling

- Use try/catch for async operations
- Throw custom Error subclasses with descriptive messages
- Validate inputs at function boundaries

### General

- Max line length: 100 characters
- Use async/await over Promises
- Prefer functional programming patterns

## Project-Specific Notes

- Uses `uniorg` ecosystem for Org mode parsing and conversion
- ES modules with TypeScript
- CLI available via `org2mdx` command
- Supports frontmatter extraction from Org keywords
- Supports table conversion to Markdown tables with alignment
- Supports image captions with `#+caption:` keywords, converting to HTML `<figure>` and `<figcaption>` elements in MDX output
