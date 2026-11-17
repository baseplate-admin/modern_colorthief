export function getPaletteGpu(
    pixels: Buffer,
    width: number,
    height: number,
    colorCount?: number,
    quality?: number,
): Promise<number[][]>;

export function getColorGpu(
    pixels: Buffer,
    width: number,
    height: number,
    quality?: number,
): Promise<number[]>;
