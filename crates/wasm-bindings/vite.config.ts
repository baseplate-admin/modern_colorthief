import { defineConfig } from 'vitest/config';
import tsconfigPaths from 'vite-tsconfig-paths';
import wasm from 'vite-plugin-wasm';

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
            {
                test: {
                    name: 'chrome',
                    include: ['tests/browser.test.js'],
                    pool: 'browser',
                    browser: {
                        provider: 'playwright',
                        name: 'chrome',
                        headless: true,
                    },
                },
            },
            {
                test: {
                    name: 'firefox',
                    include: ['tests/browser.test.js'],
                    pool: 'browser',
                    browser: {
                        provider: 'playwright',
                        name: 'firefox',
                        headless: true,
                    },
                },
            },
        ],
    },
});
