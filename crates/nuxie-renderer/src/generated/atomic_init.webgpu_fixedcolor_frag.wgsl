struct SB {
    yc: f32,
    Id: f32,
    Nf: f32,
    Of: f32,
    r6_: u32,
    Sb: u32,
    zf: u32,
    Af: u32,
    X7_: vec4<i32>,
    kh: vec2<f32>,
    Jd: vec2<f32>,
    f2_: u32,
    oh: f32,
    g6_: u32,
    W2_: f32,
    Kd: f32,
    tf: u32,
    F3_: f32,
    G3_: f32,
    Ld: f32,
    hh: u32,
    Rb: u32,
    ec: f32,
    fc: f32,
}

struct A4Yd {
    g2_: array<u32>,
}

struct i0Yd {
    g2_: array<u32>,
}

struct qf {
    g2_: array<vec2<u32>>,
}

struct rf {
    g2_: array<vec4<f32>>,
}

@id(0) override Kh: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: SB;
@group(2) @binding(3)
var<storage, read_write> A4_: A4Yd;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Yd;
@group(3) @binding(9)
var ha: sampler;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(1) @binding(13)
var Z5_: sampler;
@group(0) @binding(3)
var<storage> CD: qf;
@group(0) @binding(4)
var<storage> PB: rf;
var<private> F1_: vec4<f32>;

fn main_1() {
    let _e25 = gl_FragCoord_1;
    let _e29 = bitcast<vec2<u32>>(vec2<i32>(floor(_e25.xy)));
    let _e31 = j.r6_;
    let _e60 = bitcast<i32>((((((_e29.y >> bitcast<u32>(5u)) * (((_e31 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e29.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e29.x & 28u) << bitcast<u32>(5u)) + ((_e29.y & 28u) << bitcast<u32>(2i)))) + (((_e29.y & 3u) << bitcast<u32>(2i)) + (_e29.x & 3u))));
    let _e62 = j.Af;
    A4_.g2_[_e60] = _e62;
    if Kh {
        i0_.g2_[_e60] = 0u;
    }
    discard;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = F1_;
    return _e3;
}
