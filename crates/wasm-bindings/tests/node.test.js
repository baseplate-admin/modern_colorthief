import { describe, it, expect, beforeAll } from 'vitest';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function solidPixels(r, g, b, w = 100, h = 100) {
    const buf = new Uint8Array(w * h * 4);
    for (let i = 0; i < w * h; i++) {
        buf[i * 4] = r; buf[i * 4 + 1] = g; buf[i * 4 + 2] = b; buf[i * 4 + 3] = 255;
    }
    return { pixels: buf, width: w, height: h };
}

function twoColorPixels(r1, g1, b1, r2, g2, b2, w = 100, h = 100) {
    const buf = new Uint8Array(w * h * 4);
    const half = Math.floor(w / 2);
    for (let y = 0; y < h; y++) {
        for (let x = 0; x < w; x++) {
            const idx = (y * w + x) * 4;
            if (x < half) { buf[idx] = r1; buf[idx + 1] = g1; buf[idx + 2] = b1; }
            else         { buf[idx] = r2; buf[idx + 1] = g2; buf[idx + 2] = b2; }
            buf[idx + 3] = 255;
        }
    }
    return { pixels: buf, width: w, height: h };
}

function gradientPixels(colors) {
    const w = 100;
    const bh = Math.ceil(100 / colors.length);
    const h = bh * colors.length;
    const buf = new Uint8Array(w * h * 4);
    for (let i = 0; i < colors.length; i++) {
        const [r, g, b] = colors[i];
        for (let y = i * bh; y < (i + 1) * bh; y++) {
            for (let x = 0; x < w; x++) {
                const idx = (y * w + x) * 4;
                buf[idx] = r; buf[idx + 1] = g; buf[idx + 2] = b; buf[idx + 3] = 255;
            }
        }
    }
    return { pixels: buf, width: w, height: h };
}

// ---------------------------------------------------------------------------
// Load WASM module (raw pixel functions only — no window dependency)
// ---------------------------------------------------------------------------

let getPaletteFromPixels, getColorFromPixels;
let wasmAvailable = false;

beforeAll(async () => {
    try {
        const mod = await import('../pkg/modern_colorthief_wasm.js');
        getPaletteFromPixels = mod.getPaletteFromPixels;
        getColorFromPixels = mod.getColorFromPixels;
        wasmAvailable = typeof getPaletteFromPixels === 'function' && typeof getColorFromPixels === 'function';
    } catch { /* WASM not built yet */ }
});

function ready() { return expect.poll(() => wasmAvailable).toBe(true); }

// ---------------------------------------------------------------------------
// API surface
// ---------------------------------------------------------------------------

describe('API surface', () => {
    it('exports getPaletteFromPixels', async () => { await ready(); expect(typeof getPaletteFromPixels).toBe('function'); });
    it('exports getColorFromPixels', async () => { await ready(); expect(typeof getColorFromPixels).toBe('function'); });
});

// ---------------------------------------------------------------------------
// Solid color
// ---------------------------------------------------------------------------

describe('Solid color — raw pixels', () => {
    it('detects pure red', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(255, 0, 0);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette[0][0]).toBeGreaterThan(200);
        expect(palette[0][1]).toBeLessThan(55);
        expect(palette[0][2]).toBeLessThan(55);
    });

    it('detects pure green', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(0, 255, 0);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette[0][1]).toBeGreaterThan(200);
    });

    it('detects white', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(255, 255, 255);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette[0][0]).toBeGreaterThan(200);
        expect(palette[0][1]).toBeGreaterThan(200);
        expect(palette[0][2]).toBeGreaterThan(200);
    });

    it('detects black', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(0, 0, 0);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette[0][0]).toBeLessThan(55);
        expect(palette[0][1]).toBeLessThan(55);
        expect(palette[0][2]).toBeLessThan(55);
    });
});

// ---------------------------------------------------------------------------
// getColor — raw pixels
// ---------------------------------------------------------------------------

