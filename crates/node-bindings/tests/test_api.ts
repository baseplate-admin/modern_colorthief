import { describe, it, expect } from './test_compat';
import { getPalette, getColor } from '../index.js';
import { testImagePath, testImageBuffer, kaijuImagePath } from './test_helper.js';

describe('API surface', () => {
    it('getPalette is a function', () => {
        expect(typeof getPalette).toBe('function');
    });

    it('getColor is a function', () => {
        expect(typeof getColor).toBe('function');
    });

    it('getPalette accepts path', async () => {
        const palette = await getPalette(testImagePath());
        expect(Array.isArray(palette)).toBe(true);
    });

    it('getPalette accepts buffer', async () => {
        const palette = await getPalette(testImageBuffer());
        expect(Array.isArray(palette)).toBe(true);
    });

    it('getColor accepts buffer', async () => {
        const color = await getColor(testImageBuffer());
        expect(Array.isArray(color)).toBe(true);
    });

    it('getColor accepts path', async () => {
        const color = await getColor(testImagePath());
        expect(Array.isArray(color)).toBe(true);
    });

    // -- Different images produce different results --

    it('different images produce different dominant colors', async () => {
        const c1 = await getColor(testImagePath());
        const c2 = await getColor(kaijuImagePath());
        expect(c1).not.toEqual(c2);
    });
});
