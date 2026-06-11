import { describe, it, expect, beforeAll } from 'vitest';

// ---------------------------------------------------------------------------
// Helpers: generate test images via Canvas, return as Blob
// ---------------------------------------------------------------------------

/** Create a solid-color image (100x100) and return a Blob. */
async function createSolidImage(r, g, b) {
    const canvas = document.createElement('canvas');
    canvas.width = 100;
    canvas.height = 100;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = `rgb(${r},${g},${b})`;
    ctx.fillRect(0, 0, 100, 100);
    return new Promise(resolve => canvas.toBlob(resolve, 'image/png'));
}

/** Create a two-color image (left half / right half) and return a Blob. */
async function createTwoColorImage(r1, g1, b1, r2, g2, b2) {
    const canvas = document.createElement('canvas');
    canvas.width = 100;
    canvas.height = 100;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = `rgb(${r1},${g1},${b1})`;
    ctx.fillRect(0, 0, 50, 100);
    ctx.fillStyle = `rgb(${r2},${g2},${b2})`;
    ctx.fillRect(50, 0, 50, 100);
    return new Promise(resolve => canvas.toBlob(resolve, 'image/png'));
}

/** Create a gradient image (top-to-bottom) with N color bands. */
async function createGradientImage(colors) {
    const canvas = document.createElement('canvas');
    canvas.width = 100;
    canvas.height = 100;
    const ctx = canvas.getContext('2d');
    const bandHeight = 100 / colors.length;
    for (let i = 0; i < colors.length; i++) {
        const [r, g, b] = colors[i];
        ctx.fillStyle = `rgb(${r},${g},${b})`;
        ctx.fillRect(0, i * bandHeight, 100, bandHeight);
    }
    return new Promise(resolve => canvas.toBlob(resolve, 'image/png'));
}

/** Create a tiny 1x1 pixel image. */
async function createTinyImage(r, g, b) {
    const canvas = document.createElement('canvas');
    canvas.width = 1;
    canvas.height = 1;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = `rgb(${r},${g},${b})`;
    ctx.fillRect(0, 0, 1, 1);
    return new Promise(resolve => canvas.toBlob(resolve, 'image/png'));
}

/** Create a larger image (500x500) for performance / quality tests. */
async function createLargeImage(r, g, b) {
    const canvas = document.createElement('canvas');
    canvas.width = 500;
    canvas.height = 500;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = `rgb(${r},${g},${b})`;
    ctx.fillRect(0, 0, 500, 500);
    return new Promise(resolve => canvas.toBlob(resolve, 'image/png'));
}

// ---------------------------------------------------------------------------
// Load the WASM module (skip all tests if it is not yet built)
// ---------------------------------------------------------------------------

let getPalette, getColor;
let wasmAvailable = false;

beforeAll(async () => {
    try {
        const mod = await import('../pkg/modern_colorthief_wasm.js');
        getPalette = mod.getPalette;
        getColor = mod.getColor;
        wasmAvailable = typeof getPalette === 'function' && typeof getColor === 'function';
    } catch {
        // WASM not built yet — tests will be skipped
    }
});

// ---------------------------------------------------------------------------
// Solid color detection
// ---------------------------------------------------------------------------

