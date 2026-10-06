struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct L4he {
    k2_: array<u32>,
}

struct m0he {
    k2_: array<u32>,
}

struct Gf {
    k2_: array<vec2<u32>>,
}

struct Hf {
    k2_: array<vec4<f32>>,
}

@id(0) override mi: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(2) @binding(3)
var<storage, read_write> L4_: L4he;
@group(2) @binding(1)
var<storage, read_write> m0_: m0he;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(1) @binding(13)
var r5_: sampler;
@group(0) @binding(3)
var<storage> WC: Gf;
@group(0) @binding(4)
var<storage> JB: Hf;
var<private> L1_: vec4<f32>;

fn main_1() {
    let _e25 = gl_FragCoord_1;
    let _e29 = bitcast<vec2<u32>>(vec2<i32>(floor(_e25.xy)));
    let _e31 = j.A6_;
    let _e60 = bitcast<i32>((((((_e29.y >> bitcast<u32>(5u)) * (((_e31 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e29.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e29.x & 28u) << bitcast<u32>(5u)) + ((_e29.y & 28u) << bitcast<u32>(2i)))) + (((_e29.y & 3u) << bitcast<u32>(2i)) + (_e29.x & 3u))));
    let _e62 = j.Qf;
    L4_.k2_[_e60] = _e62;
    if mi {
        m0_.k2_[_e60] = 0u;
    }
    discard;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = L1_;
    return _e3;
}
