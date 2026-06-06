import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';
import { readFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TEST_IMAGE = resolve(__dirname, 'test.jpg');

describe('Input types', () => {
    it('str input getColor', async () => {
        const c = await getColor(TEST_IMAGE);
        expect(c.length).toBe(3);
    });

    it('buffer input getColor', async () => {
        const buffer = readFileSync(TEST_IMAGE);
        const c = await getColor(buffer);
        expect(c.length).toBe(3);
    });

    it('str input getPalette', async () => {
        const p = await getPalette(TEST_IMAGE);
        expect(p.length).toBeGreaterThan(0);
    });

    it('buffer input getPalette', async () => {
        const buffer = readFileSync(TEST_IMAGE);
        const p = await getPalette(buffer);
        expect(p.length).toBeGreaterThan(0);
    });

    it('buffer not mutated', async () => {
        const buffer = readFileSync(TEST_IMAGE);
        const copy = Buffer.from(buffer);
        await getColor(buffer);
        await getPalette(buffer);
        expect(buffer.equals(copy)).toBe(true);
    });

    it('path and buffer produce same dominant color', async () => {
        const colorFromPath = await getColor(TEST_IMAGE);
        const colorFromBuffer = await getColor(readFileSync(TEST_IMAGE));
        expect(colorFromPath).toEqual(colorFromBuffer);
    });

    it('path and buffer produce same palette', async () => {
        const paletteFromPath = await getPalette(TEST_IMAGE, 10);
        const paletteFromBuffer = await getPalette(readFileSync(TEST_IMAGE), 10);
        expect(paletteFromPath).toEqual(paletteFromBuffer);
    });
});
