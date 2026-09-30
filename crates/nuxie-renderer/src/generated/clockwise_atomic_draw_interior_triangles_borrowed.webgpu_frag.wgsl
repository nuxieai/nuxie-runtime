struct TB {
    uc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Ob: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    ih: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    mh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    fh: u32,
    Nb: u32,
    ac: f32,
    bc: f32,
}

struct Ae {
    g2_: array<u32>,
}

struct Ae_1 {
    g2_: array<atomic<u32>>,
}

var<private> j1_1: f32;
var<private> v4_1: vec2<f32>;
var<private> k3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: TB;
@group(0) @binding(6)
var<storage, read_write> S0_: Ae_1;
@group(3) @binding(9)
var da: sampler;
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
var<private> O0_1: vec4<f32>;
var<private> g1_1: f32;
var<private> C2_1: vec3<f32>;

fn main_1() {
    let _e29 = j1_1;
    let _e30 = v4_1;
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
fn main(@location(1) @interpolate(flat, either) j1_: f32, @location(8) v4_: vec2<f32>, @location(7) @interpolate(flat, either) k3_: vec2<u32>, @location(0) X1_: vec4<f32>, @location(3) @interpolate(flat, either) D0_: f32, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @location(5) O0_: vec4<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(9) C2_: vec3<f32>) {
    j1_1 = j1_;
    v4_1 = v4_;
    k3_1 = k3_;
    X1_1 = X1_;
    D0_1 = D0_;
    Y1_1 = Y1_;
    O0_1 = O0_;
    g1_1 = g1_;
    C2_1 = C2_;
    main_1();
}
