struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct k0Rd {
    e2_: array<u32>,
}

struct x4Rd {
    e2_: array<u32>,
}

struct h0Rd {
    e2_: array<u32>,
}

struct jf {
    e2_: array<vec2<u32>>,
}

struct kf {
    e2_: array<vec4<f32>>,
}

@id(13) override Ph: bool = false;
@id(14) override Qh: bool = false;
@id(0) override Ch: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> l: BC;
@group(2) @binding(0)
var<storage, read_write> k0_: k0Rd;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> x4_: x4Rd;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Rd;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(13)
var X5_: sampler;
@group(0) @binding(3)
var<storage> DD: jf;
@group(0) @binding(4)
var<storage> QB: kf;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = l.q6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if Ph {
        let _e65 = l.rf;
        k0_.e2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if Qh {
        let _e70 = textureLoad(HC, _e31, 0i);
        k0_.e2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = l.sf;
    x4_.e2_[_e63] = _e75;
    if Ch {
        h0_.e2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
