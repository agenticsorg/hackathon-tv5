/**
 * Platform detection and binary selection
 * Determines the correct binary to download based on OS and architecture
 */

const SUPPORTED_PLATFORMS = {
  'darwin-x64': {
    binary: 'arw-macos-x64',
    artifact: 'arw-darwin-x64.tar.gz',
    checksum: 'arw-darwin-x64.tar.gz.sha256'
  },
  'darwin-arm64': {
    binary: 'arw-macos-arm64',
    artifact: 'arw-darwin-arm64.tar.gz',
    checksum: 'arw-darwin-arm64.tar.gz.sha256'
  },
  'linux-x64': {
    binary: 'arw-linux-x64',
    artifact: 'arw-linux-x64.tar.gz',
    checksum: 'arw-linux-x64.tar.gz.sha256'
  },
  'linux-arm64': {
    binary: 'arw-linux-arm64',
    artifact: 'arw-linux-arm64.tar.gz',
    checksum: 'arw-linux-arm64.tar.gz.sha256'
  },
  'win32-x64': {
    binary: 'arw-windows-x64.exe',
    artifact: 'arw-windows-x64.zip',
    checksum: 'arw-windows-x64.zip.sha256'
  }
};

/**
 * Get current platform identifier
 * @returns {string} Platform identifier (e.g., 'darwin-arm64')
 */
function getCurrentPlatform() {
  const platform = process.platform;
  const arch = process.arch;

  return `${platform}-${arch}`;
}

/**
 * Check if current platform is supported
 * @returns {boolean} True if platform is supported
 */
function isPlatformSupported() {
  const platform = getCurrentPlatform();
  return platform in SUPPORTED_PLATFORMS;
}

/**
 * Get platform configuration
 * @param {string} [platformId] Platform identifier (defaults to current)
 * @returns {Object} Platform configuration
 * @throws {Error} If platform is not supported
 */
function getPlatformConfig(platformId) {
  const platform = platformId || getCurrentPlatform();

  if (!(platform in SUPPORTED_PLATFORMS)) {
    throw new Error(
      `Unsupported platform: ${platform}\n` +
      `Supported platforms: ${Object.keys(SUPPORTED_PLATFORMS).join(', ')}\n` +
      `Consider using the WASM fallback or building from source.`
    );
  }

  return {
    platform,
    ...SUPPORTED_PLATFORMS[platform]
  };
}

/**
 * Get download URL for binary
 * @param {string} version Version to download
 * @param {string} [platformId] Platform identifier (defaults to current)
 * @returns {string} Download URL
 */
function getDownloadUrl(version, platformId) {
  const config = getPlatformConfig(platformId);
  const baseUrl = 'https://github.com/agent-ready-web/agent-ready-web/releases/download';

  return `${baseUrl}/v${version}/${config.artifact}`;
}

/**
 * Get checksum URL for binary
 * @param {string} version Version to download
 * @param {string} [platformId] Platform identifier (defaults to current)
 * @returns {string} Checksum URL
 */
function getChecksumUrl(version, platformId) {
  const config = getPlatformConfig(platformId);
  const baseUrl = 'https://github.com/agent-ready-web/agent-ready-web/releases/download';

  return `${baseUrl}/v${version}/${config.checksum}`;
}

/**
 * Get binary name with extension
 * @param {string} [platformId] Platform identifier (defaults to current)
 * @returns {string} Binary name
 */
function getBinaryName(platformId) {
  const platform = platformId || getCurrentPlatform();

  if (platform.startsWith('win32')) {
    return 'arw.exe';
  }

  return 'arw';
}

module.exports = {
  SUPPORTED_PLATFORMS,
  getCurrentPlatform,
  isPlatformSupported,
  getPlatformConfig,
  getDownloadUrl,
  getChecksumUrl,
  getBinaryName
};
