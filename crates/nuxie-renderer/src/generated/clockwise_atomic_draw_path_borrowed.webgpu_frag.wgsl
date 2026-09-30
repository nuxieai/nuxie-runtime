struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct ze {
    e2_: array<u32>,
}

struct ze_1 {
    e2_: array<atomic<u32>>,
}

@id(3) override Fh: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ea: sampler;
var<private> M_1: vec4<f32>;
var<private> p4_1: vec2<f32>;
var<private> g3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> l: BC;
@group(0) @binding(6)
var<storage, read_write> Q0_: ze_1;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(13)
var X5_: sampler;
var<private> V1_1: vec4<f32>;
var<private> C0_1: f32;
var<private> W1_1: vec2<f32>;
var<private> M0_1: vec4<f32>;
var<private> g2_1: f32;
var<private> C2_1: vec3<f32>;

fn main_1() {
    var phi_598_: bool;
    var phi_851_: f32;
    var phi_857_: f32;
    var phi_858_: f32;
    var phi_535_: bool;
    var phi_859_: f32;
    var phi_860_: f32;

    let _e48 = M_1;
    switch bitcast<i32>(0u) {
        default: {
            if (_e48.y >= 0f) {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_535_ = Fh;
                        if Fh {
                            phi_535_ = (_e48.x < -1.5f);
                        }
                        let _e119 = phi_535_;
                        if _e119 {
                            let _e125 = textureSampleLevel(YC, ea, vec2<f32>((3f + _e48.x), 0f), 0f);
                            let _e130 = textureSampleLevel(YC, ea, vec2<f32>((1f - _e48.y), 0f), 0f);
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
                        phi_598_ = Fh;
                        if Fh {
                            phi_598_ = (_e48.y < -1.5f);
                        }
                        let _e55 = phi_598_;
                        if _e55 {
                            let _e59 = max(_e48.w, 0f);
                            if (_e48.z >= 0f) {
                                let _e62 = textureSampleLevel(YC, ea, vec2<f32>(_e59, 0f), 0f);
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
                                let _e85 = textureSampleLevel(YC, ea, vec2<f32>(_e82.x, 0f), 0f);
                                let _e88 = textureSampleLevel(YC, ea, vec2<f32>(_e82.y, 0f), 0f);
                                let _e91 = textureSampleLevel(YC, ea, vec2<f32>(_e82.z, 0f), 0f);
                                let _e94 = textureSampleLevel(YC, ea, vec2<f32>(_e82.w, 0f), 0f);
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
    let _e137 = p4_1;
    let _e139 = vec2<u32>(floor(_e137));
    let _e141 = g3_1[1u];
    let _e143 = g3_1[0u];
    let _e174 = u32(((abs(_e136) * 1024f) + 0.5f));
    let _e176 = l.d2_;
    let _e178 = (_e176 | (262144u - _e174));
    let _e181 = atomicMax((&Q0_.e2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), _e178);
    if (_e181 >= _e176) {
        let _e186 = atomicAdd((&Q0_.e2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), ((_e181 - max(_e181, _e178)) - _e174));
    }
    return;
}

@fragment
fn main(@location(2) M: vec4<f32>, @location(8) p4_: vec2<f32>, @location(7) @interpolate(flat, either) g3_: vec2<u32>, @location(0) V1_: vec4<f32>, @location(3) @interpolate(flat, either) C0_: f32, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @location(5) M0_: vec4<f32>, @location(6) @interpolate(flat, either) g2_: f32, @location(9) C2_: vec3<f32>) {
    M_1 = M;
    p4_1 = p4_;
    g3_1 = g3_;
    V1_1 = V1_;
    C0_1 = C0_;
    W1_1 = W1_;
    M0_1 = M0_;
    g2_1 = g2_;
    C2_1 = C2_;
    main_1();
}
