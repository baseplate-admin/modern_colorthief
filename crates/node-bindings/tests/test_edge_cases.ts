import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';
import { readFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import sharp from 'sharp';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TEST_IMAGE = resolve(__dirname, 'test.jpg');
const KAIJU_IMAGE = resolve(__dirname, 'kaiju_no_8.jpg');

describe('Edge cases', () => {
    // -- Quality bounds --

    it('quality=1 is valid', async () => {
        const color = await getColor(TEST_IMAGE, 1);
        expect(color.length).toBe(3);
    });

    it('quality=10 is valid', async () => {
        const color = await getColor(TEST_IMAGE, 10);
        expect(color.length).toBe(3);
    });

    it('quality=5 is valid', async () => {
        const color = await getColor(TEST_IMAGE, 5);
        expect(color.length).toBe(3);
    });

    // -- Different images produce different results --

    it('different images produce different colors', async () => {
        const c1 = await getColor(TEST_IMAGE);
        const c2 = await getColor(KAIJU_IMAGE);
        expect(c1).not.toEqual(c2);
    });

    it('different images produce different palettes', async () => {
        const p1 = await getPalette(TEST_IMAGE, 5);
        const p2 = await getPalette(KAIJU_IMAGE, 5);
        expect(p1).not.toEqual(p2);
    });

    // -- Small images --

    it('handles 1x1 pixel image', async () => {
        const buffer = await sharp({
            create: { width: 1, height: 1, channels: 3, background: { r: 128, g: 64, b: 32 } },
        }).toBuffer();
        const color = await getColor(buffer);
        expect(color.length).toBe(3);
    });

    it('handles 2x2 pixel image', async () => {
        const buffer = await sharp({
            create: { width: 2, height: 2, channels: 3, background: { r: 50, g: 100, b: 150 } },
        }).toBuffer();
        const color = await getColor(buffer);
        expect(color.length).toBe(3);
    });

    it('handles small 10x10 image', async () => {
        const buffer = await sharp({
            create: { width: 10, height: 10, channels: 3, background: { r: 200, g: 100, b: 50 } },
        }).toBuffer();
        const palette = await getPalette(buffer, 5);
        expect(palette.length).toBeGreaterThan(0);
    });

    // -- Large quality values --

    it('handles large quality value', async () => {
        const color = await getColor(TEST_IMAGE, 10);
        expect(color.length).toBe(3);
    });

    // -- Determinism --

    it('deterministic across repeated calls', async () => {
        const results = await Promise.all(
            Array.from({ length: 5 }, () => getColor(TEST_IMAGE)),
        );
        expect(results.every(r => r[0] === results[0][0] && r[1] === results[0][1] && r[2] === results[0][2])).toBe(true);
    });

    // -- Consistent across input types --

    it('path and buffer input produce same dominant color', async () => {
        const colorFromPath = await getColor(TEST_IMAGE);
        const colorFromBuffer = await getColor(readFileSync(TEST_IMAGE));
        expect(colorFromPath).toEqual(colorFromBuffer);
    });
});
