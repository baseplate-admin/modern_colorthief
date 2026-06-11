import { defineConfig } from 'vitest/config';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
    plugins: [wasm()],
    test: {
        name: 'chrome',
        include: ['tests/browser.test.js'],
        pool: 'browser',
        browser: {
            enabled: true,
            name: 'chrome',
            channel: 'Chrome',
            headless: true,
        },
    },
});
