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

module.exports = { convert };
