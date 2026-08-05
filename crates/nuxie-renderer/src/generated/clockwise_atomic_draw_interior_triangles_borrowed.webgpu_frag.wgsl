struct CC {
    ec: f32,
    od: f32,
    ff: f32,
    gf: f32,
    m6_: u32,
    Fg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Bg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Gg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    yg: u32,
}

struct ge {
    c2_: array<u32>,
}

struct ge_1 {
    c2_: array<atomic<u32>>,
}

var<private> i1_1: f32;
var<private> l4_1: vec2<f32>;
var<private> d3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> n: CC;
@group(0) @binding(6)
var<storage, read_write> P0_: ge_1;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(3) @binding(8)
var Mb: sampler;
@group(1) @binding(13)
var S5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> B0_1: f32;
var<private> U1_1: vec2<f32>;
var<private> L0_1: vec4<f32>;
var<private> e2_1: f32;

fn main_1() {
    let _e28 = i1_1;
    let _e29 = l4_1;
    let _e31 = vec2<u32>(floor(_e29));
    let _e33 = d3_1[1u];
    let _e35 = d3_1[0u];
    let _e66 = u32(((abs(_e28) * 1024f) + 0.5f));
    let _e68 = n.a2_;
    let _e70 = (_e68 | (262144u - _e66));
    let _e73 = atomicMax((&P0_.c2_[(_e35 + (((((_e31.y >> bitcast<u32>(5u)) * (_e33 << bitcast<u32>(5u))) + ((_e31.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e31.x & 28u) << bitcast<u32>(5u)) + ((_e31.y & 28u) << bitcast<u32>(2i)))) + (((_e31.y & 3u) << bitcast<u32>(2i)) + (_e31.x & 3u))))]), _e70);
    if (_e73 >= _e68) {
        let _e78 = atomicAdd((&P0_.c2_[(_e35 + (((((_e31.y >> bitcast<u32>(5u)) * (_e33 << bitcast<u32>(5u))) + ((_e31.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e31.x & 28u) << bitcast<u32>(5u)) + ((_e31.y & 28u) << bitcast<u32>(2i)))) + (((_e31.y & 3u) << bitcast<u32>(2i)) + (_e31.x & 3u))))]), ((_e73 - max(_e73, _e70)) - _e66));
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) i1_: f32, @location(8) l4_: vec2<f32>, @location(7) @interpolate(flat, either) d3_: vec2<u32>, @location(0) f1_: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @location(5) L0_: vec4<f32>, @location(6) @interpolate(flat, either) e2_: f32) {
    i1_1 = i1_;
    l4_1 = l4_;
    d3_1 = d3_;
    f1_1 = f1_;
    B0_1 = B0_;
    U1_1 = U1_;
    L0_1 = L0_;
    e2_1 = e2_;
    main_1();
}
