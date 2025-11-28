import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  trailingSlash: true,
  output: 'export',
  images: {
    unoptimized: true
  },
  turbopack: {
    root: __dirname,
  }
};

export default nextConfig;
