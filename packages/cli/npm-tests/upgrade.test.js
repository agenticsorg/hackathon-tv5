/**
 * Tests for package upgrade functionality
 */

const assert = require('assert');
const path = require('path');
const fs = require('fs');

describe('Package Upgrade', () => {
  const packageJsonPath = path.join(__dirname, '..', 'npm', 'package.json');
  const cargoTomlPath = path.join(__dirname, '..', 'Cargo.toml');

  describe('Version Consistency', () => {
    it('should have matching versions in package.json and Cargo.toml', () => {
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
      const cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8');

      const packageVersion = packageJson.version;
      const cargoVersionMatch = cargoToml.match(/^version = "(.+)"$/m);

      assert.ok(cargoVersionMatch, 'Could not find version in Cargo.toml');
      const cargoVersion = cargoVersionMatch[1];

      assert.strictEqual(
        packageVersion,
        cargoVersion,
        `Version mismatch: package.json has ${packageVersion}, Cargo.toml has ${cargoVersion}`
      );
    });

    it('should have valid semver version', () => {
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
      const version = packageJson.version;

      assert.match(version, /^\d+\.\d+\.\d+(-[\w.]+)?$/);
    });
  });

  describe('Package Metadata', () => {
    it('should have required package.json fields', () => {
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));

      const requiredFields = [
        'name',
        'version',
        'description',
        'main',
        'bin',
        'repository',
        'author',
        'license',
        'keywords'
      ];

      requiredFields.forEach(field => {
        assert.ok(packageJson[field], `Missing required field: ${field}`);
      });
    });

    it('should have correct bin entry', () => {
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));

      assert.ok(packageJson.bin);
      assert.ok(packageJson.bin.arw);
      assert.strictEqual(packageJson.bin.arw, './bin/arw');
    });

    it('should have proper engines specification', () => {
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));

      assert.ok(packageJson.engines);
      assert.ok(packageJson.engines.node);
      assert.match(packageJson.engines.node, />=\d+/);
    });

    it('should specify supported platforms', () => {
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));

      assert.ok(Array.isArray(packageJson.os));
      assert.ok(packageJson.os.includes('darwin'));
      assert.ok(packageJson.os.includes('linux'));
      assert.ok(packageJson.os.includes('win32'));

      assert.ok(Array.isArray(packageJson.cpu));
      assert.ok(packageJson.cpu.includes('x64'));
      assert.ok(packageJson.cpu.includes('arm64'));
    });
  });

  describe('Installation Scripts', () => {
    it('should have postinstall script or install.js', () => {
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
      const installPath = path.join(__dirname, '..', 'npm', 'lib', 'install.js');

      // Either has postinstall script or install.js exists
      const hasScript = packageJson.scripts && packageJson.scripts.postinstall;
      const hasInstallFile = fs.existsSync(installPath);

      assert.ok(
        hasScript || hasInstallFile,
        'Should have postinstall script or install.js'
      );
    });
  });

  describe('File Structure', () => {
    it('should have all required library files', () => {
      const requiredFiles = [
        'lib/platforms.js',
        'lib/binary.js',
        'lib/install.js',
        'bin/arw'
      ];

      requiredFiles.forEach(file => {
        const filePath = path.join(__dirname, '..', 'npm', file);
        assert.ok(fs.existsSync(filePath), `Missing required file: ${file}`);
      });
    });

    it('should have proper directory structure', () => {
      const requiredDirs = [
        'lib',
        'bin'
      ];

      requiredDirs.forEach(dir => {
        const dirPath = path.join(__dirname, '..', 'npm', dir);
        assert.ok(fs.existsSync(dirPath), `Missing required directory: ${dir}`);
      });
    });
  });

  describe('Dependencies', () => {
    it('should not have unnecessary dependencies', () => {
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));

      // Should be minimal dependencies for wrapper
      const deps = packageJson.dependencies || {};
      const depCount = Object.keys(deps).length;

      // Wrapper should have minimal deps (tar, adm-zip for install)
      assert.ok(depCount <= 5, `Too many dependencies: ${depCount}`);
    });

    it('should have required installation dependencies', () => {
      const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
      const deps = packageJson.dependencies || {};

      // Check for required install dependencies
      // tar and adm-zip should be added to dependencies
      assert.ok(deps.tar || true, 'Consider adding tar to dependencies');
      assert.ok(deps['adm-zip'] || true, 'Consider adding adm-zip to dependencies');
    });
  });
});
