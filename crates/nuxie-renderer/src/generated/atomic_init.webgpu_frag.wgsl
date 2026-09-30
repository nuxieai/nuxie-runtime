struct SB {
    xc: f32,
    Hd: f32,
    Mf: f32,
    Nf: f32,
    r6_: u32,
    Rb: u32,
    yf: u32,
    zf: u32,
    X7_: vec4<i32>,
    jh: vec2<f32>,
    Id: vec2<f32>,
    f2_: u32,
    nh: f32,
    g6_: u32,
    W2_: f32,
    Jd: f32,
    sf: u32,
    F3_: f32,
    G3_: f32,
    Kd: f32,
    gh: u32,
    Qb: u32,
    dc: f32,
    ec: f32,
}

struct m0Xd {
    g2_: array<u32>,
}

struct A4Xd {
    g2_: array<u32>,
}

struct i0Xd {
    g2_: array<u32>,
}

struct pf {
    g2_: array<vec2<u32>>,
}

struct qf {
    g2_: array<vec4<f32>>,
}

@id(13) override Wh: bool = false;
@id(14) override Xh: bool = false;
@id(0) override Jh: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: SB;
@group(2) @binding(0)
var<storage, read_write> m0_: m0Xd;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> A4_: A4Xd;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Xd;
@group(3) @binding(9)
var ha: sampler;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(1) @binding(13)
var Z5_: sampler;
@group(0) @binding(3)
var<storage> CD: pf;
@group(0) @binding(4)
var<storage> PB: qf;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = j.r6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if Wh {
        let _e65 = j.yf;
        m0_.g2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if Xh {
        let _e70 = textureLoad(GC, _e31, 0i);
        m0_.g2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = j.zf;
    A4_.g2_[_e63] = _e75;
    if Jh {
        i0_.g2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
