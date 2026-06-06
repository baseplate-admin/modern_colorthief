import { describe, it, expect } from 'vitest';
import { getPalette } from '../index.js';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const KAIJU_IMAGE = resolve(__dirname, 'kaiju_no_8.jpg');

describe('Deduplication', () => {
    it('no duplicate colors in palette', async () => {
        const palette = await getPalette(KAIJU_IMAGE, 255);
        const serialized = palette.map(c => c.join(','));
        expect(new Set(serialized).size).toBe(serialized.length);
        expect(palette.length).toBeGreaterThan(0);
        expect(palette.length).toBeLessThanOrEqual(255);
    });
});
