#!/usr/bin/env node
/**
 * CLI wrapper for ARW
 *
 * This script provides a cross-platform executable that shells out to the
 * native Rust binary for optimal performance.
 *
 * For pure JavaScript/WASM usage, use the exported functions from index.js instead.
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Determine the binary path based on platform
function getBinaryPath() {
  const platform = process.platform;
  const arch = process.arch;

  let binaryName = 'arw';
  if (platform === 'win32') {
    binaryName = 'arw.exe';
  }

  // Try to find the binary in various locations
  const possiblePaths = [
    // Release build (after npm run build)
    path.join(__dirname, '..', 'target', 'release', binaryName),
    // Debug build (for development)
    path.join(__dirname, '..', 'target', 'debug', binaryName),
    // System-wide installation
    path.join('/usr/local/bin', binaryName),
  ];

  for (const binaryPath of possiblePaths) {
    if (fs.existsSync(binaryPath)) {
      return binaryPath;
    }
  }

  console.error('Error: ARW binary not found.');
  console.error('Please run `npm run build` in the npx-arw directory to compile the binary.');
  console.error('Or install the binary system-wide.');
  process.exit(1);
}

// Execute the binary
function main() {
  const binaryPath = getBinaryPath();
  const args = process.argv.slice(2);

  const child = spawn(binaryPath, args, {
    stdio: 'inherit',
    env: process.env
  });

  child.on('error', (error) => {
    console.error(`Failed to execute ARW binary: ${error.message}`);
    process.exit(1);
  });

  child.on('exit', (code) => {
    process.exit(code || 0);
  });
}

// Only run if executed directly (not required as module)
if (require.main === module) {
  main();
}

module.exports = { getBinaryPath };
