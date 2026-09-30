struct Ue {
    k2_: array<u32>,
}

struct UB {
    Rc: f32,
    Ud: f32,
    cg: f32,
    dg: f32,
    B6_: u32,
    Y9_: u32,
    Of: u32,
    Pf: u32,
    k8_: vec4<i32>,
    Mh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Qh: f32,
    U4_: u32,
    c3_: f32,
    Wd: f32,
    If: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Jh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct Ue_1 {
    k2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(10) override vi: bool = false;

var<private> S_1: vec4<f32>;
var<private> q3_1: vec2<u32>;
var<private> G4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> W0_: Ue_1;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> m0_: vec4<f32>;
var<private> K1_: vec4<f32>;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var DC: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(1) @binding(13)
var v5_: sampler;
var<private> a1_1: vec4<f32>;
var<private> F0_1: f32;
var<private> l1_1: vec2<f32>;
var<private> S0_1: vec4<f32>;
var<private> Q0_1: f32;
var<private> r1_1: vec3<f32>;

fn main_1() {
    var phi_183_: bool;
    var phi_184_: bool;
    var phi_465_: f32;
    var phi_464_: f32;
    var phi_463_: f32;
    var phi_466_: f32;
    var phi_470_: f32;

    let _e39 = S_1[0u];
    if vi {
        let _e41 = q3_1[1u];
        let _e43 = q3_1[0u];
        let _e44 = G4_1;
        let _e46 = vec2<u32>(floor(_e44));
        let _e76 = atomicLoad((&W0_.k2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]));
        let _e77 = (_e39 >= 1f);
        phi_184_ = _e77;
        if _e77 {
            let _e79 = j.j2_;
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
            phi_470_ = 0f;
        } else {
            let _e89 = j.j2_;
            phi_463_ = _e39;
            if (_e76 < _e89) {
                let _e96 = (_e89 | (262144u + u32(((abs(_e39) * 1024f) + 0.5f))));
                let _e97 = atomicMax((&W0_.k2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]), _e96);
                if (_e97 <= _e89) {
                    phi_464_ = 0f;
                } else {
                    phi_465_ = _e39;
                    if (_e97 < _e96) {
                        phi_465_ = (f32(bitcast<i32>(((_e97 & 524287u) - 262144u))) * 0.0009765625f);
                    }
                    let _e106 = phi_465_;
                    phi_464_ = _e106;
                }
                let _e108 = phi_464_;
                phi_463_ = _e108;
            }
            let _e110 = phi_463_;
            phi_466_ = _e39;
            if (_e110 > 0f) {
                let _e116 = atomicAdd((&W0_.k2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]), u32(((abs(_e110) * 1024f) + 0.5f)));
                phi_466_ = ((f32(bitcast<i32>(((_e116 & 524287u) - 262144u))) * 0.0009765625f) + _e39);
            }
            let _e124 = phi_466_;
            phi_470_ = (1f - _e124);
        }
        let _e127 = phi_470_;
        m0_ = vec4(_e127);
        K1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        m0_ = vec4(_e39);
        K1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(2) S: vec4<f32>, @location(7) @interpolate(flat, either) q3_: vec2<u32>, @location(8) G4_: vec2<f32>, @location(0) a1_: vec4<f32>, @location(3) @interpolate(flat, either) F0_: f32, @location(4) @interpolate(flat, either) l1_: vec2<f32>, @location(5) S0_: vec4<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(9) r1_: vec3<f32>) -> FragmentOutput {
    S_1 = S;
    q3_1 = q3_;
    G4_1 = G4_;
    a1_1 = a1_;
    F0_1 = F0_;
    l1_1 = l1_;
    S0_1 = S0_;
    Q0_1 = Q0_;
    r1_1 = r1_;
    main_1();
    let _e20 = m0_;
    let _e21 = K1_;
    return FragmentOutput(_e20, _e21);
}
