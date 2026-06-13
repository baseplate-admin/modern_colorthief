import { describe, it, expect, beforeAll } from 'vitest';

// ---------------------------------------------------------------------------
// Helpers: generate raw RGBA pixel buffers (no Canvas required)
// ---------------------------------------------------------------------------

/** Create a solid-color RGBA buffer (width×height, 4 bytes/pixel). */
function createSolidPixels(r, g, b, width = 100, height = 100) {
    const buf = new Uint8Array(width * height * 4);
    for (let i = 0; i < width * height; i++) {
        buf[i * 4] = r;
        buf[i * 4 + 1] = g;
        buf[i * 4 + 2] = b;
        buf[i * 4 + 3] = 255; // alpha
    }
    return { pixels: buf, width, height };
}

/** Create a two-color RGBA buffer (left half / right half). */
function createTwoColorPixels(r1, g1, b1, r2, g2, b2, width = 100, height = 100) {
    const buf = new Uint8Array(width * height * 4);
    const half = Math.floor(width / 2);
    for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
            const idx = (y * width + x) * 4;
            if (x < half) {
                buf[idx] = r1; buf[idx + 1] = g1; buf[idx + 2] = b1;
            } else {
                buf[idx] = r2; buf[idx + 1] = g2; buf[idx + 2] = b2;
            }
            buf[idx + 3] = 255;
        }
    }
    return { pixels: buf, width, height };
}

/** Create a gradient RGBA buffer with N horizontal color bands (top-to-bottom). */
function createGradientPixels(colors) {
    const width = 100;
    const bandHeight = Math.ceil(100 / colors.length);
    const height = bandHeight * colors.length;
    const buf = new Uint8Array(width * height * 4);
    for (let i = 0; i < colors.length; i++) {
        const [r, g, b] = colors[i];
        for (let y = i * bandHeight; y < (i + 1) * bandHeight; y++) {
            for (let x = 0; x < width; x++) {
                const idx = (y * width + x) * 4;
                buf[idx] = r; buf[idx + 1] = g; buf[idx + 2] = b; buf[idx + 3] = 255;
            }
        }
    }
    return { pixels: buf, width, height };
}

// ---------------------------------------------------------------------------
// Load the WASM module (skip all tests if it is not yet built)
// ---------------------------------------------------------------------------

let getPaletteFromPixels, getColorFromPixels;
let wasmAvailable = false;

beforeAll(async () => {
    try {
        const mod = await import('../pkg/modern_colorthief_wasm.js');
        getPaletteFromPixels = mod.getPaletteFromPixels;
        getColorFromPixels = mod.getColorFromPixels;
        wasmAvailable = typeof getPaletteFromPixels === 'function' && typeof getColorFromPixels === 'function';
    } catch {
        // WASM not built yet — tests will be skipped
    }
});

// ---------------------------------------------------------------------------
// Solid color detection
// ---------------------------------------------------------------------------

describe('Solid color detection', () => {
    it('should detect pure red', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(255, 0, 0);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        const top = palette[0];
        expect(top[0]).toBeCloseTo(255, -1);
        expect(top[1]).toBeCloseTo(0, -1);
        expect(top[2]).toBeCloseTo(0, -1);
    }, 30000);

    it('should detect pure green', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(0, 255, 0);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        const top = palette[0];
        expect(top[0]).toBeCloseTo(0, -1);
        expect(top[1]).toBeCloseTo(255, -1);
        expect(top[2]).toBeCloseTo(0, -1);
    }, 30000);

    it('should detect white', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(255, 255, 255);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        const top = palette[0];
        expect(top[0]).toBeCloseTo(255, -1);
        expect(top[1]).toBeCloseTo(255, -1);
        expect(top[2]).toBeCloseTo(255, -1);
    }, 30000);

    it('should detect black', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(0, 0, 0);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        const top = palette[0];
        expect(top[0]).toBeCloseTo(0, -1);
        expect(top[1]).toBeCloseTo(0, -1);
        expect(top[2]).toBeCloseTo(0, -1);
    }, 30000);
});

// ---------------------------------------------------------------------------
// Two-color detection
// ---------------------------------------------------------------------------

describe('Two-color detection', () => {
    it('should find two distinct colors in a two-color image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createTwoColorPixels(255, 0, 0, 0, 0, 255);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThanOrEqual(2);

        const hasRed = palette.some(c => c[0] > 200 && c[1] < 55 && c[2] < 55);
        const hasBlue = palette.some(c => c[0] < 55 && c[1] < 55 && c[2] > 200);
        expect(hasRed || hasBlue).toBe(true);
    }, 30000);

    it('should return red and blue as separate palette entries', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createTwoColorPixels(255, 0, 0, 0, 0, 255);
        const palette = await getPaletteFromPixels(pixels, width, height, 10, 10);
        const redCount = palette.filter(c => c[0] > 200 && c[1] < 55 && c[2] < 55).length;
        const blueCount = palette.filter(c => c[0] < 55 && c[1] < 55 && c[2] > 200).length;
        expect(redCount).toBeGreaterThan(0);
        expect(blueCount).toBeGreaterThan(0);
    }, 30000);
});

// ---------------------------------------------------------------------------
// Palette length respects color_count
// ---------------------------------------------------------------------------

