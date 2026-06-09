import { defineConfig } from 'tsdown';

export default defineConfig({
    entry: ['index.ts', 'native.ts'],
    format: ['cjs', 'esm'],
    dts: true,
    outDir: 'dist',
    deps: {
        neverBundle: ['sharp'],
    },
    minify: true,
    target: 'node20',
});
