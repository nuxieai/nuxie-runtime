struct zf {
    r2_: array<u32>,
}

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

struct zf_1 {
    r2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(10) override gj: bool = false;

var<private> n1_1: f32;
var<private> z3_1: vec2<u32>;
var<private> L4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> Y0_: zf_1;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> m0_: vec4<f32>;
var<private> L1_: vec4<f32>;
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
    var phi_181_: bool;
    var phi_182_: bool;
    var phi_466_: f32;
    var phi_465_: f32;
    var phi_464_: f32;
    var phi_467_: f32;
    var phi_471_: f32;

    let _e38 = n1_1;
    if gj {
        let _e40 = z3_1[1u];
        let _e42 = z3_1[0u];
        let _e43 = L4_1;
        let _e45 = vec2<u32>(floor(_e43));
        let _e75 = atomicLoad((&Y0_.r2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]));
        let _e76 = (_e38 >= 1f);
        phi_182_ = _e76;
        if _e76 {
            let _e78 = j.q2_;
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
            phi_471_ = 0f;
        } else {
            let _e88 = j.q2_;
            phi_464_ = _e38;
            if (_e75 < _e88) {
                let _e95 = (_e88 | (262144u + u32(((abs(_e38) * 1024f) + 0.5f))));
                let _e96 = atomicMax((&Y0_.r2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), _e95);
                if (_e96 <= _e88) {
                    phi_465_ = 0f;
                } else {
                    phi_466_ = _e38;
                    if (_e96 < _e95) {
                        phi_466_ = (f32(bitcast<i32>(((_e96 & 524287u) - 262144u))) * 0.0009765625f);
                    }
                    let _e105 = phi_466_;
                    phi_465_ = _e105;
                }
                let _e107 = phi_465_;
                phi_464_ = _e107;
            }
            let _e109 = phi_464_;
            phi_467_ = _e38;
            if (_e109 > 0f) {
                let _e115 = atomicAdd((&Y0_.r2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), u32(((abs(_e109) * 1024f) + 0.5f)));
                phi_467_ = ((f32(bitcast<i32>(((_e115 & 524287u) - 262144u))) * 0.0009765625f) + _e38);
            }
            let _e123 = phi_467_;
            phi_471_ = (1f - _e123);
        }
        let _e126 = phi_471_;
        m0_ = vec4(_e126);
        L1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        m0_ = vec4(_e38);
        L1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) n1_: f32, @location(7) @interpolate(flat, either) z3_: vec2<u32>, @location(8) L4_: vec2<f32>, @location(0) O0_: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32, @location(4) @interpolate(flat, either) i2_: vec2<f32>, @location(5) V0_: vec4<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(9) U0_: vec3<f32>) -> FragmentOutput {
    n1_1 = n1_;
    z3_1 = z3_;
    L4_1 = L4_;
    O0_1 = O0_;
    G0_1 = G0_;
    i2_1 = i2_;
    V0_1 = V0_;
    P0_1 = P0_;
    U0_1 = U0_;
    main_1();
    let _e20 = m0_;
    let _e21 = L1_;
    return FragmentOutput(_e20, _e21);
}
