import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'standalone', // Required for Cloud Run deployment
  experimental: {
    // Turbopack is for dev only, not needed in production
  },
};

export default nextConfig;
