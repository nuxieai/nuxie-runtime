struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct n0cf {
    v2_: array<u32>,
}

struct W4cf {
    v2_: array<u32>,
}

struct m0cf {
    v2_: array<u32>,
}

struct ug {
    v2_: array<vec2<u32>>,
}

struct vg {
    v2_: array<vec4<f32>>,
}

@id(13) override wj: bool = false;
@id(14) override xj: bool = false;
@id(0) override jj: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(2) @binding(0)
var<storage, read_write> n0_: n0cf;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
@group(1) @binding(13)
var v4_: sampler;
@group(0) @binding(3)
var<storage> XC: ug;
@group(0) @binding(4)
var<storage> JB: vg;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = j.T6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if wj {
        let _e65 = j.Eg;
        n0_.v2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if xj {
        let _e70 = textureLoad(RB, _e31, 0i);
        n0_.v2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = j.Fg;
    W4_.v2_[_e63] = _e75;
    if jj {
        m0_.v2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