describe('getColor — raw pixels', () => {
    it('returns an RGB array of length 3', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(255, 0, 0);
        const color = await getColorFromPixels(pixels, width, height, 10);
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
    });

    it('returns valid RGB values (0-255)', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(100, 150, 200);
        const color = await getColorFromPixels(pixels, width, height, 10);
        for (const v of color) {
            expect(v).toBeGreaterThanOrEqual(0);
            expect(v).toBeLessThanOrEqual(255);
        }
    });

    it('returns red for a red image', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(255, 0, 0);
        const color = await getColorFromPixels(pixels, width, height, 10);
        expect(color[0]).toBeGreaterThan(200);
        expect(color[1]).toBeLessThan(55);
        expect(color[2]).toBeLessThan(55);
    });

    it('returns green for a green image', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(0, 255, 0);
        const color = await getColorFromPixels(pixels, width, height, 10);
        expect(color[0]).toBeLessThan(55);
        expect(color[1]).toBeGreaterThan(200);
        expect(color[2]).toBeLessThan(55);
    });
});

// ---------------------------------------------------------------------------
// Palette properties
// ---------------------------------------------------------------------------

describe('Palette properties', () => {
    it('returns valid RGB values', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(100, 150, 200);
        const palette = await getPaletteFromPixels(pixels, width, height, 10, 10);
        for (const color of palette) {
            expect(color.length).toBe(3);
            for (const v of color) {
                expect(v).toBeGreaterThanOrEqual(0);
                expect(v).toBeLessThanOrEqual(255);
            }
        }
    });

    it('palette length does not exceed color_count', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(128, 64, 32);
        for (const count of [3, 5]) {
            const palette = await getPaletteFromPixels(pixels, width, height, count, 10);
            expect(palette.length).toBeLessThanOrEqual(count);
        }
    });

    it('returns no duplicate colors', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(100, 150, 200);
        const palette = await getPaletteFromPixels(pixels, width, height, 20, 10);
        const keys = palette.map(c => `${c[0]},${c[1]},${c[2]}`);
        expect(new Set(keys).size).toBe(keys.length);
    });
});

// ---------------------------------------------------------------------------
// Two-color detection
// ---------------------------------------------------------------------------

describe('Two-color detection', () => {
    it('finds two distinct colors', async () => {
        await ready();
        const { pixels, width, height } = twoColorPixels(255, 0, 0, 0, 0, 255);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThanOrEqual(2);
        const hasRed = palette.some(c => c[0] > 200 && c[1] < 55 && c[2] < 55);
        const hasBlue = palette.some(c => c[0] < 55 && c[1] < 55 && c[2] > 200);
        expect(hasRed || hasBlue).toBe(true);
    });

    it('getColor returns a dominant color from two-color image', async () => {
        await ready();
        const { pixels, width, height } = twoColorPixels(255, 0, 0, 0, 0, 255);
        const color = await getColorFromPixels(pixels, width, height, 10);
        const isRed = color[0] > 200 && color[1] < 55 && color[2] < 55;
        const isBlue = color[0] < 55 && color[1] < 55 && color[2] > 200;
        expect(isRed || isBlue).toBe(true);
    });
});

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

describe('Determinism', () => {
    it('same palette for same pixels', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(180, 90, 45);
        const p1 = await getPaletteFromPixels(pixels, width, height, 5, 10);
        const p2 = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(p1).toEqual(p2);
    });

    it('same color for same pixels', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(180, 90, 45);
        const c1 = await getColorFromPixels(pixels, width, height, 10);
        const c2 = await getColorFromPixels(pixels, width, height, 10);
        expect(c1).toEqual(c2);
    });

    it('deterministic on multi-color image', async () => {
        await ready();
        const { pixels, width, height } = gradientPixels([[255, 0, 0], [0, 255, 0], [0, 0, 255]]);
        const p1 = await getPaletteFromPixels(pixels, width, height, 10, 5);
        const p2 = await getPaletteFromPixels(pixels, width, height, 10, 5);
        expect(p1).toEqual(p2);
    });
});

