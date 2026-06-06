import { describe, it, expect } from 'vitest';
import { getPalette, getColor } from '../index.js';
import { readFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TEST_IMAGE = resolve(__dirname, 'test.jpg');

describe('API surface', () => {
    it('getPalette is a function', () => {
        expect(typeof getPalette).toBe('function');
    });

    it('getColor is a function', () => {
        expect(typeof getColor).toBe('function');
    });

    it('getPalette accepts file path', async () => {
        const palette = await getPalette(TEST_IMAGE, 10, 10);
        expect(Array.isArray(palette)).toBe(true);
        expect(palette.length).toBeGreaterThan(0);
    });

    it('getColor accepts file path', async () => {
        const color = await getColor(TEST_IMAGE, 10);
        expect(Array.isArray(color)).toBe(true);
        expect(color.length).toBe(3);
    });

    it('getPalette accepts buffer', async () => {
        const buffer = readFileSync(TEST_IMAGE);
        const palette = await getPalette(buffer, 10, 10);
        expect(Array.isArray(palette)).toBe(true);
    });

    it('getColor accepts buffer', async () => {
        const buffer = readFileSync(TEST_IMAGE);
        const color = await getColor(buffer, 10);
        expect(Array.isArray(color)).toBe(true);
    });
});
