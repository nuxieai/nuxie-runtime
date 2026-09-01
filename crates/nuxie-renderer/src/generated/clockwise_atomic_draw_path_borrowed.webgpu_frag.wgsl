struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

struct je {
    c2_: array<u32>,
}

struct je_1 {
    c2_: array<atomic<u32>>,
}

@id(3) override hh: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;
var<private> O_1: vec4<f32>;
var<private> n4_1: vec2<f32>;
var<private> f3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> m: DC;
@group(0) @binding(6)
var<storage, read_write> P0_: je_1;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(3) @binding(8)
var Ob: sampler;
@group(1) @binding(13)
var U5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> B0_1: f32;
var<private> U1_1: vec2<f32>;
var<private> M0_1: vec4<f32>;
var<private> e2_1: f32;
var<private> A2_1: vec3<f32>;

fn main_1() {
    var phi_598_: bool;
    var phi_851_: f32;
    var phi_857_: f32;
    var phi_858_: f32;
    var phi_535_: bool;
    var phi_859_: f32;
    var phi_860_: f32;

    let _e48 = O_1;
    switch bitcast<i32>(0u) {
        default: {
            if (_e48.y >= 0f) {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_535_ = hh;
                        if hh {
                            phi_535_ = (_e48.x < -1.5f);
                        }
                        let _e119 = phi_535_;
                        if _e119 {
                            let _e125 = textureSampleLevel(YC, aa, vec2<f32>((3f + _e48.x), 0f), 0f);
                            let _e130 = textureSampleLevel(YC, aa, vec2<f32>((1f - _e48.y), 0f), 0f);
                            phi_859_ = ((1f - _e125.x) - _e130.x);
                            break;
                        } else {
                            phi_859_ = min(_e48.x, _e48.y);
                            break;
                        }
                    }
                }
                let _e134 = phi_859_;
                phi_860_ = _e134;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_598_ = hh;
                        if hh {
                            phi_598_ = (_e48.y < -1.5f);
                        }
                        let _e55 = phi_598_;
                        if _e55 {
                            let _e59 = max(_e48.w, 0f);
                            if (_e48.z >= 0f) {
                                let _e62 = textureSampleLevel(YC, aa, vec2<f32>(_e59, 0f), 0f);
                                phi_851_ = _e62.x;
                            } else {
                                phi_851_ = 0f;
                            }
                            let _e65 = phi_851_;
                            phi_857_ = _e65;
                            if (abs(_e48.z) < 1000f) {
                                let _e71 = (-2f - _e48.y);
                                let _e73 = ((_e71 - _e59) * 0.5984134f);
                                let _e76 = (vec4(_e59) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e73));
                                let _e82 = ((_e76 * -(_e48.z)) + vec4(((_e71 * _e48.z) + (abs(_e48.x) - 0.25f))));
                                let _e85 = textureSampleLevel(YC, aa, vec2<f32>(_e82.x, 0f), 0f);
                                let _e88 = textureSampleLevel(YC, aa, vec2<f32>(_e82.y, 0f), 0f);
                                let _e91 = textureSampleLevel(YC, aa, vec2<f32>(_e82.z, 0f), 0f);
                                let _e94 = textureSampleLevel(YC, aa, vec2<f32>(_e82.w, 0f), 0f);
                                let _e100 = (_e76 * 5.0959306f);
                                phi_857_ = (_e65 + (dot(vec4<f32>(_e85.x, _e88.x, _e91.x, _e94.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e100) * (_e100 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e73));
                            }
                            let _e109 = phi_857_;
                            phi_858_ = (_e109 * sign(_e48.x));
                            break;
                        } else {
                            phi_858_ = _e48.x;
                            break;
                        }
                    }
                }
                let _e114 = phi_858_;
                phi_860_ = _e114;
                break;
            }
        }
    }
    let _e136 = phi_860_;
    let _e137 = n4_1;
    let _e139 = vec2<u32>(floor(_e137));
    let _e141 = f3_1[1u];
    let _e143 = f3_1[0u];
    let _e174 = u32(((abs(_e136) * 1024f) + 0.5f));
    let _e176 = m.a2_;
    let _e178 = (_e176 | (262144u - _e174));
    let _e181 = atomicMax((&P0_.c2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), _e178);
    if (_e181 >= _e176) {
        let _e186 = atomicAdd((&P0_.c2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), ((_e181 - max(_e181, _e178)) - _e174));
    }
    return;
}

@fragment
fn main(@location(2) O: vec4<f32>, @location(8) n4_: vec2<f32>, @location(7) @interpolate(flat, either) f3_: vec2<u32>, @location(0) f1_: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @location(5) M0_: vec4<f32>, @location(6) @interpolate(flat, either) e2_: f32, @location(9) A2_: vec3<f32>) {
    O_1 = O;
    n4_1 = n4_;
    f3_1 = f3_;
    f1_1 = f1_;
    B0_1 = B0_;
    U1_1 = U1_;
    M0_1 = M0_;
    e2_1 = e2_;
    A2_1 = A2_;
    main_1();
}
