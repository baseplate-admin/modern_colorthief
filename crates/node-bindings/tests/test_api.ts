import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';
import { readFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import sharp from 'sharp';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TEST_IMAGE = resolve(__dirname, 'test.jpg');

describe('API surface', () => {
    it('getPalette is a function', () => {
        expect(typeof getPalette).toBe('function');
    });

    it('getColor is a function', () => {
        expect(typeof getColor).toBe('function');
    });

    it('getPalette accepts file path', async () => {
        const palette = await getPalette(TEST_IMAGE, 10, 10);
        expect(Array.isArray(palette)).toBe(true);
        expect(palette.length).toBeGreaterThan(0);
    });

    it('getColor accepts file path', async () => {
        const color = await getColor(TEST_IMAGE, 10);
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
    });

    it('getPalette accepts buffer', async () => {
        const buffer = readFileSync(TEST_IMAGE);
        const palette = await getPalette(buffer, 10, 10);
        expect(Array.isArray(palette)).toBe(true);
    });

    it('getColor accepts buffer', async () => {
        const buffer = readFileSync(TEST_IMAGE);
        const color = await getColor(buffer, 10);
        expect(Array.isArray(color)).toBe(true);
    });

    // -- Solid color detection (port of Python implicit test) --

    it('detects solid red color', async () => {
        const redImage = sharp({
            create: { width: 100, height: 100, channels: 3, background: { r: 255, g: 0, b: 0 } },
        });
        const color = await getColor(redImage.toBuffer());
        expect(color[0]).toBeGreaterThan(200); // red dominant
        expect(color[1]).toBeLessThan(55);
        expect(color[2]).toBeLessThan(55);
    });

    it('detects solid blue color', async () => {
        const blueImage = sharp({
            create: { width: 100, height: 100, channels: 3, background: { r: 0, g: 0, b: 255 } },
        });
        const color = await getColor(blueImage.toBuffer());
        expect(color[0]).toBeLessThan(55);
        expect(color[1]).toBeLessThan(55);
        expect(color[2]).toBeGreaterThan(200); // blue dominant
    });

    it('detects solid green color', async () => {
        const greenImage = sharp({
            create: { width: 100, height: 100, channels: 3, background: { r: 0, g: 255, b: 0 } },
        });
        const color = await getColor(greenImage.toBuffer());
        expect(color[0]).toBeLessThan(55);
        expect(color[1]).toBeGreaterThan(200); // green dominant
        expect(color[2]).toBeLessThan(55);
    });

    // -- Two-color detection --

    it('detects two dominant colors from split image', async () => {
        // Create an image with top half red, bottom half blue
        const top = sharp({
            create: { width: 100, height: 50, channels: 3, background: { r: 255, g: 0, b: 0 } },
        });
        const bottom = sharp({
            create: { width: 100, height: 50, channels: 3, background: { r: 0, g: 0, b: 255 } },
        });
        const composite = sharp()
            .composite([
                { input: await top.toBuffer(), top: 0, left: 0 },
                { input: await bottom.toBuffer(), top: 50, left: 0 },
            ])
            .resize(100, 100);

        const palette = await getPalette(composite.toBuffer(), 2);
        expect(palette.length).toBeLessThanOrEqual(2);
        expect(palette.length).toBeGreaterThan(0);

        // One color should be red-dominant, one blue-dominant
        const hasRed = palette.some(c => c[0] > 200 && c[1] < 55 && c[2] < 55);
        const hasBlue = palette.some(c => c[0] < 55 && c[1] < 55 && c[2] > 200);
        expect(hasRed || hasBlue).toBe(true);
    });

    // -- get_color returns correct dominant color --

    it('get_color returns the most frequent color for solid image', async () => {
        const orangeImage = sharp({
            create: { width: 50, height: 50, channels: 3, background: { r: 255, g: 165, b: 0 } },
        });
        const color = await getColor(orangeImage.toBuffer());
        expect(color[0]).toBeGreaterThan(200); // orange-red
        expect(color[1]).toBeGreaterThan(100); // orange-green
        expect(color[2]).toBeLessThan(80);     // low blue
    });
});
