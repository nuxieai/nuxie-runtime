struct Re {
    j2_: array<u32>,
}

struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct Re_1 {
    j2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(10) override li: bool = false;

var<private> m1_1: f32;
var<private> q3_1: vec2<u32>;
var<private> F4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> V0_: Re_1;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> m0_: vec4<f32>;
var<private> J1_: vec4<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(1) @binding(13)
var f6_: sampler;
var<private> a1_1: vec4<f32>;
var<private> F0_1: f32;
var<private> l1_1: vec2<f32>;
var<private> R0_1: vec4<f32>;
var<private> Q0_1: f32;
var<private> F1_1: vec3<f32>;

fn main_1() {
    var phi_181_: bool;
    var phi_182_: bool;
    var phi_464_: f32;
    var phi_463_: f32;
    var phi_462_: f32;
    var phi_465_: f32;
    var phi_469_: f32;

    let _e38 = m1_1;
    if li {
        let _e40 = q3_1[1u];
        let _e42 = q3_1[0u];
        let _e43 = F4_1;
        let _e45 = vec2<u32>(floor(_e43));
        let _e75 = atomicLoad((&V0_.j2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]));
        let _e76 = (_e38 >= 1f);
        phi_182_ = _e76;
        if _e76 {
            let _e78 = j.i2_;
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
            phi_469_ = 0f;
        } else {
            let _e88 = j.i2_;
            phi_462_ = _e38;
            if (_e75 < _e88) {
                let _e95 = (_e88 | (262144u + u32(((abs(_e38) * 1024f) + 0.5f))));
                let _e96 = atomicMax((&V0_.j2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), _e95);
                if (_e96 <= _e88) {
                    phi_463_ = 0f;
                } else {
                    phi_464_ = _e38;
                    if (_e96 < _e95) {
                        phi_464_ = (f32(bitcast<i32>(((_e96 & 524287u) - 262144u))) * 0.0009765625f);
                    }
                    let _e105 = phi_464_;
                    phi_463_ = _e105;
                }
                let _e107 = phi_463_;
                phi_462_ = _e107;
            }
            let _e109 = phi_462_;
            phi_465_ = _e38;
            if (_e109 > 0f) {
                let _e115 = atomicAdd((&V0_.j2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), u32(((abs(_e109) * 1024f) + 0.5f)));
                phi_465_ = ((f32(bitcast<i32>(((_e115 & 524287u) - 262144u))) * 0.0009765625f) + _e38);
            }
            let _e123 = phi_465_;
            phi_469_ = (1f - _e123);
        }
        let _e126 = phi_469_;
        m0_ = vec4(_e126);
        J1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        m0_ = vec4(_e38);
        J1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) m1_: f32, @location(7) @interpolate(flat, either) q3_: vec2<u32>, @location(8) F4_: vec2<f32>, @location(0) a1_: vec4<f32>, @location(3) @interpolate(flat, either) F0_: f32, @location(4) @interpolate(flat, either) l1_: vec2<f32>, @location(5) R0_: vec4<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(9) F1_: vec3<f32>) -> FragmentOutput {
    m1_1 = m1_;
    q3_1 = q3_;
    F4_1 = F4_;
    a1_1 = a1_;
    F0_1 = F0_;
    l1_1 = l1_;
    R0_1 = R0_;
    Q0_1 = Q0_;
    F1_1 = F1_;
    main_1();
    let _e20 = m0_;
    let _e21 = J1_;
    return FragmentOutput(_e20, _e21);
}
