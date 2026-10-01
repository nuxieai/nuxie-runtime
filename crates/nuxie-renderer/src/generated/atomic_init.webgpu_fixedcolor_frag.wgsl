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

@id(0) override ki: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
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
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(13)
var r5_: sampler;
@group(0) @binding(3)
var<storage> WC: Ef;
@group(0) @binding(4)
var<storage> JB: Ff;
var<private> K1_: vec4<f32>;

fn main_1() {
    let _e25 = gl_FragCoord_1;
    let _e29 = bitcast<vec2<u32>>(vec2<i32>(floor(_e25.xy)));
    let _e31 = j.A6_;
    let _e60 = bitcast<i32>((((((_e29.y >> bitcast<u32>(5u)) * (((_e31 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e29.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e29.x & 28u) << bitcast<u32>(5u)) + ((_e29.y & 28u) << bitcast<u32>(2i)))) + (((_e29.y & 3u) << bitcast<u32>(2i)) + (_e29.x & 3u))));
    let _e62 = j.Of;
    K4_.k2_[_e60] = _e62;
    if ki {
        m0_.k2_[_e60] = 0u;
    }
    discard;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = K1_;
    return _e3;
}
