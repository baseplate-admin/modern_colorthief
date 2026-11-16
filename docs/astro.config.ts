import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import starlight from '@astrojs/starlight';

// Read the Docs serves the built site from a sub-path (`/en/latest/`,
// `/en/stable/`, `/en/<pr-number>/`, ...), so every asset and internal link has
// to be emitted with that prefix. `.readthedocs.yml` derives it from the RTD
// build environment; locally it stays `/`.
const base = process.env.DOCS_BASE_PATH || '/';

// https://astro.build/config
export default defineConfig({
    site: 'https://modern-colorthief.readthedocs.io/',
    base,
    integrations: [
        react(),
        starlight({
            title: 'Modern Colorthief',
            description:
                'High-performance color palette extraction from images. Rust core with bindings for Python, Node.js, WASM, JVM, Ruby, and PHP.',
            defaultLocale: 'en',
            lastUpdated: true,
            head: [
                {
                    tag: 'meta',
                    attrs: { name: 'theme-color', content: '#4f46e5' },
                },
            ],
            social: [
                {
                    icon: 'github',
                    label: 'GitHub',
                    href: 'https://github.com/baseplate-admin/modern_colorthief',
                },
            ],
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
                        { label: 'Roadmaps', slug: 'roadmaps' },
                        { label: 'License', slug: 'license' },
                    ],
                },
            ],
        }),
    ],
});
