struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

struct o0cf {
    x2_: array<u32>,
}

struct W4cf {
    x2_: array<u32>,
}

struct m0cf {
    x2_: array<u32>,
}

struct jd {
    x2_: array<vec4<u32>>,
}

struct ug {
    x2_: array<vec4<f32>>,
}

@id(13) override wj: bool = false;
@id(14) override xj: bool = false;
@id(0) override jj: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(2) @binding(0)
var<storage, read_write> o0_: o0cf;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(7)
var Q8_: sampler;
@group(1) @binding(12)
var v4_: sampler;
@group(0) @binding(2)
var<storage> JB: jd;
@group(0) @binding(3)
var<storage> KB: ug;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = j.R6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if wj {
        let _e65 = j.Dg;
        o0_.x2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if xj {
        let _e70 = textureLoad(RB, _e31, 0i);
        o0_.x2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = j.Eg;
    W4_.x2_[_e63] = _e75;
    if jj {
        m0_.x2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
