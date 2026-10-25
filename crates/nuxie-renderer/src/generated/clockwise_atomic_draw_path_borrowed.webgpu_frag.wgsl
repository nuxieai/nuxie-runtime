struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct Kf {
    v2_: array<u32>,
}

struct Kf_1 {
    v2_: array<atomic<u32>>,
}

@id(3) override mj: bool = true;

@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var eb: sampler;
var<private> T_1: vec4<f32>;
var<private> P4_1: vec2<f32>;
var<private> A3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(6)
var<storage, read_write> Y0_: Kf_1;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
@group(1) @binding(13)
var v4_: sampler;
var<private> N0_1: vec4<f32>;
var<private> I0_1: f32;
var<private> k2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
var<private> S0_1: f32;
var<private> O0_1: vec3<f32>;

fn main_1() {
    var phi_600_: bool;
    var phi_853_: f32;
    var phi_859_: f32;
    var phi_860_: f32;
    var phi_537_: bool;
    var phi_861_: f32;
    var phi_862_: f32;

    let _e48 = T_1;
    switch bitcast<i32>(0u) {
        default: {
            if (_e48.y >= 0f) {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_537_ = mj;
                        if mj {
                            phi_537_ = (_e48.x < -1.5f);
                        }
                        let _e119 = phi_537_;
                        if _e119 {
                            let _e125 = textureSampleLevel(AD, eb, vec2<f32>((3f + _e48.x), 0f), 0f);
                            let _e130 = textureSampleLevel(AD, eb, vec2<f32>((1f - _e48.y), 0f), 0f);
                            phi_861_ = ((1f - _e125.x) - _e130.x);
                            break;
                        } else {
                            phi_861_ = min(_e48.x, _e48.y);
                            break;
                        }
                    }
                }
                let _e134 = phi_861_;
                phi_862_ = _e134;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_600_ = mj;
                        if mj {
                            phi_600_ = (_e48.y < -1.5f);
                        }
                        let _e55 = phi_600_;
                        if _e55 {
                            let _e59 = max(_e48.w, 0f);
                            if (_e48.z >= 0f) {
                                let _e62 = textureSampleLevel(AD, eb, vec2<f32>(_e59, 0f), 0f);
                                phi_853_ = _e62.x;
                            } else {
                                phi_853_ = 0f;
                            }
                            let _e65 = phi_853_;
                            phi_859_ = _e65;
                            if (abs(_e48.z) < 1000f) {
                                let _e71 = (-2f - _e48.y);
                                let _e73 = ((_e71 - _e59) * 0.5984134f);
                                let _e76 = (vec4(_e59) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e73));
                                let _e82 = ((_e76 * -(_e48.z)) + vec4(((_e71 * _e48.z) + (abs(_e48.x) - 0.25f))));
                                let _e85 = textureSampleLevel(AD, eb, vec2<f32>(_e82.x, 0f), 0f);
                                let _e88 = textureSampleLevel(AD, eb, vec2<f32>(_e82.y, 0f), 0f);
                                let _e91 = textureSampleLevel(AD, eb, vec2<f32>(_e82.z, 0f), 0f);
                                let _e94 = textureSampleLevel(AD, eb, vec2<f32>(_e82.w, 0f), 0f);
                                let _e100 = (_e76 * 5.0959306f);
                                phi_859_ = (_e65 + (dot(vec4<f32>(_e85.x, _e88.x, _e91.x, _e94.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e100) * (_e100 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e73));
                            }
                            let _e109 = phi_859_;
                            phi_860_ = (_e109 * sign(_e48.x));
                            break;
                        } else {
                            phi_860_ = _e48.x;
                            break;
                        }
                    }
                }
                let _e114 = phi_860_;
                phi_862_ = _e114;
                break;
            }
        }
    }
    let _e136 = phi_862_;
    let _e137 = P4_1;
    let _e139 = vec2<u32>(floor(_e137));
    let _e141 = A3_1[1u];
    let _e143 = A3_1[0u];
    let _e174 = u32(((abs(_e136) * 1024f) + 0.5f));
    let _e176 = j.r2_;
    let _e178 = (_e176 | (262144u - _e174));
    let _e181 = atomicMax((&Y0_.v2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), _e178);
    if (_e181 >= _e176) {
        let _e186 = atomicAdd((&Y0_.v2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), ((_e181 - max(_e181, _e178)) - _e174));
    }
    return;
}

@fragment
fn main(@location(2) T: vec4<f32>, @location(8) P4_: vec2<f32>, @location(7) @interpolate(flat, either) A3_: vec2<u32>, @location(0) N0_: vec4<f32>, @location(3) @interpolate(flat, either) I0_: f32, @location(4) @interpolate(flat, either) k2_: vec2<f32>, @location(5) W0_: vec4<f32>, @location(6) @interpolate(flat, either) S0_: f32, @location(9) O0_: vec3<f32>) {
    T_1 = T;
    P4_1 = P4_;
    A3_1 = A3_;
    N0_1 = N0_;
    I0_1 = I0_;
    k2_1 = k2_;
    W0_1 = W0_;
    S0_1 = S0_;
    O0_1 = O0_;
    main_1();
}
