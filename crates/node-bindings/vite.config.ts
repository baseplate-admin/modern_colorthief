import { defineConfig } from 'vite';
import { dts } from 'rolldown-plugin-dts';
import { resolve } from 'node:path';

export default defineConfig({
    plugins: dts({ generator: 'oxc' }).map((plugin) =>
        plugin.name.endsWith('fake-js') ? { ...plugin, enforce: 'pre' } : plugin
    ),
    build: {
        lib: {
            entry: [
                resolve(import.meta.dirname, 'index.ts'),
                resolve(import.meta.dirname, 'native.ts'),
            ],
            formats: ['es'],
            fileName: (format, entryName) => `${entryName}.js`,
        },
        outDir: 'dist',
        rolldownOptions: {
            external: ['sharp', /^node:/],
            output: {
                entryFileNames: '[name].js',
            },
        },
        minify: true,
        target: 'node18',
    },
});
