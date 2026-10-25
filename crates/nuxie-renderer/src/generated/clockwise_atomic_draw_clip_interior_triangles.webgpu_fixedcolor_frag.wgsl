struct Ef {
    v2_: array<u32>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct Ef_1 {
    v2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(11) override qj: bool = false;

var<private> p1_1: f32;
var<private> A3_1: vec2<u32>;
var<private> N4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> Y0_: Ef_1;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> m0_: vec4<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(3) @binding(8)
var R8_: sampler;
@group(1) @binding(13)
var W4_: sampler;
var<private> P0_1: vec4<f32>;
var<private> G0_1: f32;
var<private> k2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
var<private> Q0_1: f32;
var<private> V0_1: vec3<f32>;

fn main_1() {
    var phi_181_: bool;
    var phi_182_: bool;
    var phi_467_: f32;
    var phi_466_: f32;
    var phi_465_: f32;
    var phi_468_: f32;
    var phi_472_: f32;

    let _e38 = p1_1;
    if qj {
        let _e40 = A3_1[1u];
        let _e42 = A3_1[0u];
        let _e43 = N4_1;
        let _e45 = vec2<u32>(floor(_e43));
        let _e75 = atomicLoad((&Y0_.v2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]));
        let _e76 = (_e38 >= 1f);
        phi_182_ = _e76;
        if _e76 {
            let _e78 = j.r2_;
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
            phi_472_ = 0f;
        } else {
            let _e88 = j.r2_;
            phi_465_ = _e38;
            if (_e75 < _e88) {
                let _e95 = (_e88 | (262144u + u32(((abs(_e38) * 1024f) + 0.5f))));
                let _e96 = atomicMax((&Y0_.v2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), _e95);
                if (_e96 <= _e88) {
                    phi_466_ = 0f;
                } else {
                    phi_467_ = _e38;
                    if (_e96 < _e95) {
                        phi_467_ = (f32(bitcast<i32>(((_e96 & 524287u) - 262144u))) * 0.0009765625f);
                    }
                    let _e105 = phi_467_;
                    phi_466_ = _e105;
                }
                let _e107 = phi_466_;
                phi_465_ = _e107;
            }
            let _e109 = phi_465_;
            phi_468_ = _e38;
            if (_e109 > 0f) {
                let _e115 = atomicAdd((&Y0_.v2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), u32(((abs(_e109) * 1024f) + 0.5f)));
                phi_468_ = ((f32(bitcast<i32>(((_e115 & 524287u) - 262144u))) * 0.0009765625f) + _e38);
            }
            let _e123 = phi_468_;
            phi_472_ = (1f - _e123);
        }
        let _e126 = phi_472_;
        m0_ = vec4(_e126);
        Q1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        m0_ = vec4(_e38);
        Q1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) p1_: f32, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(8) N4_: vec2<f32>, @location(0) P0_: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32, @location(4) @interpolate(flat, either) k2_: vec2<f32>, @location(5) W0_: vec4<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(9) V0_: vec3<f32>) -> FragmentOutput {
    p1_1 = p1_;
    A3_1 = A3_;
    N4_1 = N4_;
    P0_1 = P0_;
    G0_1 = G0_;
    k2_1 = k2_;
    W0_1 = W0_;
    Q0_1 = Q0_;
    V0_1 = V0_;
    main_1();
    let _e20 = m0_;
    let _e21 = Q1_;
    return FragmentOutput(_e20, _e21);
}
