import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';
import { testImagePath, testImageBuffer } from './test_helper.js';

describe('Input types', () => {
    it('path input works for getColor', async () => {
        const color = await getColor(testImagePath());
        expect(color.length).toBe(3);
    });

    it('buffer input works for getColor', async () => {
        const color = await getColor(testImageBuffer());
        expect(color.length).toBe(3);
    });

    it('path input works for getPalette', async () => {
        const palette = await getPalette(testImagePath());
        expect(palette.length).toBeGreaterThan(0);
    });

    it('buffer input works for getPalette', async () => {
        const palette = await getPalette(testImageBuffer());
        expect(palette.length).toBeGreaterThan(0);
    });

    it('buffer not mutated', async () => {
        const buf = testImageBuffer();
        const copy = Buffer.from(buf);
        await getColor(buf);
        await getPalette(buf);
        expect(buf.equals(copy)).toBe(true);
    });

    it('path and buffer produce same dominant color', async () => {
        const colorFromPath = await getColor(testImagePath());
        const colorFromBuffer = await getColor(testImageBuffer());
        expect(colorFromPath).toEqual(colorFromBuffer);
    });

    it('path and buffer produce same palette', async () => {
        const paletteFromPath = await getPalette(testImagePath(), 10);
        const paletteFromBuffer = await getPalette(testImageBuffer(), 10);
        expect(paletteFromPath).toEqual(paletteFromBuffer);
    });
});
