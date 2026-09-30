struct Ae {
    g2_: array<u32>,
}

struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
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

@id(10) override Rh: bool = false;

var<private> O_1: vec4<f32>;
var<private> k3_1: vec2<u32>;
var<private> v4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> S0_: Ae_1;
@group(0) @binding(0)
var<uniform> j: TB;
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
    var phi_183_: bool;
    var phi_184_: bool;
    var phi_466_: f32;
    var phi_465_: f32;
    var phi_464_: f32;
    var phi_467_: f32;
    var phi_471_: f32;

    let _e39 = O_1[0u];
    if Rh {
        let _e41 = k3_1[1u];
        let _e43 = k3_1[0u];
        let _e44 = v4_1;
        let _e46 = vec2<u32>(floor(_e44));
        let _e76 = atomicLoad((&S0_.g2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]));
        let _e77 = (_e39 >= 1f);
        phi_184_ = _e77;
        if _e77 {
            let _e79 = j.f2_;
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
            let _e89 = j.f2_;
            phi_464_ = _e39;
            if (_e76 < _e89) {
                let _e96 = (_e89 | (262144u + u32(((abs(_e39) * 1024f) + 0.5f))));
                let _e97 = atomicMax((&S0_.g2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]), _e96);
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
                let _e116 = atomicAdd((&S0_.g2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]), u32(((abs(_e110) * 1024f) + 0.5f)));
                phi_467_ = ((f32(bitcast<i32>(((_e116 & 524287u) - 262144u))) * 0.0009765625f) + _e39);
            }
            let _e124 = phi_467_;
            phi_471_ = (1f - _e124);
        }
        let _e127 = phi_471_;
        i0_ = vec4(_e127);
        F1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        i0_ = vec4(_e39);
        F1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(2) O: vec4<f32>, @location(7) @interpolate(flat, either) k3_: vec2<u32>, @location(8) v4_: vec2<f32>, @location(0) X1_: vec4<f32>, @location(3) @interpolate(flat, either) D0_: f32, @location(4) @interpolate(flat, either) Y1_: vec2<f32>, @location(5) O0_: vec4<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(9) C2_: vec3<f32>) -> FragmentOutput {
    O_1 = O;
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
