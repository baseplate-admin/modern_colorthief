import * as esbuild from 'esbuild';

esbuild.build({
    entryPoints: ['index.ts'],
    bundle: true,
    minify: true,
    platform: 'node',
    target: 'node20',
    outfile: 'dist/index.js',
    external: ['./native.js', 'sharp'],
    sourcemap: true,
}).catch(() => process.exit(1));
