import { describe, it, expect, beforeAll } from 'vitest';
import { tryImportGpu, createPixels, createGradientPixels } from './test_helper.js';

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
        // Allow some tolerance for quantization
        expect(dominant[0]).toBeGreaterThan(200); // R is high
        expect(dominant[1]).toBeLessThan(55);     // G is low
        expect(dominant[2]).toBeLessThan(55);     // B is low
    });

    it.skipIf(!gpuAvailable)('dominant color of solid green is close to green', () => {
        const pixels = createPixels(100, 100, [0, 255, 0]);
        const palette = getPaletteGpu(pixels, 100, 100);
        const dominant = palette[0];
        expect(dominant[0]).toBeLessThan(55);     // R is low
        expect(dominant[1]).toBeGreaterThan(200); // G is high
        expect(dominant[2]).toBeLessThan(55);     // B is low
    });

    it.skipIf(!gpuAvailable)('dominant color of solid blue is close to blue', () => {
        const pixels = createPixels(100, 100, [0, 0, 255]);
        const palette = getPaletteGpu(pixels, 100, 100);
        const dominant = palette[0];
        expect(dominant[0]).toBeLessThan(55);     // R is low
        expect(dominant[1]).toBeLessThan(55);     // G is low
        expect(dominant[2]).toBeGreaterThan(200); // B is high
    });

    it.skipIf(!gpuAvailable)('gradient image returns multiple distinct colors', () => {
        const pixels = createGradientPixels(200, 200, [255, 0, 0], [0, 0, 255]);
        const palette = getPaletteGpu(pixels, 200, 200, 10);
        expect(palette.length).toBeGreaterThan(1);
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
        // Allow tolerance for quantization
        expect(color[0]).toBeGreaterThan(150);
        expect(color[1]).toBeLessThan(100);
        expect(color[2]).toBeGreaterThan(50);
    });
});
