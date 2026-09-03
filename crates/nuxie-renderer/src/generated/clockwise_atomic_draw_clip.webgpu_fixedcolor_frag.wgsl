struct ke {
    d2_: array<u32>,
}

struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

struct ke_1 {
    d2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(10) override ph: bool = false;

var<private> O_1: vec4<f32>;
var<private> f3_1: vec2<u32>;
var<private> n4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> P0_: ke_1;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> h0_: vec4<f32>;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(3) @binding(8)
var Pb: sampler;
@group(1) @binding(13)
var V5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> B0_1: f32;
var<private> V1_1: vec2<f32>;
var<private> M0_1: vec4<f32>;
var<private> f2_1: f32;
var<private> A2_1: vec3<f32>;

fn main_1() {
    var phi_183_: bool;
    var phi_184_: bool;
    var phi_466_: f32;
    var phi_465_: f32;
    var phi_464_: f32;
    var phi_467_: f32;
    var phi_471_: f32;

    let _e39 = O_1[0u];
    if ph {
        let _e41 = f3_1[1u];
        let _e43 = f3_1[0u];
        let _e44 = n4_1;
        let _e46 = vec2<u32>(floor(_e44));
        let _e76 = atomicLoad((&P0_.d2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]));
        let _e77 = (_e39 >= 1f);
        phi_184_ = _e77;
        if _e77 {
            let _e79 = m.c2_;
            let _e80 = (_e76 < _e79);
            phi_183_ = _e80;
            if !(_e80) {
                phi_183_ = (_e76 >= (_e79 | 262144u));
            }
            let _e85 = phi_183_;
            phi_184_ = _e85;
        }
        let _e87 = phi_184_;
        if _e87 {
            phi_471_ = 0f;
        } else {
            let _e89 = m.c2_;
            phi_464_ = _e39;
            if (_e76 < _e89) {
                let _e96 = (_e89 | (262144u + u32(((abs(_e39) * 1024f) + 0.5f))));
                let _e97 = atomicMax((&P0_.d2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]), _e96);
                if (_e97 <= _e89) {
                    phi_465_ = 0f;
                } else {
                    phi_466_ = _e39;
                    if (_e97 < _e96) {
                        phi_466_ = (f32(bitcast<i32>(((_e97 & 524287u) - 262144u))) * 0.0009765625f);
                    }
                    let _e106 = phi_466_;
                    phi_465_ = _e106;
                }
                let _e108 = phi_465_;
                phi_464_ = _e108;
            }
            let _e110 = phi_464_;
            phi_467_ = _e39;
            if (_e110 > 0f) {
                let _e116 = atomicAdd((&P0_.d2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]), u32(((abs(_e110) * 1024f) + 0.5f)));
                phi_467_ = ((f32(bitcast<i32>(((_e116 & 524287u) - 262144u))) * 0.0009765625f) + _e39);
            }
            let _e124 = phi_467_;
            phi_471_ = (1f - _e124);
        }
        let _e127 = phi_471_;
        h0_ = vec4(_e127);
        C1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        h0_ = vec4(_e39);
        C1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(2) O: vec4<f32>, @location(7) @interpolate(flat, either) f3_: vec2<u32>, @location(8) n4_: vec2<f32>, @location(0) f1_: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(4) @interpolate(flat, either) V1_: vec2<f32>, @location(5) M0_: vec4<f32>, @location(6) @interpolate(flat, either) f2_: f32, @location(9) A2_: vec3<f32>) -> FragmentOutput {
    O_1 = O;
    f3_1 = f3_;
    n4_1 = n4_;
    f1_1 = f1_;
    B0_1 = B0_;
    V1_1 = V1_;
    M0_1 = M0_;
    f2_1 = f2_;
    A2_1 = A2_;
    main_1();
    let _e20 = h0_;
    let _e21 = C1_;
    return FragmentOutput(_e20, _e21);
}
