import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import starlight from '@astrojs/starlight';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Read the Docs serves the built site from a sub-path (`/en/latest/`,
// `/en/stable/`, `/en/<pr-number>/`, ...), so every asset and internal link has
// to be emitted with that prefix. `.readthedocs.yml` derives it from the RTD
// build environment; locally it stays `/`.
const base = process.env.DOCS_BASE_PATH || '/';

const customCss = readFileSync(join(__dirname, 'public/custom.css'), 'utf-8');

// Version the JS and WASM together so a cached copy of either file cannot be
// paired with a different build after Read the Docs publishes an update.
const webgpuBundle = join(__dirname, 'public/wasm-webgpu/modern_colorthief_webgpu');
const webgpuFiles = [webgpuBundle + '.js', webgpuBundle + '_bg.wasm'];
const webgpuHash = createHash('sha256');
for (const file of webgpuFiles) {
    if (existsSync(file)) webgpuHash.update(readFileSync(file));
}
const webgpuRevision = webgpuHash.digest('hex').slice(0, 16);

// https://astro.build/config
export default defineConfig({
    site: 'https://modern-colorthief.readthedocs.io/',
    base,
    vite: {
        define: {
            'import.meta.env.PUBLIC_WEBGPU_REVISION': JSON.stringify(webgpuRevision),
        },
    },
    integrations: [
        react(),
        starlight({
            title: 'Modern Colorthief',
            description:
                'High-performance color palette extraction from images. Rust core with bindings for Python, Node.js, WASM, JVM, Ruby, and PHP.',
            defaultLocale: 'en',
            lastUpdated: true,
            expressiveCode: {
                styleOverrides: { borderRadius: '0.5rem' },
            },
            head: [
                {
                    tag: 'meta',
                    attrs: { name: 'theme-color', content: '#18181b' },
                },
                {
                    tag: 'style',
                    content: customCss,
                },
            ],
            social: [
                {
                    icon: 'github',
                    label: 'GitHub',
                    href: 'https://github.com/baseplate-admin/modern_colorthief',
                },
            ],
            logo: { src: './public/favicon.svg', alt: '' },
            favicon: '/favicon.svg',
            editLink: {
                baseUrl: 'https://github.com/baseplate-admin/modern_colorthief',
            },
            sidebar: [
                {
                    label: 'Introduction',
                    items: [
                        { label: 'Overview', slug: 'index' },
                        { label: 'Installation', slug: 'installation' },
                        { label: 'Quick Start', slug: 'quickstart' },
                        { label: 'Live Demo', slug: 'demo' },
                    ],
                },
                {
                    label: 'Language Guides',
                    items: [
                        { label: 'Python', slug: 'languages/python' },
                        { label: 'Ruby', slug: 'languages/ruby' },
                        { label: 'Node.js', slug: 'languages/nodejs' },
                        { label: 'Java / Kotlin', slug: 'languages/java' },
                        { label: 'PHP', slug: 'languages/php' },
                        { label: 'WebAssembly', slug: 'languages/wasm' },
                    ],
                },
                {
                    label: 'GPU',
                    items: [
                        { label: 'WebGPU', slug: 'webgpu' },
                    ],
                },
                {
                    label: 'Reference',
                    items: [
                        { label: 'API', slug: 'api' },
                        { label: 'Command Line', slug: 'cli' },
                        {
                            label: 'Differences vs. colorthief',
                            slug: 'differences',
                        },
                        { label: 'Benchmarks', slug: 'benchmarks' },
                        { label: 'Median Cut (MMCQ)', slug: 'mmcq' },
                        { label: 'LLMs & AI Assistants', slug: 'llms' },
                        { label: 'Roadmaps', slug: 'roadmaps' },
                        { label: 'License', slug: 'license' },
                    ],
                },
            ],
        }),
    ],
});
