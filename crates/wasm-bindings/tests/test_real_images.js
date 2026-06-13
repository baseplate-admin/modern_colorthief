import { describe, it, expect, beforeAll } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Resolve the absolute path to a test image file. */
function testImagePath(filename) {
    return join(__dirname, filename);
}

/** Read a test image file as a Uint8Array. */
function loadImageBytes(filename) {
    return new Uint8Array(readFileSync(testImagePath(filename)));
}

// ---------------------------------------------------------------------------
// Load the WASM module
// ---------------------------------------------------------------------------

let getPalette, getColor, decodeImage;
let wasmAvailable = false;

beforeAll(async () => {
    try {
        const mod = await import('../pkg/colorthief_wasm.js');
        getPalette = mod.getPalette;
        getColor = mod.getColor;
        decodeImage = mod.decodeImage;
        wasmAvailable =
            typeof getPalette === 'function' &&
            typeof getColor === 'function' &&
            typeof decodeImage === 'function';
    } catch {
        // WASM not built yet — tests will be skipped
    }
});

// ---------------------------------------------------------------------------
// test.jpg — basic palette extraction
// ---------------------------------------------------------------------------

describe('test.jpg — getPalette', () => {
    it('should return a non-empty palette', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const palette = await getPalette(bytes, 10, 10);
        expect(palette.length).toBeGreaterThan(0);
    }, 30000);

    it('should return colors with valid RGB values (0-255)', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const palette = await getPalette(bytes, 10, 10);
        for (const color of palette) {
            expect(color.length).toBe(3);
            for (const channel of color) {
                expect(channel).toBeGreaterThanOrEqual(0);
                expect(channel).toBeLessThanOrEqual(255);
            }
        }
    }, 30000);

    it('should return at most color_count colors', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const palette = await getPalette(bytes, 5, 10);
        expect(palette.length).toBeLessThanOrEqual(5);
    }, 30000);

    it('should return no duplicate colors', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const palette = await getPalette(bytes, 20, 10);
        const serialized = palette.map(c => `${c[0]},${c[1]},${c[2]}`);
        const unique = new Set(serialized);
        expect(unique.size).toBe(serialized.length);
    }, 30000);

    it('should return more colors with higher color_count', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const small = await getPalette(bytes, 3, 10);
        const large = await getPalette(bytes, 15, 10);
        expect(large.length).toBeGreaterThanOrEqual(small.length);
    }, 30000);

    it('should produce deterministic results', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const p1 = await getPalette(bytes, 10, 10);
        const p2 = await getPalette(bytes, 10, 10);
        expect(p1).toEqual(p2);
    }, 30000);
});

// ---------------------------------------------------------------------------
// test.jpg — getColor
// ---------------------------------------------------------------------------

describe('test.jpg — getColor', () => {
    it('should return an RGB array of length 3', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const color = await getColor(bytes, 10);
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
    }, 30000);

    it('should return valid RGB values (0-255)', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const color = await getColor(bytes, 10);
        for (const channel of color) {
            expect(channel).toBeGreaterThanOrEqual(0);
            expect(channel).toBeLessThanOrEqual(255);
        }
    }, 30000);

    it('should produce deterministic results', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const c1 = await getColor(bytes, 10);
        const c2 = await getColor(bytes, 10);
        expect(c1).toEqual(c2);
    }, 30000);

    it('should return the same dominant color regardless of quality setting', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const cLow = await getColor(bytes, 1);
        const cHigh = await getColor(bytes, 10);
        // Colors may differ slightly with quality, but should be in the same ballpark
        expect(Math.abs(cLow[0] - cHigh[0])).toBeLessThan(100);
        expect(Math.abs(cLow[1] - cHigh[1])).toBeLessThan(100);
        expect(Math.abs(cLow[2] - cHigh[2])).toBeLessThan(100);
    }, 30000);
});

