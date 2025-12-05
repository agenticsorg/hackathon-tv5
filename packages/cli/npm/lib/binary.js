/**
 * Binary execution wrapper
 * Manages binary location, execution, and fallback to WASM
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const platforms = require('./platforms');

const BINARIES_DIR = path.join(__dirname, '..', 'binaries');

/**
 * Get the path to the native binary
 * @returns {string|null} Path to binary or null if not found
 */
function getBinaryPath() {
  const binaryName = platforms.getBinaryName();
  const binaryPath = path.join(BINARIES_DIR, binaryName);

  if (fs.existsSync(binaryPath)) {
    return binaryPath;
  }

  return null;
}

/**
 * Check if native binary is available
 * @returns {boolean} True if binary exists and is executable
 */
function isBinaryAvailable() {
  const binaryPath = getBinaryPath();

  if (!binaryPath) {
    return false;
  }

  try {
    fs.accessSync(binaryPath, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

/**
 * Execute the native binary
 * @param {string[]} args Command line arguments
 * @returns {Promise<number>} Exit code
 */
async function executeBinary(args) {
  const binaryPath = getBinaryPath();

  if (!binaryPath) {
    throw new Error(
      'ARW binary not found.\n' +
      'Run `npm install` to download the binary or `npm run build:rust` to compile from source.'
    );
  }

  return new Promise((resolve, reject) => {
    const child = spawn(binaryPath, args, {
      stdio: 'inherit',
      env: process.env
    });

    child.on('error', (error) => {
      reject(new Error(`Failed to execute ARW: ${error.message}`));
    });

    child.on('exit', (code) => {
      resolve(code || 0);
    });
  });
}

/**
 * Load and execute WASM fallback
 * @param {string[]} args Command line arguments
 * @returns {Promise<number>} Exit code
 */
async function executeWasm(args) {
  try {
    // Check if WASM module exists
    const wasmPath = path.join(__dirname, '..', 'pkg', 'arw_lib.js');
    if (!fs.existsSync(wasmPath)) {
      throw new Error(
        'WASM module not found.\n' +
        'Run `npm run build:wasm` to compile the WASM module.'
      );
    }

    console.log('ℹ Using WASM fallback (native binary not available)');

    // Load WASM module
    const wasm = require(wasmPath);

    // Parse command and execute
    const command = args[0];
    const commandArgs = args.slice(1);

    switch (command) {
      case 'validate':
        return await executeWasmValidate(wasm, commandArgs);
      case 'generate':
        return await executeWasmGenerate(wasm, commandArgs);
      default:
        console.error(`Command '${command}' is not supported in WASM mode.`);
        console.error('Please use the native binary for full functionality.');
        return 1;
    }
  } catch (error) {
    console.error('Failed to execute WASM fallback:', error.message);
    return 1;
  }
}

/**
 * Execute validate command via WASM
 */
async function executeWasmValidate(wasm, args) {
  const filePath = args[0];

  if (!filePath) {
    console.error('Usage: arw validate <file>');
    return 1;
  }

  try {
    const content = fs.readFileSync(filePath, 'utf-8');
    const result = await wasm.validate_manifest_wasm(content);

    if (result.valid) {
      console.log('✓ Validation passed');
      return 0;
    } else {
      console.error('✗ Validation failed:');
      result.errors.forEach(error => {
        console.error(`  - ${error.path}: ${error.message}`);
      });
      return 1;
    }
  } catch (error) {
    console.error('Validation error:', error.message);
    return 1;
  }
}

/**
 * Execute generate command via WASM
 */
async function executeWasmGenerate(wasm, args) {
  console.error('Generate command not yet supported in WASM mode.');
  console.error('Please use the native binary: npm run build:rust');
  return 1;
}

/**
 * Execute ARW CLI with automatic fallback
 * @param {string[]} args Command line arguments
 * @returns {Promise<number>} Exit code
 */
async function execute(args) {
  if (isBinaryAvailable()) {
    return executeBinary(args);
  }

  console.warn('⚠ Native binary not available, trying WASM fallback...');
  return executeWasm(args);
}

module.exports = {
  getBinaryPath,
  isBinaryAvailable,
  executeBinary,
  executeWasm,
  execute
};
