import { execFileSync } from 'node:child_process';
import { copyFileSync } from 'node:fs';
import { extname } from 'node:path';

const output = execFileSync(
    'cargo',
    ['build', '--release', '--locked', '--message-format=json'],
    { cwd: import.meta.dirname, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }
);

for (const line of output.trim().split('\n')) {
    const message = JSON.parse(line);
    if (message.reason === 'compiler-message' && message.message.rendered) {
        process.stderr.write(message.message.rendered);
    }
    if (message.reason !== 'compiler-artifact' || message.target.name !== 'modern_colorthief_node_gpu') {
        continue;
    }
    const library = message.filenames.find((file) => ['.so', '.dylib', '.dll'].includes(extname(file)));
    if (library) {
        copyFileSync(library, new URL('modern_colorthief_gpu.node', import.meta.url));
        process.exit(0);
    }
}

throw new Error('Cargo did not produce the Node GPU native library');
