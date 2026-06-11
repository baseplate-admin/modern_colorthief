import { describe, it, expect, beforeAll } from 'vitest';

// ---------------------------------------------------------------------------
// Load the WASM module (browser API with Canvas-based image decoding)
// ---------------------------------------------------------------------------

let getPalette, getColor, decodeImage, getPaletteFromPixels, getColorFromPixels;
let wasmAvailable = false;

beforeAll(async () => {
    try {
        const mod = await import('../pkg/modern_colorthief_wasm.js');
        await mod.default();
        getPalette = mod.getPalette;
        getColor = mod.getColor;
        decodeImage = mod.decodeImage;
        getPaletteFromPixels = mod.getPaletteFromPixels;
        getColorFromPixels = mod.getColorFromPixels;
        wasmAvailable =
            typeof getPalette === 'function' &&
            typeof getColor === 'function' &&
            typeof decodeImage === 'function' &&
            typeof getPaletteFromPixels === 'function' &&
            typeof getColorFromPixels === 'function';
    } catch { /* WASM not built yet */ }
});

function ready() { return expect.poll(() => wasmAvailable).toBe(true); }

// ---------------------------------------------------------------------------
// Helpers: create test images via Canvas
// ---------------------------------------------------------------------------

async function solidBlob(r, g, b, w = 100, h = 100) {
    const c = document.createElement('canvas');
    c.width = w; c.height = h;
    c.getContext('2d').fillStyle = `rgb(${r},${g},${b})`;
    c.getContext('2d').fillRect(0, 0, w, h);
    return new Promise(res => c.toBlob(res, 'image/png'));
}

async function twoColorBlob(r1, g1, b1, r2, g2, b2, w = 100, h = 100) {
    const c = document.createElement('canvas');
    c.width = w; c.height = h;
    const ctx = c.getContext('2d');
    ctx.fillStyle = `rgb(${r1},${g1},${b1})`;
    ctx.fillRect(0, 0, w / 2, h);
    ctx.fillStyle = `rgb(${r2},${g2},${b2})`;
    ctx.fillRect(w / 2, 0, w / 2, h);
    return new Promise(res => c.toBlob(res, 'image/png'));
}

function solidPixels(r, g, b, w = 100, h = 100) {
    const buf = new Uint8Array(w * h * 4);
    for (let i = 0; i < w * h; i++) {
        buf[i * 4] = r; buf[i * 4 + 1] = g; buf[i * 4 + 2] = b; buf[i * 4 + 3] = 255;
    }
    return { pixels: buf, width: w, height: h };
}

// ---------------------------------------------------------------------------
// API surface
// ---------------------------------------------------------------------------

describe('API surface', () => {
    it('exports getPalette', async () => { await ready(); expect(typeof getPalette).toBe('function'); });
    it('exports getColor', async () => { await ready(); expect(typeof getColor).toBe('function'); });
    it('exports decodeImage', async () => { await ready(); expect(typeof decodeImage).toBe('function'); });
    it('exports getPaletteFromPixels', async () => { await ready(); expect(typeof getPaletteFromPixels).toBe('function'); });
    it('exports getColorFromPixels', async () => { await ready(); expect(typeof getColorFromPixels).toBe('function'); });
});

// ---------------------------------------------------------------------------
// getPalette — via Canvas-decoded image
// ---------------------------------------------------------------------------

