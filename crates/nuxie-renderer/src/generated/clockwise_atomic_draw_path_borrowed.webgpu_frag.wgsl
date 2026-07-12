struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct Yd {
    X1_: array<u32>,
}

struct Yd_1 {
    X1_: array<atomic<u32>>,
}

@id(3) override Pg: bool = true;

@group(0) @binding(10)
var QC: texture_2d<f32>;
@group(3) @binding(10)
var T9_: sampler;
var<private> I_1: vec4<f32>;
var<private> j4_1: vec2<f32>;
var<private> e3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> k: NB;
@group(0) @binding(7)
var<storage, read_write> S0_: Yd_1;
@group(0) @binding(9)
var DD: texture_2d<f32>;
@group(1) @binding(12)
var AC: texture_2d<f32>;
@group(3) @binding(9)
var Bb: sampler;
@group(1) @binding(14)
var R5_: sampler;
var<private> i1_1: vec4<f32>;
var<private> z0_1: f32;
var<private> S1_1: vec2<f32>;
var<private> N0_1: vec4<f32>;
var<private> Z1_1: f32;

fn main_1() {
    var phi_593_: bool;
    var phi_846_: f32;
    var phi_852_: f32;
    var phi_853_: f32;
    var phi_530_: bool;
    var phi_854_: f32;
    var phi_855_: f32;

    let _e47 = I_1;
    switch bitcast<i32>(0u) {
        default: {
            if (_e47.y >= 0f) {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_530_ = Pg;
                        if Pg {
                            phi_530_ = (_e47.x < -1.5f);
                        }
                        let _e118 = phi_530_;
                        if _e118 {
                            let _e124 = textureSampleLevel(QC, T9_, vec2<f32>((3f + _e47.x), 0f), 0f);
                            let _e129 = textureSampleLevel(QC, T9_, vec2<f32>((1f - _e47.y), 0f), 0f);
                            phi_854_ = ((1f - _e124.x) - _e129.x);
                            break;
                        } else {
                            phi_854_ = min(_e47.x, _e47.y);
                            break;
                        }
                    }
                }
                let _e133 = phi_854_;
                phi_855_ = _e133;
                break;
            } else {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_593_ = Pg;
                        if Pg {
                            phi_593_ = (_e47.y < -1.5f);
                        }
                        let _e54 = phi_593_;
                        if _e54 {
                            let _e58 = max(_e47.w, 0f);
                            if (_e47.z >= 0f) {
                                let _e61 = textureSampleLevel(QC, T9_, vec2<f32>(_e58, 0f), 0f);
                                phi_846_ = _e61.x;
                            } else {
                                phi_846_ = 0f;
                            }
                            let _e64 = phi_846_;
                            phi_852_ = _e64;
                            if (abs(_e47.z) < 1000f) {
                                let _e70 = (-2f - _e47.y);
                                let _e72 = ((_e70 - _e58) * 0.5984134f);
                                let _e75 = (vec4(_e58) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e72));
                                let _e81 = ((_e75 * -(_e47.z)) + vec4(((_e70 * _e47.z) + (abs(_e47.x) - 0.25f))));
                                let _e84 = textureSampleLevel(QC, T9_, vec2<f32>(_e81.x, 0f), 0f);
                                let _e87 = textureSampleLevel(QC, T9_, vec2<f32>(_e81.y, 0f), 0f);
                                let _e90 = textureSampleLevel(QC, T9_, vec2<f32>(_e81.z, 0f), 0f);
                                let _e93 = textureSampleLevel(QC, T9_, vec2<f32>(_e81.w, 0f), 0f);
                                let _e99 = (_e75 * 5.0959306f);
                                phi_852_ = (_e64 + (dot(vec4<f32>(_e84.x, _e87.x, _e90.x, _e93.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e99) * (_e99 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e72));
                            }
                            let _e108 = phi_852_;
                            phi_853_ = (_e108 * sign(_e47.x));
                            break;
                        } else {
                            phi_853_ = _e47.x;
                            break;
                        }
                    }
                }
                let _e113 = phi_853_;
                phi_855_ = _e113;
                break;
            }
        }
    }
    let _e135 = phi_855_;
    let _e136 = j4_1;
    let _e138 = vec2<u32>(floor(_e136));
    let _e140 = e3_1[1u];
    let _e142 = e3_1[0u];
    let _e173 = u32(((abs(_e135) * 1024f) + 0.5f));
    let _e175 = k.W1_;
    let _e177 = (_e175 | (262144u - _e173));
    let _e180 = atomicMax((&S0_.X1_[(_e142 + (((((_e138.y >> bitcast<u32>(5u)) * (_e140 << bitcast<u32>(5u))) + ((_e138.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e138.x & 28u) << bitcast<u32>(5u)) + ((_e138.y & 28u) << bitcast<u32>(2i)))) + (((_e138.y & 3u) << bitcast<u32>(2i)) + (_e138.x & 3u))))]), _e177);
    if (_e180 >= _e175) {
        let _e185 = atomicAdd((&S0_.X1_[(_e142 + (((((_e138.y >> bitcast<u32>(5u)) * (_e140 << bitcast<u32>(5u))) + ((_e138.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e138.x & 28u) << bitcast<u32>(5u)) + ((_e138.y & 28u) << bitcast<u32>(2i)))) + (((_e138.y & 3u) << bitcast<u32>(2i)) + (_e138.x & 3u))))]), ((_e180 - max(_e180, _e177)) - _e173));
    }
    return;
}

@fragment
fn main(@location(2) I: vec4<f32>, @location(8) j4_: vec2<f32>, @location(7) @interpolate(flat) e3_: vec2<u32>, @location(0) i1_: vec4<f32>, @location(3) @interpolate(flat) z0_: f32, @location(4) @interpolate(flat) S1_: vec2<f32>, @location(5) N0_: vec4<f32>, @location(6) @interpolate(flat) Z1_: f32) {
    I_1 = I;
    j4_1 = j4_;
    e3_1 = e3_;
    i1_1 = i1_;
    z0_1 = z0_;
    S1_1 = S1_;
    N0_1 = N0_;
    Z1_1 = Z1_;
    main_1();
}
