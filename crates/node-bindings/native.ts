import { createRequire } from 'node:module';
import { platform } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const NAME = 'modern_colorthief';

function getSuffix(): string {
    const p = platform();
    if (p === 'linux') return 'gnu-x64-linux';
    if (p === 'darwin') return 'apple-x64-darwin';
    if (p === 'win32') return 'msvc-x64-windows';
    return '';
}

function resolveNativePath(): string {
    const suffix = getSuffix();
    const basenames = [
        `artifacts/${NAME}-${suffix}.node`,
        `artifacts/${NAME}.node`,
        `${NAME}-${suffix}.node`,
        `${NAME}.node`,
    ];
    // Search from __dirname and parent directories (handles dist/ output)
    const bases = [__dirname, join(__dirname, '..'), join(__dirname, '..', '..')];
    const candidates = bases.flatMap(base => basenames.map(name => join(base, name)));
    for (const candidate of candidates) {
        try {
            return require.resolve(candidate);
        } catch {
            // try next
        }
    }
    throw new Error(`Cannot find native binding ${NAME}. Searched: ${candidates.join(', ')}`);
}

const native = require(resolveNativePath()) as {
    getPalette: (pixels: Buffer, width: number, height: number, colorCount: number, quality: number) => number[][];
    getColor: (pixels: Buffer, width: number, height: number, quality: number) => number[];
};

export const getPalette = native.getPalette;
export const getColor = native.getColor;
