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

var<private> O_1: vec4<f32>;
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
    var phi_183_: bool;
    var phi_184_: bool;
    var phi_461_: f32;
    var phi_460_: f32;
    var phi_459_: f32;
    var phi_462_: f32;
    var phi_466_: f32;

    let _e38 = O_1[0u];
    if gh {
        let _e40 = d3_1[1u];
        let _e42 = d3_1[0u];
        let _e43 = l4_1;
        let _e45 = vec2<u32>(floor(_e43));
        let _e75 = atomicLoad((&P0_.c2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]));
        let _e76 = (_e38 >= 1f);
        phi_184_ = _e76;
        if _e76 {
            let _e78 = n.a2_;
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
            let _e88 = n.a2_;
            phi_459_ = _e38;
            if (_e75 < _e88) {
                let _e95 = (_e88 | (262144u + u32(((abs(_e38) * 1024f) + 0.5f))));
                let _e96 = atomicMax((&P0_.c2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), _e95);
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
                let _e115 = atomicAdd((&P0_.c2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), u32(((abs(_e109) * 1024f) + 0.5f)));
                phi_462_ = ((f32(bitcast<i32>(((_e115 & 524287u) - 262144u))) * 0.0009765625f) + _e38);
            }
            let _e123 = phi_462_;
            phi_466_ = (1f - _e123);
        }
        let _e126 = phi_466_;
        h0_ = vec4(_e126);
        C1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        h0_ = vec4(_e38);
        C1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(2) O: vec4<f32>, @location(7) @interpolate(flat, either) d3_: vec2<u32>, @location(8) l4_: vec2<f32>, @location(0) f1_: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @location(5) L0_: vec4<f32>, @location(6) @interpolate(flat, either) e2_: f32) -> FragmentOutput {
    O_1 = O;
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
