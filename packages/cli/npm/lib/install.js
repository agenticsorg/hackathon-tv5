#!/usr/bin/env node
/**
 * Post-install script to download platform-specific binary
 * Downloads the correct ARW binary for the current platform
 */

const fs = require('fs');
const path = require('path');
const https = require('https');
const { createHash } = require('crypto');
const { promisify } = require('util');
const { pipeline } = require('stream');
const { createGunzip } = require('zlib');
const tar = require('tar');

const platforms = require('./platforms');

const streamPipeline = promisify(pipeline);

const PACKAGE_VERSION = require('../package.json').version;
const BINARIES_DIR = path.join(__dirname, '..', 'binaries');
const CACHE_DIR = path.join(require('os').homedir(), '.cache', 'arw');

/**
 * Ensure directory exists
 */
function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

/**
 * Download file from URL
 */
async function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);

    https.get(url, {
      headers: {
        'User-Agent': 'arw-npm-installer'
      }
    }, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        // Follow redirect
        return downloadFile(response.headers.location, dest).then(resolve).catch(reject);
      }

      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode} ${response.statusMessage}`));
        return;
      }

      response.pipe(file);

      file.on('finish', () => {
        file.close();
        resolve();
      });
    }).on('error', (err) => {
      fs.unlink(dest, () => {});
      reject(err);
    });
  });
}

/**
 * Calculate SHA256 checksum of file
 */
async function calculateChecksum(filePath) {
  return new Promise((resolve, reject) => {
    const hash = createHash('sha256');
    const stream = fs.createReadStream(filePath);

    stream.on('data', (data) => hash.update(data));
    stream.on('end', () => resolve(hash.digest('hex')));
    stream.on('error', reject);
  });
}

/**
 * Verify checksum of downloaded file
 */
async function verifyChecksum(filePath, checksumUrl) {
  try {
    const checksumFile = path.join(CACHE_DIR, 'checksum.txt');
    await downloadFile(checksumUrl, checksumFile);

    const expectedChecksum = fs.readFileSync(checksumFile, 'utf-8').trim().split(' ')[0];
    const actualChecksum = await calculateChecksum(filePath);

    if (expectedChecksum !== actualChecksum) {
      throw new Error(
        `Checksum verification failed!\n` +
        `Expected: ${expectedChecksum}\n` +
        `Actual: ${actualChecksum}`
      );
    }

    console.log('✓ Checksum verified successfully');
  } catch (error) {
    console.warn('⚠ Warning: Could not verify checksum:', error.message);
    console.warn('  Continuing anyway...');
  }
}

/**
 * Extract tar.gz file
 */
async function extractTarGz(archivePath, destDir) {
  ensureDir(destDir);

  await tar.extract({
    file: archivePath,
    cwd: destDir,
    strip: 1
  });
}

/**
 * Extract zip file (for Windows)
 */
async function extractZip(archivePath, destDir) {
  const AdmZip = require('adm-zip');
  const zip = new AdmZip(archivePath);

  ensureDir(destDir);
  zip.extractAllTo(destDir, true);
}

/**
 * Main installation function
 */
async function install() {
  try {
    console.log('🔧 Installing ARW CLI...');

    // Check platform support
    if (!platforms.isPlatformSupported()) {
      console.error(
        '❌ Unsupported platform:', platforms.getCurrentPlatform(), '\n',
        '   Supported platforms:', Object.keys(platforms.SUPPORTED_PLATFORMS).join(', '), '\n',
        '   Please build from source or use the WASM version.'
      );
      process.exit(1);
    }

    const config = platforms.getPlatformConfig();
    console.log(`📦 Platform: ${config.platform}`);

    // Check if binary already exists
    const binaryPath = path.join(BINARIES_DIR, platforms.getBinaryName());
    if (fs.existsSync(binaryPath)) {
      console.log('✓ Binary already installed');
      return;
    }

    // Prepare directories
    ensureDir(BINARIES_DIR);
    ensureDir(CACHE_DIR);

    // Download URLs
    const downloadUrl = platforms.getDownloadUrl(PACKAGE_VERSION);
    const checksumUrl = platforms.getChecksumUrl(PACKAGE_VERSION);

    console.log(`📥 Downloading from: ${downloadUrl}`);

    // Download archive
    const archivePath = path.join(CACHE_DIR, config.artifact);
    await downloadFile(downloadUrl, archivePath);
    console.log('✓ Download complete');

    // Verify checksum
    await verifyChecksum(archivePath, checksumUrl);

    // Extract archive
    console.log('📦 Extracting archive...');
    if (config.artifact.endsWith('.tar.gz')) {
      await extractTarGz(archivePath, BINARIES_DIR);
    } else if (config.artifact.endsWith('.zip')) {
      await extractZip(archivePath, BINARIES_DIR);
    }

    // Make binary executable (Unix-like systems)
    if (process.platform !== 'win32') {
      fs.chmodSync(binaryPath, 0o755);
    }

    console.log('✓ Installation complete!');
    console.log(`\n🎉 ARW CLI is ready to use!`);
    console.log(`   Try: npx arw --help\n`);

  } catch (error) {
    console.error('\n❌ Installation failed:', error.message);
    console.error('\nFallback options:');
    console.error('  1. Build from source: npm run build:rust');
    console.error('  2. Use WASM version: npm run build:wasm');
    console.error('  3. Download manually from GitHub releases\n');
    process.exit(1);
  }
}

// Only run if executed directly
if (require.main === module) {
  install();
}

module.exports = { install };
