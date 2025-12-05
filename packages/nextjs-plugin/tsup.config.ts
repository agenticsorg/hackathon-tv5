import { defineConfig } from 'tsup';

export default defineConfig([
  // Server-side plugin (no 'use client' needed)
  {
    entry: {
      index: 'src/index.ts'
    },
    format: ['cjs', 'esm'],
    dts: true,
    splitting: false,
    sourcemap: true,
    clean: true,
    external: ['react', 'react-dom', 'next', 'chokidar'],
    treeshake: true,
    minify: false
  },
  // CLI tool
  {
    entry: {
      cli: 'src/cli.ts'
    },
    format: ['cjs'],
    dts: false,
    splitting: false,
    sourcemap: true,
    external: ['chokidar'],
    treeshake: true,
    minify: false
  },
  // Runtime utilities (no 'use client' needed)
  {
    entry: {
      runtime: 'src/runtime.ts'
    },
    format: ['cjs', 'esm'],
    dts: true,
    splitting: false,
    sourcemap: true,
    external: ['react', 'react-dom', 'next'],
    treeshake: true,
    minify: false
  },
  // Client components (needs 'use client')
  {
    entry: {
      components: 'src/components/index.ts'
    },
    format: ['cjs', 'esm'],
    dts: true,
    splitting: false,
    sourcemap: true,
    external: ['react', 'react-dom', 'next'],
    treeshake: false,  // Disable treeshaking to preserve directives
    minify: false,
    esbuildOptions(options) {
      options.banner = {
        js: '"use client";'
      };
    }
  }
]);
