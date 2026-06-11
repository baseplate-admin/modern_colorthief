import { defineConfig } from 'vitest/config';
import { playwright } from '@vitest/browser-playwright';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
    plugins: [wasm()],
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
                        provider: playwright({ launch: { headless: true } }),
                        name: 'chrome',
                    },
                },
            },
            {
                test: {
                    name: 'firefox',
                    include: ['tests/browser.test.js'],
                    pool: 'browser',
                    browser: {
                        provider: playwright({ launch: { headless: true } }),
                        name: 'firefox',
                    },
                },
            },
        ],
    },
});
