struct WB {
    Hd: f32,
    Qe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    I8_: vec4<i32>,
    Ki: vec2<f32>,
    Re: vec2<f32>,
    w2_: u32,
    Oi: f32,
    w6_: u32,
    j3_: f32,
    Se: f32,
    xg: u32,
    I3_: f32,
    J3_: f32,
    Te: f32,
    Hi: u32,
    Ea: u32,
    P8_: f32,
    Q8_: f32,
}

struct Lf {
    x2_: array<u32>,
}

struct Lf_1 {
    x2_: array<atomic<u32>>,
}

var<private> o1_1: f32;
var<private> P4_1: vec2<f32>;
var<private> B3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(5)
var<storage, read_write> X0_: Lf_1;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(3) @binding(7)
var R8_: sampler;
@group(1) @binding(12)
var v4_: sampler;
var<private> M0_1: vec4<f32>;
var<private> I0_1: f32;
var<private> l2_1: vec2<f32>;
var<private> V0_1: vec4<f32>;
var<private> R0_1: f32;
var<private> N0_1: vec3<f32>;

fn main_1() {
    let _e29 = o1_1;
    let _e30 = P4_1;
    let _e32 = vec2<u32>(floor(_e30));
    let _e34 = B3_1[1u];
    let _e36 = B3_1[0u];
    let _e67 = u32(((abs(_e29) * 1024f) + 0.5f));
    let _e69 = j.w2_;
    let _e71 = (_e69 | (262144u - _e67));
    let _e74 = atomicMax((&X0_.x2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), _e71);
    if (_e74 >= _e69) {
        let _e79 = atomicAdd((&X0_.x2_[(_e36 + (((((_e32.y >> bitcast<u32>(5u)) * (_e34 << bitcast<u32>(5u))) + ((_e32.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e32.x & 28u) << bitcast<u32>(5u)) + ((_e32.y & 28u) << bitcast<u32>(2i)))) + (((_e32.y & 3u) << bitcast<u32>(2i)) + (_e32.x & 3u))))]), ((_e74 - max(_e74, _e71)) - _e67));
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) o1_: f32, @location(8) P4_: vec2<f32>, @location(7) @interpolate(flat, either) B3_: vec2<u32>, @location(0) M0_: vec4<f32>, @location(3) @interpolate(flat, either) I0_: f32, @location(4) @interpolate(flat, either) l2_: vec2<f32>, @location(5) V0_: vec4<f32>, @location(6) @interpolate(flat, either) R0_: f32, @location(9) N0_: vec3<f32>) {
    o1_1 = o1_;
    P4_1 = P4_;
    B3_1 = B3_;
    M0_1 = M0_;
    I0_1 = I0_;
    l2_1 = l2_;
    V0_1 = V0_;
    R0_1 = R0_;
    N0_1 = N0_;
    main_1();
}
