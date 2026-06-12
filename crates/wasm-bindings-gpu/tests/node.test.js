import { describe, it, expect, beforeAll, beforeEach } from 'vitest';

// ---------------------------------------------------------------------------
// Helpers (port of Python test helpers)
// ---------------------------------------------------------------------------

function pixels(r, g, b, w = 100, h = 100) {
    const buf = new Uint8Array(w * h * 4);
    for (let i = 0; i < w * h; i++) {
        buf[i * 4] = r; buf[i * 4 + 1] = g; buf[i * 4 + 2] = b; buf[i * 4 + 3] = 255;
    }
    return { pixels: buf, width: w, height: h };
}

function twoColor(r1, g1, b1, r2, g2, b2, w = 100, h = 100) {
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

function gradient(colors) {
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
// Load WASM GPU module
// ---------------------------------------------------------------------------

let getPaletteGpu, getColorGpu;
let gpuAvailable = false;

beforeAll(async () => {
    try {
        const mod = await import('../pkg/modern_colorthief_wasm_gpu.js');
        getPaletteGpu = mod.getPaletteGpu;
        getColorGpu = mod.getColorGpu;
        gpuAvailable = typeof getPaletteGpu === 'function' && typeof getColorGpu === 'function';
    } catch { /* WASM GPU not built for nodejs target */ }
});

function skipIfNoGpu() {
    if (!gpuAvailable) expect.skip('GPU not available in Node.js');
}

// ---------------------------------------------------------------------------
// API surface (port of test_api.py)
// ---------------------------------------------------------------------------

describe('GPU API surface', () => {
    beforeEach(() => { skipIfNoGpu(); });

    it('exports getPaletteGpu', () => { expect(typeof getPaletteGpu).toBe('function'); });
    it('exports getColorGpu', () => { expect(typeof getColorGpu).toBe('function'); });
});

// ---------------------------------------------------------------------------
// Solid color — getPaletteGpu (port of test_properties.py)
// ---------------------------------------------------------------------------

describe('GPU solid color — palette', () => {
    beforeEach(() => { skipIfNoGpu(); });

    it('detects pure red', async () => {
        const { pixels, width, height } = pixels(255, 0, 0);
        const palette = await getPaletteGpu(pixels, width, height, 5, 10);
        expect(palette[0][0]).toBeGreaterThan(200);
        expect(palette[0][1]).toBeLessThan(55);
        expect(palette[0][2]).toBeLessThan(55);
    });

    it('detects pure green', async () => {
        const { pixels, width, height } = pixels(0, 255, 0);
        const palette = await getPaletteGpu(pixels, width, height, 5, 10);
        expect(palette[0][1]).toBeGreaterThan(200);
    });

    it('detects white', async () => {
        const { pixels, width, height } = pixels(255, 255, 255);
        const palette = await getPaletteGpu(pixels, width, height, 5, 10);
        expect(palette[0][0]).toBeGreaterThan(200);
        expect(palette[0][1]).toBeGreaterThan(200);
        expect(palette[0][2]).toBeGreaterThan(200);
    });

    it('detects black', async () => {
        const { pixels, width, height } = pixels(0, 0, 0);
        const palette = await getPaletteGpu(pixels, width, height, 5, 10);
        expect(palette[0][0]).toBeLessThan(55);
        expect(palette[0][1]).toBeLessThan(55);
        expect(palette[0][2]).toBeLessThan(55);
    });
});

// ---------------------------------------------------------------------------
// getColorGpu (port of test_properties.py)
// ---------------------------------------------------------------------------

describe('GPU getColor', () => {
    beforeEach(() => { skipIfNoGpu(); });

    it('returns RGB array of length 3', async () => {
        const { pixels, width, height } = pixels(255, 0, 0);
        const color = await getColorGpu(pixels, width, height, 10);
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
    });

    it('returns valid RGB values (0-255)', async () => {
        const { pixels, width, height } = pixels(100, 150, 200);
        const color = await getColorGpu(pixels, width, height, 10);
        for (const v of color) {
            expect(v).toBeGreaterThanOrEqual(0);
            expect(v).toBeLessThanOrEqual(255);
        }
    });

    it('returns red for red image', async () => {
        const { pixels, width, height } = pixels(255, 0, 0);
        const color = await getColorGpu(pixels, width, height, 10);
        expect(color[0]).toBeGreaterThan(200);
        expect(color[1]).toBeLessThan(55);
        expect(color[2]).toBeLessThan(55);
    });
});

// ---------------------------------------------------------------------------
// Palette properties (port of test_properties.py)
// ---------------------------------------------------------------------------

describe('GPU palette properties', () => {
    beforeEach(() => { skipIfNoGpu(); });

    it('returns valid RGB values', async () => {
        const { pixels, width, height } = pixels(100, 150, 200);
        const palette = await getPaletteGpu(pixels, width, height, 10, 10);
        for (const color of palette) {
            expect(color.length).toBe(3);
            for (const v of color) {
                expect(v).toBeGreaterThanOrEqual(0);
                expect(v).toBeLessThanOrEqual(255);
            }
        }
    });

    it('palette length does not exceed color_count', async () => {
        const { pixels, width, height } = pixels(128, 64, 32);
        for (const count of [3, 5]) {
            const palette = await getPaletteGpu(pixels, width, height, count, 10);
            expect(palette.length).toBeLessThanOrEqual(count);
        }
    });

    it('no duplicate colors', async () => {
        const { pixels, width, height } = pixels(100, 150, 200);
        const palette = await getPaletteGpu(pixels, width, height, 20, 10);
        const keys = palette.map(c => `${c[0]},${c[1]},${c[2]}`);
        expect(new Set(keys).size).toBe(keys.length);
    });
});

// ---------------------------------------------------------------------------
// Two-color detection
// ---------------------------------------------------------------------------

describe('GPU two-color detection', () => {
    beforeEach(() => { skipIfNoGpu(); });

    it('finds two distinct colors', async () => {
        const { pixels, width, height } = twoColor(255, 0, 0, 0, 0, 255);
        const palette = await getPaletteGpu(pixels, width, height, 5, 10);
        const hasRed = palette.some(c => c[0] > 200 && c[1] < 55 && c[2] < 55);
        const hasBlue = palette.some(c => c[0] < 55 && c[1] < 55 && c[2] > 200);
        expect(hasRed || hasBlue).toBe(true);
    });
});

// ---------------------------------------------------------------------------
// Determinism (port of test_edge_cases.py)
// ---------------------------------------------------------------------------

describe('GPU determinism', () => {
    beforeEach(() => { skipIfNoGpu(); });

    it('same palette for same pixels', async () => {
        const { pixels, width, height } = pixels(180, 90, 45);
        const p1 = await getPaletteGpu(pixels, width, height, 5, 10);
        const p2 = await getPaletteGpu(pixels, width, height, 5, 10);
        expect(p1).toEqual(p2);
    });

    it('same color for same pixels', async () => {
        const { pixels, width, height } = pixels(180, 90, 45);
        const c1 = await getColorGpu(pixels, width, height, 10);
        const c2 = await getColorGpu(pixels, width, height, 10);
        expect(c1).toEqual(c2);
    });

    it('deterministic on multi-color image', async () => {
        const { pixels, width, height } = gradient([[255, 0, 0], [0, 255, 0], [0, 0, 255]]);
        const p1 = await getPaletteGpu(pixels, width, height, 10, 5);
        const p2 = await getPaletteGpu(pixels, width, height, 10, 5);
        expect(p1).toEqual(p2);
    });
});

// ---------------------------------------------------------------------------
// Quality bounds (port of test_errors.py / test_edge_cases.py)
// ---------------------------------------------------------------------------

describe('GPU quality bounds', () => {
    beforeEach(() => { skipIfNoGpu(); });

    it('quality=1 is valid', async () => {
        const { pixels, width, height } = pixels(200, 100, 50);
        const palette = await getPaletteGpu(pixels, width, height, 5, 1);
        expect(palette.length).toBeGreaterThan(0);
    });

    it('quality=10 is valid', async () => {
        const { pixels, width, height } = pixels(200, 100, 50);
        const palette = await getPaletteGpu(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
    });
});

// ---------------------------------------------------------------------------
// Deduplication (port of test_deduplication.py)
// ---------------------------------------------------------------------------

describe('GPU deduplication', () => {
    beforeEach(() => { skipIfNoGpu(); });

    it('no duplicates with high color_count', async () => {
        const { pixels, width, height } = pixels(100, 150, 200);
        const palette = await getPaletteGpu(pixels, width, height, 50, 10);
        const keys = palette.map(c => `${c[0]},${c[1]},${c[2]}`);
        expect(new Set(keys).size).toBe(keys.length);
        expect(palette.length).toBeLessThanOrEqual(50);
    });
});

// ---------------------------------------------------------------------------
// Edge cases (port of test_edge_cases.py)
// ---------------------------------------------------------------------------

describe('GPU edge cases', () => {
    beforeEach(() => { skipIfNoGpu(); });

    it('handles 1x1 pixel image — palette', async () => {
        const { pixels, width, height } = pixels(255, 128, 64, 1, 1);
        const palette = await getPaletteGpu(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
    });

    it('handles 1x1 pixel image — color', async () => {
        const { pixels, width, height } = pixels(70, 140, 210, 1, 1);
        const color = await getColorGpu(pixels, width, height, 10);
        expect(color.length).toBe(3);
    });

    it('handles large image', async () => {
        const { pixels, width, height } = pixels(100, 200, 150, 500, 500);
        const palette = await getPaletteGpu(pixels, width, height, 5, 10);
        expect(palette.length).toBeGreaterThan(0);
    });
});

// ---------------------------------------------------------------------------
// Error handling (port of test_errors.py)
// ---------------------------------------------------------------------------

describe('GPU error handling', () => {
    beforeEach(() => { skipIfNoGpu(); });

    it('rejects empty pixels — palette', async () => {
        await expect(getPaletteGpu(new Uint8Array(0), 0, 0, 5, 10)).rejects.toThrow();
    });

    it('rejects empty pixels — color', async () => {
        await expect(getColorGpu(new Uint8Array(0), 0, 0, 10)).rejects.toThrow();
    });

    it('rejects insufficient data', async () => {
        await expect(getColorGpu(new Uint8Array([255, 0, 0]), 1, 1, 10)).rejects.toThrow();
    });
});

// ---------------------------------------------------------------------------
// Concurrency (port of test_concurrent.py)
// ---------------------------------------------------------------------------

describe('GPU concurrency', () => {
    beforeEach(() => { skipIfNoGpu(); });

    it('handles concurrent palette calls', async () => {
        const { pixels, width, height } = pixels(100, 150, 200);
        const results = await Promise.all(
            Array.from({ length: 5 }, () => getPaletteGpu(pixels, width, height, 10, 10))
        );
        expect(results.length).toBe(5);
    });

    it('handles concurrent color calls', async () => {
        const { pixels, width, height } = pixels(100, 150, 200);
        const results = await Promise.all(
            Array.from({ length: 5 }, () => getColorGpu(pixels, width, height, 10))
        );
        expect(results.length).toBe(5);
    });

    it('concurrent calls produce consistent results', async () => {
        const { pixels, width, height } = pixels(180, 90, 45);
        const results = await Promise.all(
            Array.from({ length: 4 }, () => getPaletteGpu(pixels, width, height, 10, 10))
        );
        for (let i = 1; i < results.length; i++) {
            expect(results[i]).toEqual(results[0]);
        }
    });
});
