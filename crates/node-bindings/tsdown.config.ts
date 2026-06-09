import { defineConfig } from 'tsdown';

export default defineConfig([
    {
        entry: ['index.ts'],
        format: ['cjs', 'esm'],
        dts: true,
        outDir: 'dist',
        external: ['./native', 'sharp'],
        minify: true,
        target: 'node20',
    },
    {
        entry: ['native.ts'],
        format: ['esm'],
        dts: true,
        outDir: '.',
        minify: true,
        target: 'node20',
    },
]);
