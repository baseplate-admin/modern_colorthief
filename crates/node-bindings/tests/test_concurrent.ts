import { describe, it, expect } from './test_compat';
import { getPalette, getColor } from '../index.js';
import { testImagePath, testImageBuffer } from './test_helper.js';

describe('Concurrent access', () => {
    it('concurrent color and palette calls', async () => {
        const results = await Promise.all([
            getColor(testImagePath()),
            getPalette(testImagePath(), 3),
            getColor(testImageBuffer()),
        ]);
        expect(results.length).toBe(3);
    });

    it('concurrent calls produce consistent results', async () => {
        const results = await Promise.all(
            Array.from({ length: 5 }, () => getColor(testImagePath())),
        );
        expect(results.every(r => r[0] === results[0][0] && r[1] === results[0][1] && r[2] === results[0][2])).toBe(true);
    });

    it('concurrent palette calls', async () => {
        const results = await Promise.all(
            Array.from({ length: 3 }, () => getPalette(testImagePath(), 5)),
        );
        expect(results.length).toBe(3);
        expect(results.every(r => r.length > 0)).toBe(true);
    });

    it('high concurrency stress test', async () => {
        const results = await Promise.all(
            Array.from({ length: 10 }, (_, i) =>
                i % 2 === 0 ? getColor(testImagePath()) : getPalette(testImagePath(), 5),
            ),
        );
        expect(results.length).toBe(10);
    });
});
