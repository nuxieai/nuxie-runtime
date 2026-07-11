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

@id(0) override Mg: bool = true;

var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0) 
var<uniform> k: NB;
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
@group(1) @binding(12) 
var AC: texture_2d<f32>;
@group(3) @binding(9) 
var Bb: sampler;
@group(1) @binding(14) 
var R5_: sampler;
@group(0) @binding(4) 
var<storage> TC: Be;
@group(0) @binding(5) 
var<storage> PB: Ce;
var<private> l1_: vec4<f32>;

fn main_1() {
    let _e25 = gl_FragCoord_1;
    let _e29 = bitcast<vec2<u32>>(vec2<i32>(floor(_e25.xy)));
    let _e31 = k.q5_;
    let _e60 = bitcast<i32>((((((_e29.y >> bitcast<u32>(5u)) * (((_e31 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e29.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e29.x & 28u) << bitcast<u32>(5u)) + ((_e29.y & 28u) << bitcast<u32>(2i)))) + (((_e29.y & 3u) << bitcast<u32>(2i)) + (_e29.x & 3u))));
    let _e62 = k.Ke;
    p4_.X1_[_e60] = _e62;
    if Mg {
        d0_.X1_[_e60] = 0u;
    }
    discard;
}

@fragment 
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = l1_;
    return _e3;
}
