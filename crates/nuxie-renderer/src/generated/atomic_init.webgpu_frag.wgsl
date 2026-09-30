struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

struct l0Td {
    g2_: array<u32>,
}

struct z4Td {
    g2_: array<u32>,
}

struct i0Td {
    g2_: array<u32>,
}

struct lf {
    g2_: array<vec2<u32>>,
}

struct mf {
    g2_: array<vec4<f32>>,
}

@id(13) override Rh: bool = false;
@id(14) override Sh: bool = false;
@id(0) override Eh: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: AC;
@group(2) @binding(0)
var<storage, read_write> l0_: l0Td;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Td;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(1) @binding(13)
var Y5_: sampler;
@group(0) @binding(3)
var<storage> CD: lf;
@group(0) @binding(4)
var<storage> PB: mf;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = j.q6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if Rh {
        let _e65 = j.tf;
        l0_.g2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if Sh {
        let _e70 = textureLoad(GC, _e31, 0i);
        l0_.g2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = j.uf;
    z4_.g2_[_e63] = _e75;
    if Eh {
        i0_.g2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