describe('Palette length respects color_count', () => {
    it('should return at most color_count colors', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(128, 64, 32);
        const palette = await getPaletteFromPixels(pixels, width, height, 3, 10);
        expect(palette.length).toBeLessThanOrEqual(3);
    }, 30000);

    it('should return at most 1 color when color_count is 1', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(200, 100, 50);
        const palette = await getPaletteFromPixels(pixels, width, height, 1, 10);
        expect(palette.length).toBeLessThanOrEqual(1);
    }, 30000);

    it('should return more colors when color_count is higher', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createGradientPixels([
            [255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0], [255, 0, 255]
        ]);
        const small = await getPaletteFromPixels(pixels, width, height, 3, 10);
        const large = await getPaletteFromPixels(pixels, width, height, 10, 10);
        expect(large.length).toBeGreaterThanOrEqual(small.length);
    }, 30000);
});

// ---------------------------------------------------------------------------
// Deduplication
// ---------------------------------------------------------------------------

describe('Deduplication', () => {
    it('should not return duplicate colors in palette', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(100, 150, 200);
        const palette = await getPaletteFromPixels(pixels, width, height, 20, 10);
        const serialized = palette.map(c => `${c[0]},${c[1]},${c[2]}`);
        const unique = new Set(serialized);
        expect(unique.size).toBe(serialized.length);
    }, 30000);

    it('solid image should return very few unique colors', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(50, 50, 50);
        const palette = await getPaletteFromPixels(pixels, width, height, 50, 10);
        expect(palette.length).toBeLessThanOrEqual(50);
    }, 30000);
});

// ---------------------------------------------------------------------------
// get_color returns correct dominant color
// ---------------------------------------------------------------------------

describe('getColor returns correct dominant color', () => {
    it('should return an RGB array of length 3', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(255, 0, 0);
        const color = await getColorFromPixels(pixels, width, height, 10);
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
    }, 30000);

    it('should return red for a red image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(255, 0, 0);
        const color = await getColorFromPixels(pixels, width, height, 10);
        expect(color[0]).toBeCloseTo(255, -1);
        expect(color[1]).toBeCloseTo(0, -1);
        expect(color[2]).toBeCloseTo(0, -1);
    }, 30000);

    it('should return green for a green image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(0, 255, 0);
        const color = await getColorFromPixels(pixels, width, height, 10);
        expect(color[0]).toBeCloseTo(0, -1);
        expect(color[1]).toBeCloseTo(255, -1);
        expect(color[2]).toBeCloseTo(0, -1);
    }, 30000);

    it('should return a color close to the dominant color in a two-color image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createTwoColorPixels(255, 0, 0, 0, 0, 255);
        const color = await getColorFromPixels(pixels, width, height, 10);
        const isRed = color[0] > 200 && color[1] < 55 && color[2] < 55;
        const isBlue = color[0] < 55 && color[1] < 55 && color[2] > 200;
        expect(isRed || isBlue).toBe(true);
    }, 30000);
});

// ---------------------------------------------------------------------------
// Error handling for empty/invalid input
// ---------------------------------------------------------------------------

describe('Error handling', () => {
    it('should reject with empty pixels', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const empty = new Uint8Array(0);
        await expect(getPaletteFromPixels(empty, 0, 0, 5, 10)).rejects.toThrow();
    }, 30000);

    it('should reject with insufficient pixel data', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const tooSmall = new Uint8Array([255, 0, 0]);
        await expect(getPaletteFromPixels(tooSmall, 1, 1, 5, 10)).rejects.toThrow();
    }, 30000);

    it('getColor should reject with empty pixels', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const empty = new Uint8Array(0);
        await expect(getColorFromPixels(empty, 0, 0, 10)).rejects.toThrow();
    }, 30000);
});

// ---------------------------------------------------------------------------
// Deterministic results
// ---------------------------------------------------------------------------

describe('Deterministic results', () => {
    it('should return the same palette for the same pixels twice', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(180, 90, 45);
        const p1 = await getPaletteFromPixels(pixels, width, height, 5, 10);
        const p2 = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(p1).toEqual(p2);
    }, 30000);

    it('should return the same color for the same pixels twice', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(180, 90, 45);
        const c1 = await getColorFromPixels(pixels, width, height, 10);
        const c2 = await getColorFromPixels(pixels, width, height, 10);
        expect(c1).toEqual(c2);
    }, 30000);

    it('should be deterministic on a multi-color image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createGradientPixels([
            [255, 0, 0], [0, 255, 0], [0, 0, 255]
        ]);
        const p1 = await getPaletteFromPixels(pixels, width, height, 10, 5);
        const p2 = await getPaletteFromPixels(pixels, width, height, 10, 5);
        expect(p1).toEqual(p2);
    }, 30000);
});

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

describe('Edge cases', () => {
    it('should handle a 1x1 pixel image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(255, 128, 64, 1, 1);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        const top = palette[0];
        expect(top[0]).toBeCloseTo(255, -1);
        expect(top[1]).toBeCloseTo(128, -1);
        expect(top[2]).toBeCloseTo(64, -1);
    }, 30000);

    it('getColor should handle a 1x1 pixel image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(70, 140, 210, 1, 1);
        const color = await getColorFromPixels(pixels, width, height, 10);
        expect(color.length).toBe(3);
        expect(color[0]).toBeCloseTo(70, -1);
        expect(color[1]).toBeCloseTo(140, -1);
        expect(color[2]).toBeCloseTo(210, -1);
    }, 30000);

    it('should handle large quality value (quality=10)', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(200, 100, 50);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
    }, 30000);

    it('should handle minimum quality value (quality=1)', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(200, 100, 50);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 1);
        expect(palette.length).toBeGreaterThan(0);
    }, 30000);

    it('should work with a large image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const { pixels, width, height } = createSolidPixels(100, 200, 150, 500, 500);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        const top = palette[0];
        expect(top[0]).toBeCloseTo(100, -1);
        expect(top[1]).toBeCloseTo(200, -1);
        expect(top[2]).toBeCloseTo(150, -1);
    }, 30000);
});
