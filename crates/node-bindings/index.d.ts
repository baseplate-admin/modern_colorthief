export function getPalette(
    image: string | Buffer,
    colorCount?: number,
    quality?: number,
): Promise<number[][]>;

export function getColor(
    image: string | Buffer,
    quality?: number,
): Promise<number[]>;
