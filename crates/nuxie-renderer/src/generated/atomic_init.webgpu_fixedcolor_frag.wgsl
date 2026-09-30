struct BC {
    qc: f32,
    Ad: f32,
    Ef: f32,
    Ff: f32,
    q6_: u32,
    Nb: u32,
    qf: u32,
    rf: u32,
    V7_: vec4<i32>,
    bh: vec2<f32>,
    Bd: vec2<f32>,
    d2_: u32,
    fh: f32,
    f6_: u32,
    U2_: f32,
    Cd: f32,
    lf: u32,
    C3_: f32,
    D3_: f32,
    Dd: f32,
    Yg: u32,
}

struct x4Qd {
    e2_: array<u32>,
}

struct h0Qd {
    e2_: array<u32>,
}

struct hf {
    e2_: array<vec2<u32>>,
}

struct jf {
    e2_: array<vec4<f32>>,
}

@id(0) override Bh: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: BC;
@group(2) @binding(3)
var<storage, read_write> x4_: x4Qd;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Qd;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(13)
var X5_: sampler;
@group(0) @binding(3)
var<storage> DD: hf;
@group(0) @binding(4)
var<storage> QB: jf;
var<private> C1_: vec4<f32>;

fn main_1() {
    let _e25 = gl_FragCoord_1;
    let _e29 = bitcast<vec2<u32>>(vec2<i32>(floor(_e25.xy)));
    let _e31 = n.q6_;
    let _e60 = bitcast<i32>((((((_e29.y >> bitcast<u32>(5u)) * (((_e31 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e29.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e29.x & 28u) << bitcast<u32>(5u)) + ((_e29.y & 28u) << bitcast<u32>(2i)))) + (((_e29.y & 3u) << bitcast<u32>(2i)) + (_e29.x & 3u))));
    let _e62 = n.rf;
    x4_.e2_[_e60] = _e62;
    if Bh {
        h0_.e2_[_e60] = 0u;
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
