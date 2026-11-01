// One pass of a box's backdrop blur (backdrop_blur.rs). A pass draws part of
// one canvas from part of another: each texel it draws sums up to ten linear
// reads of the source, each with its own place and weight. A halving pass
// reads the two or four texels a texel covers. A blur pass reads a Gaussian
// along one axis, two of its texels to a read. The last pass applies CSS's
// saturate.
//
// Places are device pixels from the box's top left corner. Every read folds
// about the box's edges, so the blur sees the box mirrored past them, and
// stays inside the texels of the source that hold this frame's picture.
//
// Written with WebGPU conventions (+Y up, texture rows from the top); the
// WebGL2 lowering adjusts the coordinate space.

struct Pass {
    // The device pixels one source texel spans, on each axis.
    source_texel: vec2<f32>,
    // How many of the source's texels hold this frame's picture.
    source_used: vec2<f32>,
    // The source canvas's whole size, in texels.
    source_size: vec2<f32>,
    // The device pixels one texel this pass draws spans.
    target_texel: vec2<f32>,
    // How many texels this pass draws, which is its viewport.
    target_used: vec2<f32>,
    // The box's size in device pixels, where reads fold.
    extent: vec2<f32>,
    // CSS's saturate amount: 1 on every pass but the last.
    saturate: f32,
    // How many of `taps` this pass reads.
    tap_count: u32,
    // Each read: x and y in device pixels from the drawn texel's center, then
    // its weight.
    taps: array<vec4<f32>, 10>,
};

@group(0) @binding(0) var<uniform> blur: Pass;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var source_sampler: sampler;

struct Varyings {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_blur(@builtin(vertex_index) index: u32) -> Varyings {
    // (0, 0), (2, 0) and (0, 2): a triangle whose middle covers the viewport.
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: Varyings;
    out.clip = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}

// A place folded into [0, edge], as mirrors at 0 and at the edge repeat it.
fn fold(place: f32, edge: f32) -> f32 {
    let period = 2.0 * edge;
    let wrapped = place - period * floor(place / period);
    return select(wrapped, period - wrapped, wrapped > edge);
}

fn read(place: vec2<f32>) -> vec4<f32> {
    let folded = vec2<f32>(fold(place.x, blur.extent.x), fold(place.y, blur.extent.y));
    let texel = clamp(
        folded / blur.source_texel,
        vec2<f32>(0.5),
        blur.source_used - vec2<f32>(0.5),
    );
    return textureSampleLevel(source, source_sampler, texel / blur.source_size, 0.0);
}

@fragment
fn fs_blur(in: Varyings) -> @location(0) vec4<f32> {
    let center = in.uv * blur.target_used * blur.target_texel;
    var color = vec4<f32>(0.0);
    for (var i = 0u; i < blur.tap_count; i = i + 1u) {
        let tap = blur.taps[i];
        color = color + tap.z * read(center + tap.xy);
    }
    // CSS's saturate matrix, on premultiplied color: each channel moves from
    // the luma by the amount, and stops at 0 and at the pixel's alpha.
    let luma = dot(color.rgb, vec3<f32>(0.213, 0.715, 0.072));
    let saturated = vec3<f32>(luma) + blur.saturate * (color.rgb - vec3<f32>(luma));
    return vec4<f32>(clamp(saturated, vec3<f32>(0.0), vec3<f32>(color.a)), color.a);
}
