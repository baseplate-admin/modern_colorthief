const { createRequire } = require('module');
const require = createRequire(import.meta.url);
const { platform } = require('os');
const { join, dirname } = require('path');
const { fileURLToPath } = require('url');

const __dirname = dirname(fileURLToPath(import.meta.url));

const NAME = 'modern_colorthief';

function getSuffix() {
    const p = platform();
    if (p === 'linux') return 'gnu-x64-linux';
    if (p === 'darwin') return 'apple-x64-darwin';
    if (p === 'win32') return 'msvc-x64-windows';
    return '';
}

function resolve() {
    const suffix = getSuffix();
    const candidates = [
        join(__dirname, 'artifacts', `${NAME}-${suffix}.node`),
        join(__dirname, 'artifacts', `${NAME}.node`),
        join(__dirname, `${NAME}-${suffix}.node`),
        join(__dirname, `${NAME}.node`),
    ];
    for (const candidate of candidates) {
        try {
            return require.resolve(candidate);
        } catch { /* try next */ }
    }
    throw new Error(`Cannot find native binding ${NAME}. Searched: ${candidates.join(', ')}`);
}

module.exports = require(resolve());
