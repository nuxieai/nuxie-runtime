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

struct Ue {
    k2_: array<u32>,
}

struct Ue_1 {
    k2_: array<atomic<u32>>,
}

var<private> m1_1: f32;
var<private> G4_1: vec2<f32>;
var<private> q3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(6)
var<storage, read_write> W0_: Ue_1;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var DC: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(1) @binding(13)
var v5_: sampler;
var<private> a1_1: vec4<f32>;
var<private> F0_1: f32;
var<private> l1_1: vec2<f32>;
var<private> S0_1: vec4<f32>;
var<private> Q0_1: f32;
var<private> r1_1: vec3<f32>;

fn main_1() {
    let _e29 = m1_1;
    let _e30 = G4_1;
    let _e32 = vec2<u32>(floor(_e30));
    let _e34 = q3_1[1u];
    let _e36 = q3_1[0u];
    let _e67 = u32(((abs(_e29) * 1024f) + 0.5f));
    let _e69 = j.j2_;
    let _e71 = (_e69 | (262144u - _e67));
    let _e74 = atomicMax((&W0_.k2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), _e71);
    if (_e74 >= _e69) {
        let _e79 = atomicAdd((&W0_.k2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), ((_e74 - max(_e74, _e71)) - _e67));
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) m1_: f32, @location(8) G4_: vec2<f32>, @location(7) @interpolate(flat, either) q3_: vec2<u32>, @location(0) a1_: vec4<f32>, @location(3) @interpolate(flat, either) F0_: f32, @location(4) @interpolate(flat, either) l1_: vec2<f32>, @location(5) S0_: vec4<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(9) r1_: vec3<f32>) {
    m1_1 = m1_;
    G4_1 = G4_;
    q3_1 = q3_;
    a1_1 = a1_;
    F0_1 = F0_;
    l1_1 = l1_;
    S0_1 = S0_;
    Q0_1 = Q0_;
    r1_1 = r1_;
    main_1();
}
