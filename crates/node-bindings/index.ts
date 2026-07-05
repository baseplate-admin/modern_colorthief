import { getPalette as _getPalette, getColor as _getColor } from './native';
import sharp from 'sharp';

/**
 * Extract a palette of dominant colors from an image.
 *
 * @param image - File path or image buffer
 * @param colorCount - Number of colors to extract (default: 10)
 * @param quality - Sampling quality (default: 10)
 * @returns Array of [R, G, B] color tuples
 */
export async function getPalette(
    image: string | Buffer,
    colorCount = 10,
    quality = 10,
): Promise<number[][]> {
    const { data, info } = await sharp(image)
        .raw()
        .ensureAlpha()
        .toBuffer({ resolveWithObject: true });
    return _getPalette(data, info.width, info.height, colorCount, quality);
}

/**
 * Extract the dominant color from an image.
 *
 * @param image - File path or image buffer
 * @param quality - Sampling quality (default: 10)
 * @returns [R, G, B] color tuple
 */
export async function getColor(
    image: string | Buffer,
    quality = 10,
): Promise<number[]> {
    const { data, info } = await sharp(image)
        .raw()
        .ensureAlpha()
        .toBuffer({ resolveWithObject: true });
    return _getColor(data, info.width, info.height, quality);
}
