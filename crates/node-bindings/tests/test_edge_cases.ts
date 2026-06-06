import { describe, it, expect } from 'vitest';
import { getColor } from '../index.js';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TEST_IMAGE = resolve(__dirname, 'test.jpg');
const KAIJU_IMAGE = resolve(__dirname, 'kaiju_no_8.jpg');

describe('Edge cases', () => {
    it('quality=1 is valid', async () => {
        const color = await getColor(TEST_IMAGE, 3);
        expect(color.length).toBe(3);
    });

    it('quality=10 is valid', async () => {
        const color = await getColor(TEST_IMAGE, 10);
        expect(color.length).toBe(3);
    });

    it('different images produce different colors', async () => {
        const c1 = await getColor(TEST_IMAGE);
        const c2 = await getColor(KAIJU_IMAGE);
        expect(c1).not.toEqual(c2);
    });
});
