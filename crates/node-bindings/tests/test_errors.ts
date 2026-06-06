import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';

describe('Error handling', () => {
    it('throws on nonexistent file - getColor', async () => {
        await expect(getColor('does_not_exist.jpg')).rejects.toThrow();
    });

    it('throws on nonexistent file - getPalette', async () => {
        await expect(getPalette('does_not_exist.jpg')).rejects.toThrow();
    });

    it('throws on empty buffer - getColor', async () => {
        await expect(getColor(Buffer.from(''))).rejects.toThrow();
    });

    it('throws on empty buffer - getPalette', async () => {
        await expect(getPalette(Buffer.from(''))).rejects.toThrow();
    });

    it('throws on invalid data', async () => {
        await expect(getColor(Buffer.from('not an image'))).rejects.toThrow();
    });
});
