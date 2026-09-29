struct ye {
    e2_: array<u32>,
}

struct BC {
    qc: f32,
    Ad: f32,
    Ef: f32,
    Ff: f32,
    q6_: u32,
    Nb: u32,
    qf: u32,
    rf: u32,
    V7_: vec4<i32>,
    bh: vec2<f32>,
    Bd: vec2<f32>,
    d2_: u32,
    fh: f32,
    e6_: u32,
    T2_: f32,
    Cd: f32,
    lf: u32,
    B3_: f32,
    C3_: f32,
    Dd: f32,
    Yg: u32,
}

struct ye_1 {
    e2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(10) override Lh: bool = false;

var<private> h1_1: f32;
var<private> g3_1: vec2<u32>;
var<private> p4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> Q0_: ye_1;
@group(0) @binding(0)
var<uniform> n: BC;
var<private> h0_: vec4<f32>;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(13)
var W5_: sampler;
var<private> V1_1: vec4<f32>;
var<private> C0_1: f32;
var<private> W1_1: vec2<f32>;
var<private> M0_1: vec4<f32>;
var<private> g2_1: f32;
var<private> C2_1: vec3<f32>;

fn main_1() {
    var phi_181_: bool;
    var phi_182_: bool;
    var phi_465_: f32;
    var phi_464_: f32;
    var phi_463_: f32;
    var phi_466_: f32;
    var phi_470_: f32;

    let _e38 = h1_1;
    if Lh {
        let _e40 = g3_1[1u];
        let _e42 = g3_1[0u];
        let _e43 = p4_1;
        let _e45 = vec2<u32>(floor(_e43));
        let _e75 = atomicLoad((&Q0_.e2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]));
        let _e76 = (_e38 >= 1f);
        phi_182_ = _e76;
        if _e76 {
            let _e78 = n.d2_;
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
            let _e88 = n.d2_;
            phi_463_ = _e38;
            if (_e75 < _e88) {
                let _e95 = (_e88 | (262144u + u32(((abs(_e38) * 1024f) + 0.5f))));
                let _e96 = atomicMax((&Q0_.e2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), _e95);
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
                let _e115 = atomicAdd((&Q0_.e2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), u32(((abs(_e109) * 1024f) + 0.5f)));
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
fn main(@location(1) @interpolate(flat, either) h1_: f32, @location(7) @interpolate(flat, either) g3_: vec2<u32>, @location(8) p4_: vec2<f32>, @location(0) V1_: vec4<f32>, @location(3) @interpolate(flat, either) C0_: f32, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @location(5) M0_: vec4<f32>, @location(6) @interpolate(flat, either) g2_: f32, @location(9) C2_: vec3<f32>) -> FragmentOutput {
    h1_1 = h1_;
    g3_1 = g3_;
    p4_1 = p4_;
    V1_1 = V1_;
    C0_1 = C0_;
    W1_1 = W1_;
    M0_1 = M0_;
    g2_1 = g2_;
    C2_1 = C2_;
    main_1();
    let _e20 = h0_;
    let _e21 = C1_;
    return FragmentOutput(_e20, _e21);
}
