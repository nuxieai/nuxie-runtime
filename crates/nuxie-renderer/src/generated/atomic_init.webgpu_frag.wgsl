struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct g0qd {
    X1_: array<u32>,
}

struct p4qd {
    X1_: array<u32>,
}

struct d0qd {
    X1_: array<u32>,
}

struct Be {
    X1_: array<vec2<u32>>,
}

struct Ce {
    X1_: array<vec4<f32>>,
}

@id(11) override Yg: bool = false;
@id(12) override Zg: bool = false;
@id(0) override Mg: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0) 
var<uniform> k: NB;
@group(2) @binding(0) 
var<storage, read_write> g0_: g0qd;
@group(1) @binding(12) 
var AC: texture_2d<f32>;
@group(2) @binding(3) 
var<storage, read_write> p4_: p4qd;
@group(2) @binding(1) 
var<storage, read_write> d0_: d0qd;
@group(3) @binding(10) 
var T9_: sampler;
@group(0) @binding(9) 
var DD: texture_2d<f32>;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
@group(3) @binding(9) 
var Bb: sampler;
@group(1) @binding(14) 
var R5_: sampler;
@group(0) @binding(4) 
var<storage> TC: Be;
@group(0) @binding(5) 
var<storage> PB: Ce;

fn main_1() {
    let _e28 = gl_FragCoord_1;
    let _e31 = vec2<i32>(floor(_e28.xy));
    let _e32 = bitcast<vec2<u32>>(_e31);
    let _e34 = k.q5_;
    let _e63 = bitcast<i32>((((((_e32.y >> bitcast<u32>(5u)) * (((_e34 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))));
    if Yg {
        let _e65 = k.Je;
        g0_.X1_[_e63] = pack4x8unorm(unpack4x8unorm(_e65));
    }
    if Zg {
        let _e70 = textureLoad(AC, _e31, 0i);
        g0_.X1_[_e63] = pack4x8unorm(_e70);
    }
    let _e75 = k.Ke;
    p4_.X1_[_e63] = _e75;
    if Mg {
        d0_.X1_[_e63] = 0u;
    }
    return;
}

@fragment 
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
