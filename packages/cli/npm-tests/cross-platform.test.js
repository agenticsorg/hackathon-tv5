/**
 * Tests for cross-platform compatibility
 */

const assert = require('assert');
const platforms = require('../npm/lib/platforms');

describe('Cross-Platform Compatibility', () => {
  describe('Platform Detection Accuracy', () => {
    it('should correctly identify platform type', () => {
      const platform = platforms.getCurrentPlatform();
      const [os, arch] = platform.split('-');

      assert.ok(['darwin', 'linux', 'win32'].includes(os));
      assert.ok(['x64', 'arm64'].includes(arch));

      assert.strictEqual(os, process.platform);
      assert.strictEqual(arch, process.arch);
    });

    it('should provide config for all supported platforms', () => {
      const supportedPlatforms = Object.keys(platforms.SUPPORTED_PLATFORMS);

      supportedPlatforms.forEach(platform => {
        const config = platforms.getPlatformConfig(platform);

        assert.ok(config.platform);
        assert.ok(config.binary);
        assert.ok(config.artifact);
        assert.ok(config.checksum);

        // Verify artifact format
        if (platform.startsWith('win32')) {
          assert.ok(config.artifact.endsWith('.zip'));
          assert.ok(config.binary.endsWith('.exe'));
        } else {
          assert.ok(config.artifact.endsWith('.tar.gz'));
          assert.ok(!config.binary.endsWith('.exe'));
        }
      });
    });
  });

  describe('Binary Naming', () => {
    it('should use .exe extension on Windows', () => {
      const name = platforms.getBinaryName('win32-x64');
      assert.strictEqual(name, 'arw.exe');
    });

    it('should not use extension on Unix-like systems', () => {
      const darwinName = platforms.getBinaryName('darwin-arm64');
      const linuxName = platforms.getBinaryName('linux-x64');

      assert.strictEqual(darwinName, 'arw');
      assert.strictEqual(linuxName, 'arw');
    });
  });

  describe('Download URLs', () => {
    const testVersion = '1.0.0';

    it('should generate valid URLs for all platforms', () => {
      const supportedPlatforms = Object.keys(platforms.SUPPORTED_PLATFORMS);

      supportedPlatforms.forEach(platform => {
        const url = platforms.getDownloadUrl(testVersion, platform);

        assert.ok(url.startsWith('https://github.com'));
        assert.ok(url.includes('/releases/download/'));
        assert.ok(url.includes(`v${testVersion}`));

        const config = platforms.getPlatformConfig(platform);
        assert.ok(url.endsWith(config.artifact));
      });
    });

    it('should generate valid checksum URLs', () => {
      const supportedPlatforms = Object.keys(platforms.SUPPORTED_PLATFORMS);

      supportedPlatforms.forEach(platform => {
        const url = platforms.getChecksumUrl(testVersion, platform);

        assert.ok(url.includes('.sha256'));
      });
    });
  });

  describe('Archive Format Detection', () => {
    it('should use tar.gz for Unix-like systems', () => {
      const darwinConfig = platforms.getPlatformConfig('darwin-arm64');
      const linuxConfig = platforms.getPlatformConfig('linux-x64');

      assert.ok(darwinConfig.artifact.endsWith('.tar.gz'));
      assert.ok(linuxConfig.artifact.endsWith('.tar.gz'));
    });

    it('should use zip for Windows', () => {
      const windowsConfig = platforms.getPlatformConfig('win32-x64');
      assert.ok(windowsConfig.artifact.endsWith('.zip'));
    });
  });

  describe('Architecture Support', () => {
    it('should support x64 architecture', () => {
      const x64Platforms = ['darwin-x64', 'linux-x64', 'win32-x64'];

      x64Platforms.forEach(platform => {
        assert.ok(platforms.SUPPORTED_PLATFORMS[platform]);
      });
    });

    it('should support ARM64 architecture', () => {
      const arm64Platforms = ['darwin-arm64', 'linux-arm64'];

      arm64Platforms.forEach(platform => {
        assert.ok(platforms.SUPPORTED_PLATFORMS[platform]);
      });
    });
  });

  describe('Error Messages', () => {
    it('should provide helpful error for unsupported platform', () => {
      try {
        platforms.getPlatformConfig('freebsd-x64');
        assert.fail('Should have thrown error');
      } catch (error) {
        assert.ok(error.message.includes('Unsupported platform'));
        assert.ok(error.message.includes('darwin-x64'));
        assert.ok(error.message.includes('WASM fallback'));
      }
    });
  });
});
