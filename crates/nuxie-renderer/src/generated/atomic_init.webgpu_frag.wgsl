struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
}

struct j0Id {
    d2_: array<u32>,
}

struct w4Id {
    d2_: array<u32>,
}

struct g0Id {
    d2_: array<u32>,
}

struct Re {
    d2_: array<vec2<u32>>,
}

struct Se {
    d2_: array<vec4<f32>>,
}

@id(13) override wh: bool = false;
@id(14) override xh: bool = false;
@id(0) override jh: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: BC;
@group(2) @binding(0)
var<storage, read_write> j0_: j0Id;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> w4_: w4Id;
@group(2) @binding(1)
var<storage, read_write> g0_: g0Id;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var Rb: sampler;
@group(1) @binding(13)
var V5_: sampler;
@group(0) @binding(3)
var<storage> AD: Re;
@group(0) @binding(4)
var<storage> QB: Se;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = m.p6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if wh {
        let _e65 = m.Ze;
        j0_.d2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if xh {
        let _e70 = textureLoad(HC, _e31, 0i);
        j0_.d2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = m.af;
    w4_.d2_[_e63] = _e75;
    if jh {
        g0_.d2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
