struct wf {
    v2_: array<u32>,
}

struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct wf_1 {
    v2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(10) override ej: bool = false;

var<private> o1_1: f32;
var<private> y3_1: vec2<u32>;
var<private> J4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> Z0_: wf_1;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> m0_: vec4<f32>;
var<private> N1_: vec4<f32>;
@group(3) @binding(9)
var Ta: sampler;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(3) @binding(8)
var H8_: sampler;
@group(1) @binding(13)
var S4_: sampler;
var<private> O0_1: vec4<f32>;
var<private> G0_1: f32;
var<private> j2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
var<private> P0_1: f32;
var<private> V0_1: vec3<f32>;

fn main_1() {
    var phi_181_: bool;
    var phi_182_: bool;
    var phi_465_: f32;
    var phi_464_: f32;
    var phi_463_: f32;
    var phi_466_: f32;
    var phi_470_: f32;

    let _e38 = o1_1;
    if ej {
        let _e40 = y3_1[1u];
        let _e42 = y3_1[0u];
        let _e43 = J4_1;
        let _e45 = vec2<u32>(floor(_e43));
        let _e75 = atomicLoad((&Z0_.v2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]));
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
            phi_470_ = 0f;
        } else {
            let _e88 = j.r2_;
            phi_463_ = _e38;
            if (_e75 < _e88) {
                let _e95 = (_e88 | (262144u + u32(((abs(_e38) * 1024f) + 0.5f))));
                let _e96 = atomicMax((&Z0_.v2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), _e95);
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
                let _e115 = atomicAdd((&Z0_.v2_[(_e42 + (((((_e45.y >> bitcast<u32>(5u)) * (_e40 << bitcast<u32>(5u))) + ((_e45.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e45.x & 28u) << bitcast<u32>(5u)) + ((_e45.y & 28u) << bitcast<u32>(2i)))) + (((_e45.y & 3u) << bitcast<u32>(2i)) + (_e45.x & 3u))))]), u32(((abs(_e109) * 1024f) + 0.5f)));
                phi_466_ = ((f32(bitcast<i32>(((_e115 & 524287u) - 262144u))) * 0.0009765625f) + _e38);
            }
            let _e123 = phi_466_;
            phi_470_ = (1f - _e123);
        }
        let _e126 = phi_470_;
        m0_ = vec4(_e126);
        N1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        m0_ = vec4(_e38);
        N1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) o1_: f32, @location(7) @interpolate(flat, either) y3_: vec2<u32>, @location(8) J4_: vec2<f32>, @location(0) O0_: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32, @location(4) @interpolate(flat, either) j2_: vec2<f32>, @location(5) W0_: vec4<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(9) V0_: vec3<f32>) -> FragmentOutput {
    o1_1 = o1_;
    y3_1 = y3_;
    J4_1 = J4_;
    O0_1 = O0_;
    G0_1 = G0_;
    j2_1 = j2_;
    W0_1 = W0_;
    P0_1 = P0_;
    V0_1 = V0_;
    main_1();
    let _e20 = m0_;
    let _e21 = N1_;
    return FragmentOutput(_e20, _e21);
}
