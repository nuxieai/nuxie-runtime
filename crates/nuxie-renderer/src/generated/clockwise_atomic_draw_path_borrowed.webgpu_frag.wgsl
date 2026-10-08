struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

struct zf {
    v2_: array<u32>,
}

struct zf_1 {
    v2_: array<atomic<u32>>,
}

@id(3) override bj: bool = true;

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Va: sampler;
var<private> S_1: vec4<f32>;
var<private> J4_1: vec2<f32>;
var<private> y3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(6)
var<storage, read_write> Z0_: zf_1;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(3) @binding(8)
var I8_: sampler;
@group(1) @binding(13)
var S4_: sampler;
var<private> P0_1: vec4<f32>;
var<private> G0_1: f32;
var<private> j2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
var<private> Q0_1: f32;
var<private> V0_1: vec3<f32>;

fn main_1() {
    var phi_599_: bool;
    var phi_852_: f32;
    var phi_858_: f32;
    var phi_859_: f32;
    var phi_536_: bool;
    var phi_860_: f32;
    var phi_861_: f32;

    let _e48 = S_1;
    switch bitcast<i32>(0u) {
        default: {
            if (_e48.y >= 0f) {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_536_ = bj;
                        if bj {
                            phi_536_ = (_e48.x < -1.5f);
                        }
                        let _e119 = phi_536_;
                        if _e119 {
                            let _e125 = textureSampleLevel(ZC, Va, vec2<f32>((3f + _e48.x), 0f), 0f);
                            let _e130 = textureSampleLevel(ZC, Va, vec2<f32>((1f - _e48.y), 0f), 0f);
                            phi_860_ = ((1f - _e125.x) - _e130.x);
                            break;
                        } else {
                            phi_860_ = min(_e48.x, _e48.y);
                            break;
                        }
                    }
                }
                let _e134 = phi_860_;
                phi_861_ = _e134;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_599_ = bj;
                        if bj {
                            phi_599_ = (_e48.y < -1.5f);
                        }
                        let _e55 = phi_599_;
                        if _e55 {
                            let _e59 = max(_e48.w, 0f);
                            if (_e48.z >= 0f) {
                                let _e62 = textureSampleLevel(ZC, Va, vec2<f32>(_e59, 0f), 0f);
                                phi_852_ = _e62.x;
                            } else {
                                phi_852_ = 0f;
                            }
                            let _e65 = phi_852_;
                            phi_858_ = _e65;
                            if (abs(_e48.z) < 1000f) {
                                let _e71 = (-2f - _e48.y);
                                let _e73 = ((_e71 - _e59) * 0.5984134f);
                                let _e76 = (vec4(_e59) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e73));
                                let _e82 = ((_e76 * -(_e48.z)) + vec4(((_e71 * _e48.z) + (abs(_e48.x) - 0.25f))));
                                let _e85 = textureSampleLevel(ZC, Va, vec2<f32>(_e82.x, 0f), 0f);
                                let _e88 = textureSampleLevel(ZC, Va, vec2<f32>(_e82.y, 0f), 0f);
                                let _e91 = textureSampleLevel(ZC, Va, vec2<f32>(_e82.z, 0f), 0f);
                                let _e94 = textureSampleLevel(ZC, Va, vec2<f32>(_e82.w, 0f), 0f);
                                let _e100 = (_e76 * 5.0959306f);
                                phi_858_ = (_e65 + (dot(vec4<f32>(_e85.x, _e88.x, _e91.x, _e94.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e100) * (_e100 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e73));
                            }
                            let _e109 = phi_858_;
                            phi_859_ = (_e109 * sign(_e48.x));
                            break;
                        } else {
                            phi_859_ = _e48.x;
                            break;
                        }
                    }
                }
                let _e114 = phi_859_;
                phi_861_ = _e114;
                break;
            }
        }
    }
    let _e136 = phi_861_;
    let _e137 = J4_1;
    let _e139 = vec2<u32>(floor(_e137));
    let _e141 = y3_1[1u];
    let _e143 = y3_1[0u];
    let _e174 = u32(((abs(_e136) * 1024f) + 0.5f));
    let _e176 = j.r2_;
    let _e178 = (_e176 | (262144u - _e174));
    let _e181 = atomicMax((&Z0_.v2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), _e178);
    if (_e181 >= _e176) {
        let _e186 = atomicAdd((&Z0_.v2_[(_e143 + (((((_e139.y >> bitcast<u32>(5u)) * (_e141 << bitcast<u32>(5u))) + ((_e139.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e139.x & 28u) << bitcast<u32>(5u)) + ((_e139.y & 28u) << bitcast<u32>(2i)))) + (((_e139.y & 3u) << bitcast<u32>(2i)) + (_e139.x & 3u))))]), ((_e181 - max(_e181, _e178)) - _e174));
    }
    return;
}

@fragment
fn main(@location(2) S: vec4<f32>, @location(8) J4_: vec2<f32>, @location(7) @interpolate(flat, either) y3_: vec2<u32>, @location(0) P0_: vec4<f32>, @location(3) @interpolate(flat, either) G0_: f32, @location(4) @interpolate(flat, either) j2_: vec2<f32>, @location(5) W0_: vec4<f32>, @location(6) @interpolate(flat, either) Q0_: f32, @location(9) V0_: vec3<f32>) {
    S_1 = S;
    J4_1 = J4_;
    y3_1 = y3_;
    P0_1 = P0_;
    G0_1 = G0_;
    j2_1 = j2_;
    W0_1 = W0_;
    Q0_1 = Q0_;
    V0_1 = V0_;
    main_1();
}
