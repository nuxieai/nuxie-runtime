struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct wf {
    v2_: array<u32>,
}

struct wf_1 {
    v2_: array<atomic<u32>>,
}

var<private> o1_1: f32;
var<private> J4_1: vec2<f32>;
var<private> y3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(6)
var<storage, read_write> Z0_: wf_1;
@group(3) @binding(9)
var Ta: sampler;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(3) @binding(8)
var H8_: sampler;
@group(1) @binding(13)
var S4_: sampler;
var<private> O0_1: vec4<f32>;
var<private> G0_1: f32;
var<private> j2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
var<private> P0_1: f32;
var<private> V0_1: vec3<f32>;

fn main_1() {
    let _e29 = o1_1;
    let _e30 = J4_1;
    let _e32 = vec2<u32>(floor(_e30));
    let _e34 = y3_1[1u];
    let _e36 = y3_1[0u];
    let _e67 = u32(((abs(_e29) * 1024f) + 0.5f));
    let _e69 = j.r2_;
    let _e71 = (_e69 | (262144u - _e67));
    let _e74 = atomicMax((&Z0_.v2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), _e71);
    if (_e74 >= _e69) {
        let _e79 = atomicAdd((&Z0_.v2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), ((_e74 - max(_e74, _e71)) - _e67));
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) o1_: f32, @location(8) J4_: vec2<f32>, @location(7) @interpolate(flat, either) y3_: vec2<u32>, @location(0) O0_: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32, @location(4) @interpolate(flat, either) j2_: vec2<f32>, @location(5) W0_: vec4<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(9) V0_: vec3<f32>) {
    o1_1 = o1_;
    J4_1 = J4_;
    y3_1 = y3_;
    O0_1 = O0_;
    G0_1 = G0_;
    j2_1 = j2_;
    W0_1 = W0_;
    P0_1 = P0_;
    V0_1 = V0_;
    main_1();
}
