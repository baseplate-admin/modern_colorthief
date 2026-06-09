import { getPalette as _getPalette, getColor as _getColor } from './native';
import sharp from 'sharp';

/**
 * Extract a palette of dominant colors from an image.
 *
 * @param {string|Buffer} image - File path or image buffer
 * @param {number} [colorCount=10] - Number of colors to extract
 * @param {number} [quality=10] - Sampling quality
 * @returns {Promise<number[][]>} Array of [R, G, B] color tuples
 */
export async function getPalette(image, colorCount = 10, quality = 10) {
    const { data, info } = await sharp(image).raw().ensureAlpha().toBuffer({ resolveWithObject: true });
    return _getPalette(data, info.width, info.height, colorCount, quality);
}

/**
 * Extract the dominant color from an image.
 *
 * @param {string|Buffer} image - File path or image buffer
 * @param {number} [quality=10] - Sampling quality
 * @returns {Promise<number[]>} [R, G, B] color tuple
 */
export async function getColor(image, quality = 10) {
    const { data, info } = await sharp(image).raw().ensureAlpha().toBuffer({ resolveWithObject: true });
    return _getColor(data, info.width, info.height, quality);
}
