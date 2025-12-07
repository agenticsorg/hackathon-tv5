/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  output: 'standalone',
  images: {
    remotePatterns: [
      {
        protocol: 'https',
        hostname: 'artworks.thetvdb.com',
        pathname: '/**',
      },
      {
        protocol: 'https',
        hostname: 'thetvdb.com',
        pathname: '/**',
      },
    ],
  },
}

module.exports = nextConfig
