import { createRequire } from 'node:module';
import { platform, arch } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const NAME = 'modern_colorthief_gpu';

function getSuffix(): string {
    const p = platform();
    const a = arch();
    if (p === 'linux') return a === 'arm64' ? 'linux-arm64-gnu' : 'linux-x64-gnu';
    if (p === 'darwin') return a === 'arm64' ? 'darwin-arm64' : 'darwin-x64';
    if (p === 'win32') return 'win32-x64-msvc';
    return '';
}

function resolveNativePath(): string {
    const suffix = getSuffix();
    const basenames = [
        `artifacts/${NAME}.${suffix}.node`,
        `artifacts/${NAME}.node`,
        `${NAME}.${suffix}.node`,
        `${NAME}.node`,
    ];
    const bases = [
        __dirname,
        join(__dirname, '..'),
        join(__dirname, '..', '..'),
    ];
    const candidates = bases.flatMap((base) =>
        basenames.map((name) => join(base, name)),
    );
    for (const candidate of candidates) {
        try {
            return require.resolve(candidate);
        } catch {
            // try next
        }
    }
    throw new Error(
        `Cannot find native GPU binding ${NAME}. Searched: ${candidates.join(', ')}`,
    );
}

const native = require(resolveNativePath()) as {
    getPaletteGpu: (
        pixels: Buffer,
        width: number,
        height: number,
        colorCount?: number,
        quality?: number,
    ) => Promise<number[][]>;
    getColorGpu: (
        pixels: Buffer,
        width: number,
        height: number,
        quality?: number,
    ) => Promise<number[]>;
};

export const getPaletteGpu: typeof native.getPaletteGpu = native.getPaletteGpu;
export const getColorGpu: typeof native.getColorGpu = native.getColorGpu;
