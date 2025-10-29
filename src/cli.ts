#!/usr/bin/env node

import { readFileSync, writeFileSync } from 'fs';
import { orgToMdx } from './index.js';

const args = process.argv.slice(2);

if (args.length < 2) {
  console.error('Usage: org2mdx <input.org> <output.mdx> [--frontmatter]');
  process.exit(1);
}

const [inputFile, outputFile] = args;
const frontmatter = args.includes('--frontmatter');

try {
  const orgContent = readFileSync(inputFile, 'utf-8');
  const mdxContent = orgToMdx(orgContent, { frontmatter });
  writeFileSync(outputFile, mdxContent, 'utf-8');
  console.log(`Converted ${inputFile} to ${outputFile}`);
} catch (error) {
  console.error(`Error: ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
}