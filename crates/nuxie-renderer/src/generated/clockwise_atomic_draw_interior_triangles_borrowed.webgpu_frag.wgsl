struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct Yd {
    X1_: array<u32>,
}

struct Yd_1 {
    X1_: array<atomic<u32>>,
}

var<private> j1_1: f32;
var<private> j4_1: vec2<f32>;
var<private> e3_1: vec2<u32>;
@group(0) @binding(0) 
var<uniform> k: NB;
@group(0) @binding(7) 
var<storage, read_write> S0_: Yd_1;
@group(3) @binding(10) 
var T9_: sampler;
@group(0) @binding(9) 
var DD: texture_2d<f32>;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
@group(1) @binding(12) 
var AC: texture_2d<f32>;
@group(3) @binding(9) 
var Bb: sampler;
@group(1) @binding(14) 
var R5_: sampler;
var<private> i1_1: vec4<f32>;
var<private> z0_1: f32;
var<private> S1_1: vec2<f32>;
var<private> N0_1: vec4<f32>;
var<private> Z1_1: f32;

fn main_1() {
    let _e28 = j1_1;
    let _e29 = j4_1;
    let _e31 = vec2<u32>(floor(_e29));
    let _e33 = e3_1[1u];
    let _e35 = e3_1[0u];
    let _e66 = u32(((abs(_e28) * 1024f) + 0.5f));
    let _e68 = k.W1_;
    let _e70 = (_e68 | (262144u - _e66));
    let _e73 = atomicMax((&S0_.X1_[(_e35 + (((((_e31.y >> bitcast<u32>(5u)) * (_e33 << bitcast<u32>(5u))) + ((_e31.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e31.x & 28u) << bitcast<u32>(5u)) + ((_e31.y & 28u) << bitcast<u32>(2i)))) + (((_e31.y & 3u) << bitcast<u32>(2i)) + (_e31.x & 3u))))]), _e70);
    if (_e73 >= _e68) {
        let _e78 = atomicAdd((&S0_.X1_[(_e35 + (((((_e31.y >> bitcast<u32>(5u)) * (_e33 << bitcast<u32>(5u))) + ((_e31.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e31.x & 28u) << bitcast<u32>(5u)) + ((_e31.y & 28u) << bitcast<u32>(2i)))) + (((_e31.y & 3u) << bitcast<u32>(2i)) + (_e31.x & 3u))))]), ((_e73 - max(_e73, _e70)) - _e66));
    }
    return;
}

@fragment 
fn main(@location(1) @interpolate(flat) j1_: f32, @location(8) j4_: vec2<f32>, @location(7) @interpolate(flat) e3_: vec2<u32>, @location(0) i1_: vec4<f32>, @location(3) @interpolate(flat) z0_: f32, @location(4) @interpolate(flat) S1_: vec2<f32>, @location(5) N0_: vec4<f32>, @location(6) @interpolate(flat) Z1_: f32) {
    j1_1 = j1_;
    j4_1 = j4_;
    e3_1 = e3_;
    i1_1 = i1_;
    z0_1 = z0_;
    S1_1 = S1_;
    N0_1 = N0_;
    Z1_1 = Z1_;
    main_1();
}
