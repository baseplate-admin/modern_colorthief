import { getPalette, getColor } from '@modern_colorthief/wasm';

const response = await fetch(new URL('test.jpg', import.meta.url));
const buffer = await response.arrayBuffer();

const palette = await getPalette(buffer, 10, 10);
console.log('Palette:', palette);

const color = await getColor(buffer, 10);
console.log('Dominant color:', color);
