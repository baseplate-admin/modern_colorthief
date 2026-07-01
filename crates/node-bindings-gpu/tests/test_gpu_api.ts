import { describe, it, expect, beforeAll } from 'vitest';
import { tryImportGpu, createPixels, createGradientPixels, createTwoColorPixels, createCheckerboardPixels } from './test_helper.js';

// ---------------------------------------------------------------------------
// Lazy-import the native GPU module so we can skip all tests when it is
// unavailable (native binary not built or Vulkan not present).
// ---------------------------------------------------------------------------

let getPaletteGpu: ((pixels: Uint8Array, w: number, h: number, colorCount?: number, quality?: number) => number[][]);
let getColorGpu: ((pixels: Uint8Array, w: number, h: number, quality?: number) => number[]);
let gpuAvailable = false;

beforeAll(async () => {
    const mod = await tryImportGpu();
    if (mod && typeof mod.getPaletteGpu === 'function' && typeof mod.getColorGpu === 'function') {
        getPaletteGpu = mod.getPaletteGpu;
        getColorGpu = mod.getColorGpu;
        gpuAvailable = true;
    }
});

describe('GPU API surface', () => {
    it.skipIf(!gpuAvailable)('getPaletteGpu is a function', () => {
        expect(typeof getPaletteGpu).toBe('function');
    });

    it.skipIf(!gpuAvailable)('getColorGpu is a function', () => {
        expect(typeof getColorGpu).toBe('function');
    });
});

describe('GPU palette extraction', () => {
    it.skipIf(!gpuAvailable)('returns a palette for solid red pixels', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const palette = getPaletteGpu(pixels, 100, 100);
        expect(Array.isArray(palette)).toBe(true);
        expect(palette.length).toBeGreaterThan(0);
    });

    it.skipIf(!gpuAvailable)('returns a palette for solid green pixels', () => {
        const pixels = createPixels(100, 100, [0, 128, 0]);
        const palette = getPaletteGpu(pixels, 100, 100);
        expect(Array.isArray(palette)).toBe(true);
        expect(palette.length).toBeGreaterThan(0);
    });

    it.skipIf(!gpuAvailable)('returns a palette for solid blue pixels', () => {
        const pixels = createPixels(100, 100, [0, 0, 255]);
        const palette = getPaletteGpu(pixels, 100, 100);
        expect(Array.isArray(palette)).toBe(true);
        expect(palette.length).toBeGreaterThan(0);
    });

    it.skipIf(!gpuAvailable)('palette contains valid RGB values', () => {
        const pixels = createPixels(200, 200, [100, 150, 200]);
        const palette = getPaletteGpu(pixels, 200, 200);
        for (const color of palette) {
            expect(Array.isArray(color)).toBe(true);
            expect(color.length).toBe(3);
            for (const v of color) {
                expect(typeof v).toBe('number');
                expect(v).toBeGreaterThanOrEqual(0);
                expect(v).toBeLessThanOrEqual(255);
            }
        }
    });

    it.skipIf(!gpuAvailable)('dominant color of solid red is close to red', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const palette = getPaletteGpu(pixels, 100, 100);
        const dominant = palette[0];
        expect(dominant[0]).toBeGreaterThan(200);
        expect(dominant[1]).toBeLessThan(55);
        expect(dominant[2]).toBeLessThan(55);
    });

    it.skipIf(!gpuAvailable)('dominant color of solid green is close to green', () => {
        const pixels = createPixels(100, 100, [0, 255, 0]);
        const palette = getPaletteGpu(pixels, 100, 100);
        const dominant = palette[0];
        expect(dominant[0]).toBeLessThan(55);
        expect(dominant[1]).toBeGreaterThan(200);
        expect(dominant[2]).toBeLessThan(55);
    });

    it.skipIf(!gpuAvailable)('dominant color of solid blue is close to blue', () => {
        const pixels = createPixels(100, 100, [0, 0, 255]);
        const palette = getPaletteGpu(pixels, 100, 100);
        const dominant = palette[0];
        expect(dominant[0]).toBeLessThan(55);
        expect(dominant[1]).toBeLessThan(55);
        expect(dominant[2]).toBeGreaterThan(200);
    });

    it.skipIf(!gpuAvailable)('gradient image returns multiple distinct colors', () => {
        const pixels = createGradientPixels(200, 200, [255, 0, 0], [0, 0, 255]);
        const palette = getPaletteGpu(pixels, 200, 200, 10);
        expect(palette.length).toBeGreaterThan(1);
    });

    // -- Two-color detection --

    it.skipIf(!gpuAvailable)('detects both red and blue', () => {
        const pixels = createTwoColorPixels(50, 50, [255, 0, 0], [0, 0, 255]);
        const palette = getPaletteGpu(pixels, 10, 10, 5, 1);
        expect(palette.some(c => c[0] > 200 && c[1] < 55 && c[2] < 55)).toBe(true);
        expect(palette.some(c => c[0] < 55 && c[1] < 55 && c[2] > 200)).toBe(true);
    });

    // -- Checkerboard --

    it.skipIf(!gpuAvailable)('checkerboard returns palette', () => {
        const pixels = createCheckerboardPixels(10, 10, [200, 50, 50], [50, 50, 200]);
        const palette = getPaletteGpu(pixels, 10, 10, 5, 1);
        expect(palette.length).toBeGreaterThan(0);
    });

    // -- 1x1 single pixel --

    it.skipIf(!gpuAvailable)('single pixel returns that color', () => {
        const pixel = new Uint8Array([42, 100, 200, 255]);
        const palette = getPaletteGpu(pixel, 1, 1, 5, 1);
        expect(palette.length).toBeGreaterThan(0);
        expect(palette).toContainEqual([42, 100, 200]);
    });

    // -- Non-square wide --

    it.skipIf(!gpuAvailable)('wide image returns palette', () => {
        const pixels = createPixels(200, 50, [255, 0, 0]);
        const palette = getPaletteGpu(pixels, 200, 50);
        expect(palette.length).toBeGreaterThan(0);
    });

    // -- Non-square tall --

    it.skipIf(!gpuAvailable)('tall image returns palette', () => {
        const pixels = createPixels(50, 200, [255, 0, 0]);
        const palette = getPaletteGpu(pixels, 50, 200);
        expect(palette.length).toBeGreaterThan(0);
    });

    // -- Quality=0 clamped --

    it.skipIf(!gpuAvailable)('quality=0 clamped works', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const palette = getPaletteGpu(pixels, 100, 100, 5, 0);
        expect(palette.length).toBeGreaterThan(0);
    });

    // -- Quality=100 --

    it.skipIf(!gpuAvailable)('quality=100 works', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const palette = getPaletteGpu(pixels, 100, 100, 5, 100);
        expect(palette.length).toBeGreaterThan(0);
    });

    // -- Different images produce different palettes --

    it.skipIf(!gpuAvailable)('different images different palette', () => {
        const red = createPixels(100, 100, [255, 0, 0]);
        const blue = createPixels(100, 100, [0, 0, 255]);
        const p1 = getPaletteGpu(red, 100, 100);
        const p2 = getPaletteGpu(blue, 100, 100);
        expect(p1).not.toEqual(p2);
    });

    // -- Dominant color appears in palette --

    it.skipIf(!gpuAvailable)('dominant color in palette', () => {
        const pixels = createTwoColorPixels(50, 50, [255, 0, 0], [0, 0, 255]);
        const color = getColorGpu(pixels, 10, 10, 1);
        const palette = getPaletteGpu(pixels, 10, 10, 5, 1);
        expect(palette).toContainEqual(color);
    });
});

