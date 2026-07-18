import { defineConfig } from 'vite';
import { resolve } from 'node:path';

export default defineConfig({
    plugins: [],
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
