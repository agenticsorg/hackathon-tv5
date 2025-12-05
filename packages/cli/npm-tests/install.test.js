/**
 * Tests for binary installation and download functionality
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const platforms = require('../npm/lib/platforms');
const binary = require('../npm/lib/binary');

describe('Binary Installation', () => {
  describe('Platform Detection', () => {
    it('should detect current platform', () => {
      const platform = platforms.getCurrentPlatform();
      assert.ok(platform);
      assert.match(platform, /^(darwin|linux|win32)-(x64|arm64)$/);
    });

    it('should have valid platform config', () => {
      const config = platforms.getPlatformConfig();
      assert.ok(config.platform);
      assert.ok(config.binary);
      assert.ok(config.artifact);
      assert.ok(config.checksum);
    });

    it('should generate correct download URLs', () => {
      const url = platforms.getDownloadUrl('0.1.0');
      assert.ok(url.includes('github.com'));
      assert.ok(url.includes('releases/download'));
      assert.ok(url.includes('v0.1.0'));
    });

    it('should generate correct checksum URLs', () => {
      const url = platforms.getChecksumUrl('0.1.0');
      assert.ok(url.includes('.sha256'));
    });

    it('should get correct binary name', () => {
      const name = platforms.getBinaryName();
      if (process.platform === 'win32') {
        assert.strictEqual(name, 'arw.exe');
      } else {
        assert.strictEqual(name, 'arw');
      }
    });
  });

  describe('Platform Support', () => {
    it('should list all supported platforms', () => {
      const platforms_list = Object.keys(platforms.SUPPORTED_PLATFORMS);
      assert.ok(platforms_list.length >= 5);
      assert.ok(platforms_list.includes('darwin-x64'));
      assert.ok(platforms_list.includes('darwin-arm64'));
      assert.ok(platforms_list.includes('linux-x64'));
      assert.ok(platforms_list.includes('linux-arm64'));
      assert.ok(platforms_list.includes('win32-x64'));
    });

    it('should support current platform', () => {
      const supported = platforms.isPlatformSupported();
      assert.strictEqual(supported, true);
    });

    it('should throw error for unsupported platform', () => {
      assert.throws(() => {
        platforms.getPlatformConfig('unsupported-platform');
      }, /Unsupported platform/);
    });
  });

  describe('Binary Management', () => {
    it('should check binary availability', () => {
      const available = binary.isBinaryAvailable();
      assert.strictEqual(typeof available, 'boolean');
    });

    it('should get binary path or null', () => {
      const binaryPath = binary.getBinaryPath();
      if (binaryPath) {
        assert.ok(typeof binaryPath === 'string');
        assert.ok(binaryPath.includes('arw'));
      } else {
        assert.strictEqual(binaryPath, null);
      }
    });
  });

  describe('Error Handling', () => {
    it('should handle missing binary gracefully', async () => {
      // This test assumes binary is not installed
      // In CI, binary might not be available yet
      try {
        await binary.executeBinary(['--version']);
      } catch (error) {
        assert.ok(error.message.includes('not found') || error.message.includes('ENOENT'));
      }
    });
  });
});

describe('Installation Process', () => {
  it('should have install script', () => {
    const installPath = path.join(__dirname, '..', 'npm', 'lib', 'install.js');
    assert.ok(fs.existsSync(installPath));
  });

  it('should have platform detection', () => {
    const platformsPath = path.join(__dirname, '..', 'npm', 'lib', 'platforms.js');
    assert.ok(fs.existsSync(platformsPath));
  });

  it('should have binary management', () => {
    const binaryPath = path.join(__dirname, '..', 'npm', 'lib', 'binary.js');
    assert.ok(fs.existsSync(binaryPath));
  });
});
