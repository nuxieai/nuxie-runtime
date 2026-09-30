struct TB {
    tc: f32,
    Bd: f32,
    Hf: f32,
    If: f32,
    o6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    T7_: vec4<i32>,
    hh: vec2<f32>,
    Cd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    X2_: f32,
    Dd: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Ed: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct l0Rd {
    g2_: array<u32>,
}

struct B4Rd {
    g2_: array<u32>,
}

struct i0Rd {
    g2_: array<u32>,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct lf {
    g2_: array<vec4<f32>>,
}

@id(13) override Uh: bool = false;
@id(14) override Vh: bool = false;
@id(0) override Hh: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: TB;
@group(2) @binding(0)
var<storage, read_write> l0_: l0Rd;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> B4_: B4Rd;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Rd;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(3)
var<storage> DD: kf;
@group(0) @binding(4)
var<storage> PB: lf;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = j.o6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if Uh {
        let _e65 = j.tf;
        l0_.g2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if Vh {
        let _e70 = textureLoad(HC, _e31, 0i);
        l0_.g2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = j.uf;
    B4_.g2_[_e63] = _e75;
    if Hh {
        i0_.g2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