// ---------------------------------------------------------------------------
// test.jpg — decodeImage
// ---------------------------------------------------------------------------

describe('test.jpg — decodeImage', () => {
    it('should decode and return valid dimensions', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const result = await decodeImage(bytes);
        expect(result.width).toBeGreaterThan(0);
        expect(result.height).toBeGreaterThan(0);
        expect(result.pixels.length).toBe(result.width * result.height * 4);
    }, 30000);

    it('should return a Uint8Array of pixels', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const result = await decodeImage(bytes);
        expect(result.pixels instanceof Uint8Array).toBe(true);
    }, 30000);
});

// ---------------------------------------------------------------------------
// kaiju_no_8.jpg — getPalette
// ---------------------------------------------------------------------------

describe('kaiju_no_8.jpg — getPalette', () => {
    it('should return a non-empty palette', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('kaiju_no_8.jpg');
        const palette = await getPalette(bytes, 10, 10);
        expect(palette.length).toBeGreaterThan(0);
    }, 30000);

    it('should return colors with valid RGB values (0-255)', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('kaiju_no_8.jpg');
        const palette = await getPalette(bytes, 10, 10);
        for (const color of palette) {
            expect(color.length).toBe(3);
            for (const channel of color) {
                expect(channel).toBeGreaterThanOrEqual(0);
                expect(channel).toBeLessThanOrEqual(255);
            }
        }
    }, 30000);

    it('should return no duplicate colors', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('kaiju_no_8.jpg');
        const palette = await getPalette(bytes, 20, 10);
        const serialized = palette.map(c => `${c[0]},${c[1]},${c[2]}`);
        const unique = new Set(serialized);
        expect(unique.size).toBe(serialized.length);
    }, 30000);

    it('should produce deterministic results', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('kaiju_no_8.jpg');
        const p1 = await getPalette(bytes, 10, 10);
        const p2 = await getPalette(bytes, 10, 10);
        expect(p1).toEqual(p2);
    }, 30000);
});

// ---------------------------------------------------------------------------
// kaiju_no_8.jpg — getColor
// ---------------------------------------------------------------------------

describe('kaiju_no_8.jpg — getColor', () => {
    it('should return an RGB array of length 3', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('kaiju_no_8.jpg');
        const color = await getColor(bytes, 10);
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
    }, 30000);

    it('should return valid RGB values (0-255)', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('kaiju_no_8.jpg');
        const color = await getColor(bytes, 10);
        for (const channel of color) {
            expect(channel).toBeGreaterThanOrEqual(0);
            expect(channel).toBeLessThanOrEqual(255);
        }
    }, 30000);
});

// ---------------------------------------------------------------------------
// kaiju_no_8.jpg — decodeImage
// ---------------------------------------------------------------------------

describe('kaiju_no_8.jpg — decodeImage', () => {
    it('should decode and return valid dimensions', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('kaiju_no_8.jpg');
        const result = await decodeImage(bytes);
        expect(result.width).toBeGreaterThan(0);
        expect(result.height).toBeGreaterThan(0);
        expect(result.pixels.length).toBe(result.width * result.height * 4);
    }, 30000);
});

// ---------------------------------------------------------------------------
// URL input to getPalette/getColor
// ---------------------------------------------------------------------------

describe('URL input to getPalette/getColor', () => {
    it('should accept a file URL for getPalette', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const url = `file:///${testImagePath('test.jpg').replace(/\\/g, '/')}`;
        const palette = await getPalette(url, 10, 10);
        expect(palette.length).toBeGreaterThan(0);
    }, 30000);

    it('should accept a file URL for getColor', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const url = `file:///${testImagePath('test.jpg').replace(/\\/g, '/')}`;
        const color = await getColor(url, 10);
        expect(color.length).toBe(3);
    }, 30000);

    it('should accept a file URL for decodeImage', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const url = `file:///${testImagePath('test.jpg').replace(/\\/g, '/')}`;
        const result = await decodeImage(url);
        expect(result.width).toBeGreaterThan(0);
        expect(result.height).toBeGreaterThan(0);
    }, 30000);

    it('should reject an invalid URL', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        await expect(getPalette('https://example.com/nonexistent.png', 5, 10)).rejects.toThrow();
    }, 30000);
});

