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

@id(0) override kj: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
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
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(3) @binding(7)
var R8_: sampler;
@group(1) @binding(12)
var v4_: sampler;
@group(0) @binding(2)
var<storage> JB: kd;
@group(0) @binding(3)
var<storage> KB: vg;
var<private> R1_: vec4<f32>;

fn main_1() {
    let _e25 = gl_FragCoord_1;
    let _e29 = bitcast<vec2<u32>>(vec2<i32>(floor(_e25.xy)));
    let _e31 = j.T6_;
    let _e60 = bitcast<i32>((((((_e29.y >> bitcast<u32>(5u)) * (((_e31 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e29.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e29.x & 28u) << bitcast<u32>(5u)) + ((_e29.y & 28u) << bitcast<u32>(2i)))) + (((_e29.y & 3u) << bitcast<u32>(2i)) + (_e29.x & 3u))));
    let _e62 = j.Fg;
    W4_.x2_[_e60] = _e62;
    if kj {
        m0_.x2_[_e60] = 0u;
    }
    discard;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = R1_;
    return _e3;
}