// ---------------------------------------------------------------------------
// Quality bounds
// ---------------------------------------------------------------------------

describe('Quality bounds', () => {
    it('quality=1 is valid', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(200, 100, 50);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 1);
        expect(palette.length).toBeGreaterThan(0);
    });

    it('quality=10 is valid', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(200, 100, 50);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
    });

    it('quality=5 is valid', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(200, 100, 50);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 5);
        expect(palette.length).toBeGreaterThan(0);
    });
});

// ---------------------------------------------------------------------------
// Deduplication
// ---------------------------------------------------------------------------

describe('Deduplication', () => {
    it('no duplicates with high color_count', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(100, 150, 200);
        const palette = await getPaletteFromPixels(pixels, width, height, 50, 10);
        const keys = palette.map(c => `${c[0]},${c[1]},${c[2]}`);
        expect(new Set(keys).size).toBe(keys.length);
        expect(palette.length).toBeLessThanOrEqual(50);
    });
});

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

describe('Edge cases', () => {
    it('handles 1x1 pixel image — palette', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(255, 128, 64, 1, 1);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
        expect(palette[0][0]).toBeGreaterThan(200);
    });

    it('handles 1x1 pixel image — color', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(70, 140, 210, 1, 1);
        const color = await getColorFromPixels(pixels, width, height, 10);
        expect(color.length).toBe(3);
    });

    it('handles large image', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(100, 200, 150, 500, 500);
        const palette = await getPaletteFromPixels(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
    });

    it('higher color_count returns more colors', async () => {
        await ready();
        const { pixels, width, height } = gradientPixels([
            [255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0], [255, 0, 255]
        ]);
        const small = await getPaletteFromPixels(pixels, width, height, 3, 10);
        const large = await getPaletteFromPixels(pixels, width, height, 10, 10);
        expect(large.length).toBeGreaterThanOrEqual(small.length);
    });
});

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

describe('Error handling', () => {
    it('rejects empty pixels — palette', async () => {
        await ready();
        await expect(getPaletteFromPixels(new Uint8Array(0), 0, 0, 5, 10)).rejects.toThrow();
    });

    it('rejects insufficient pixel data — palette', async () => {
        await ready();
        await expect(getPaletteFromPixels(new Uint8Array([255, 0, 0]), 1, 1, 5, 10)).rejects.toThrow();
    });

    it('rejects empty pixels — color', async () => {
        await ready();
        await expect(getColorFromPixels(new Uint8Array(0), 0, 0, 10)).rejects.toThrow();
    });

    it('rejects insufficient pixel data — color', async () => {
        await ready();
        await expect(getColorFromPixels(new Uint8Array([255, 0, 0]), 1, 1, 10)).rejects.toThrow();
    });
});

// ---------------------------------------------------------------------------
// Concurrency
// ---------------------------------------------------------------------------

describe('Concurrency', () => {
    it('handles concurrent palette calls', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(100, 150, 200);
        const results = await Promise.all(
            Array.from({ length: 5 }, () => getPaletteFromPixels(pixels, width, height, 10, 10))
        );
        expect(results.length).toBe(5);
        for (const p of results) expect(p.length).toBeGreaterThan(0);
    });

    it('handles concurrent color calls', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(100, 150, 200);
        const results = await Promise.all(
            Array.from({ length: 5 }, () => getColorFromPixels(pixels, width, height, 10))
        );
        expect(results.length).toBe(5);
        for (const c of results) expect(c.length).toBe(3);
    });

    it('concurrent calls produce consistent results', async () => {
        await ready();
        const { pixels, width, height } = solidPixels(180, 90, 45);
        const results = await Promise.all(
            Array.from({ length: 4 }, () => getPaletteFromPixels(pixels, width, height, 10, 10))
        );
        for (let i = 1; i < results.length; i++) {
            expect(results[i]).toEqual(results[0]);
        }
    });
});
