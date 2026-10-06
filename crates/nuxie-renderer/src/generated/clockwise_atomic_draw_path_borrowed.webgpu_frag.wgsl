struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct Ve {
    k2_: array<u32>,
}

struct Ve_1 {
    k2_: array<atomic<u32>>,
}

@id(3) override pi: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var xa: sampler;
var<private> S_1: vec4<f32>;
var<private> G4_1: vec2<f32>;
var<private> r3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(6)
var<storage, read_write> V0_: Ve_1;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(1) @binding(13)
var r5_: sampler;
var<private> a1_1: vec4<f32>;
var<private> F0_1: f32;
var<private> l1_1: vec2<f32>;
var<private> R0_1: vec4<f32>;
var<private> Q0_1: f32;
var<private> v1_1: vec3<f32>;

fn main_1() {
    var phi_597_: bool;
    var phi_850_: f32;
    var phi_856_: f32;
    var phi_857_: f32;
    var phi_534_: bool;
    var phi_858_: f32;
    var phi_859_: f32;

    let _e48 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            if (_e48.y >= 0f) {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_534_ = pi;
                        if pi {
                            phi_534_ = (_e48.x < -1.5f);
                        }
                        let _e119 = phi_534_;
                        if _e119 {
                            let _e125 = textureSampleLevel(YC, xa, vec2<f32>((3f + _e48.x), 0f), 0f);
                            let _e130 = textureSampleLevel(YC, xa, vec2<f32>((1f - _e48.y), 0f), 0f);
                            phi_858_ = ((1f - _e125.x) - _e130.x);
                            break;
                        } else {
                            phi_858_ = min(_e48.x, _e48.y);
                            break;
                        }
                    }
                }
                let _e134 = phi_858_;
                phi_859_ = _e134;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_597_ = pi;
                        if pi {
                            phi_597_ = (_e48.y < -1.5f);
                        }
                        let _e55 = phi_597_;
                        if _e55 {
                            let _e59 = max(_e48.w, 0f);
                            if (_e48.z >= 0f) {
                                let _e62 = textureSampleLevel(YC, xa, vec2<f32>(_e59, 0f), 0f);
                                phi_850_ = _e62.x;
                            } else {
                                phi_850_ = 0f;
                            }
                            let _e65 = phi_850_;
                            phi_856_ = _e65;
                            if (abs(_e48.z) < 1000f) {
                                let _e71 = (-2f - _e48.y);
                                let _e73 = ((_e71 - _e59) * 0.5984134f);
                                let _e76 = (vec4(_e59) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e73));
                                let _e82 = ((_e76 * -(_e48.z)) + vec4(((_e71 * _e48.z) + (abs(_e48.x) - 0.25f))));
                                let _e85 = textureSampleLevel(YC, xa, vec2<f32>(_e82.x, 0f), 0f);
                                let _e88 = textureSampleLevel(YC, xa, vec2<f32>(_e82.y, 0f), 0f);
                                let _e91 = textureSampleLevel(YC, xa, vec2<f32>(_e82.z, 0f), 0f);
                                let _e94 = textureSampleLevel(YC, xa, vec2<f32>(_e82.w, 0f), 0f);
                                let _e100 = (_e76 * 5.0959306f);
                                phi_856_ = (_e65 + (dot(vec4<f32>(_e85.x, _e88.x, _e91.x, _e94.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e100) * (_e100 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e73));
                            }
                            let _e109 = phi_856_;
                            phi_857_ = (_e109 * sign(_e48.x));
                            break;
                        } else {
                            phi_857_ = _e48.x;
                            break;
                        }
                    }
                }
                let _e114 = phi_857_;
                phi_859_ = _e114;
                break;
            }
        }
    }
    let _e136 = phi_859_;
    let _e137 = G4_1;
    let _e139 = vec2<u32>(floor(_e137));
    let _e141 = r3_1[1u];
    let _e143 = r3_1[0u];
    let _e174 = u32(((abs(_e136) * 1024f) + 0.5f));
    let _e176 = j.j2_;
    let _e178 = (_e176 | (262144u - _e174));
    let _e181 = atomicMax((&V0_.k2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), _e178);
    if (_e181 >= _e176) {
        let _e186 = atomicAdd((&V0_.k2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), ((_e181 - max(_e181, _e178)) - _e174));
    }
    return;
}

@fragment
fn main(@location(2) S: vec4<f32>, @location(8) G4_: vec2<f32>, @location(7) @interpolate(flat, either) r3_: vec2<u32>, @location(0) a1_: vec4<f32>, @location(3) @interpolate(flat, either) F0_: f32, @location(4) @interpolate(flat, either) l1_: vec2<f32>, @location(5) R0_: vec4<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(9) v1_: vec3<f32>) {
    S_1 = S;
    G4_1 = G4_;
    r3_1 = r3_;
    a1_1 = a1_;
    F0_1 = F0_;
    l1_1 = l1_;
    R0_1 = R0_;
    Q0_1 = Q0_;
    v1_1 = v1_;
    main_1();
}
