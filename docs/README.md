# Modern Colorthief Docs

Astro + Starlight documentation site for [Modern Colorthief](https://github.com/baseplate-admin/modern_colorthief).

## Stack

- [Astro](https://astro.build/) 7 — static site generator
- [Starlight](https://starlight.astro.build/) 0.43 — docs theme
- [Pagefind](https://pagefind.app/) — full-text search (built-in to Starlight)
- [modern-colorthief-wasm](https://github.com/baseplate-admin/modern_colorthief/tree/main/crates/wasm-bindings) — interactive demo widget

## Development

```bash
npm install
npm run dev       # start dev server at http://localhost:4321
npm run build     # production build to dist/
npm run preview   # preview the production build
```

## Structure

```
src/
  content.config.ts          # Starlight content collection
  components/
    PaletteDemo.astro        # Interactive WASM demo widget
  content/
    docs/
      index.mdx              # Home / overview
      installation.mdx
      quickstart.mdx
      api.mdx
      cli.mdx
      differences.mdx
      benchmarks.mdx
      mmcq.mdx
      roadmaps.mdx
      license.mdx
      languages/
        python.mdx
        ruby.mdx
        nodejs.mdx
        java.mdx
        php.mdx
        wasm.mdx
```

## Interactive demo

The home page includes a `<PaletteDemo />` widget that loads the WASM build of Modern Colorthief in the browser. It supports:

- Drag-and-drop image upload
- Paste an image URL
- Click a swatch to copy its hex code

The WASM bundle is committed at `crates/wasm-bindings/pkg/` and linked via a `file:` dependency in `package.json`. Rebuild it with:

```bash
cd ../crates/wasm-bindings
wasm-pack build --release --target web
```
