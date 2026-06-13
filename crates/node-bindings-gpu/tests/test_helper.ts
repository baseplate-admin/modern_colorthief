// ---------------------------------------------------------------------------
// Helpers for GPU binding tests (raw pixel data, not image files)
// ---------------------------------------------------------------------------

/**
 * Create a Uint8Array of RGBA pixel data.
 * Each pixel is 4 bytes: [R, G, B, A].
 */
export function createPixels(
    width: number,
    height: number,
    color: [number, number, number],
    alpha: number = 255,
): Uint8Array {
    const [r, g, b] = color;
    const data = new Uint8Array(width * height * 4);
    for (let i = 0; i < width * height; i++) {
        data[i * 4 + 0] = r;
        data[i * 4 + 1] = g;
        data[i * 4 + 2] = b;
        data[i * 4 + 3] = alpha;
    }
    return data;
}

/**
 * Create a two-color gradient image (left half color1, right half color2).
 */
export function createGradientPixels(
    width: number,
    height: number,
    color1: [number, number, number],
    color2: [number, number, number],
): Uint8Array {
    const data = new Uint8Array(width * height * 4);
    const mid = Math.floor(width / 2);
    for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
            const idx = (y * width + x) * 4;
            const [r, g, b] = x < mid ? color1 : color2;
            data[idx + 0] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = 255;
        }
    }
    return data;
}

/**
 * Try to import the native GPU module.
 * Returns null if the module is not available (e.g., native binary not built).
 */
export async function tryImportGpu() {
    try {
        const mod = await import('../index.js');
        return mod;
    } catch {
        return null;
    }
}
