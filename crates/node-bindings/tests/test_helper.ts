import { resolve } from 'path';
import { readFileSync } from 'fs';

// ---------------------------------------------------------------------------
// Committed test images (shared across all language bindings)
// ---------------------------------------------------------------------------

const TEST_IMAGE_PATH = resolve(__dirname, 'test.jpg');
const KAIJU_IMAGE_PATH = resolve(__dirname, 'kaiju_no_8.jpg');

/** Get the file path for the primary test image. */
export function testImagePath(): string {
    return TEST_IMAGE_PATH;
}

/** Get the file path for the secondary test image (different dominant color). */
export function kaijuImagePath(): string {
    return KAIJU_IMAGE_PATH;
}

/** Read the primary test image as a Buffer. */
export function testImageBuffer(): Buffer {
    return readFileSync(TEST_IMAGE_PATH);
}

/** Read the secondary test image as a Buffer. */
export function kaijuImageBuffer(): Buffer {
    return readFileSync(KAIJU_IMAGE_PATH);
}
