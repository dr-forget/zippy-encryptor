#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

// Path to the index.js file
const indexJsPath = path.resolve(__dirname, "zippy-encryptor/index.js");
const indexMjsPath = path.resolve(__dirname, "zippy-encryptor/index.mjs");

// Check if index.js exists
if (!fs.existsSync(indexJsPath)) {
  console.error("Error: index.js file does not exist at path:", indexJsPath);
  process.exit(1);
}

// Read the content of index.js
let content = fs.readFileSync(indexJsPath, "utf8");

// Add ESM header
let esmHeader = `// Converted from CommonJS to ESM
import { createRequire } from 'module';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

`;

// Remove CommonJS exports and destructuring
content = content.replace(/module\.exports\.\w+\s*=\s*\w+;?/g, "");
content = content.replace(
  /const\s*{\s*[\w\s,]+\s*}\s*=\s*nativeBinding;?/g,
  ""
);

// Add ESM export
const esmFooter =
  "\n// Export the nativeBinding directly\nexport default nativeBinding;\n";

// Combine to create the final ESM module
const finalContent = esmHeader + content + esmFooter;

// Write the content to index.mjs
fs.writeFileSync(indexMjsPath, finalContent);

console.log(`Created: ${indexMjsPath}`);