// ---------------------------------------------------------------------------
// Blob input with real image data
// ---------------------------------------------------------------------------

describe('Blob input with real image data', () => {
    it('should accept a Blob created from test.jpg bytes', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const blob = new Blob([bytes]);
        const palette = await getPalette(blob, 10, 10);
        expect(palette.length).toBeGreaterThan(0);
    }, 30000);

    it('should accept a Blob created from kaiju_no_8.jpg bytes', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('kaiju_no_8.jpg');
        const blob = new Blob([bytes]);
        const color = await getColor(blob, 10);
        expect(color.length).toBe(3);
    }, 30000);
});

// ---------------------------------------------------------------------------
// Concurrent Promise execution with real images
// ---------------------------------------------------------------------------

describe('Concurrent Promise execution', () => {
    it('should handle concurrent getPalette calls for the same image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const promises = Array.from({ length: 5 }, () => getPalette(bytes, 10, 10));
        const results = await Promise.all(promises);
        expect(results.length).toBe(5);
        for (const palette of results) {
            expect(palette.length).toBeGreaterThan(0);
        }
    }, 30000);

    it('should handle concurrent getColor calls for the same image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const promises = Array.from({ length: 5 }, () => getColor(bytes, 10));
        const results = await Promise.all(promises);
        expect(results.length).toBe(5);
        for (const color of results) {
            expect(color.length).toBe(3);
        }
    }, 30000);

    it('should handle concurrent decodeImage calls for the same image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const promises = Array.from({ length: 3 }, () => decodeImage(bytes));
        const results = await Promise.all(promises);
        expect(results.length).toBe(3);
        for (const result of results) {
            expect(result.width).toBeGreaterThan(0);
            expect(result.height).toBeGreaterThan(0);
        }
    }, 30000);

    it('should handle concurrent calls across both test images', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const testBytes = loadImageBytes('test.jpg');
        const kaijuBytes = loadImageBytes('kaiju_no_8.jpg');
        const results = await Promise.all([
            getPalette(testBytes, 10, 10),
            getColor(kaijuBytes, 10),
            decodeImage(testBytes),
            getPalette(kaijuBytes, 5, 10),
            getColor(testBytes, 10),
        ]);
        expect(results[0].length).toBeGreaterThan(0); // palette from test.jpg
        expect(results[1].length).toBe(3);            // color from kaiju
        expect(results[2].width).toBeGreaterThan(0);  // decoded test.jpg
        expect(results[3].length).toBeGreaterThan(0); // palette from kaiju
        expect(results[4].length).toBe(3);            // color from test.jpg
    }, 30000);

    it('should handle concurrent calls with different parameters', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('kaiju_no_8.jpg');
        const results = await Promise.all([
            getPalette(bytes, 3, 10),
            getPalette(bytes, 10, 10),
            getPalette(bytes, 20, 1),
            getColor(bytes, 1),
            getColor(bytes, 10),
        ]);
        expect(results[0].length).toBeLessThanOrEqual(3);
        expect(results[1].length).toBeLessThanOrEqual(10);
        expect(results[2].length).toBeLessThanOrEqual(20);
        expect(results[3].length).toBe(3);
        expect(results[4].length).toBe(3);
    }, 30000);

    it('should produce consistent results across concurrent calls', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const bytes = loadImageBytes('test.jpg');
        const promises = Array.from({ length: 4 }, () => getPalette(bytes, 10, 10));
        const results = await Promise.all(promises);
        // All results should be identical
        for (let i = 1; i < results.length; i++) {
            expect(results[i]).toEqual(results[0]);
        }
    }, 30000);
});
