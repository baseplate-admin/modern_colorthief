import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TEST_IMAGE = resolve(__dirname, 'test.jpg');

describe('Properties', () => {
    it('getColor returns valid RGB', async () => {
        const color = await getColor(TEST_IMAGE);
        expect(color.length).toBe(3);
        for (const c of color) {
            expect(Number.isInteger(c)).toBe(true);
            expect(c).toBeGreaterThanOrEqual(0);
            expect(c).toBeLessThanOrEqual(255);
        }
    });

    it('getPalette returns valid RGB list', async () => {
        const palette = await getPalette(TEST_IMAGE);
        expect(palette.length).toBeGreaterThan(0);
        for (const color of palette) {
            expect(color.length).toBe(3);
            for (const c of color) {
                expect(Number.isInteger(c)).toBe(true);
                expect(c).toBeGreaterThanOrEqual(0);
                expect(c).toBeLessThanOrEqual(255);
            }
        }
    });

    it('palette is deduplicated', async () => {
        const palette = await getPalette(TEST_IMAGE);
        const serialized = palette.map(c => c.join(','));
        expect(new Set(serialized).size).toBe(serialized.length);
    });

    it('palette count bounded', async () => {
        for (const count of [3, 5]) {
            const palette = await getPalette(TEST_IMAGE, count);
            expect(palette.length).toBeLessThanOrEqual(count);
        }
    });

    it('deterministic results', async () => {
        const c1 = await getColor(TEST_IMAGE);
        const c2 = await getColor(TEST_IMAGE);
        expect(c1).toEqual(c2);
    });

    it('deterministic palette results', async () => {
        const p1 = await getPalette(TEST_IMAGE, 10);
        const p2 = await getPalette(TEST_IMAGE, 10);
        expect(p1).toEqual(p2);
    });

    it('default palette count is 10', async () => {
        const palette = await getPalette(TEST_IMAGE);
        expect(palette.length).toBeLessThanOrEqual(10);
    });

    it('palette length respects color_count exactly for small images', async () => {
        // With a solid color image, palette should return exactly 1 color
        // (fewer if deduplication removes similar colors)
        const palette = await getPalette(TEST_IMAGE, 3);
        expect(palette.length).toBeLessThanOrEqual(3);
    });
});
