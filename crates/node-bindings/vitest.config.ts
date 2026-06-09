import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        include: ['tests/test_*.ts'],
        exclude: ['tests/test_helper.ts'],
        testTimeout: 10000,
    },
});