describe('getPalette (browser)', () => {
    it('returns a non-empty palette for solid red', async () => {
        await ready();
        const blob = await solidBlob(255, 0, 0);
        const palette = await getPalette(blob, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        expect(palette[0][0]).toBeGreaterThan(200);
        expect(palette[0][1]).toBeLessThan(55);
        expect(palette[0][2]).toBeLessThan(55);
    });

    it('returns valid RGB values (0-255)', async () => {
        await ready();
        const blob = await solidBlob(100, 150, 200);
        const palette = await getPalette(blob, 10, 10);
        for (const color of palette) {
            expect(color.length).toBe(3);
            for (const v of color) {
                expect(v).toBeGreaterThanOrEqual(0);
                expect(v).toBeLessThanOrEqual(255);
            }
        }
    });

    it('respects color_count', async () => {
        await ready();
        const blob = await solidBlob(255, 0, 0);
        for (const count of [3, 5]) {
            const palette = await getPalette(blob, count, 10);
            expect(palette.length).toBeLessThanOrEqual(count);
        }
    });

    it('no duplicate colors', async () => {
        await ready();
        const blob = await solidBlob(100, 150, 200);
        const palette = await getPalette(blob, 20, 10);
        const keys = palette.map(c => `${c[0]},${c[1]},${c[2]}`);
        expect(new Set(keys).size).toBe(keys.length);
    });

    it('deterministic results', async () => {
        await ready();
        const blob = await solidBlob(180, 90, 45);
        const p1 = await getPalette(blob, 10, 10);
        const p2 = await getPalette(blob, 10, 10);
        expect(p1).toEqual(p2);
    });

    it('detects two distinct colors', async () => {
        await ready();
        const blob = await twoColorBlob(255, 0, 0, 0, 0, 255);
        const palette = await getPalette(blob, 5, 10);
        expect(palette.length).toBeGreaterThanOrEqual(2);
        const hasRed = palette.some(c => c[0] > 200 && c[1] < 55 && c[2] < 55);
        const hasBlue = palette.some(c => c[0] < 55 && c[1] < 55 && c[2] > 200);
        expect(hasRed || hasBlue).toBe(true);
    });
});

// ---------------------------------------------------------------------------
// getColor — via Canvas-decoded image
// ---------------------------------------------------------------------------

describe('getColor (browser)', () => {
    it('returns an RGB array of length 3', async () => {
        await ready();
        const blob = await solidBlob(255, 0, 0);
        const color = await getColor(blob, 10);
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
    });

    it('returns valid RGB values', async () => {
        await ready();
        const blob = await solidBlob(100, 150, 200);
        const color = await getColor(blob, 10);
        for (const v of color) {
            expect(v).toBeGreaterThanOrEqual(0);
            expect(v).toBeLessThanOrEqual(255);
        }
    });

    it('returns red for a red image', async () => {
        await ready();
        const blob = await solidBlob(255, 0, 0);
        const color = await getColor(blob, 10);
        expect(color[0]).toBeGreaterThan(200);
        expect(color[1]).toBeLessThan(55);
        expect(color[2]).toBeLessThan(55);
    });

    it('deterministic results', async () => {
        await ready();
        const blob = await solidBlob(100, 150, 200);
        const c1 = await getColor(blob, 10);
        const c2 = await getColor(blob, 10);
        expect(c1).toEqual(c2);
    });
});

// ---------------------------------------------------------------------------
// decodeImage — pixel data extraction
// ---------------------------------------------------------------------------

describe('decodeImage (browser)', () => {
    it('returns correct dimensions', async () => {
        await ready();
        const blob = await solidBlob(255, 0, 0, 200, 100);
        const result = await decodeImage(blob);
        expect(result.width).toBe(200);
        expect(result.height).toBe(100);
    });

    it('returns correct pixel count (width * height * 4)', async () => {
        await ready();
        const blob = await solidBlob(255, 0, 0, 50, 30);
        const result = await decodeImage(blob);
        expect(result.pixels.length).toBe(50 * 30 * 4);
    });

    it('returns correct RGBA for solid red', async () => {
        await ready();
        const blob = await solidBlob(255, 0, 0);
        const result = await decodeImage(blob);
        expect(result.pixels[0]).toBe(255);
        expect(result.pixels[1]).toBe(0);
        expect(result.pixels[2]).toBe(0);
        expect(result.pixels[3]).toBe(255);
    });

    it('returns correct RGBA for solid green', async () => {
        await ready();
        const blob = await solidBlob(0, 255, 0);
        const result = await decodeImage(blob);
        expect(result.pixels[0]).toBe(0);
        expect(result.pixels[1]).toBe(255);
        expect(result.pixels[2]).toBe(0);
        expect(result.pixels[3]).toBe(255);
    });

    it('returns correct RGBA for white', async () => {
        await ready();
        const blob = await solidBlob(255, 255, 255);
        const result = await decodeImage(blob);
        expect(result.pixels[0]).toBe(255);
        expect(result.pixels[1]).toBe(255);
        expect(result.pixels[2]).toBe(255);
        expect(result.pixels[3]).toBe(255);
    });

    it('returns correct RGBA for black', async () => {
        await ready();
        const blob = await solidBlob(0, 0, 0);
        const result = await decodeImage(blob);
        expect(result.pixels[0]).toBe(0);
        expect(result.pixels[1]).toBe(0);
        expect(result.pixels[2]).toBe(0);
        expect(result.pixels[3]).toBe(255);
    });

    it('rejects non-image data', async () => {
        await ready();
        const fake = new Blob([new TextEncoder().encode('not an image')]);
        await expect(decodeImage(fake)).rejects.toThrow();
    });

    it('rejects empty bytes', async () => {
        await ready();
        const empty = new Blob([new Uint8Array(0)]);
        await expect(decodeImage(empty)).rejects.toThrow();
    });

    it('rejects invalid value', async () => {
        await ready();
        await expect(decodeImage(null)).rejects.toThrow();
    });
});

// ---------------------------------------------------------------------------
// Raw pixel functions (also work in browser)
// ---------------------------------------------------------------------------

describe('getPaletteFromPixels (browser)', () => {
    it('detects solid red', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(255, 0, 0);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette[0][0]).toBeGreaterThan(200);
        expect(palette[0][1]).toBeLessThan(55);
    });

    it('deterministic results', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(180, 90, 45);
        const p1 = await getPaletteFromPixels(pixels, width, height, 5, 10);
        const p2 = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(p1).toEqual(p2);
    });
});

