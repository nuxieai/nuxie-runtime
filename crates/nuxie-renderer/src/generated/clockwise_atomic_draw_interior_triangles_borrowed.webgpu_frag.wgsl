struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    H8_: vec4<i32>,
    Fi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ji: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Ci: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct Ef {
    v2_: array<u32>,
}

struct Ef_1 {
    v2_: array<atomic<u32>>,
}

var<private> p1_1: f32;
var<private> N4_1: vec2<f32>;
var<private> A3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(6)
var<storage, read_write> Y0_: Ef_1;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
@group(1) @binding(13)
var W4_: sampler;
var<private> P0_1: vec4<f32>;
var<private> G0_1: f32;
var<private> k2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
var<private> Q0_1: f32;
var<private> V0_1: vec3<f32>;

fn main_1() {
    let _e29 = p1_1;
    let _e30 = N4_1;
    let _e32 = vec2<u32>(floor(_e30));
    let _e34 = A3_1[1u];
    let _e36 = A3_1[0u];
    let _e67 = u32(((abs(_e29) * 1024f) + 0.5f));
    let _e69 = j.r2_;
    let _e71 = (_e69 | (262144u - _e67));
    let _e74 = atomicMax((&Y0_.v2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), _e71);
    if (_e74 >= _e69) {
        let _e79 = atomicAdd((&Y0_.v2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), ((_e74 - max(_e74, _e71)) - _e67));
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) p1_: f32, @location(8) N4_: vec2<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(0) P0_: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32, @location(4) @interpolate(flat, either) k2_: vec2<f32>, @location(5) W0_: vec4<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(9) V0_: vec3<f32>) {
    p1_1 = p1_;
    N4_1 = N4_;
    A3_1 = A3_;
    P0_1 = P0_;
    G0_1 = G0_;
    k2_1 = k2_;
    W0_1 = W0_;
    Q0_1 = Q0_;
    V0_1 = V0_;
    main_1();
}
