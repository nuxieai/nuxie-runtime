struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

struct j0Ed {
    d2_: array<u32>,
}

struct v4Ed {
    d2_: array<u32>,
}

struct h0Ed {
    d2_: array<u32>,
}

struct Ne {
    d2_: array<vec2<u32>>,
}

struct Oe {
    d2_: array<vec4<f32>>,
}

@id(13) override sh: bool = false;
@id(14) override th: bool = false;
@id(0) override fh: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: DC;
@group(2) @binding(0)
var<storage, read_write> j0_: j0Ed;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> v4_: v4Ed;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Ed;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var Pb: sampler;
@group(1) @binding(13)
var V5_: sampler;
@group(0) @binding(3)
var<storage> BD: Ne;
@group(0) @binding(4)
var<storage> RB: Oe;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = m.p6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if sh {
        let _e65 = m.Ve;
        j0_.d2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if th {
        let _e70 = textureLoad(JC, _e31, 0i);
        j0_.d2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = m.We;
    v4_.d2_[_e63] = _e75;
    if fh {
        h0_.d2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
