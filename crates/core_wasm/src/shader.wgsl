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
@group(0) @binding(2) var<storage, read>       chunk_colors: array<vec3<f32>>;
@group(0) @binding(3) var<storage, read_write> unique:       array<vec3<f32>>;
@group(0) @binding(4) var<storage, read_write> ucount:       array<u32>;

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

@compute @workgroup_size(64) @entry_point("dedup")
fn dedup(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= params.num_chunks) { return; }
    let col = chunk_colors[idx];
    var cnt: u32 = ucount[0];
    var found: bool = false;
    for (var i: u32 = 0u; i < cnt; i = i + 1u) {
        let e = unique[i];
        if (abs(e.x - col.x) < 1.5 && abs(e.y - col.y) < 1.5 && abs(e.z - col.z) < 1.5) {
            found = true;
            break;
        }
    }
    if (!found && cnt < params.color_count) {
        unique[cnt] = col;
        ucount[0] = cnt + 1u;
    }
}
