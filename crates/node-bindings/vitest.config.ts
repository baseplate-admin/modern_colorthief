import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        include: ['tests/test_*.ts'],
        exclude: ['tests/test_helper.ts', 'tests/test_compat.ts', 'tests/test_runner.test.ts'],
        testTimeout: 10000,
    },
});
