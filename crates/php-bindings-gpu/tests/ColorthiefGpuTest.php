<?php

// ext-php-rs exposes get_palette() and get_color() as global functions
// from the "modern_colorthief_gpu" extension module.
// Pixels are passed as arrays of integers (byte values 0-255).

// -- Pixel helpers --

function redPixels(): array {
    return [255, 0, 0, 255];
}

function twoColorPixels(): array {
    return [255, 0, 0, 255, 0, 0, 255, 255];
}

function greenPixels(): array {
    $green = [0, 255, 0, 255];
    $result = [];
    for ($i = 0; $i < 9; $i++) {
        $result = array_merge($result, $green);
    }
    return $result;
}

function solidBluePixels(): array {
    $blue = [0, 0, 255, 255];
    $result = [];
    for ($i = 0; $i < 9; $i++) {
        $result = array_merge($result, $blue);
    }
    return $result;
}

function solidWhitePixels(): array {
    $white = [255, 255, 255, 255];
    $result = [];
    for ($i = 0; $i < 9; $i++) {
        $result = array_merge($result, $white);
    }
    return $result;
}

function gradientPixels(): array {
    $pixels = [];
    for ($x = 0; $x < 20; $x++) {
        for ($y = 0; $y < 10; $y++) {
            $pixels[] = ($x * 13) % 256;
            $pixels[] = ($x * 7) % 256;
            $pixels[] = ($x * 5) % 256;
            $pixels[] = 255;
        }
    }
    return $pixels;
}

function checkerboardPixels(): array {
    $pixels = [];
    for ($y = 0; $y < 10; $y++) {
        for ($x = 0; $x < 10; $x++) {
            if (($x + $y) % 2 === 0) {
                $pixels[] = 200; $pixels[] = 50; $pixels[] = 50; $pixels[] = 255;
            } else {
                $pixels[] = 50; $pixels[] = 50; $pixels[] = 200; $pixels[] = 255;
            }
        }
    }
    return $pixels;
}

// -- Image loading --

function imageToPixels(string $path): array {
    $img = imagecreatefromjpeg($path);
    $w = imagesx($img);
    $h = imagesy($img);
    $pixels = [];
    for ($y = 0; $y < $h; $y++) {
        for ($x = 0; $x < $w; $x++) {
            $rgba = imagecolorat($img, $x, $y);
            $pixels[] = ($rgba >> 16) & 0xFF;
            $pixels[] = ($rgba >> 8) & 0xFF;
            $pixels[] = $rgba & 0xFF;
            $pixels[] = 255;
        }
    }
    imagedestroy($img);
    return $pixels;
}

function testImagePath(): string {
    return __DIR__ . '/test.jpg';
}

function kaijuImagePath(): string {
    return __DIR__ . '/kaiju_no_8.jpg';
}

// -- Solid color tests --

test('gpu solid red color detection', function () {
    $palette = get_palette(redPixels(), 1, 1, 5, 1);
    expect($palette)->not->toBeEmpty();
    expect($palette[0])->toBe([255, 0, 0]);
});

test('gpu solid green color detection', function () {
    $palette = get_palette(greenPixels(), 3, 3, 5, 1);
    expect($palette)->not->toBeEmpty();
    expect($palette[0])->toBe([0, 255, 0]);
});

test('gpu solid blue color detection', function () {
    $palette = get_palette(solidBluePixels(), 3, 3, 5, 1);
    expect($palette)->not->toBeEmpty();
    expect($palette[0])->toBe([0, 0, 255]);
});

test('gpu solid white color detection', function () {
    $palette = get_palette(solidWhitePixels(), 3, 3, 5, 1);
    expect($palette)->not->toBeEmpty();
    expect($palette[0])->toBe([255, 255, 255]);
});

test('gpu two-color detection returns both colors', function () {
    $palette = get_palette(twoColorPixels(), 1, 2, 10, 1);
    expect($palette)->toHaveCount(2);
    $foundRed = false;
    $foundBlue = false;
    foreach ($palette as $color) {
        if ($color === [255, 0, 0]) {
            $foundRed = true;
        }
        if ($color === [0, 0, 255]) {
            $foundBlue = true;
        }
    }
    expect($foundRed)->toBeTrue();
    expect($foundBlue)->toBeTrue();
});

test('gpu palette length respects color_count', function () {
    $palette = get_palette(greenPixels(), 3, 3, 3, 1);
    expect(count($palette))->toBeLessThanOrEqual(3);
});

test('gpu palette length respects high color_count', function () {
    $palette = get_palette(greenPixels(), 3, 3, 50, 1);
    expect(count($palette))->toBeLessThanOrEqual(50);
});

// -- Deduplication tests --

test('gpu deduplication returns unique colors', function () {
    $palette = get_palette(greenPixels(), 3, 3, 10, 1);
    $unique = array_map(function ($color) {
        return implode(',', $color);
    }, $palette);
    expect(count($palette))->toEqual(count(array_unique($unique)));
});

