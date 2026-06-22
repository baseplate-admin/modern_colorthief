struct Params {
    width:       u32,
    height:      u32,
    color_count: u32,
    quality:     u32,
    num_pixels:  u32,
    num_chunks:  u32,
}

@group(0) @binding(0) var<storage, read>       pixels:       array<vec4<f32>>;
@group(0) @binding(1) var<uniform>             params:       Params;
@group(0) @binding(2) var<storage, read_write> chunk_colors: array<vec3<f32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.x;
    if (c >= params.num_chunks) { return; }
    let ppc = (params.num_pixels + params.num_chunks - 1u) / params.num_chunks;
    let s = c * ppc;
    let e = min(s + ppc, params.num_pixels);
    let step = max(params.quality, 1u);
    var r: f32 = 0.0;
    var g: f32 = 0.0;
    var b: f32 = 0.0;
    var n: u32 = 0u;
    for (var i: u32 = s; i < e; i = i + step) {
        let p = pixels[i];
        r = r + p.r;
        g = g + p.g;
        b = b + p.b;
        n = n + 1u;
    }
    if (n > 0u) {
        chunk_colors[c] = vec3<f32>(r / f32(n), g / f32(n), b / f32(n));
    }
}
