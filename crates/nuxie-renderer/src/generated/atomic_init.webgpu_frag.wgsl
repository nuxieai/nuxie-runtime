struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct p0hf {
    z2_: array<u32>,
}

struct d5hf {
    z2_: array<u32>,
}

struct n0hf {
    z2_: array<u32>,
}

struct nd {
    z2_: array<vec4<u32>>,
}

struct Cg {
    z2_: array<vec4<f32>>,
}

@id(13) override Hj: bool = false;
@id(14) override Ij: bool = false;
@id(0) override uj: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: RB;
@group(2) @binding(0)
var<storage, read_write> p0_: p0hf;
@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> d5_: d5hf;
@group(2) @binding(1)
var<storage, read_write> n0_: n0hf;
@group(3) @binding(8)
var ib: sampler;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(7)
var V8_: sampler;
@group(1) @binding(12)
var x4_: sampler;
@group(0) @binding(2)
var<storage> JB: nd;
@group(0) @binding(3)
var<storage> KB: Cg;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = j.V5_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if Hj {
        let _e65 = j.Lg;
        p0_.z2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if Ij {
        let _e70 = textureLoad(SB, _e31, 0i);
        p0_.z2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = j.Ng;
    d5_.z2_[_e63] = _e75;
    if uj {
        n0_.z2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
