import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  // Enable standalone output for Docker/Cloud Run deployment
  output: 'standalone',

  // Image optimization for TMDB images
  images: {
    remotePatterns: [
      {
        protocol: 'https',
        hostname: 'image.tmdb.org',
        pathname: '/t/p/**',
      },
    ],
    // Optimize for common poster sizes
    deviceSizes: [640, 750, 828, 1080, 1200, 1920],
    imageSizes: [16, 32, 48, 64, 96, 128, 256, 384, 500],
  },

  // Headers for ARW discovery
  async headers() {
    return [
      {
        source: '/:path*',
        headers: [
          {
            key: 'X-ARW-Version',
            value: '0.1',
          },
        ],
      },
      {
        source: '/.well-known/arw-manifest.json',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=3600, stale-while-revalidate=86400',
          },
          {
            key: 'Content-Type',
            value: 'application/json',
          },
        ],
      },
      {
        source: '/llms.txt',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=3600, stale-while-revalidate=86400',
          },
          {
            key: 'Content-Type',
            value: 'text/plain; charset=utf-8',
          },
        ],
      },
      {
        source: '/llms/:path*.llm.md',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=3600, stale-while-revalidate=86400',
          },
          {
            key: 'Content-Type',
            value: 'text/markdown; charset=utf-8',
          },
        ],
      },
    ];
  },

  // Redirects for legacy paths
  async redirects() {
    return [
      {
        source: '/arw-manifest.json',
        destination: '/.well-known/arw-manifest.json',
        permanent: true,
      },
    ];
  },

  // Experimental features
  experimental: {
    // Enable server actions
    serverActions: {
      bodySizeLimit: '2mb',
    },
  },

  // Exclude native modules from webpack bundling
  serverExternalPackages: [
    'ruvector',
    '@ruvector/core',
    '@ruvector/sona',
    '@ruvector/sona-darwin-arm64',
    '@ruvector/sona-darwin-x64',
    '@ruvector/sona-linux-x64-gnu',
    '@ruvector/sona-linux-arm64-gnu',
    '@ruvector/sona-win32-x64-msvc',
  ],
};

export default nextConfig;