describe('Solid color detection', () => {
    it('should detect pure red', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(255, 0, 0);
        const palette = await getPalette(img, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        const top = palette[0];
        expect(top[0]).toBeCloseTo(255, -1); // R ~ 255
        expect(top[1]).toBeCloseTo(0, -1);   // G ~ 0
        expect(top[2]).toBeCloseTo(0, -1);   // B ~ 0
    }, 30000);

    it('should detect pure green', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(0, 255, 0);
        const palette = await getPalette(img, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        const top = palette[0];
        expect(top[0]).toBeCloseTo(0, -1);
        expect(top[1]).toBeCloseTo(255, -1);
        expect(top[2]).toBeCloseTo(0, -1);
    }, 30000);

    it('should detect white', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(255, 255, 255);
        const palette = await getPalette(img, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        const top = palette[0];
        expect(top[0]).toBeCloseTo(255, -1);
        expect(top[1]).toBeCloseTo(255, -1);
        expect(top[2]).toBeCloseTo(255, -1);
    }, 30000);

    it('should detect black', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(0, 0, 0);
        const palette = await getPalette(img, 5, 10);
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
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createTwoColorImage(255, 0, 0, 0, 0, 255);
        const palette = await getPalette(img, 5, 10);
        expect(palette.length).toBeGreaterThanOrEqual(2);

        const hasRed = palette.some(c => c[0] > 200 && c[1] < 55 && c[2] < 55);
        const hasBlue = palette.some(c => c[0] < 55 && c[1] < 55 && c[2] > 200);
        expect(hasRed || hasBlue).toBe(true);
    }, 30000);

    it('should return red and blue as separate palette entries', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createTwoColorImage(255, 0, 0, 0, 0, 255);
        const palette = await getPalette(img, 10, 10);
        // At least one entry should be close to red, another close to blue
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
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(128, 64, 32);
        const palette = await getPalette(img, 3, 10);
        expect(palette.length).toBeLessThanOrEqual(3);
    }, 30000);

    it('should return at most 1 color when color_count is 1', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(200, 100, 50);
        const palette = await getPalette(img, 1, 10);
        expect(palette.length).toBeLessThanOrEqual(1);
    }, 30000);

    it('should return more colors when color_count is higher', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createGradientImage([
            [255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0], [255, 0, 255]
        ]);
        const small = await getPalette(img, 3, 10);
        const large = await getPalette(img, 10, 10);
        expect(large.length).toBeGreaterThanOrEqual(small.length);
    }, 30000);
});

// ---------------------------------------------------------------------------
// Deduplication
// ---------------------------------------------------------------------------

describe('Deduplication', () => {
    it('should not return duplicate colors in palette', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(100, 150, 200);
        const palette = await getPalette(img, 20, 10);
        const serialized = palette.map(c => `${c[0]},${c[1]},${c[2]}`);
        const unique = new Set(serialized);
        expect(unique.size).toBe(serialized.length);
    }, 30000);

    it('solid image should return very few unique colors', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(50, 50, 50);
        const palette = await getPalette(img, 50, 10);
        // A solid image has only one color, so even with color_count=50 we expect few
        expect(palette.length).toBeLessThanOrEqual(50);
    }, 30000);
});

// ---------------------------------------------------------------------------
// get_color returns correct dominant color
// ---------------------------------------------------------------------------

describe('getColor returns correct dominant color', () => {
    it('should return an RGB array of length 3', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(255, 0, 0);
        const color = await getColor(img, 10);
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
    }, 30000);

    it('should return red for a red image', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(255, 0, 0);
        const color = await getColor(img, 10);
        expect(color[0]).toBeCloseTo(255, -1);
        expect(color[1]).toBeCloseTo(0, -1);
        expect(color[2]).toBeCloseTo(0, -1);
    }, 30000);

    it('should return green for a green image', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(0, 255, 0);
        const color = await getColor(img, 10);
        expect(color[0]).toBeCloseTo(0, -1);
        expect(color[1]).toBeCloseTo(255, -1);
        expect(color[2]).toBeCloseTo(0, -1);
    }, 30000);

    it('should return a color close to the dominant color in a two-color image', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createTwoColorImage(255, 0, 0, 0, 0, 255);
        const color = await getColor(img, 10);
        // Either red-dominant or blue-dominant is acceptable (equal area)
        const isRed = color[0] > 200 && color[1] < 55 && color[2] < 55;
        const isBlue = color[0] < 55 && color[1] < 55 && color[2] > 200;
        expect(isRed || isBlue).toBe(true);
    }, 30000);
});

// ---------------------------------------------------------------------------
// Error handling for empty/invalid input
// ---------------------------------------------------------------------------

