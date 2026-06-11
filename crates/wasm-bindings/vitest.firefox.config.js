import { defineConfig } from 'vitest/config';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
    plugins: [wasm()],
    test: {
        name: 'firefox',
        include: ['tests/browser.test.js'],
        pool: 'browser',
        browser: {
            enabled: true,
            name: 'firefox',
            channel: 'Firefox',
            headless: true,
        },
    },
});
