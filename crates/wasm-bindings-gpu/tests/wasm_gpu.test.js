import { describe, it, expect, beforeAll } from 'vitest';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Create solid-color RGBA pixel data. */
function createPixels(w, h, [r, g, b]) {
    const pixels = new Uint8Array(w * h * 4);
    for (let i = 0; i < w * h; i++) {
        pixels[i * 4] = r;
        pixels[i * 4 + 1] = g;
        pixels[i * 4 + 2] = b;
        pixels[i * 4 + 3] = 255;
    }
    return pixels;
}

// ---------------------------------------------------------------------------
// Load the WASM GPU module
// ---------------------------------------------------------------------------

let getPaletteGpu, getColorGpu;
let gpuAvailable = false;

beforeAll(async () => {
    try {
        const mod = await import('../pkg/colorthief_wasm_gpu.js');
        getPaletteGpu = mod.getPaletteGpu;
        getColorGpu = mod.getColorGpu;
        gpuAvailable =
            typeof getPaletteGpu === 'function' &&
            typeof getColorGpu === 'function';
    } catch {
        // WASM GPU not built yet
    }
});

// ---------------------------------------------------------------------------
// GPU palette extraction
// ---------------------------------------------------------------------------

describe('GPU getPaletteGpu', () => {
    it.skipIf(!gpuAvailable)('returns a palette for solid red', async () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const palette = await getPaletteGpu(pixels, 100, 100, 10, 10);
        expect(Array.isArray(palette)).toBe(true);
        expect(palette.length).toBeGreaterThan(0);
    });

    it.skipIf(!gpuAvailable)('returns valid RGB values', async () => {
        const pixels = createPixels(100, 100, [100, 150, 200]);
        const palette = await getPaletteGpu(pixels, 100, 100, 10, 10);
        for (const color of palette) {
            expect(color.length).toBe(3);
            for (const v of color) {
                expect(v).toBeGreaterThanOrEqual(0);
                expect(v).toBeLessThanOrEqual(255);
            }
        }
    });

    it.skipIf(!gpuAvailable)('solid red dominant color is close to red', async () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const palette = await getPaletteGpu(pixels, 100, 100, 10, 10);
        const [r, g, b] = palette[0];
        expect(r).toBeGreaterThan(200);
        expect(g).toBeLessThan(55);
        expect(b).toBeLessThan(55);
    });

    it.skipIf(!gpuAvailable)('respects color_count', async () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const palette = await getPaletteGpu(pixels, 100, 100, 5, 10);
        expect(palette.length).toBeLessThanOrEqual(5);
    });

    it.skipIf(!gpuAvailable)('no duplicate colors', async () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const palette = await getPaletteGpu(pixels, 100, 100, 10, 10);
        const keys = palette.map(c => `${c[0]},${c[1]},${c[2]}`);
        expect(new Set(keys).size).toBe(keys.length);
    });

    it.skipIf(!gpuAvailable)('deterministic results', async () => {
        const pixels = createPixels(100, 100, [200, 100, 50]);
        const p1 = await getPaletteGpu(pixels, 100, 100, 10, 10);
        const p2 = await getPaletteGpu(pixels, 100, 100, 10, 10);
        expect(p1).toEqual(p2);
    });

    it.skipIf(!gpuAvailable)('quality parameter variation', async () => {
        const pixels = createPixels(200, 200, [100, 150, 200]);
        for (const q of [1, 5, 10]) {
            const palette = await getPaletteGpu(pixels, 200, 200, 10, q);
            expect(palette.length).toBeGreaterThan(0);
        }
    });

    it.skipIf(!gpuAvailable)('rejects on empty pixels', async () => {
        const empty = new Uint8Array(0);
        await expect(getPaletteGpu(empty, 0, 0, 10, 10)).rejects.toThrow();
    });

    it.skipIf(!gpuAvailable)('rejects on zero width', async () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        await expect(getPaletteGpu(pixels, 0, 100, 10, 10)).rejects.toThrow();
    });

    it.skipIf(!gpuAvailable)('rejects on mismatched size', async () => {
        const small = new Uint8Array(100);
        await expect(getPaletteGpu(small, 10, 10, 10, 10)).rejects.toThrow();
    });
});

// ---------------------------------------------------------------------------
// GPU dominant color extraction
// ---------------------------------------------------------------------------

describe('GPU getColorGpu', () => {
    it.skipIf(!gpuAvailable)('returns a 3-element array', async () => {
        const pixels = createPixels(100, 100, [255, 128, 64]);
        const color = await getColorGpu(pixels, 100, 100, 10);
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
    });

    it.skipIf(!gpuAvailable)('valid RGB values', async () => {
        const pixels = createPixels(100, 100, [50, 100, 150]);
        const color = await getColorGpu(pixels, 100, 100, 10);
        for (const v of color) {
            expect(v).toBeGreaterThanOrEqual(0);
            expect(v).toBeLessThanOrEqual(255);
        }
    });

    it.skipIf(!gpuAvailable)('solid red color is close to red', async () => {
        const pixels = createPixels(100, 100, [255, 0, 0]);
        const [r, g, b] = await getColorGpu(pixels, 100, 100, 10);
        expect(r).toBeGreaterThan(200);
        expect(g).toBeLessThan(55);
        expect(b).toBeLessThan(55);
    });

    it.skipIf(!gpuAvailable)('deterministic results', async () => {
        const pixels = createPixels(100, 100, [200, 100, 50]);
        const c1 = await getColorGpu(pixels, 100, 100, 10);
        const c2 = await getColorGpu(pixels, 100, 100, 10);
        expect(c1).toEqual(c2);
    });

    it.skipIf(!gpuAvailable)('rejects on empty pixels', async () => {
        const empty = new Uint8Array(0);
        await expect(getColorGpu(empty, 0, 0, 10)).rejects.toThrow();
    });
});
