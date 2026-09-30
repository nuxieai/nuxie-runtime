struct SB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    eh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    ih: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    bh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct A4Sd {
    g2_: array<u32>,
}

struct i0Sd {
    g2_: array<u32>,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct lf {
    g2_: array<vec4<f32>>,
}

@id(0) override Eh: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: SB;
@group(2) @binding(3)
var<storage, read_write> A4_: A4Sd;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Sd;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(3) @binding(8)
var M9_: sampler;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(3)
var<storage> CD: kf;
@group(0) @binding(4)
var<storage> PB: lf;
var<private> F1_: vec4<f32>;

fn main_1() {
    let _e25 = gl_FragCoord_1;
    let _e29 = bitcast<vec2<u32>>(vec2<i32>(floor(_e25.xy)));
    let _e31 = j.n6_;
    let _e60 = bitcast<i32>((((((_e29.y >> bitcast<u32>(5u)) * (((_e31 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e29.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e29.x & 28u) << bitcast<u32>(5u)) + ((_e29.y & 28u) << bitcast<u32>(2i)))) + (((_e29.y & 3u) << bitcast<u32>(2i)) + (_e29.x & 3u))));
    let _e62 = j.uf;
    A4_.g2_[_e60] = _e62;
    if Eh {
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
