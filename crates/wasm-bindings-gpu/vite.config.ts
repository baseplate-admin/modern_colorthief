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
                    name: 'browser',
                    include: ['tests/browser.test.js'],
                    pool: 'browser',
                    browser: {
                        enabled: true,
                        provider: playwright({ launch: { headless: true } }),
                        instances: [
                            { name: 'chrome', browser: 'chromium' },
                            { name: 'firefox', browser: 'firefox' },
                        ],
                    },
                },
            },
        ],
    },
});