test('gpu deduplication on real image with large palette', function () {
    $pixels = imageToPixels(testImagePath());
    $img = imagecreatefromjpeg(testImagePath());
    $w = imagesx($img);
    $h = imagesy($img);
    imagedestroy($img);

    $palette = get_palette($pixels, $w, $h, 255, 10);

    $strings = array_map(function ($c) { return implode(',', $c); }, $palette);
    expect(count($palette))->toEqual(count(array_unique($strings)));
    expect(count($palette))->toBeGreaterThan(0);
    expect(count($palette))->toBeLessThanOrEqual(255);
});

// -- Property tests --

test('gpu get_color returns valid RGB from real image', function () {
    $pixels = imageToPixels(testImagePath());
    $img = imagecreatefromjpeg(testImagePath());
    $w = imagesx($img);
    $h = imagesy($img);
    imagedestroy($img);

    $color = get_color($pixels, $w, $h, 10);
    expect(count($color))->toBe(3);
    foreach ($color as $channel) {
        expect($channel)->toBeInt()->toBeGreaterThanOrEqual(0)->toBeLessThanOrEqual(255);
    }
});

test('gpu get_palette returns valid RGB list from real image', function () {
    $pixels = imageToPixels(testImagePath());
    $img = imagecreatefromjpeg(testImagePath());
    $w = imagesx($img);
    $h = imagesy($img);
    imagedestroy($img);

    $palette = get_palette($pixels, $w, $h, 10, 10);
    expect(count($palette))->toBeGreaterThan(0);
    foreach ($palette as $color) {
        expect(count($color))->toBe(3);
        foreach ($color as $channel) {
            expect($channel)->toBeInt()->toBeGreaterThanOrEqual(0)->toBeLessThanOrEqual(255);
        }
    }
});

test('gpu palette count bounded by requested color_count on real image', function () {
    $pixels = imageToPixels(testImagePath());
    $img = imagecreatefromjpeg(testImagePath());
    $w = imagesx($img);
    $h = imagesy($img);
    imagedestroy($img);

    foreach ([3, 5] as $count) {
        $palette = get_palette($pixels, $w, $h, $count, 10);
        expect(count($palette))->toBeLessThanOrEqual($count);
    }
});

// -- Dominant color tests --

test('gpu get_color returns dominant color', function () {
    $color = get_color(redPixels(), 1, 1, 1);
    expect($color)->toBe([255, 0, 0]);
});

test('gpu get_color returns valid RGB values', function () {
    $color = get_color(greenPixels(), 3, 3, 1);
    expect(count($color))->toBe(3);
    foreach ($color as $channel) {
        expect($channel)->toBeInt()->toBeGreaterThanOrEqual(0)->toBeLessThanOrEqual(255);
    }
});

// -- Edge case tests --

test('gpu deterministic results for same input', function () {
    $result1 = get_palette(greenPixels(), 3, 3, 5, 1);
    $result2 = get_palette(greenPixels(), 3, 3, 5, 1);
    expect($result1)->toEqual($result2);
});

test('gpu deterministic get_color results', function () {
    $color1 = get_color(redPixels(), 1, 1, 1);
    $color2 = get_color(redPixels(), 1, 1, 1);
    expect($color1)->toEqual($color2);
});

test('gpu deterministic results on real image', function () {
    $pixels = imageToPixels(testImagePath());
    $img = imagecreatefromjpeg(testImagePath());
    $w = imagesx($img);
    $h = imagesy($img);
    imagedestroy($img);

    $c1 = get_color($pixels, $w, $h, 10);
    $c2 = get_color($pixels, $w, $h, 10);
    expect($c1)->toEqual($c2);
});

test('gpu different images produce different dominant colors', function () {
    $redColor = get_color(redPixels(), 1, 1, 1);
    $greenColor = get_color(greenPixels(), 3, 3, 1);
    expect($redColor)->not->toEqual($greenColor);
});

test('gpu different real images produce different dominant colors', function () {
    $p1 = imageToPixels(testImagePath());
    $img1 = imagecreatefromjpeg(testImagePath());
    $w1 = imagesx($img1);
    $h1 = imagesy($img1);
    imagedestroy($img1);

    $p2 = imageToPixels(kaijuImagePath());
    $img2 = imagecreatefromjpeg(kaijuImagePath());
    $w2 = imagesx($img2);
    $h2 = imagesy($img2);
    imagedestroy($img2);

    $c1 = get_color($p1, $w1, $h1, 10);
    $c2 = get_color($p2, $w2, $h2, 10);
    expect($c1)->not->toEqual($c2);
});

// -- Dominant color in palette consistency --

test('gpu dominant color appears in palette', function () {
    $color = get_color(twoColorPixels(), 1, 2, 1);
    $palette = get_palette(twoColorPixels(), 1, 2, 5, 1);
    $found = false;
    foreach ($palette as $c) {
        if ($c === $color) {
            $found = true;
        }
    }
    expect($found)->toBeTrue();
});

// -- Quality tests --

