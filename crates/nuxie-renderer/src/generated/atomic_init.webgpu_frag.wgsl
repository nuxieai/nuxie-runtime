struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

struct j0Dd {
    c2_: array<u32>,
}

struct v4Dd {
    c2_: array<u32>,
}

struct h0Dd {
    c2_: array<u32>,
}

struct Me {
    c2_: array<vec2<u32>>,
}

struct Ne {
    c2_: array<vec4<f32>>,
}

@id(13) override rh: bool = false;
@id(14) override sh: bool = false;
@id(0) override eh: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: DC;
@group(2) @binding(0)
var<storage, read_write> j0_: j0Dd;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(2) @binding(3)
var<storage, read_write> v4_: v4Dd;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Dd;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var Ob: sampler;
@group(1) @binding(13)
var U5_: sampler;
@group(0) @binding(3)
var<storage> BD: Me;
@group(0) @binding(4)
var<storage> RB: Ne;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = m.o6_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if rh {
        let _e65 = m.Ue;
        j0_.c2_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if sh {
        let _e70 = textureLoad(JC, _e31, 0i);
        j0_.c2_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = m.Ve;
    v4_.c2_[_e63] = _e75;
    if eh {
        h0_.c2_[_e63] = 0u;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
