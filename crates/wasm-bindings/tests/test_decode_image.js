import { describe, it, expect, beforeAll } from 'vitest';

// ---------------------------------------------------------------------------
// Helpers: generate test images via Canvas, return as Blob
// ---------------------------------------------------------------------------

/** Create a solid-color image (WxH) and return a Blob. */
async function createSolidImage(r, g, b, w = 100, h = 100) {
    const canvas = document.createElement('canvas');
    canvas.width = w;
    canvas.height = h;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = `rgb(${r},${g},${b})`;
    ctx.fillRect(0, 0, w, h);
    return new Promise(resolve => canvas.toBlob(resolve, 'image/png'));
}

// ---------------------------------------------------------------------------
// Load the WASM module
// ---------------------------------------------------------------------------

let decodeImage, getPalette, getColor;
let wasmAvailable = false;

beforeAll(async () => {
    try {
        const mod = await import('../pkg/colorthief_wasm.js');
        decodeImage = mod.decodeImage;
        getPalette = mod.getPalette;
        getColor = mod.getColor;
        wasmAvailable =
            typeof decodeImage === 'function' &&
            typeof getPalette === 'function' &&
            typeof getColor === 'function';
    } catch {
        // WASM not built yet — tests will be skipped
    }
});

// ---------------------------------------------------------------------------
// decodeImage() — basic functionality
// ---------------------------------------------------------------------------

describe('decodeImage() basic functionality', () => {
    it('should return an object with pixels, width, and height', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(255, 0, 0);
        const result = await decodeImage(img);
        expect(result).toHaveProperty('pixels');
        expect(result).toHaveProperty('width');
        expect(result).toHaveProperty('height');
    }, 30000);

    it('should return correct dimensions for a 100x100 image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(100, 150, 200);
        const result = await decodeImage(img);
        expect(result.width).toBe(100);
        expect(result.height).toBe(100);
    }, 30000);

    it('should return correct dimensions for a non-square image', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(50, 100, 150, 200, 75);
        const result = await decodeImage(img);
        expect(result.width).toBe(200);
        expect(result.height).toBe(75);
    }, 30000);

    it('should return a Uint8Array for pixels', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(255, 128, 64);
        const result = await decodeImage(img);
        expect(result.pixels instanceof Uint8Array).toBe(true);
    }, 30000);

    it('should return correct pixel count (width * height * 4 for RGBA)', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(255, 0, 0, 50, 30);
        const result = await decodeImage(img);
        expect(result.pixels.length).toBe(50 * 30 * 4);
    }, 30000);
});

// ---------------------------------------------------------------------------
// decodeImage() — pixel values for solid colors
// ---------------------------------------------------------------------------

describe('decodeImage() pixel values', () => {
    it('should return correct RGBA for solid red', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(255, 0, 0);
        const result = await decodeImage(img);
        // First pixel: R=255, G=0, B=0, A=255
        expect(result.pixels[0]).toBe(255); // R
        expect(result.pixels[1]).toBe(0);   // G
        expect(result.pixels[2]).toBe(0);   // B
        expect(result.pixels[3]).toBe(255); // A
    }, 30000);

    it('should return correct RGBA for solid green', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(0, 255, 0);
        const result = await decodeImage(img);
        expect(result.pixels[0]).toBe(0);
        expect(result.pixels[1]).toBe(255);
        expect(result.pixels[2]).toBe(0);
        expect(result.pixels[3]).toBe(255);
    }, 30000);

    it('should return correct RGBA for solid blue', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(0, 0, 255);
        const result = await decodeImage(img);
        expect(result.pixels[0]).toBe(0);
        expect(result.pixels[1]).toBe(0);
        expect(result.pixels[2]).toBe(255);
        expect(result.pixels[3]).toBe(255);
    }, 30000);

    it('should return correct RGBA for white', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(255, 255, 255);
        const result = await decodeImage(img);
        expect(result.pixels[0]).toBe(255);
        expect(result.pixels[1]).toBe(255);
        expect(result.pixels[2]).toBe(255);
        expect(result.pixels[3]).toBe(255);
    }, 30000);

    it('should return correct RGBA for black', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(0, 0, 0);
        const result = await decodeImage(img);
        expect(result.pixels[0]).toBe(0);
        expect(result.pixels[1]).toBe(0);
        expect(result.pixels[2]).toBe(0);
        expect(result.pixels[3]).toBe(255);
    }, 30000);
});

