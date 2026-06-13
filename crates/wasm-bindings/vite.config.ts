import { defineConfig } from 'vite';
import tsconfigPaths from 'vite-tsconfig-paths';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
    plugins: [tsconfigPaths(), wasm()],
});
