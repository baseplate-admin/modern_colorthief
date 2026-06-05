import argparse
import sys
from modern_colorthief import get_color, get_palette


def rgb_to_hex(rgb: tuple[int, int, int]) -> str:
    """Convert RGB tuple to hex string."""
    return "#{:02x}{:02x}{:02x}".format(rgb[0], rgb[1], rgb[2])


def main():
    parser = argparse.ArgumentParser(
        prog="modern_colorthief",
        description="Get dominant color or palette from an image.",
    )
    parser.add_argument("file", help="Path to the image file")

    parser.add_argument(
        "--palette",
        action="store_true",
        help="Get the palette of dominant colors instead of a single dominant color",
    )
    parser.add_argument(
        "--quality",
        type=int,
        default=10,
        help="Quality (default: 10). Higher is faster but less accurate.",
    )
    parser.add_argument(
        "--count",
        type=int,
        default=5,
        help="Color count for palette (default: 5). Only used with --palette.",
    )

    args = parser.parse_args()

    try:
        if args.palette:
            colors = get_palette(
                args.file, color_count=args.count, quality=args.quality
            )
            for color in colors:
                print(rgb_to_hex(color))
        else:
            color = get_color(args.file, quality=args.quality)
            print(rgb_to_hex(color))

    except Exception as e:
        print(f"Error processing image: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
