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
    e6_: u32,
    T2_: f32,
    Cd: f32,
    lf: u32,
    B3_: f32,
    C3_: f32,
    Dd: f32,
    Yg: u32,
}

struct k0Qd {
    e2_: array<u32>,
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

@id(13) override Oh: bool = false;
@id(14) override Ph: bool = false;
@id(0) override Bh: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: BC;
@group(2) @binding(0)
var<storage, read_write> k0_: k0Qd;
@group(1) @binding(11)
var HC: texture_2d<f32>;
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
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(3)
var<storage> DD: hf;
@group(0) @binding(4)
var<storage> QB: jf;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = n.q6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if Oh {
        let _e65 = n.qf;
        k0_.e2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if Ph {
        let _e70 = textureLoad(HC, _e31, 0i);
        k0_.e2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = n.rf;
    x4_.e2_[_e63] = _e75;
    if Bh {
        h0_.e2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
