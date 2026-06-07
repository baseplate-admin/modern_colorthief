import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';
import { testImagePath, testImageBuffer, kaijuImagePath, kaijuImageBuffer } from './test_helper.js';

describe('Edge cases', () => {
    // -- Quality bounds --

    it('quality=1 is valid', async () => {
        const color = await getColor(testImagePath(), 1);
        expect(color.length).toBe(3);
    });

    it('quality=10 is valid', async () => {
        const color = await getColor(testImagePath(), 10);
        expect(color.length).toBe(3);
    });

    it('quality=5 is valid', async () => {
        const color = await getColor(testImagePath(), 5);
        expect(color.length).toBe(3);
    });

    // -- Different images produce different results --

    it('different images produce different colors', async () => {
        const c1 = await getColor(testImageBuffer());
        const c2 = await getColor(kaijuImagePath());
        expect(c1).not.toEqual(c2);
    });

    it('different images produce different palettes', async () => {
        const p1 = await getPalette(testImageBuffer(), 5);
        const p2 = await getPalette(kaijuImagePath(), 5);
        expect(p1).not.toEqual(p2);
    });

    // -- Determinism --

    it('deterministic across repeated calls', async () => {
        const results = await Promise.all(
            Array.from({ length: 5 }, () => getColor(testImagePath())),
        );
        expect(results.every(r => r[0] === results[0][0] && r[1] === results[0][1] && r[2] === results[0][2])).toBe(true);
    });

    // -- Consistent across input types --

    it('path and buffer input produce same dominant color', async () => {
        const colorFromPath = await getColor(testImagePath());
        const colorFromBuffer = await getColor(testImageBuffer());
        expect(colorFromPath).toEqual(colorFromBuffer);
    });
});
