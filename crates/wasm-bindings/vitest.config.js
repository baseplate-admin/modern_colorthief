import { defineConfig } from 'vitest/config';
import wasm from 'vite-plugin-wasm';
import tsconfigPaths from 'vite-tsconfig-paths';

export default defineConfig({
    plugins: [tsconfigPaths(), wasm()],
    test: {
        environment: 'happy-dom',
    },
});
