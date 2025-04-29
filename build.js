#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

const srcDir = __dirname;
const destDir = path.join(__dirname, "zippy-encryptor");

fs.mkdirSync(destDir, { recursive: true });

const targetFiles = ["index.js", "index.d.ts", "README.md"];
const targetExtensions = [".node"];

const filesToMove = fs.readdirSync(srcDir).filter((file) => {
  return (
    targetFiles.includes(file) ||
    targetExtensions.some((ext) => file.endsWith(ext))
  );
});

for (const file of filesToMove) {
  const srcPath = path.join(srcDir, file);
  const destPath = path.join(destDir, file);
  fs.copyFileSync(srcPath, destPath);
  console.log(`✔ Moved ${file} → ${path.relative(__dirname, destPath)}`);
}

console.log("\n✅ All files moved successfully.");
