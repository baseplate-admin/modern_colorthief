import { describe, it, expect } from 'vitest';

describe('WASM API', () => {
    it('should export getPalette function', async () => {
        const mod = await import('../src/lib');
        expect(typeof mod.getPalette).toBe('function');
    });

    it('should export getColor function', async () => {
        const mod = await import('../src/lib');
        expect(typeof mod.getColor).toBe('function');
    });

    it('should export decodeImage function', async () => {
        const mod = await import('../src/lib');
        expect(typeof mod.decodeImage).toBe('function');
    });
});
