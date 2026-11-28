struct Params {
    start: u32,
    length: u32,
    axis: u32,
    quality: u32,
}

const TILE_SIZE: u32 = 1024u;
const LANES: u32 = 64u;
const COLORS: u32 = 256u;
const SUMMARY_SIZE: u32 = 5u;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> samples: array<u32>;
@group(0) @binding(2) var<storage, read_write> scratch: array<u32>;
@group(0) @binding(3) var<storage, read_write> tiles: array<u32>;

var<workgroup> bins: array<atomic<u32>, 256>;

fn rgb(pixel: u32) -> vec3<u32> {
    return vec3<u32>(pixel & 255u, (pixel >> 8u) & 255u, (pixel >> 16u) & 255u);
}

fn pack_rgb(color: vec3<u32>) -> u32 {
    return color.r | (color.g << 8u) | (color.b << 16u);
}

fn channel(pixel: u32) -> u32 {
    return (pixel >> (params.axis * 8u)) & 255u;
}

@compute @workgroup_size(64)
fn sample_pixels(
    @builtin(global_invocation_id) id: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
) {
    for (var i = id.x; i < params.length; i += groups.x * LANES) {
        samples[i] = scratch[i * params.quality];
    }
}

@compute @workgroup_size(64)
fn summarize(@builtin(global_invocation_id) id: vec3<u32>) {
    let first = id.x * TILE_SIZE;
    if (first >= params.length) { return; }
    let end = min(first + TILE_SIZE, params.length);
    var low = vec3<u32>(255u);
    var high = vec3<u32>(0u);
    var sum = vec3<u32>(0u);

    for (var i = first; i < end; i += 1u) {
        let color = rgb(samples[params.start + i]);
        low = min(low, color);
        high = max(high, color);
        sum += color;
    }

    let offset = id.x * SUMMARY_SIZE;
    tiles[offset] = pack_rgb(low);
    tiles[offset + 1u] = pack_rgb(high);
    tiles[offset + 2u] = sum.r;
    tiles[offset + 3u] = sum.g;
    tiles[offset + 4u] = sum.b;
}

@compute @workgroup_size(64)
fn histogram(
    @builtin(workgroup_id) group: vec3<u32>,
    @builtin(local_invocation_index) lane: u32,
) {
    for (var value = lane; value < COLORS; value += LANES) {
        atomicStore(&bins[value], 0u);
    }
    workgroupBarrier();

    let first = group.x * TILE_SIZE;
    let end = min(first + TILE_SIZE, params.length);
    for (var i = first + lane; i < end; i += LANES) {
        atomicAdd(&bins[channel(samples[params.start + i])], 1u);
    }
    workgroupBarrier();

    for (var value = lane; value < COLORS; value += LANES) {
        tiles[group.x * COLORS + value] = atomicLoad(&bins[value]);
    }
}

@compute @workgroup_size(1)
fn scatter(@builtin(workgroup_id) group: vec3<u32>) {
    let first = group.x * TILE_SIZE;
    let end = min(first + TILE_SIZE, params.length);
    // Each tile owns its cursors. Walking it in order preserves equal-key order.
    for (var i = first; i < end; i += 1u) {
        let pixel = samples[params.start + i];
        let cursor = group.x * COLORS + channel(pixel);
        let offset = tiles[cursor];
        scratch[params.start + offset] = pixel;
        tiles[cursor] = offset + 1u;
    }
}