describe('GPU dominant color extraction', () => {
    it.skipIf(!gpuAvailable)('getColorGpu returns a 3-element array', () => {
        const pixels = createPixels(100, 100, [255, 128, 64]);
        const color = getColorGpu(pixels, 100, 100);
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
    });

    it.skipIf(!gpuAvailable)('getColorGpu returns valid RGB values', () => {
        const pixels = createPixels(100, 100, [50, 100, 150]);
        const color = getColorGpu(pixels, 100, 100);
        for (const v of color) {
            expect(typeof v).toBe('number');
            expect(v).toBeGreaterThanOrEqual(0);
            expect(v).toBeLessThanOrEqual(255);
        }
    });

    it.skipIf(!gpuAvailable)('dominant color matches solid color input', () => {
        const pixels = createPixels(100, 100, [200, 50, 100]);
        const color = getColorGpu(pixels, 100, 100);
        expect(color[0]).toBeGreaterThan(150);
        expect(color[1]).toBeLessThan(100);
        expect(color[2]).toBeGreaterThan(50);
    });

    // -- Solid green dominant --

    it.skipIf(!gpuAvailable)('solid green dominant color', () => {
        const pixels = createPixels(100, 100, [0, 255, 0]);
        const color = getColorGpu(pixels, 100, 100);
        expect(color[0]).toBeLessThan(55);
        expect(color[1]).toBeGreaterThan(200);
        expect(color[2]).toBeLessThan(55);
    });

    // -- Solid blue dominant --

    it.skipIf(!gpuAvailable)('solid blue dominant color', () => {
        const pixels = createPixels(100, 100, [0, 0, 255]);
        const color = getColorGpu(pixels, 100, 100);
        expect(color[0]).toBeLessThan(55);
        expect(color[1]).toBeLessThan(55);
        expect(color[2]).toBeGreaterThan(200);
    });

    // -- 1x1 single pixel --

    it.skipIf(!gpuAvailable)('single pixel returns exact color', () => {
        const pixel = new Uint8Array([200, 100, 50, 255]);
        const color = getColorGpu(pixel, 1, 1);
        expect(color).toEqual([200, 100, 50]);
    });

    // -- Different images different color --

    it.skipIf(!gpuAvailable)('different images different color', () => {
        const red = createPixels(100, 100, [255, 0, 0]);
        const green = createPixels(100, 100, [0, 255, 0]);
        const c1 = getColorGpu(red, 100, 100);
        const c2 = getColorGpu(green, 100, 100);
        expect(c1).not.toEqual(c2);
    });

    // -- Determinism --

    it.skipIf(!gpuAvailable)('deterministic dominant color', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const c1 = getColorGpu(pixels, 100, 100);
        const c2 = getColorGpu(pixels, 100, 100);
        expect(c1).toEqual(c2);
    });

    // -- GC stress --

    it.skipIf(!gpuAvailable)('gc stress palette', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        for (let i = 0; i < 50; i++) {
            const palette = getPaletteGpu(pixels, 100, 100);
            expect(palette.length).toBeGreaterThan(0);
        }
    });

    it.skipIf(!gpuAvailable)('gc stress color', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        for (let i = 0; i < 50; i++) {
            const color = getColorGpu(pixels, 100, 100);
            expect(color.length).toBe(3);
        }
    });

    it.skipIf(!gpuAvailable)('gc stress mixed', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        for (let i = 0; i < 25; i++) {
            const palette = getPaletteGpu(pixels, 100, 100);
            const color = getColorGpu(pixels, 100, 100);
            expect(palette.length).toBeGreaterThan(0);
            expect(color.length).toBe(3);
        }
    });
});
