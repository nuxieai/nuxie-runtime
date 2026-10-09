struct zf {
    r2_: array<u32>,
}

struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

struct zf_1 {
    r2_: array<atomic<u32>>,
}

struct FragmentOutput {
    @location(1) member: vec4<f32>,
    @location(0) member_1: vec4<f32>,
}

@id(10) override gj: bool = false;

var<private> S_1: vec4<f32>;
var<private> z3_1: vec2<u32>;
var<private> L4_1: vec2<f32>;
@group(0) @binding(6)
var<storage, read_write> Y0_: zf_1;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> m0_: vec4<f32>;
var<private> L1_: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
@group(1) @binding(13)
var U4_: sampler;
var<private> O0_1: vec4<f32>;
var<private> G0_1: f32;
var<private> i2_1: vec2<f32>;
var<private> V0_1: vec4<f32>;
var<private> P0_1: f32;
var<private> U0_1: vec3<f32>;

fn main_1() {
    var phi_183_: bool;
    var phi_184_: bool;
    var phi_467_: f32;
    var phi_466_: f32;
    var phi_465_: f32;
    var phi_468_: f32;
    var phi_472_: f32;

    let _e39 = S_1[0u];
    if gj {
        let _e41 = z3_1[1u];
        let _e43 = z3_1[0u];
        let _e44 = L4_1;
        let _e46 = vec2<u32>(floor(_e44));
        let _e76 = atomicLoad((&Y0_.r2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]));
        let _e77 = (_e39 >= 1f);
        phi_184_ = _e77;
        if _e77 {
            let _e79 = j.q2_;
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
            phi_472_ = 0f;
        } else {
            let _e89 = j.q2_;
            phi_465_ = _e39;
            if (_e76 < _e89) {
                let _e96 = (_e89 | (262144u + u32(((abs(_e39) * 1024f) + 0.5f))));
                let _e97 = atomicMax((&Y0_.r2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]), _e96);
                if (_e97 <= _e89) {
                    phi_466_ = 0f;
                } else {
                    phi_467_ = _e39;
                    if (_e97 < _e96) {
                        phi_467_ = (f32(bitcast<i32>(((_e97 & 524287u) - 262144u))) * 0.0009765625f);
                    }
                    let _e106 = phi_467_;
                    phi_466_ = _e106;
                }
                let _e108 = phi_466_;
                phi_465_ = _e108;
            }
            let _e110 = phi_465_;
            phi_468_ = _e39;
            if (_e110 > 0f) {
                let _e116 = atomicAdd((&Y0_.r2_[(_e43 + (((((_e46.y >> bitcast<u32>(5u)) * (_e41 << bitcast<u32>(5u))) + ((_e46.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e46.x & 28u) << bitcast<u32>(5u)) + ((_e46.y & 28u) << bitcast<u32>(2i)))) + (((_e46.y & 3u) << bitcast<u32>(2i)) + (_e46.x & 3u))))]), u32(((abs(_e110) * 1024f) + 0.5f)));
                phi_468_ = ((f32(bitcast<i32>(((_e116 & 524287u) - 262144u))) * 0.0009765625f) + _e39);
            }
            let _e124 = phi_468_;
            phi_472_ = (1f - _e124);
        }
        let _e127 = phi_472_;
        m0_ = vec4(_e127);
        L1_ = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        m0_ = vec4(_e39);
        L1_ = vec4<f32>(0f, 0f, 0f, 0f);
    }
    return;
}

@fragment
fn main(@location(2) S: vec4<f32>, @location(7) @interpolate(flat, either) z3_: vec2<u32>, @location(8) L4_: vec2<f32>, @location(0) O0_: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32, @location(4) @interpolate(flat, either) i2_: vec2<f32>, @location(5) V0_: vec4<f32>, @location(6) @interpolate(flat, either) P0_: f32, @location(9) U0_: vec3<f32>) -> FragmentOutput {
    S_1 = S;
    z3_1 = z3_;
    L4_1 = L4_;
    O0_1 = O0_;
    G0_1 = G0_;
    i2_1 = i2_;
    V0_1 = V0_;
    P0_1 = P0_;
    U0_1 = U0_;
    main_1();
    let _e20 = m0_;
    let _e21 = L1_;
    return FragmentOutput(_e20, _e21);
}
