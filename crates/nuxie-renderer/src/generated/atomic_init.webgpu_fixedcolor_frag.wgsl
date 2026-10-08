struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct P4Me {
    v2_: array<u32>,
}

struct m0Me {
    v2_: array<u32>,
}

struct gg {
    v2_: array<vec2<u32>>,
}

struct hg {
    v2_: array<vec4<f32>>,
}

@id(0) override Ui: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(2) @binding(3)
var<storage, read_write> P4_: P4Me;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Me;
@group(3) @binding(9)
var Ta: sampler;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(3) @binding(8)
var H8_: sampler;
@group(1) @binding(13)
var S4_: sampler;
@group(0) @binding(3)
var<storage> WC: gg;
@group(0) @binding(4)
var<storage> JB: hg;
var<private> N1_: vec4<f32>;

fn main_1() {
    let _e25 = gl_FragCoord_1;
    let _e29 = bitcast<vec2<u32>>(vec2<i32>(floor(_e25.xy)));
    let _e31 = j.L6_;
    let _e60 = bitcast<i32>((((((_e29.y >> bitcast<u32>(5u)) * (((_e31 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e29.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e29.x & 28u) << bitcast<u32>(5u)) + ((_e29.y & 28u) << bitcast<u32>(2i)))) + (((_e29.y & 3u) << bitcast<u32>(2i)) + (_e29.x & 3u))));
    let _e62 = j.qg;
    P4_.v2_[_e60] = _e62;
    if Ui {
        m0_.v2_[_e60] = 0u;
    }
    discard;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = N1_;
    return _e3;
}
