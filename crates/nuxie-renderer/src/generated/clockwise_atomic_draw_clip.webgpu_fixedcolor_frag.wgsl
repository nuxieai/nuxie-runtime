struct Yd {
    X1_: array<u32>,
}

struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct Yd_1 {
    X1_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(9) override Wg: bool = false;

var<private> I_1: vec4<f32>;
var<private> e3_1: vec2<u32>;
var<private> j4_1: vec2<f32>;
@group(0) @binding(7)
var<storage, read_write> S0_: Yd_1;
@group(0) @binding(0)
var<uniform> k: NB;
var<private> d0_: vec4<f32>;
var<private> l1_: vec4<f32>;
@group(3) @binding(10)
var T9_: sampler;
@group(0) @binding(9)
var DD: texture_2d<f32>;
@group(0) @binding(10)
var QC: texture_2d<f32>;
@group(1) @binding(12)
var AC: texture_2d<f32>;
@group(3) @binding(9)
var Bb: sampler;
@group(1) @binding(14)
var R5_: sampler;
var<private> i1_1: vec4<f32>;
var<private> z0_1: f32;
var<private> S1_1: vec2<f32>;
var<private> N0_1: vec4<f32>;
var<private> Z1_1: f32;

fn main_1() {
    var phi_183_: bool;
    var phi_184_: bool;
    var phi_461_: f32;
    var phi_460_: f32;
    var phi_459_: f32;
    var phi_462_: f32;
    var phi_466_: f32;

    let _e38 = I_1[0u];
    if Wg {
        let _e40 = e3_1[1u];
        let _e42 = e3_1[0u];
        let _e43 = j4_1;
        let _e45 = vec2<u32>(floor(_e43));
        let _e75 = atomicLoad((&S0_.X1_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]));
        let _e76 = (_e38 >= 1f);
        phi_184_ = _e76;
        if _e76 {
            let _e78 = k.W1_;
            let _e79 = (_e75 < _e78);
            phi_183_ = _e79;
            if !(_e79) {
                phi_183_ = (_e75 >= (_e78 | 262144u));
            }
            let _e84 = phi_183_;
            phi_184_ = _e84;
        }
        let _e86 = phi_184_;
        if _e86 {
            phi_466_ = 0f;
        } else {
            let _e88 = k.W1_;
            phi_459_ = _e38;
            if (_e75 < _e88) {
                let _e95 = (_e88 | (262144u + u32(((abs(_e38) * 1024f) + 0.5f))));
                let _e96 = atomicMax((&S0_.X1_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), _e95);
                if (_e96 <= _e88) {
                    phi_460_ = 0f;
                } else {
                    phi_461_ = _e38;
                    if (_e96 < _e95) {
                        phi_461_ = (f32(bitcast<i32>(((_e96 & 524287u) - 262144u))) * 0.0009765625f);
                    }
                    let _e105 = phi_461_;
                    phi_460_ = _e105;
                }
                let _e107 = phi_460_;
                phi_459_ = _e107;
            }
            let _e109 = phi_459_;
            phi_462_ = _e38;
            if (_e109 > 0f) {
                let _e115 = atomicAdd((&S0_.X1_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), u32(((abs(_e109) * 1024f) + 0.5f)));
                phi_462_ = ((f32(bitcast<i32>(((_e115 & 524287u) - 262144u))) * 0.0009765625f) + _e38);
            }
            let _e123 = phi_462_;
            phi_466_ = (1f - _e123);
        }
        let _e126 = phi_466_;
        d0_ = vec4(_e126);
        l1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        d0_ = vec4(_e38);
        l1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(2) I: vec4<f32>, @location(7) @interpolate(flat) e3_: vec2<u32>, @location(8) j4_: vec2<f32>, @location(0) i1_: vec4<f32>, @location(3) @interpolate(flat) z0_: f32, @location(4) @interpolate(flat) S1_: vec2<f32>, @location(5) N0_: vec4<f32>, @location(6) @interpolate(flat) Z1_: f32) -> FragmentOutput {
    I_1 = I;
    e3_1 = e3_;
    j4_1 = j4_;
    i1_1 = i1_;
    z0_1 = z0_;
    S1_1 = S1_;
    N0_1 = N0_;
    Z1_1 = Z1_;
    main_1();
    let _e18 = d0_;
    let _e19 = l1_;
    return FragmentOutput(_e18, _e19);
}
