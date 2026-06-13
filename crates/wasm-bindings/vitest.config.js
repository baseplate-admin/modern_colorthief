import { defineConfig } from 'vitest/config';
import wasm from 'vite-plugin-wasm';
import tsconfigPaths from 'vite-tsconfig-paths';

export default defineConfig({
    plugins: [tsconfigPaths(), wasm()],
    test: {
        include: ['tests/wasm.test.js'],
        // Browser-only tests (decodeImage, getPalette/getColor with image blobs)
        // require Canvas API which is unavailable in Node.js.
        // Run those manually in a browser or via Playwright.
    },
});
