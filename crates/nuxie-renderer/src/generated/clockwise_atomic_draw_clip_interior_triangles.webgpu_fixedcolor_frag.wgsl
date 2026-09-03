struct ke {
    d2_: array<u32>,
}

struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

struct ke_1 {
    d2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(10) override ph: bool = false;

var<private> i1_1: f32;
var<private> f3_1: vec2<u32>;
var<private> n4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> P0_: ke_1;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> h0_: vec4<f32>;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(3) @binding(8)
var Pb: sampler;
@group(1) @binding(13)
var V5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> B0_1: f32;
var<private> V1_1: vec2<f32>;
var<private> M0_1: vec4<f32>;
var<private> f2_1: f32;
var<private> A2_1: vec3<f32>;

fn main_1() {
    var phi_181_: bool;
    var phi_182_: bool;
    var phi_465_: f32;
    var phi_464_: f32;
    var phi_463_: f32;
    var phi_466_: f32;
    var phi_470_: f32;

    let _e38 = i1_1;
    if ph {
        let _e40 = f3_1[1u];
        let _e42 = f3_1[0u];
        let _e43 = n4_1;
        let _e45 = vec2<u32>(floor(_e43));
        let _e75 = atomicLoad((&P0_.d2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]));
        let _e76 = (_e38 >= 1f);
        phi_182_ = _e76;
        if _e76 {
            let _e78 = m.c2_;
            let _e79 = (_e75 < _e78);
            phi_181_ = _e79;
            if !(_e79) {
                phi_181_ = (_e75 >= (_e78 | 262144u));
            }
            let _e84 = phi_181_;
            phi_182_ = _e84;
        }
        let _e86 = phi_182_;
        if _e86 {
            phi_470_ = 0f;
        } else {
            let _e88 = m.c2_;
            phi_463_ = _e38;
            if (_e75 < _e88) {
                let _e95 = (_e88 | (262144u + u32(((abs(_e38) * 1024f) + 0.5f))));
                let _e96 = atomicMax((&P0_.d2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), _e95);
                if (_e96 <= _e88) {
                    phi_464_ = 0f;
                } else {
                    phi_465_ = _e38;
                    if (_e96 < _e95) {
                        phi_465_ = (f32(bitcast<i32>(((_e96 & 524287u) - 262144u))) * 0.0009765625f);
                    }
                    let _e105 = phi_465_;
                    phi_464_ = _e105;
                }
                let _e107 = phi_464_;
                phi_463_ = _e107;
            }
            let _e109 = phi_463_;
            phi_466_ = _e38;
            if (_e109 > 0f) {
                let _e115 = atomicAdd((&P0_.d2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), u32(((abs(_e109) * 1024f) + 0.5f)));
                phi_466_ = ((f32(bitcast<i32>(((_e115 & 524287u) - 262144u))) * 0.0009765625f) + _e38);
            }
            let _e123 = phi_466_;
            phi_470_ = (1f - _e123);
        }
        let _e126 = phi_470_;
        h0_ = vec4(_e126);
        C1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        h0_ = vec4(_e38);
        C1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) i1_: f32, @location(7) @interpolate(flat, either) f3_: vec2<u32>, @location(8) n4_: vec2<f32>, @location(0) f1_: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(4) @interpolate(flat, either) V1_: vec2<f32>, @location(5) M0_: vec4<f32>, @location(6) @interpolate(flat, either) f2_: f32, @location(9) A2_: vec3<f32>) -> FragmentOutput {
    i1_1 = i1_;
    f3_1 = f3_;
    n4_1 = n4_;
    f1_1 = f1_;
    B0_1 = B0_;
    V1_1 = V1_;
    M0_1 = M0_;
    f2_1 = f2_;
    A2_1 = A2_;
    main_1();
    let _e20 = h0_;
    let _e21 = C1_;
    return FragmentOutput(_e20, _e21);
}
