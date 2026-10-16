struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Ci: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Gi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    G3_: f32,
    H3_: f32,
    He: f32,
    zi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct Bf {
    r2_: array<u32>,
}

struct Bf_1 {
    r2_: array<atomic<u32>>,
}

var<private> o1_1: f32;
var<private> L4_1: vec2<f32>;
var<private> A3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(6)
var<storage, read_write> Y0_: Bf_1;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
@group(1) @binding(13)
var U4_: sampler;
var<private> P0_1: vec4<f32>;
var<private> G0_1: f32;
var<private> i2_1: vec2<f32>;
var<private> V0_1: vec4<f32>;
var<private> Q0_1: f32;
var<private> U0_1: vec3<f32>;

fn main_1() {
    let _e29 = o1_1;
    let _e30 = L4_1;
    let _e32 = vec2<u32>(floor(_e30));
    let _e34 = A3_1[1u];
    let _e36 = A3_1[0u];
    let _e67 = u32(((abs(_e29) * 1024f) + 0.5f));
    let _e69 = j.q2_;
    let _e71 = (_e69 | (262144u - _e67));
    let _e74 = atomicMax((&Y0_.r2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), _e71);
    if (_e74 >= _e69) {
        let _e79 = atomicAdd((&Y0_.r2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), ((_e74 - max(_e74, _e71)) - _e67));
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) o1_: f32, @location(8) L4_: vec2<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(0) P0_: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32, @location(4) @interpolate(flat, either) i2_: vec2<f32>, @location(5) V0_: vec4<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(9) U0_: vec3<f32>) {
    o1_1 = o1_;
    L4_1 = L4_;
    A3_1 = A3_;
    P0_1 = P0_;
    G0_1 = G0_;
    i2_1 = i2_;
    V0_1 = V0_;
    Q0_1 = Q0_;
    U0_1 = U0_;
    main_1();
}
