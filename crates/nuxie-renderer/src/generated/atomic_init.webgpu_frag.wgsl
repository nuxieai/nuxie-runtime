struct WB {
    Hd: f32,
    Qe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    I8_: vec4<i32>,
    Ki: vec2<f32>,
    Re: vec2<f32>,
    w2_: u32,
    Oi: f32,
    w6_: u32,
    j3_: f32,
    Se: f32,
    xg: u32,
    I3_: f32,
    J3_: f32,
    Te: f32,
    Hi: u32,
    Ea: u32,
    P8_: f32,
    Q8_: f32,
}

struct o0df {
    x2_: array<u32>,
}

struct W4df {
    x2_: array<u32>,
}

struct m0df {
    x2_: array<u32>,
}

struct kd {
    x2_: array<vec4<u32>>,
}

struct vg {
    x2_: array<vec4<f32>>,
}

@id(13) override xj: bool = false;
@id(14) override yj: bool = false;
@id(0) override kj: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(2) @binding(0)
var<storage, read_write> o0_: o0df;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> W4_: W4df;
@group(2) @binding(1)
var<storage, read_write> m0_: m0df;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(7)
var R8_: sampler;
@group(1) @binding(12)
var v4_: sampler;
@group(0) @binding(2)
var<storage> JB: kd;
@group(0) @binding(3)
var<storage> KB: vg;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = j.T6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if xj {
        let _e65 = j.Eg;
        o0_.x2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if yj {
        let _e70 = textureLoad(RB, _e31, 0i);
        o0_.x2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = j.Fg;
    W4_.x2_[_e63] = _e75;
    if kj {
        m0_.x2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
