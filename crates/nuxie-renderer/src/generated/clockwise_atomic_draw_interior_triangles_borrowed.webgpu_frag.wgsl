struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

struct je {
    c2_: array<u32>,
}

struct je_1 {
    c2_: array<atomic<u32>>,
}

var<private> i1_1: f32;
var<private> n4_1: vec2<f32>;
var<private> f3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> m: DC;
@group(0) @binding(6)
var<storage, read_write> P0_: je_1;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(3) @binding(8)
var Ob: sampler;
@group(1) @binding(13)
var U5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> B0_1: f32;
var<private> U1_1: vec2<f32>;
var<private> M0_1: vec4<f32>;
var<private> e2_1: f32;
var<private> A2_1: vec3<f32>;

fn main_1() {
    let _e29 = i1_1;
    let _e30 = n4_1;
    let _e32 = vec2<u32>(floor(_e30));
    let _e34 = f3_1[1u];
    let _e36 = f3_1[0u];
    let _e67 = u32(((abs(_e29) * 1024f) + 0.5f));
    let _e69 = m.a2_;
    let _e71 = (_e69 | (262144u - _e67));
    let _e74 = atomicMax((&P0_.c2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), _e71);
    if (_e74 >= _e69) {
        let _e79 = atomicAdd((&P0_.c2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), ((_e74 - max(_e74, _e71)) - _e67));
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) i1_: f32, @location(8) n4_: vec2<f32>, @location(7) @interpolate(flat, either) f3_: vec2<u32>, @location(0) f1_: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @location(5) M0_: vec4<f32>, @location(6) @interpolate(flat, either) e2_: f32, @location(9) A2_: vec3<f32>) {
    i1_1 = i1_;
    n4_1 = n4_;
    f3_1 = f3_;
    f1_1 = f1_;
    B0_1 = B0_;
    U1_1 = U1_;
    M0_1 = M0_;
    e2_1 = e2_;
    A2_1 = A2_;
    main_1();
}
