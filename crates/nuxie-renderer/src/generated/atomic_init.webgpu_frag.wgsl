struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct j0xd {
    c2_: array<u32>,
}

struct q4xd {
    c2_: array<u32>,
}

struct h0xd {
    c2_: array<u32>,
}

struct Fe {
    c2_: array<vec2<u32>>,
}

struct Ge {
    c2_: array<vec4<f32>>,
}

@id(11) override gh: bool = false;
@id(12) override hh: bool = false;
@id(0) override Ug: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: CC;
@group(2) @binding(0)
var<storage, read_write> j0_: j0xd;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> q4_: q4xd;
@group(2) @binding(1)
var<storage, read_write> h0_: h0xd;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var Jb: sampler;
@group(1) @binding(13)
var S5_: sampler;
@group(0) @binding(3)
var<storage> AD: Fe;
@group(0) @binding(4)
var<storage> RB: Ge;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = m.m6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if gh {
        let _e65 = m.Ne;
        j0_.c2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if hh {
        let _e70 = textureLoad(IC, _e31, 0i);
        j0_.c2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = m.Oe;
    q4_.c2_[_e63] = _e75;
    if Ug {
        h0_.c2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
