struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct o0ge {
    j2_: array<u32>,
}

struct K4ge {
    j2_: array<u32>,
}

struct m0ge {
    j2_: array<u32>,
}

struct Bf {
    j2_: array<vec2<u32>>,
}

struct Cf {
    j2_: array<vec4<f32>>,
}

@id(12) override ni: bool = false;
@id(13) override oi: bool = false;
@id(0) override bi: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(2) @binding(0)
var<storage, read_write> o0_: o0ge;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> K4_: K4ge;
@group(2) @binding(1)
var<storage, read_write> m0_: m0ge;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(13)
var f6_: sampler;
@group(0) @binding(3)
var<storage> XC: Bf;
@group(0) @binding(4)
var<storage> JB: Cf;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = j.z6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if ni {
        let _e65 = j.Kf;
        o0_.j2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if oi {
        let _e70 = textureLoad(IC, _e31, 0i);
        o0_.j2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = j.Lf;
    K4_.j2_[_e63] = _e75;
    if bi {
        m0_.j2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
