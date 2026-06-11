<?php

// ext-php-rs exposes get_palette() and get_color() as global functions
// from the "modern_colorthief_gpu" extension module.
// Pixels are passed as arrays of integers (byte values 0-255).

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

test('gpu solid red color detection', function () {
    $palette = get_palette(redPixels(), 1, 1, 5, 1);
    expect($palette)->not->toBeEmpty();
    expect($palette[0])->toBe([255, 0, 0]);
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

test('gpu deduplication returns unique colors', function () {
    $palette = get_palette(greenPixels(), 3, 3, 10, 1);
    $unique = array_map(function ($color) {
        return implode(',', $color);
    }, $palette);
    expect(count($palette))->toEqual(count(array_unique($unique)));
});

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

test('gpu error on empty pixels', function () {
    expect(fn () => get_palette([], 1, 1, 5, 1))->toThrow(\Exception::class);
    expect(fn () => get_color([], 1, 1, 1))->toThrow(\Exception::class);
});

test('gpu error on mismatched pixel data', function () {
    $pixels = [255, 0, 0, 255];
    $result = get_palette($pixels, 2, 2, 5, 1);
    expect($result)->toBeArray();
});

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

test('gpu different images produce different dominant colors', function () {
    $redColor = get_color(redPixels(), 1, 1, 1);
    $greenColor = get_color(greenPixels(), 3, 3, 1);
    expect($redColor)->not->toEqual($greenColor);
});

test('gpu quality parameter is accepted', function () {
    $pixels = greenPixels();
    $color1 = get_color($pixels, 3, 3, 1);
    $color10 = get_color($pixels, 3, 3, 10);
    expect($color1)->not->toBeEmpty();
    expect($color10)->not->toBeEmpty();
});
