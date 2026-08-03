struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

struct q4zd {
    c2_: array<u32>,
}

struct h0zd {
    c2_: array<u32>,
}

struct He {
    c2_: array<vec2<u32>>,
}

struct Ie {
    c2_: array<vec4<f32>>,
}

@id(0) override Wg: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: CC;
@group(2) @binding(3)
var<storage, read_write> q4_: q4zd;
@group(2) @binding(1)
var<storage, read_write> h0_: h0zd;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(3) @binding(8)
var Kb: sampler;
@group(1) @binding(13)
var S5_: sampler;
@group(0) @binding(3)
var<storage> AD: He;
@group(0) @binding(4)
var<storage> RB: Ie;
var<private> C1_: vec4<f32>;

fn main_1() {
    let _e25 = gl_FragCoord_1;
    let _e29 = bitcast<vec2<u32>>(vec2<i32>(floor(_e25.xy)));
    let _e31 = n.m6_;
    let _e60 = bitcast<i32>((((((_e29.y >> bitcast<u32>(5u)) * (((_e31 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e29.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e29.x & 28u) << bitcast<u32>(5u)) + ((_e29.y & 28u) << bitcast<u32>(2i)))) + (((_e29.y & 3u) << bitcast<u32>(2i)) + (_e29.x & 3u))));
    let _e62 = n.Qe;
    q4_.c2_[_e60] = _e62;
    if Wg {
        h0_.c2_[_e60] = 0u;
    }
    discard;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = C1_;
    return _e3;
}
