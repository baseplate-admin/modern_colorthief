import js from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
    {
        ignores: ['dist/**', 'artifacts/**', 'node_modules/**', 'pkg/**'],
    },
    js.configs.recommended,
    ...tseslint.configs.recommended,
    {
        rules: {
            '@typescript-eslint/no-explicit-any': 'error',
            '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
            'no-console': ['error', { allow: ['warn', 'error'] }],
            'prefer-const': 'error',
        },
    },
    {
        // Test suites and examples exercise runtime APIs (Deno/Bun globals,
        // console output) — relax the strictest rules there.
        files: ['tests/*.ts', 'examples/**'],
        rules: {
            'no-console': 'off',
            '@typescript-eslint/no-explicit-any': 'off',
            '@typescript-eslint/no-unused-vars': 'off',
            'no-undef': 'off',
        },
    }
);
