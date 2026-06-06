import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TEST_IMAGE = resolve(__dirname, 'test.jpg');

describe('Error handling', () => {
    // -- Invalid paths --

    it('throws on nonexistent file - getColor', async () => {
        await expect(getColor('does_not_exist.jpg')).rejects.toThrow();
    });

    it('throws on nonexistent file - getPalette', async () => {
        await expect(getPalette('does_not_exist.jpg')).rejects.toThrow();
    });

    it('error message contains path', async () => {
        try {
            await getColor('no_such_image.png');
            expect.fail('should have thrown');
        } catch (e: unknown) {
            const msg = (e as Error).message ?? String(e);
            expect(msg.toLowerCase()).toContain('no_such_image');
        }
    });

    // -- Invalid bytes --

    it('throws on empty buffer - getColor', async () => {
        await expect(getColor(Buffer.from(''))).rejects.toThrow();
    });

    it('throws on empty buffer - getPalette', async () => {
        await expect(getPalette(Buffer.from(''))).rejects.toThrow();
    });

    it('throws on invalid data', async () => {
        await expect(getColor(Buffer.from('not an image'))).rejects.toThrow();
    });

    it('throws on truncated image data', async () => {
        // JPEG magic bytes followed by garbage
        const truncated = Buffer.from([0xff, 0xd8, 0xff, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        await expect(getColor(truncated)).rejects.toThrow();
    });

    // -- Unsupported types --

    it('throws on null input - getColor', async () => {
        await expect(getColor(null as unknown as string)).rejects.toThrow();
    });

    it('throws on null input - getPalette', async () => {
        await expect(getPalette(null as unknown as string)).rejects.toThrow();
    });

    it('throws on number input - getColor', async () => {
        await expect(getColor(42 as unknown as string)).rejects.toThrow();
    });

    it('throws on number input - getPalette', async () => {
        await expect(getPalette(42 as unknown as string)).rejects.toThrow();
    });

    it('throws on float input - getColor', async () => {
        await expect(getColor(3.14 as unknown as string)).rejects.toThrow();
    });

    it('throws on array input - getColor', async () => {
        await expect(getColor([] as unknown as string)).rejects.toThrow();
    });

    it('throws on array input - getPalette', async () => {
        await expect(getPalette([] as unknown as string)).rejects.toThrow();
    });

    it('throws on object input - getColor', async () => {
        await expect(getColor({} as unknown as string)).rejects.toThrow();
    });

    it('throws on object input - getPalette', async () => {
        await expect(getPalette({} as unknown as string)).rejects.toThrow();
    });

    it('throws on boolean input - getColor', async () => {
        await expect(getColor(true as unknown as string)).rejects.toThrow();
    });

    it('throws on dict-like path input - getPalette', async () => {
        await expect(getPalette({ path: 'test.jpg' } as unknown as string)).rejects.toThrow();
    });

    // -- Quality bounds --

    it('quality=1 is valid', async () => {
        const color = await getColor(TEST_IMAGE, 1);
        expect(color.length).toBe(3);
    });

    it('quality=5 is valid', async () => {
        const color = await getColor(TEST_IMAGE, 5);
        expect(color.length).toBe(3);
    });

    it('quality=10 is valid', async () => {
        const color = await getColor(TEST_IMAGE, 10);
        expect(color.length).toBe(3);
    });
});
