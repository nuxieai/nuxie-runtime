struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct n0ge {
    k2_: array<u32>,
}

struct K4ge {
    k2_: array<u32>,
}

struct m0ge {
    k2_: array<u32>,
}

struct Ef {
    k2_: array<vec2<u32>>,
}

struct Ff {
    k2_: array<vec4<f32>>,
}

@id(12) override wi: bool = false;
@id(13) override xi: bool = false;
@id(0) override ki: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(2) @binding(0)
var<storage, read_write> n0_: n0ge;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> K4_: K4ge;
@group(2) @binding(1)
var<storage, read_write> m0_: m0ge;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(13)
var r5_: sampler;
@group(0) @binding(3)
var<storage> WC: Ef;
@group(0) @binding(4)
var<storage> JB: Ff;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = j.A6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if wi {
        let _e65 = j.Nf;
        n0_.k2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if xi {
        let _e70 = textureLoad(CC, _e31, 0i);
        n0_.k2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = j.Of;
    K4_.k2_[_e63] = _e75;
    if ki {
        m0_.k2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
