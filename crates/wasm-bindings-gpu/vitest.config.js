import { defineConfig } from 'vitest/config';
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
                    browser: {
                        enabled: true,
                        name: 'chrome',
                        channel: 'Chrome',
                        headless: true,
                    },
                },
            },
            {
                test: {
                    name: 'firefox',
                    include: ['tests/browser.test.js'],
                    browser: {
                        enabled: true,
                        name: 'firefox',
                        channel: 'Firefox',
                        headless: true,
                    },
                },
            },
        ],
    },
});
