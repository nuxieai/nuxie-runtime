struct ee {
    c2_: array<u32>,
}

struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

struct ee_1 {
    c2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(9) override gh: bool = false;

var<private> i1_1: f32;
var<private> d3_1: vec2<u32>;
var<private> l4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> P0_: ee_1;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> h0_: vec4<f32>;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(3) @binding(8)
var Kb: sampler;
@group(1) @binding(13)
var S5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> B0_1: f32;
var<private> U1_1: vec2<f32>;
var<private> L0_1: vec4<f32>;
var<private> e2_1: f32;

fn main_1() {
    var phi_181_: bool;
    var phi_182_: bool;
    var phi_460_: f32;
    var phi_459_: f32;
    var phi_458_: f32;
    var phi_461_: f32;
    var phi_465_: f32;

    let _e37 = i1_1;
    if gh {
        let _e39 = d3_1[1u];
        let _e41 = d3_1[0u];
        let _e42 = l4_1;
        let _e44 = vec2<u32>(floor(_e42));
        let _e74 = atomicLoad((&P0_.c2_[(_e41 + (((((_e44.y >> bitcast<u32>(5u)) * (_e39 << bitcast<u32>(5u))) + ((_e44.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e44.x & 28u) << bitcast<u32>(5u)) + ((_e44.y & 28u) << bitcast<u32>(2i)))) + (((_e44.y & 3u) << bitcast<u32>(2i)) + (_e44.x & 3u))))]));
        let _e75 = (_e37 >= 1f);
        phi_182_ = _e75;
        if _e75 {
            let _e77 = n.a2_;
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
            let _e87 = n.a2_;
            phi_458_ = _e37;
            if (_e74 < _e87) {
                let _e94 = (_e87 | (262144u + u32(((abs(_e37) * 1024f) + 0.5f))));
                let _e95 = atomicMax((&P0_.c2_[(_e41 + (((((_e44.y >> bitcast<u32>(5u)) * (_e39 << bitcast<u32>(5u))) + ((_e44.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e44.x & 28u) << bitcast<u32>(5u)) + ((_e44.y & 28u) << bitcast<u32>(2i)))) + (((_e44.y & 3u) << bitcast<u32>(2i)) + (_e44.x & 3u))))]), _e94);
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
                let _e114 = atomicAdd((&P0_.c2_[(_e41 + (((((_e44.y >> bitcast<u32>(5u)) * (_e39 << bitcast<u32>(5u))) + ((_e44.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e44.x & 28u) << bitcast<u32>(5u)) + ((_e44.y & 28u) << bitcast<u32>(2i)))) + (((_e44.y & 3u) << bitcast<u32>(2i)) + (_e44.x & 3u))))]), u32(((abs(_e108) * 1024f) + 0.5f)));
                phi_461_ = ((f32(bitcast<i32>(((_e114 & 524287u) - 262144u))) * 0.0009765625f) + _e37);
            }
            let _e122 = phi_461_;
            phi_465_ = (1f - _e122);
        }
        let _e125 = phi_465_;
        h0_ = vec4(_e125);
        C1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        h0_ = vec4(_e37);
        C1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) i1_: f32, @location(7) @interpolate(flat, either) d3_: vec2<u32>, @location(8) l4_: vec2<f32>, @location(0) f1_: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @location(5) L0_: vec4<f32>, @location(6) @interpolate(flat, either) e2_: f32) -> FragmentOutput {
    i1_1 = i1_;
    d3_1 = d3_;
    l4_1 = l4_;
    f1_1 = f1_;
    B0_1 = B0_;
    U1_1 = U1_;
    L0_1 = L0_;
    e2_1 = e2_;
    main_1();
    let _e18 = h0_;
    let _e19 = C1_;
    return FragmentOutput(_e18, _e19);
}
