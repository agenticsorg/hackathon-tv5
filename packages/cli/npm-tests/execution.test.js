/**
 * Tests for CLI execution functionality
 */

const assert = require('assert');
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const binary = require('../npm/lib/binary');

describe('CLI Execution', () => {
  const cliPath = path.join(__dirname, '..', 'npm', 'bin', 'arw');

  before(function() {
    // Skip tests if binary is not available
    if (!binary.isBinaryAvailable() && !hasWasmFallback()) {
      this.skip();
    }
  });

  describe('Help Command', () => {
    it('should display help message', (done) => {
      const child = spawn('node', [cliPath, '--help']);
      let output = '';

      child.stdout.on('data', (data) => {
        output += data.toString();
      });

      child.on('close', (code) => {
        assert.strictEqual(code, 0);
        assert.ok(output.includes('ARW CLI'));
        assert.ok(output.includes('Usage'));
        assert.ok(output.includes('Commands'));
        done();
      });
    });

    it('should display help with -h flag', (done) => {
      const child = spawn('node', [cliPath, '-h']);
      let output = '';

      child.stdout.on('data', (data) => {
        output += data.toString();
      });

      child.on('close', (code) => {
        assert.strictEqual(code, 0);
        assert.ok(output.includes('ARW CLI'));
        done();
      });
    });
  });

  describe('Binary Execution', () => {
    it('should execute without errors', async function() {
      if (!binary.isBinaryAvailable()) {
        this.skip();
      }

      try {
        const exitCode = await binary.executeBinary(['--help']);
        assert.strictEqual(exitCode, 0);
      } catch (error) {
        assert.fail(`Binary execution failed: ${error.message}`);
      }
    });
  });

  describe('Error Handling', () => {
    it('should handle invalid commands', (done) => {
      const child = spawn('node', [cliPath, 'invalid-command']);

      child.on('close', (code) => {
        assert.notStrictEqual(code, 0);
        done();
      });
    });

    it('should show error for missing required arguments', (done) => {
      const child = spawn('node', [cliPath, 'validate']);
      let stderr = '';

      child.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      child.on('close', (code) => {
        assert.notStrictEqual(code, 0);
        done();
      });
    });
  });
});

function hasWasmFallback() {
  const wasmPath = path.join(__dirname, '..', 'npm', 'pkg', 'arw_lib.js');
  return fs.existsSync(wasmPath);
}
