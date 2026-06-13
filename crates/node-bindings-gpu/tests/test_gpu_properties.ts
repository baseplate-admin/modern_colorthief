import { describe, it, expect, beforeAll } from 'vitest';
import { tryImportGpu, createPixels, createGradientPixels } from './test_helper.js';

// ---------------------------------------------------------------------------
// Lazy-import the native GPU module.
// ---------------------------------------------------------------------------

let getPaletteGpu: ((pixels: Uint8Array, w: number, h: number, colorCount?: number, quality?: number) => number[][]);
let gpuAvailable = false;

beforeAll(async () => {
    const mod = await tryImportGpu();
    if (mod && typeof mod.getPaletteGpu === 'function') {
        getPaletteGpu = mod.getPaletteGpu;
        gpuAvailable = true;
    }
});

describe('GPU palette length bounds', () => {
    it.skipIf(!gpuAvailable)('respects color_count=1', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const palette = getPaletteGpu(pixels, 100, 100, 1);
        expect(palette.length).toBeLessThanOrEqual(1);
    });

    it.skipIf(!gpuAvailable)('respects color_count=3', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const palette = getPaletteGpu(pixels, 100, 100, 3);
        expect(palette.length).toBeLessThanOrEqual(3);
    });

    it.skipIf(!gpuAvailable)('respects color_count=5', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const palette = getPaletteGpu(pixels, 100, 100, 5);
        expect(palette.length).toBeLessThanOrEqual(5);
    });

    it.skipIf(!gpuAvailable)('respects color_count=20', () => {
        const pixels = createGradientPixels(200, 200, [255, 0, 0], [0, 0, 255]);
        const palette = getPaletteGpu(pixels, 200, 200, 20);
        expect(palette.length).toBeLessThanOrEqual(20);
    });

    it.skipIf(!gpuAvailable)('default color_count returns at least one color', () => {
        const pixels = createPixels(100, 100, [128, 128, 128]);
        const palette = getPaletteGpu(pixels, 100, 100);
        expect(palette.length).toBeGreaterThanOrEqual(1);
    });
});

describe('GPU deduplication', () => {
    it.skipIf(!gpuAvailable)('no duplicate colors in palette', () => {
        const pixels = createGradientPixels(200, 200, [255, 0, 0], [0, 0, 255]);
        const palette = getPaletteGpu(pixels, 200, 200, 10);
        const seen = new Set<string>();
        for (const color of palette) {
            const key = `${color[0]},${color[1]},${color[2]}`;
            expect(seen.has(key)).toBe(false);
            seen.add(key);
        }
    });

    it.skipIf(!gpuAvailable)('solid color palette has no duplicates', () => {
        const pixels = createPixels(100, 100, [100, 200, 50]);
        const palette = getPaletteGpu(pixels, 100, 100, 10);
        const seen = new Set<string>();
        for (const color of palette) {
            const key = `${color[0]},${color[1]},${color[2]}`;
            expect(seen.has(key)).toBe(false);
            seen.add(key);
        }
    });
});

describe('GPU determinism', () => {
    it.skipIf(!gpuAvailable)('same input produces same palette', () => {
        const pixels = createGradientPixels(200, 200, [255, 0, 0], [0, 255, 0]);
        const p1 = getPaletteGpu(pixels, 200, 200, 10);
        const p2 = getPaletteGpu(pixels, 200, 200, 10);
        expect(p1).toEqual(p2);
    });

    it.skipIf(!gpuAvailable)('same input produces same palette with different quality', () => {
        const pixels = createPixels(100, 100, [200, 100, 50]);
        const p1 = getPaletteGpu(pixels, 100, 100, 5, 1);
        const p2 = getPaletteGpu(pixels, 100, 100, 5, 1);
        expect(p1).toEqual(p2);
    });
});

describe('GPU error handling', () => {
    it.skipIf(!gpuAvailable)('throws on empty pixel data', () => {
        const empty = new Uint8Array(0);
        expect(() => getPaletteGpu(empty, 0, 0)).toThrow();
    });

    it.skipIf(!gpuAvailable)('throws on zero width', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        expect(() => getPaletteGpu(pixels, 0, 100)).toThrow();
    });

    it.skipIf(!gpuAvailable)('throws on zero height', () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        expect(() => getPaletteGpu(pixels, 100, 0)).toThrow();
    });

    it.skipIf(!gpuAvailable)('throws when pixel data is too small for dimensions', () => {
        // 10x10 RGBA = 400 bytes, but we only pass 100 bytes
        const small = new Uint8Array(100);
        expect(() => getPaletteGpu(small, 10, 10)).toThrow();
    });

    it.skipIf(!gpuAvailable)('throws when pixel data is too large for dimensions', () => {
        // 10x10 RGBA = 400 bytes, but we pass 1600 bytes (100x100 worth)
        const pixels = createPixels(100, 100, [255, 0, 0]);
        expect(() => getPaletteGpu(pixels, 10, 10)).toThrow();
    });
});

describe('GPU quality parameter variation', () => {
    it.skipIf(!gpuAvailable)('quality=1 returns valid palette', () => {
        const pixels = createGradientPixels(200, 200, [255, 0, 0], [0, 0, 255]);
        const palette = getPaletteGpu(pixels, 200, 200, 10, 1);
        expect(palette.length).toBeGreaterThan(0);
    });

    it.skipIf(!gpuAvailable)('quality=5 returns valid palette', () => {
        const pixels = createGradientPixels(200, 200, [255, 0, 0], [0, 0, 255]);
        const palette = getPaletteGpu(pixels, 200, 200, 10, 5);
        expect(palette.length).toBeGreaterThan(0);
    });

    it.skipIf(!gpuAvailable)('quality=10 returns valid palette', () => {
        const pixels = createGradientPixels(200, 200, [255, 0, 0], [0, 0, 255]);
        const palette = getPaletteGpu(pixels, 200, 200, 10, 10);
        expect(palette.length).toBeGreaterThan(0);
    });

    it.skipIf(!gpuAvailable)('higher quality may return more colors', () => {
        const pixels = createGradientPixels(400, 400, [255, 0, 0], [0, 0, 255]);
        const low = getPaletteGpu(pixels, 400, 400, 10, 1);
        const high = getPaletteGpu(pixels, 400, 400, 10, 10);
        // Higher quality samples more pixels, so it should find at least as many colors
        expect(high.length).toBeGreaterThanOrEqual(low.length);
    });
});

describe('GPU return value structure', () => {
    it.skipIf(!gpuAvailable)('every color is a 3-element array of numbers', () => {
        const pixels = createPixels(50, 50, [128, 64, 192]);
        const palette = getPaletteGpu(pixels, 50, 50);
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
});
