struct BC {
    sc: f32,
    Cd: f32,
    Gf: f32,
    Hf: f32,
    o6_: u32,
    Pb: u32,
    sf: u32,
    tf: u32,
    V7_: vec4<i32>,
    dh: vec2<f32>,
    Dd: vec2<f32>,
    d2_: u32,
    hh: f32,
    d6_: u32,
    T2_: f32,
    Ed: f32,
    nf: u32,
    C3_: f32,
    D3_: f32,
    Fd: f32,
    ah: u32,
    Ob: u32,
}

struct k0Sd {
    e2_: array<u32>,
}

struct x4Sd {
    e2_: array<u32>,
}

struct h0Sd {
    e2_: array<u32>,
}

struct kf {
    e2_: array<vec2<u32>>,
}

struct lf {
    e2_: array<vec4<f32>>,
}

@id(13) override Qh: bool = false;
@id(14) override Rh: bool = false;
@id(0) override Dh: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> l: BC;
@group(2) @binding(0)
var<storage, read_write> k0_: k0Sd;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> x4_: x4Sd;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Sd;
@group(3) @binding(9)
var fa: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var O9_: sampler;
@group(1) @binding(13)
var V5_: sampler;
@group(0) @binding(3)
var<storage> DD: kf;
@group(0) @binding(4)
var<storage> QB: lf;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = l.o6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if Qh {
        let _e65 = l.sf;
        k0_.e2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if Rh {
        let _e70 = textureLoad(HC, _e31, 0i);
        k0_.e2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = l.tf;
    x4_.e2_[_e63] = _e75;
    if Dh {
        h0_.e2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
