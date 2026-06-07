import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';
import { testImagePath, testImageBuffer } from './test_helper.js';

describe('Properties', () => {
    // -- Return value structure --

    it('getColor returns 3-element RGB array', async () => {
        const color = await getColor(testImagePath());
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
        color.forEach(v => {
            expect(typeof v).toBe('number');
            expect(v).toBeGreaterThanOrEqual(0);
            expect(v).toBeLessThanOrEqual(255);
        });
    });

    it('getPalette returns array of 3-element RGB arrays', async () => {
        const palette = await getPalette(testImagePath());
        expect(Array.isArray(palette)).toBe(true);
        palette.forEach(color => {
            expect(Array.isArray(color)).toBe(true);
            expect(color.length).toBe(3);
            color.forEach(v => {
                expect(typeof v).toBe('number');
                expect(v).toBeGreaterThanOrEqual(0);
                expect(v).toBeLessThanOrEqual(255);
            });
        });
    });

    it('deterministic color results', async () => {
        const c1 = await getColor(testImagePath());
        const c2 = await getColor(testImagePath());
        expect(c1).toEqual(c2);
    });

    it('deterministic palette results', async () => {
        const p1 = await getPalette(testImagePath(), 10);
        const p2 = await getPalette(testImagePath(), 10);
        expect(p1).toEqual(p2);
    });

    it('palette length respects color_count=3', async () => {
        const palette = await getPalette(testImagePath(), 3);
        expect(palette.length).toBeLessThanOrEqual(3);
    });

    it('palette length respects color_count=5', async () => {
        const palette = await getPalette(testImagePath(), 5);
        expect(palette.length).toBeLessThanOrEqual(5);
    });
});
