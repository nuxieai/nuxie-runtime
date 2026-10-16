struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Ci: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Gi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    G3_: f32,
    H3_: f32,
    He: f32,
    zi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct n0Re {
    r2_: array<u32>,
}

struct R4Re {
    r2_: array<u32>,
}

struct m0Re {
    r2_: array<u32>,
}

struct lg {
    r2_: array<vec2<u32>>,
}

struct mg {
    r2_: array<vec4<f32>>,
}

@id(12) override mj: bool = false;
@id(13) override nj: bool = false;
@id(0) override aj: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(2) @binding(0)
var<storage, read_write> n0_: n0Re;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Re;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Re;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
@group(1) @binding(13)
var U4_: sampler;
@group(0) @binding(3)
var<storage> XC: lg;
@group(0) @binding(4)
var<storage> JB: mg;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = j.S6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if mj {
        let _e65 = j.vg;
        n0_.r2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if nj {
        let _e70 = textureLoad(TB, _e31, 0i);
        n0_.r2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = j.wg;
    R4_.r2_[_e63] = _e75;
    if aj {
        m0_.r2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
