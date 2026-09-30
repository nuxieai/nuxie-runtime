struct TB {
    tc: f32,
    Bd: f32,
    Hf: f32,
    If: f32,
    o6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    T7_: vec4<i32>,
    hh: vec2<f32>,
    Cd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    X2_: f32,
    Dd: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Ed: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct Ae {
    g2_: array<u32>,
}

struct Ae_1 {
    g2_: array<atomic<u32>>,
}

var<private> i1_1: f32;
var<private> w4_1: vec2<f32>;
var<private> k3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: TB;
@group(0) @binding(6)
var<storage, read_write> S0_: Ae_1;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(13)
var W5_: sampler;
var<private> X1_1: vec4<f32>;
var<private> D0_1: f32;
var<private> Y1_1: vec2<f32>;
var<private> P0_1: vec4<f32>;
var<private> f1_1: f32;
var<private> D2_1: vec3<f32>;

fn main_1() {
    let _e29 = i1_1;
    let _e30 = w4_1;
    let _e32 = vec2<u32>(floor(_e30));
    let _e34 = k3_1[1u];
    let _e36 = k3_1[0u];
    let _e67 = u32(((abs(_e29) * 1024f) + 0.5f));
    let _e69 = j.f2_;
    let _e71 = (_e69 | (262144u - _e67));
    let _e74 = atomicMax((&S0_.g2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), _e71);
    if (_e74 >= _e69) {
        let _e79 = atomicAdd((&S0_.g2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), ((_e74 - max(_e74, _e71)) - _e67));
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) i1_: f32, @location(8) w4_: vec2<f32>, @location(7) @interpolate(flat, either) k3_: vec2<u32>, @location(0) X1_: vec4<f32>, @location(3) @interpolate(flat, either) D0_: f32, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @location(5) P0_: vec4<f32>, @location(6) @interpolate(flat, either) f1_: f32, @location(9) D2_: vec3<f32>) {
    i1_1 = i1_;
    w4_1 = w4_;
    k3_1 = k3_;
    X1_1 = X1_;
    D0_1 = D0_;
    Y1_1 = Y1_;
    P0_1 = P0_;
    f1_1 = f1_;
    D2_1 = D2_;
    main_1();
}
