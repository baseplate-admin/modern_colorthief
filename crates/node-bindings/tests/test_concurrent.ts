import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';
import { readFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TEST_IMAGE = resolve(__dirname, 'test.jpg');

describe('Concurrency', () => {
    it('concurrent getColor calls', async () => {
        const results = await Promise.all(
            Array.from({ length: 3 }, () => getColor(TEST_IMAGE)),
        );
        expect(results.length).toBe(3);
        expect(results.every(r => r.length === 3)).toBe(true);
    });

    it('concurrent mixed ops', async () => {
        const buffer = readFileSync(TEST_IMAGE);
        const results = await Promise.all([
            getColor(TEST_IMAGE),
            getPalette(TEST_IMAGE, 3),
            getColor(buffer),
        ]);
        expect(results.length).toBe(3);
    });
});
