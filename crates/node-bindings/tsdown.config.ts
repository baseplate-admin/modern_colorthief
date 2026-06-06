import { defineConfig } from 'tsdown';

export default defineConfig({
    entry: ['index.ts'],
    format: ['cjs', 'esm'],
    dts: true,
    outDir: 'dist',
    external: ['./native.js', 'sharp'],
    minify: true,
    target: 'node20',
});
