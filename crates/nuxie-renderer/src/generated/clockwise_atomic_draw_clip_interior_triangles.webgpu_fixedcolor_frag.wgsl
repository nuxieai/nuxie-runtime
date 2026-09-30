struct Ae {
    g2_: array<u32>,
}

struct SB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    eh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    ih: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    bh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct Ae_1 {
    g2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(10) override Oh: bool = false;

var<private> j1_1: f32;
var<private> k3_1: vec2<u32>;
var<private> v4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> S0_: Ae_1;
@group(0) @binding(0)
var<uniform> j: SB;
var<private> i0_: vec4<f32>;
var<private> F1_: vec4<f32>;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(3) @binding(8)
var M9_: sampler;
@group(1) @binding(13)
var W5_: sampler;
var<private> X1_1: vec4<f32>;
var<private> D0_1: f32;
var<private> Y1_1: vec2<f32>;
var<private> O0_1: vec4<f32>;
var<private> g1_1: f32;
var<private> C2_1: vec3<f32>;

fn main_1() {
    var phi_181_: bool;
    var phi_182_: bool;
    var phi_465_: f32;
    var phi_464_: f32;
    var phi_463_: f32;
    var phi_466_: f32;
    var phi_470_: f32;

    let _e38 = j1_1;
    if Oh {
        let _e40 = k3_1[1u];
        let _e42 = k3_1[0u];
        let _e43 = v4_1;
        let _e45 = vec2<u32>(floor(_e43));
        let _e75 = atomicLoad((&S0_.g2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]));
        let _e76 = (_e38 >= 1f);
        phi_182_ = _e76;
        if _e76 {
            let _e78 = j.f2_;
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
            let _e88 = j.f2_;
            phi_463_ = _e38;
            if (_e75 < _e88) {
                let _e95 = (_e88 | (262144u + u32(((abs(_e38) * 1024f) + 0.5f))));
                let _e96 = atomicMax((&S0_.g2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), _e95);
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
                let _e115 = atomicAdd((&S0_.g2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), u32(((abs(_e109) * 1024f) + 0.5f)));
                phi_466_ = ((f32(bitcast<i32>(((_e115 & 524287u) - 262144u))) * 0.0009765625f) + _e38);
            }
            let _e123 = phi_466_;
            phi_470_ = (1f - _e123);
        }
        let _e126 = phi_470_;
        i0_ = vec4(_e126);
        F1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        i0_ = vec4(_e38);
        F1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) j1_: f32, @location(7) @interpolate(flat, either) k3_: vec2<u32>, @location(8) v4_: vec2<f32>, @location(0) X1_: vec4<f32>, @location(3) @interpolate(flat, either) D0_: f32, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @location(5) O0_: vec4<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(9) C2_: vec3<f32>) -> FragmentOutput {
    j1_1 = j1_;
    k3_1 = k3_;
    v4_1 = v4_;
    X1_1 = X1_;
    D0_1 = D0_;
    Y1_1 = Y1_;
    O0_1 = O0_;
    g1_1 = g1_;
    C2_1 = C2_;
    main_1();
    let _e20 = i0_;
    let _e21 = F1_;
    return FragmentOutput(_e20, _e21);
}
