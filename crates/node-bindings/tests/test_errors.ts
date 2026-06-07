import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';
import { testImagePath } from './test_helper.js';

describe('Error handling', () => {
    // -- Invalid paths --

    it('throws on nonexistent file (color)', async () => {
        await expect(getColor('does_not_exist.jpg')).rejects.toThrow();
    });

    it('throws on nonexistent file (palette)', async () => {
        await expect(getPalette('does_not_exist.jpg')).rejects.toThrow();
    });

    it('error message contains path', async () => {
        await expect(getColor('no_such_image.png')).rejects.toThrow('no_such_image.png');
    });

    // -- Invalid buffer data --

    it('throws on empty buffer (color)', async () => {
        await expect(getColor(Buffer.from([]))).rejects.toThrow();
    });

    it('throws on empty buffer (palette)', async () => {
        await expect(getPalette(Buffer.from([]))).rejects.toThrow();
    });

    it('throws on invalid data', async () => {
        await expect(getColor(Buffer.from('this is not an image'))).rejects.toThrow();
    });

    it('throws on truncated JPEG', async () => {
        const truncated = Buffer.from([0xff, 0xd8, 0xff, 0xe0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        await expect(getColor(truncated)).rejects.toThrow();
    });

    // -- Quality bounds --

    it('quality=1 is valid', async () => {
        const color = await getColor(testImagePath(), 1);
        expect(color.length).toBe(3);
    });

    it('quality=5 is valid', async () => {
        const color = await getColor(testImagePath(), 5);
        expect(color.length).toBe(3);
    });

    it('quality=10 is valid', async () => {
        const color = await getColor(testImagePath(), 10);
        expect(color.length).toBe(3);
    });
});
