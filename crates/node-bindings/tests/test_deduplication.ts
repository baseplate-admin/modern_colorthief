import { describe, it, expect } from './test_compat';
import { getPalette } from '../index.js';
import { kaijuImagePath } from './test_helper.js';

describe('Deduplication', () => {
    it('no duplicate colors in palette', async () => {
        const palette = await getPalette(kaijuImagePath(), 255);
        const serialized = palette.map(c => c.join(','));
        expect(new Set(serialized).size).toBe(serialized.length);
        expect(palette.length).toBeGreaterThan(0);
        expect(palette.length).toBeLessThanOrEqual(255);
    });
});
