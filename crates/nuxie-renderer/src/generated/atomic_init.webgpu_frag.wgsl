struct UB {
    Rc: f32,
    Ud: f32,
    cg: f32,
    dg: f32,
    B6_: u32,
    Y9_: u32,
    Of: u32,
    Pf: u32,
    k8_: vec4<i32>,
    Mh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Qh: f32,
    U4_: u32,
    c3_: f32,
    Wd: f32,
    If: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Jh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct n0he {
    k2_: array<u32>,
}

struct L4he {
    k2_: array<u32>,
}

struct m0he {
    k2_: array<u32>,
}

struct Ff {
    k2_: array<vec2<u32>>,
}

struct Gf {
    k2_: array<vec4<f32>>,
}

@id(12) override xi: bool = false;
@id(13) override yi: bool = false;
@id(0) override li: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(2) @binding(0)
var<storage, read_write> n0_: n0he;
@group(1) @binding(11)
var DC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> L4_: L4he;
@group(2) @binding(1)
var<storage, read_write> m0_: m0he;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(1) @binding(13)
var v5_: sampler;
@group(0) @binding(3)
var<storage> XC: Ff;
@group(0) @binding(4)
var<storage> JB: Gf;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = j.B6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if xi {
        let _e65 = j.Of;
        n0_.k2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if yi {
        let _e70 = textureLoad(DC, _e31, 0i);
        n0_.k2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = j.Pf;
    L4_.k2_[_e63] = _e75;
    if li {
        m0_.k2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
