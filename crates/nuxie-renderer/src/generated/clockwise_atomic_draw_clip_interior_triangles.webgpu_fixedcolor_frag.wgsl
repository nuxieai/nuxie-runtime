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

var<private> j1_1: f32;
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
    var phi_181_: bool;
    var phi_182_: bool;
    var phi_460_: f32;
    var phi_459_: f32;
    var phi_458_: f32;
    var phi_461_: f32;
    var phi_465_: f32;

    let _e37 = j1_1;
    if Wg {
        let _e39 = e3_1[1u];
        let _e41 = e3_1[0u];
        let _e42 = j4_1;
        let _e44 = vec2<u32>(floor(_e42));
        let _e74 = atomicLoad((&S0_.X1_[(_e41 + (((((_e44.y >> bitcast<u32>(5u)) * (_e39 << bitcast<u32>(5u))) + ((_e44.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e44.x & 28u) << bitcast<u32>(5u)) + ((_e44.y & 28u) << bitcast<u32>(2i)))) + (((_e44.y & 3u) << bitcast<u32>(2i)) + (_e44.x & 3u))))]));
        let _e75 = (_e37 >= 1f);
        phi_182_ = _e75;
        if _e75 {
            let _e77 = k.W1_;
            let _e78 = (_e74 < _e77);
            phi_181_ = _e78;
            if !(_e78) {
                phi_181_ = (_e74 >= (_e77 | 262144u));
            }
            let _e83 = phi_181_;
            phi_182_ = _e83;
        }
        let _e85 = phi_182_;
        if _e85 {
            phi_465_ = 0f;
        } else {
            let _e87 = k.W1_;
            phi_458_ = _e37;
            if (_e74 < _e87) {
                let _e94 = (_e87 | (262144u + u32(((abs(_e37) * 1024f) + 0.5f))));
                let _e95 = atomicMax((&S0_.X1_[(_e41 + (((((_e44.y >> bitcast<u32>(5u)) * (_e39 << bitcast<u32>(5u))) + ((_e44.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e44.x & 28u) << bitcast<u32>(5u)) + ((_e44.y & 28u) << bitcast<u32>(2i)))) + (((_e44.y & 3u) << bitcast<u32>(2i)) + (_e44.x & 3u))))]), _e94);
                if (_e95 <= _e87) {
                    phi_459_ = 0f;
                } else {
                    phi_460_ = _e37;
                    if (_e95 < _e94) {
                        phi_460_ = (f32(bitcast<i32>(((_e95 & 524287u) - 262144u))) * 0.0009765625f);
                    }
                    let _e104 = phi_460_;
                    phi_459_ = _e104;
                }
                let _e106 = phi_459_;
                phi_458_ = _e106;
            }
            let _e108 = phi_458_;
            phi_461_ = _e37;
            if (_e108 > 0f) {
                let _e114 = atomicAdd((&S0_.X1_[(_e41 + (((((_e44.y >> bitcast<u32>(5u)) * (_e39 << bitcast<u32>(5u))) + ((_e44.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e44.x & 28u) << bitcast<u32>(5u)) + ((_e44.y & 28u) << bitcast<u32>(2i)))) + (((_e44.y & 3u) << bitcast<u32>(2i)) + (_e44.x & 3u))))]), u32(((abs(_e108) * 1024f) + 0.5f)));
                phi_461_ = ((f32(bitcast<i32>(((_e114 & 524287u) - 262144u))) * 0.0009765625f) + _e37);
            }
            let _e122 = phi_461_;
            phi_465_ = (1f - _e122);
        }
        let _e125 = phi_465_;
        d0_ = vec4(_e125);
        l1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        d0_ = vec4(_e37);
        l1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat) j1_: f32, @location(7) @interpolate(flat) e3_: vec2<u32>, @location(8) j4_: vec2<f32>, @location(0) i1_: vec4<f32>, @location(3) @interpolate(flat) z0_: f32, @location(4) @interpolate(flat) S1_: vec2<f32>, @location(5) N0_: vec4<f32>, @location(6) @interpolate(flat) Z1_: f32) -> FragmentOutput {
    j1_1 = j1_;
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
