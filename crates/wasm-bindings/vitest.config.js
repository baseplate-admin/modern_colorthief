import { defineConfig } from 'vitest/config';
import wasm from 'vite-plugin-wasm';
import tsconfigPaths from 'vite-tsconfig-paths';

export default defineConfig({
    plugins: [tsconfigPaths(), wasm()],
    test: {
        projects: [
            {
                test: {
                    name: 'node',
                    include: ['tests/node.test.js'],
                },
            },
            './vitest.chrome.config.js',
            './vitest.firefox.config.js',
        ],
    },
});
