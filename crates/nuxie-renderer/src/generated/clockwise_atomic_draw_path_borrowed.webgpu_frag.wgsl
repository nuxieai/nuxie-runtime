struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct Sf {
    z2_: array<u32>,
}

struct Sf_1 {
    z2_: array<atomic<u32>>,
}

@id(3) override xj: bool = true;

@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var ib: sampler;
var<private> S_1: vec4<f32>;
var<private> V4_1: vec2<f32>;
var<private> D3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: RB;
@group(0) @binding(5)
var<storage, read_write> c1_: Sf_1;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(3) @binding(7)
var V8_: sampler;
@group(1) @binding(12)
var x4_: sampler;
var<private> N0_1: vec4<f32>;
var<private> J0_1: f32;
var<private> l2_1: vec2<f32>;
var<private> Y0_1: vec4<f32>;
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

    let _e48 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            if (_e48.y >= 0f) {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_537_ = xj;
                        if xj {
                            phi_537_ = (_e48.x < -1.5f);
                        }
                        let _e119 = phi_537_;
                        if _e119 {
                            let _e125 = textureSampleLevel(ZC, ib, vec2<f32>((3f + _e48.x), 0f), 0f);
                            let _e130 = textureSampleLevel(ZC, ib, vec2<f32>((1f - _e48.y), 0f), 0f);
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
                        phi_600_ = xj;
                        if xj {
                            phi_600_ = (_e48.y < -1.5f);
                        }
                        let _e55 = phi_600_;
                        if _e55 {
                            let _e59 = max(_e48.w, 0f);
                            if (_e48.z >= 0f) {
                                let _e62 = textureSampleLevel(ZC, ib, vec2<f32>(_e59, 0f), 0f);
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
                                let _e85 = textureSampleLevel(ZC, ib, vec2<f32>(_e82.x, 0f), 0f);
                                let _e88 = textureSampleLevel(ZC, ib, vec2<f32>(_e82.y, 0f), 0f);
                                let _e91 = textureSampleLevel(ZC, ib, vec2<f32>(_e82.z, 0f), 0f);
                                let _e94 = textureSampleLevel(ZC, ib, vec2<f32>(_e82.w, 0f), 0f);
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
    let _e137 = V4_1;
    let _e139 = vec2<u32>(floor(_e137));
    let _e141 = D3_1[1u];
    let _e143 = D3_1[0u];
    let _e174 = u32(((abs(_e136) * 1024f) + 0.5f));
    let _e176 = j.y2_;
    let _e178 = (_e176 | (262144u - _e174));
    let _e181 = atomicMax((&c1_.z2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), _e178);
    if (_e181 >= _e176) {
        let _e186 = atomicAdd((&c1_.z2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), ((_e181 - max(_e181, _e178)) - _e174));
    }
    return;
}

@fragment
fn main(@location(2) S: vec4<f32>, @location(8) V4_: vec2<f32>, @location(7) @interpolate(flat, either) D3_: vec2<u32>, @location(0) N0_: vec4<f32>, @location(3) @interpolate(flat, either) J0_: f32, @location(4) @interpolate(flat, either) l2_: vec2<f32>, @location(5) Y0_: vec4<f32>, @location(6) @interpolate(flat, either) S0_: f32, @location(9) O0_: vec3<f32>) {
    S_1 = S;
    V4_1 = V4_;
    D3_1 = D3_;
    N0_1 = N0_;
    J0_1 = J0_;
    l2_1 = l2_;
    Y0_1 = Y0_;
    S0_1 = S0_;
    O0_1 = O0_;
    main_1();
}
