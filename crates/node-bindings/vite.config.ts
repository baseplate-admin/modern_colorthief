import { defineConfig } from 'vite';
import dts from 'vite-plugin-dts';
import { resolve } from 'node:path';

export default defineConfig({
    plugins: [
        dts({
            entryRoot: '.',
            entryRootForward: false,
        }),
    ],
    build: {
        lib: {
            entry: [
                resolve(__dirname, 'index.ts'),
                resolve(__dirname, 'native.ts'),
            ],
            formats: ['es'],
            fileName: (format, entryName) => `${entryName}.js`,
        },
        outDir: 'dist',
        rollupOptions: {
            external: ['sharp', /^node:/],
            output: {
                entryFileNames: '[name].js',
            },
        },
        minify: true,
        target: 'node18',
    },
});
