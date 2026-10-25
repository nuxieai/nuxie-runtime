struct Ef {
    v2_: array<u32>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct Ef_1 {
    v2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(11) override qj: bool = false;

var<private> T_1: vec4<f32>;
var<private> A3_1: vec2<u32>;
var<private> N4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> Y0_: Ef_1;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> m0_: vec4<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(3) @binding(8)
var R8_: sampler;
@group(1) @binding(13)
var W4_: sampler;
var<private> P0_1: vec4<f32>;
var<private> G0_1: f32;
var<private> k2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
var<private> Q0_1: f32;
var<private> V0_1: vec3<f32>;

fn main_1() {
    var phi_183_: bool;
    var phi_184_: bool;
    var phi_468_: f32;
    var phi_467_: f32;
    var phi_466_: f32;
    var phi_469_: f32;
    var phi_473_: f32;

    let _e39 = T_1[0u];
    if qj {
        let _e41 = A3_1[1u];
        let _e43 = A3_1[0u];
        let _e44 = N4_1;
        let _e46 = vec2<u32>(floor(_e44));
        let _e76 = atomicLoad((&Y0_.v2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]));
        let _e77 = (_e39 >= 1f);
        phi_184_ = _e77;
        if _e77 {
            let _e79 = j.r2_;
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
            phi_473_ = 0f;
        } else {
            let _e89 = j.r2_;
            phi_466_ = _e39;
            if (_e76 < _e89) {
                let _e96 = (_e89 | (262144u + u32(((abs(_e39) * 1024f) + 0.5f))));
                let _e97 = atomicMax((&Y0_.v2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]), _e96);
                if (_e97 <= _e89) {
                    phi_467_ = 0f;
                } else {
                    phi_468_ = _e39;
                    if (_e97 < _e96) {
                        phi_468_ = (f32(bitcast<i32>(((_e97 & 524287u) - 262144u))) * 0.0009765625f);
                    }
                    let _e106 = phi_468_;
                    phi_467_ = _e106;
                }
                let _e108 = phi_467_;
                phi_466_ = _e108;
            }
            let _e110 = phi_466_;
            phi_469_ = _e39;
            if (_e110 > 0f) {
                let _e116 = atomicAdd((&Y0_.v2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]), u32(((abs(_e110) * 1024f) + 0.5f)));
                phi_469_ = ((f32(bitcast<i32>(((_e116 & 524287u) - 262144u))) * 0.0009765625f) + _e39);
            }
            let _e124 = phi_469_;
            phi_473_ = (1f - _e124);
        }
        let _e127 = phi_473_;
        m0_ = vec4(_e127);
        Q1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        m0_ = vec4(_e39);
        Q1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(2) T: vec4<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(8) N4_: vec2<f32>, @location(0) P0_: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32, @location(4) @interpolate(flat, either) k2_: vec2<f32>, @location(5) W0_: vec4<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(9) V0_: vec3<f32>) -> FragmentOutput {
    T_1 = T;
    A3_1 = A3_;
    N4_1 = N4_;
    P0_1 = P0_;
    G0_1 = G0_;
    k2_1 = k2_;
    W0_1 = W0_;
    Q0_1 = Q0_;
    V0_1 = V0_;
    main_1();
    let _e20 = m0_;
    let _e21 = Q1_;
    return FragmentOutput(_e20, _e21);
}
