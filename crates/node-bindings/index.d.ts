export function getPalette(
    pixels: Buffer,
    width: number,
    height: number,
    colorCount?: number,
    quality?: number,
): Promise<number[][]>;

export function getColor(
    pixels: Buffer,
    width: number,
    height: number,
    quality?: number,
): Promise<number[]>;

export function getPaletteSync(
    pixels: Buffer,
    width: number,
    height: number,
    colorCount?: number,
    quality?: number,
): number[][];

export function getColorSync(
    pixels: Buffer,
    width: number,
    height: number,
    quality?: number,
): number[];