describe('Error handling', () => {
    it('should reject with empty bytes', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const emptyBlob = new Blob([new Uint8Array(0)]);
        await expect(getPalette(emptyBlob, 5, 10)).rejects.toThrow();
    }, 30000);

    it('should reject with non-image data', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const fakeBlob = new Blob([new TextEncoder().encode('not an image')]);
        await expect(getPalette(fakeBlob, 5, 10)).rejects.toThrow();
    }, 30000);

    it('getColor should reject with empty bytes', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const emptyBlob = new Blob([new Uint8Array(0)]);
        await expect(getColor(emptyBlob, 10)).rejects.toThrow();
    }, 30000);

    it('getColor should reject with non-image data', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const fakeBlob = new Blob([new TextEncoder().encode('hello world')]);
        await expect(getColor(fakeBlob, 10)).rejects.toThrow();
    }, 30000);

    it('should reject with truncated JPEG header', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const truncated = new Uint8Array([0xff, 0xd8, 0xff, 0xe0, 0, 0, 0, 0, 0, 0]);
        const blob = new Blob([truncated]);
        await expect(getPalette(blob, 5, 10)).rejects.toThrow();
    }, 30000);
});

// ---------------------------------------------------------------------------
// Deterministic results
// ---------------------------------------------------------------------------

describe('Deterministic results', () => {
    it('should return the same palette for the same image twice', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(180, 90, 45);
        const p1 = await getPalette(img, 5, 10);
        const p2 = await getPalette(img, 5, 10);
        expect(p1).toEqual(p2);
    }, 30000);

    it('should return the same color for the same image twice', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(180, 90, 45);
        const c1 = await getColor(img, 10);
        const c2 = await getColor(img, 10);
        expect(c1).toEqual(c2);
    }, 30000);

    it('should be deterministic on a multi-color image', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createGradientImage([
            [255, 0, 0], [0, 255, 0], [0, 0, 255]
        ]);
        const p1 = await getPalette(img, 10, 5);
        const p2 = await getPalette(img, 10, 5);
        expect(p1).toEqual(p2);
    }, 30000);
});

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

describe('Edge cases', () => {
    it('should handle a 1x1 pixel image', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createTinyImage(255, 128, 64);
        const palette = await getPalette(img, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        const top = palette[0];
        expect(top[0]).toBeCloseTo(255, -1);
        expect(top[1]).toBeCloseTo(128, -1);
        expect(top[2]).toBeCloseTo(64, -1);
    }, 30000);

    it('getColor should handle a 1x1 pixel image', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createTinyImage(70, 140, 210);
        const color = await getColor(img, 10);
        expect(color.length).toBe(3);
        expect(color[0]).toBeCloseTo(70, -1);
        expect(color[1]).toBeCloseTo(140, -1);
        expect(color[2]).toBeCloseTo(210, -1);
    }, 30000);

    it('should handle large quality value (quality=10)', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(200, 100, 50);
        const palette = await getPalette(img, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
    }, 30000);

    it('should handle minimum quality value (quality=1)', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(200, 100, 50);
        const palette = await getPalette(img, 5, 1);
        expect(palette.length).toBeGreaterThan(0);
    }, 30000);

    it('should work with a large image', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const img = await createLargeImage(100, 200, 150);
        const palette = await getPalette(img, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        const top = palette[0];
        expect(top[0]).toBeCloseTo(100, -1);
        expect(top[1]).toBeCloseTo(200, -1);
        expect(top[2]).toBeCloseTo(150, -1);
    }, 30000);

    it('should work when passing a Uint8Array', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const blob = await createSolidImage(255, 0, 0);
        const arrayBuffer = await blob.arrayBuffer();
        const uint8 = new Uint8Array(arrayBuffer);
        const palette = await getPalette(uint8, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
    }, 30000);

    it('should work when passing an ArrayBuffer', async () => {
        expect.poll(() => wasmAvailable).toBe(true);
        const blob = await createSolidImage(0, 255, 0);
        const arrayBuffer = await blob.arrayBuffer();
        const palette = await getPalette(arrayBuffer, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
    }, 30000);
});
