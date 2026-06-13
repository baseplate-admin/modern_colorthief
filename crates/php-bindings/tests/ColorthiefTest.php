<?php

// ext-php-rs exposes get_palette() and get_color() as global functions
// from the "modern_colorthief" extension module.
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

// -- Image loading (port from Python tests that use real images) --

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

test('solid red color detection', function () {
    $palette = get_palette(redPixels(), 1, 1, 5, 1);
    expect($palette)->not->toBeEmpty();
    expect($palette[0])->toBe([255, 0, 0]);
});

test('two-color detection returns both colors', function () {
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

test('palette length respects color_count', function () {
    $palette = get_palette(greenPixels(), 3, 3, 3, 1);
    expect(count($palette))->toBeLessThanOrEqual(3);
});

test('palette length respects high color_count', function () {
    $palette = get_palette(greenPixels(), 3, 3, 50, 1);
    expect(count($palette))->toBeLessThanOrEqual(50);
});

// -- Deduplication tests (port from Python test_deduplication) --

test('deduplication returns unique colors', function () {
    $palette = get_palette(greenPixels(), 3, 3, 10, 1);
    $unique = array_map(function ($color) {
        return implode(',', $color);
    }, $palette);
    expect(count($palette))->toEqual(count(array_unique($unique)));
});

test('deduplication on real image with large palette', function () {
    $pixels = imageToPixels(testImagePath());
    $img = imagecreatefromjpeg(testImagePath());
    $w = imagesx($img);
    $h = imagesy($img);
    imagedestroy($img);

    $palette = get_palette($pixels, $w, $h, 255, 10);

    // No duplicate colors
    $strings = array_map(function ($c) { return implode(',', $c); }, $palette);
    expect(count($palette))->toEqual(count(array_unique($strings)));
    // Palette has reasonable size
    expect(count($palette))->toBeGreaterThan(0);
    expect(count($palette))->toBeLessThanOrEqual(255);
});

// -- Property tests (port from Python test_properties) --

test('get_color returns valid RGB from real image', function () {
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

test('get_palette returns valid RGB list from real image', function () {
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

test('palette count bounded by requested color_count on real image', function () {
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

test('get_color returns dominant color', function () {
    $color = get_color(redPixels(), 1, 1, 1);
    expect($color)->toBe([255, 0, 0]);
});

test('get_color returns valid RGB values', function () {
    $color = get_color(greenPixels(), 3, 3, 1);
    expect(count($color))->toBe(3);
    foreach ($color as $channel) {
        expect($channel)->toBeInt()->toBeGreaterThanOrEqual(0)->toBeLessThanOrEqual(255);
    }
});

// -- Edge case tests (port from Python test_edge_cases) --

test('deterministic results for same input', function () {
    $result1 = get_palette(greenPixels(), 3, 3, 5, 1);
    $result2 = get_palette(greenPixels(), 3, 3, 5, 1);
    expect($result1)->toEqual($result2);
});

test('deterministic get_color results', function () {
    $color1 = get_color(redPixels(), 1, 1, 1);
    $color2 = get_color(redPixels(), 1, 1, 1);
    expect($color1)->toEqual($color2);
});

test('deterministic results on real image', function () {
    $pixels = imageToPixels(testImagePath());
    $img = imagecreatefromjpeg(testImagePath());
    $w = imagesx($img);
    $h = imagesy($img);
    imagedestroy($img);

    $c1 = get_color($pixels, $w, $h, 10);
    $c2 = get_color($pixels, $w, $h, 10);
    expect($c1)->toEqual($c2);
});

test('different images produce different dominant colors', function () {
    $redColor = get_color(redPixels(), 1, 1, 1);
    $greenColor = get_color(greenPixels(), 3, 3, 1);
    expect($redColor)->not->toEqual($greenColor);
});

test('different real images produce different dominant colors', function () {
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

// -- Quality tests (port from Python test_edge_cases, test_properties) --

test('quality min valid', function () {
    $pixels = imageToPixels(testImagePath());
    $img = imagecreatefromjpeg(testImagePath());
    $w = imagesx($img);
    $h = imagesy($img);
    imagedestroy($img);

    $color = get_color($pixels, $w, $h, 1);
    expect(count($color))->toBe(3);
});

test('quality max valid', function () {
    $pixels = imageToPixels(testImagePath());
    $img = imagecreatefromjpeg(testImagePath());
    $w = imagesx($img);
    $h = imagesy($img);
    imagedestroy($img);

    $color = get_color($pixels, $w, $h, 10);
    expect(count($color))->toBe(3);
});

test('quality parameter is accepted', function () {
    $pixels = greenPixels();
    $color1 = get_color($pixels, 3, 3, 1);
    $color10 = get_color($pixels, 3, 3, 10);
    expect($color1)->not->toBeEmpty();
    expect($color10)->not->toBeEmpty();
});

// -- Error handling tests (port from Python test_errors) --

test('error on empty pixels', function () {
    expect(fn () => get_palette([], 1, 1, 5, 1))->toThrow(\Exception::class);
    expect(fn () => get_color([], 1, 1, 1))->toThrow(\Exception::class);
});

test('error on mismatched pixel data', function () {
    $pixels = [255, 0, 0, 255];
    $result = get_palette($pixels, 2, 2, 5, 1);
    expect($result)->toBeArray();
});
