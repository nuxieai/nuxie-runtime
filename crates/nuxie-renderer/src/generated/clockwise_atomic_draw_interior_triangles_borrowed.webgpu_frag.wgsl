struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct Sf {
    z2_: array<u32>,
}

struct Sf_1 {
    z2_: array<atomic<u32>>,
}

var<private> r1_1: f32;
var<private> V4_1: vec2<f32>;
var<private> D3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: RB;
@group(0) @binding(5)
var<storage, read_write> c1_: Sf_1;
@group(3) @binding(8)
var ib: sampler;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(3) @binding(7)
var V8_: sampler;
@group(1) @binding(12)
var x4_: sampler;
var<private> N0_1: vec4<f32>;
var<private> J0_1: f32;
var<private> l2_1: vec2<f32>;
var<private> Y0_1: vec4<f32>;
var<private> S0_1: f32;
var<private> O0_1: vec3<f32>;

fn main_1() {
    let _e29 = r1_1;
    let _e30 = V4_1;
    let _e32 = vec2<u32>(floor(_e30));
    let _e34 = D3_1[1u];
    let _e36 = D3_1[0u];
    let _e67 = u32(((abs(_e29) * 1024f) + 0.5f));
    let _e69 = j.y2_;
    let _e71 = (_e69 | (262144u - _e67));
    let _e74 = atomicMax((&c1_.z2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), _e71);
    if (_e74 >= _e69) {
        let _e79 = atomicAdd((&c1_.z2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), ((_e74 - max(_e74, _e71)) - _e67));
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) r1_: f32, @location(8) V4_: vec2<f32>, @location(7) @interpolate(flat, either) D3_: vec2<u32>, @location(0) N0_: vec4<f32>, @location(3) @interpolate(flat, either) J0_: f32, @location(4) @interpolate(flat, either) l2_: vec2<f32>, @location(5) Y0_: vec4<f32>, @location(6) @interpolate(flat, either) S0_: f32, @location(9) O0_: vec3<f32>) {
    r1_1 = r1_;
    V4_1 = V4_;
    D3_1 = D3_;
    N0_1 = N0_;
    J0_1 = J0_;
    l2_1 = l2_;
    Y0_1 = Y0_;
    S0_1 = S0_;
    O0_1 = O0_;
    main_1();
}
