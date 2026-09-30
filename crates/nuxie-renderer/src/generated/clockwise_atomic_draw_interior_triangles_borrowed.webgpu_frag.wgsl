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

struct ze {
    e2_: array<u32>,
}

struct ze_1 {
    e2_: array<atomic<u32>>,
}

var<private> h1_1: f32;
var<private> p4_1: vec2<f32>;
var<private> g3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> l: BC;
@group(0) @binding(6)
var<storage, read_write> Q0_: ze_1;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(13)
var X5_: sampler;
var<private> V1_1: vec4<f32>;
var<private> C0_1: f32;
var<private> W1_1: vec2<f32>;
var<private> M0_1: vec4<f32>;
var<private> g2_1: f32;
var<private> C2_1: vec3<f32>;

fn main_1() {
    let _e29 = h1_1;
    let _e30 = p4_1;
    let _e32 = vec2<u32>(floor(_e30));
    let _e34 = g3_1[1u];
    let _e36 = g3_1[0u];
    let _e67 = u32(((abs(_e29) * 1024f) + 0.5f));
    let _e69 = l.d2_;
    let _e71 = (_e69 | (262144u - _e67));
    let _e74 = atomicMax((&Q0_.e2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), _e71);
    if (_e74 >= _e69) {
        let _e79 = atomicAdd((&Q0_.e2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), ((_e74 - max(_e74, _e71)) - _e67));
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) h1_: f32, @location(8) p4_: vec2<f32>, @location(7) @interpolate(flat, either) g3_: vec2<u32>, @location(0) V1_: vec4<f32>, @location(3) @interpolate(flat, either) C0_: f32, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @location(5) M0_: vec4<f32>, @location(6) @interpolate(flat, either) g2_: f32, @location(9) C2_: vec3<f32>) {
    h1_1 = h1_;
    p4_1 = p4_;
    g3_1 = g3_;
    V1_1 = V1_;
    C0_1 = C0_;
    W1_1 = W1_;
    M0_1 = M0_;
    g2_1 = g2_;
    C2_1 = C2_;
    main_1();
}