describe('getColorFromPixels (browser)', () => {
    it('returns correct color', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(255, 0, 0);
        const color = await getColorFromPixels(pixels, width, height, 10);
        expect(color[0]).toBeGreaterThan(200);
        expect(color[1]).toBeLessThan(55);
    });
});

// ---------------------------------------------------------------------------
// Quality bounds
// ---------------------------------------------------------------------------

describe('Quality bounds', () => {
    it('quality=1 is valid', async () => {
        await ready();
        const blob = await solidBlob(200, 100, 50);
        const palette = await getPalette(blob, 5, 1);
        expect(palette.length).toBeGreaterThan(0);
    });

    it('quality=10 is valid', async () => {
        await ready();
        const blob = await solidBlob(200, 100, 50);
        const palette = await getPalette(blob, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
    });
});

// ---------------------------------------------------------------------------
// Error handling — high-level API
// ---------------------------------------------------------------------------

describe('High-level API errors', () => {
    it('rejects empty bytes — palette', async () => {
        await ready();
        await expect(getPalette(new Uint8Array(0), 5, 10)).rejects.toThrow();
    });

    it('rejects empty bytes — color', async () => {
        await ready();
        await expect(getColor(new Uint8Array(0), 10)).rejects.toThrow();
    });

    it('rejects non-image bytes', async () => {
        await ready();
        const text = new TextEncoder().encode('this is not an image');
        await expect(getColor(text, 10)).rejects.toThrow();
    });

    it('rejects truncated JPEG', async () => {
        await ready();
        const truncated = new Uint8Array([0xff, 0xd8, 0xff, 0xe0, 0, 0, 0, 0, 0, 0]);
        await expect(getColor(truncated, 10)).rejects.toThrow();
    });
});

// ---------------------------------------------------------------------------
// Concurrency
// ---------------------------------------------------------------------------

describe('Concurrency', () => {
    it('handles concurrent palette calls', async () => {
        await ready();
        const blob = await solidBlob(100, 150, 200);
        const results = await Promise.all(
            Array.from({ length: 5 }, () => getPalette(blob, 10, 10))
        );
        expect(results.length).toBe(5);
        for (const p of results) expect(p.length).toBeGreaterThan(0);
    });

    it('handles concurrent color calls', async () => {
        await ready();
        const blob = await solidBlob(100, 150, 200);
        const results = await Promise.all(
            Array.from({ length: 5 }, () => getColor(blob, 10))
        );
        expect(results.length).toBe(5);
        for (const c of results) expect(c.length).toBe(3);
    });

    it('mixed concurrent operations', async () => {
        await ready();
        const blob = await solidBlob(100, 150, 200);
        const results = await Promise.all([
            getPalette(blob, 10, 10),
            getColor(blob, 10),
            decodeImage(blob),
        ]);
        expect(results[0].length).toBeGreaterThan(0);
        expect(results[1].length).toBe(3);
        expect(results[2].width).toBeGreaterThan(0);
    });

    it('concurrent calls produce consistent results', async () => {
        await ready();
        const blob = await solidBlob(180, 90, 45);
        const results = await Promise.all(
            Array.from({ length: 4 }, () => getPalette(blob, 10, 10))
        );
        for (let i = 1; i < results.length; i++) {
            expect(results[i]).toEqual(results[0]);
        }
    });
});
