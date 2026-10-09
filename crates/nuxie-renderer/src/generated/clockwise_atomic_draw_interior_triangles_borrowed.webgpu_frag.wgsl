struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

struct zf {
    r2_: array<u32>,
}

struct zf_1 {
    r2_: array<atomic<u32>>,
}

var<private> n1_1: f32;
var<private> L4_1: vec2<f32>;
var<private> z3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(6)
var<storage, read_write> Y0_: zf_1;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
@group(1) @binding(13)
var U4_: sampler;
var<private> O0_1: vec4<f32>;
var<private> G0_1: f32;
var<private> i2_1: vec2<f32>;
var<private> V0_1: vec4<f32>;
var<private> P0_1: f32;
var<private> U0_1: vec3<f32>;

fn main_1() {
    let _e29 = n1_1;
    let _e30 = L4_1;
    let _e32 = vec2<u32>(floor(_e30));
    let _e34 = z3_1[1u];
    let _e36 = z3_1[0u];
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
fn main(@location(1) @interpolate(flat, either) n1_: f32, @location(8) L4_: vec2<f32>, @location(7) @interpolate(flat, either) z3_: vec2<u32>, @location(0) O0_: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32, @location(4) @interpolate(flat, either) i2_: vec2<f32>, @location(5) V0_: vec4<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(9) U0_: vec3<f32>) {
    n1_1 = n1_;
    L4_1 = L4_;
    z3_1 = z3_;
    O0_1 = O0_;
    G0_1 = G0_;
    i2_1 = i2_;
    V0_1 = V0_;
    P0_1 = P0_;
    U0_1 = U0_;
    main_1();
}
