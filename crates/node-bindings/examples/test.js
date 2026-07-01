import { getPalette, getColor } from '../index.js';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const imagePath = `${__dirname}/test.jpg`;

const palette = await getPalette(imagePath, 10, 10);
console.log('Palette:', palette);

const color = await getColor(imagePath, 10);
console.log('Dominant color:', color);