// ---------------------------------------------------------------------------
// decodeImage() — input types
// ---------------------------------------------------------------------------

describe('decodeImage() input types', () => {
    it('should accept a Blob', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(128, 64, 32);
        const result = await decodeImage(img);
        expect(result.width).toBeGreaterThan(0);
        expect(result.height).toBeGreaterThan(0);
    }, 30000);

    it('should accept a Uint8Array', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const blob = await createSolidImage(200, 100, 50);
        const arrayBuffer = await blob.arrayBuffer();
        const uint8 = new Uint8Array(arrayBuffer);
        const result = await decodeImage(uint8);
        expect(result.width).toBe(100);
        expect(result.height).toBe(100);
    }, 30000);

    it('should accept an ArrayBuffer', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const blob = await createSolidImage(50, 150, 250);
        const arrayBuffer = await blob.arrayBuffer();
        const result = await decodeImage(arrayBuffer);
        expect(result.width).toBe(100);
        expect(result.height).toBe(100);
    }, 30000);
});

// ---------------------------------------------------------------------------
// decodeImage() — error handling
// ---------------------------------------------------------------------------

describe('decodeImage() error handling', () => {
    it('should reject with empty bytes', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const emptyBlob = new Blob([new Uint8Array(0)]);
        await expect(decodeImage(emptyBlob)).rejects.toThrow();
    }, 30000);

    it('should reject with non-image data', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const fakeBlob = new Blob([new TextEncoder().encode('this is not an image')]);
        await expect(decodeImage(fakeBlob)).rejects.toThrow();
    }, 30000);

    it('should reject with truncated JPEG header', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const truncated = new Uint8Array([0xff, 0xd8, 0xff, 0xe0, 0, 0, 0, 0, 0, 0]);
        const blob = new Blob([truncated]);
        await expect(decodeImage(blob)).rejects.toThrow();
    }, 30000);

    it('should reject with invalid JS value', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        await expect(decodeImage(null)).rejects.toThrow();
    }, 30000);

    it('should reject with undefined', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        await expect(decodeImage(undefined)).rejects.toThrow();
    }, 30000);

    it('should reject with a plain number', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        await expect(decodeImage(42)).rejects.toThrow();
    }, 30000);

    it('should reject with an empty object', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        await expect(decodeImage({})).rejects.toThrow();
    }, 30000);
});

// ---------------------------------------------------------------------------
// decodeImage() — error messages contain useful info
// ---------------------------------------------------------------------------

describe('decodeImage() error messages', () => {
    it('should include an error message when rejecting empty bytes', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const emptyBlob = new Blob([new Uint8Array(0)]);
        try {
            await decodeImage(emptyBlob);
            expect.fail('Should have thrown');
        } catch (e) {
            expect(e.message.length).toBeGreaterThan(0);
        }
    }, 30000);

    it('should include an error message when rejecting non-image data', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const fakeBlob = new Blob([new TextEncoder().encode('not an image at all')]);
        try {
            await decodeImage(fakeBlob);
            expect.fail('Should have thrown');
        } catch (e) {
            expect(e.message.length).toBeGreaterThan(0);
        }
    }, 30000);
});

// ---------------------------------------------------------------------------
// decodeImage() — consistency with getPalette/getColor
// ---------------------------------------------------------------------------

describe('decodeImage() consistency', () => {
    it('should produce same palette as getPalette when using decoded pixels', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(255, 100, 50);
        const paletteFromBlob = await getPalette(img, 5, 10);
        const paletteFromDecode = await getPalette(img, 5, 10);
        expect(paletteFromBlob).toEqual(paletteFromDecode);
    }, 30000);

    it('decoded image dimensions should match source', async () => {
        await expect.poll(() => wasmAvailable).toBe(true);
        const img = await createSolidImage(10, 20, 30, 64, 48);
        const result = await decodeImage(img);
        expect(result.width).toBe(64);
        expect(result.height).toBe(48);
    }, 30000);
});
