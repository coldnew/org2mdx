const path = require('path');

const getLibPath = () => {
  const platform = process.platform;
  const basePath = path.join(__dirname, 'target', 'release');
  if (platform === 'win32') return path.join(basePath, 'org2mdx_napi.dll');
  if (platform === 'darwin') return path.join(basePath, 'liborg2mdx_napi.dylib');
  return path.join(basePath, 'liborg2mdx_napi.so');
};

let native;
try {
  native = require(getLibPath());
} catch (err) {
  console.error('Failed to load native addon. Run `npm run build` first.');
  throw err;
}

/**
 * Convert org-mode content to MDX
 * @param {string} input - Org-mode formatted string
 * @returns {string} MDX formatted string
 */
function convert(input) {
  if (typeof input !== 'string') {
    throw new TypeError('Input must be a string');
  }
  return native.convert(input);
}

/**
 * Convert MDX content to Org-mode
 * @param {string} input - MDX formatted string
 * @returns {string} Org-mode formatted string
 */
function convertMdxToOrg(input) {
  if (typeof input !== 'string') {
    throw new TypeError('Input must be a string');
  }
  return native.convertMdxToOrg(input);
}

/**
 * Parse org-mode content to AST (JSON string)
 * @param {string} input - Org-mode formatted string
 * @returns {string} Pretty-printed JSON AST
 */
function parseOrgToAst(input) {
  if (typeof input !== 'string') {
    throw new TypeError('Input must be a string');
  }
  return native.parseOrgToAst(input);
}

/**
 * Parse MDX content to AST (JSON string)
 * @param {string} input - MDX formatted string
 * @returns {string} Pretty-printed JSON AST
 */
function parseMdxToAst(input) {
  if (typeof input !== 'string') {
    throw new TypeError('Input must be a string');
  }
  return native.parseMdxToAst(input);
}

module.exports = { convert, convertMdxToOrg, parseOrgToAst, parseMdxToAst };