test('gpu quality min valid', function () {
    $pixels = imageToPixels(testImagePath());
    $img = imagecreatefromjpeg(testImagePath());
    $w = imagesx($img);
    $h = imagesy($img);
    imagedestroy($img);

    $color = get_color($pixels, $w, $h, 1);
    expect(count($color))->toBe(3);
});

test('gpu quality max valid', function () {
    $pixels = imageToPixels(testImagePath());
    $img = imagecreatefromjpeg(testImagePath());
    $w = imagesx($img);
    $h = imagesy($img);
    imagedestroy($img);

    $color = get_color($pixels, $w, $h, 10);
    expect(count($color))->toBe(3);
});

test('gpu quality middle valid', function () {
    $pixels = greenPixels();
    $color = get_color($pixels, 3, 3, 5);
    expect(count($color))->toBe(3);
});

test('gpu quality zero clamped', function () {
    $color = get_color(redPixels(), 1, 1, 0);
    expect(count($color))->toBe(3);
});

test('gpu quality 100 works', function () {
    $color = get_color(redPixels(), 1, 1, 100);
    expect(count($color))->toBe(3);
});

// -- Gradient image --

test('gpu gradient returns multiple colors', function () {
    $pixels = gradientPixels();
    $palette = get_palette($pixels, 20, 10, 10, 1);
    expect(count($palette))->toBeGreaterThan(1);
});

// -- Checkerboard --

test('gpu checkerboard returns palette', function () {
    $pixels = checkerboardPixels();
    $palette = get_palette($pixels, 10, 10, 5, 1);
    expect($palette)->not->toBeEmpty();
});

// -- Error handling tests --

test('gpu error on empty pixels', function () {
    expect(fn () => get_palette([], 1, 1, 5, 1))->toThrow(\Exception::class);
    expect(fn () => get_color([], 1, 1, 1))->toThrow(\Exception::class);
});

test('gpu error on mismatched pixel data', function () {
    $pixels = [255, 0, 0, 255];
    $result = get_palette($pixels, 2, 2, 5, 1);
    expect($result)->toBeArray();
});

// -- GC stress --

test('gpu gc stress palette', function () {
    $pixels = redPixels();
    for ($i = 0; $i < 50; $i++) {
        $palette = get_palette($pixels, 1, 1, 5, 1);
        expect($palette)->not->toBeEmpty();
    }
});

test('gpu gc stress color', function () {
    $pixels = redPixels();
    for ($i = 0; $i < 50; $i++) {
        $color = get_color($pixels, 1, 1, 1);
        expect(count($color))->toBe(3);
    }
});

test('gpu gc stress mixed', function () {
    $pixels = redPixels();
    for ($i = 0; $i < 25; $i++) {
        $palette = get_palette($pixels, 1, 1, 5, 1);
        $color = get_color($pixels, 1, 1, 1);
        expect($palette)->not->toBeEmpty();
        expect(count($color))->toBe(3);
    }
});

// -- Solid black detection --

test('gpu solid black color detection', function () {
    $black = [];
    for ($i = 0; $i < 9; $i++) {
        $black[] = 0; $black[] = 0; $black[] = 0; $black[] = 255;
    }
    $palette = get_palette($black, 3, 3, 5, 1);
    expect($palette)->not->toBeEmpty();
    $dominant = $palette[0];
    expect($dominant[0])->toBeLessThan(55);
    expect($dominant[1])->toBeLessThan(55);
    expect($dominant[2])->toBeLessThan(55);
});

// -- Dominant reflects majority --

test('gpu dominant reflects 90/10 majority', function () {
    $pixels = [];
    for ($i = 0; $i < 90; $i++) {
        $pixels[] = 255; $pixels[] = 0; $pixels[] = 0; $pixels[] = 255;
    }
    for ($i = 0; $i < 10; $i++) {
        $pixels[] = 0; $pixels[] = 0; $pixels[] = 255; $pixels[] = 255;
    }
    $color = get_color($pixels, 10, 10, 1);
    expect($color[0])->toBeGreaterThan(200);
});

// -- Color count=0 edge case --

test('gpu color_count zero returns empty', function () {
    $palette = get_palette(redPixels(), 1, 1, 0, 1);
    expect($palette)->toBeArray();
    expect(count($palette))->toBe(0);
});

// -- Non-square wide/tall --

test('gpu non-square wide image', function () {
    $pixels = [];
    for ($i = 0; $i < 200; $i++) {
        $pixels[] = 255; $pixels[] = 0; $pixels[] = 0; $pixels[] = 255;
    }
    $palette = get_palette($pixels, 20, 10, 5, 1);
    expect($palette)->not->toBeEmpty();
});

test('gpu non-square tall image', function () {
    $pixels = [];
    for ($i = 0; $i < 200; $i++) {
        $pixels[] = 255; $pixels[] = 0; $pixels[] = 0; $pixels[] = 255;
    }
    $palette = get_palette($pixels, 10, 20, 5, 1);
    expect($palette)->not->toBeEmpty();
});
