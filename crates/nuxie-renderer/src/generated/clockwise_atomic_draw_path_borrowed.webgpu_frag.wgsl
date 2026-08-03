struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

struct ee {
    c2_: array<u32>,
}

struct ee_1 {
    c2_: array<atomic<u32>>,
}

@id(3) override Zg: bool = true;

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;
var<private> O_1: vec4<f32>;
var<private> l4_1: vec2<f32>;
var<private> d3_1: vec2<u32>;
@group(0) @binding(0)
var<uniform> n: CC;
@group(0) @binding(6)
var<storage, read_write> P0_: ee_1;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(3) @binding(8)
var Kb: sampler;
@group(1) @binding(13)
var S5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> B0_1: f32;
var<private> U1_1: vec2<f32>;
var<private> L0_1: vec4<f32>;
var<private> e2_1: f32;

fn main_1() {
    var phi_593_: bool;
    var phi_846_: f32;
    var phi_852_: f32;
    var phi_853_: f32;
    var phi_530_: bool;
    var phi_854_: f32;
    var phi_855_: f32;

    let _e47 = O_1;
    switch bitcast<i32>(0u) {
        default: {
            if (_e47.y >= 0f) {
                switch bitcast<i32>(0u) {
                    default: {
                        phi_530_ = Zg;
                        if Zg {
                            phi_530_ = (_e47.x < -1.5f);
                        }
                        let _e118 = phi_530_;
                        if _e118 {
                            let _e124 = textureSampleLevel(XC, aa, vec2<f32>((3f + _e47.x), 0f), 0f);
                            let _e129 = textureSampleLevel(XC, aa, vec2<f32>((1f - _e47.y), 0f), 0f);
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
                        phi_593_ = Zg;
                        if Zg {
                            phi_593_ = (_e47.y < -1.5f);
                        }
                        let _e54 = phi_593_;
                        if _e54 {
                            let _e58 = max(_e47.w, 0f);
                            if (_e47.z >= 0f) {
                                let _e61 = textureSampleLevel(XC, aa, vec2<f32>(_e58, 0f), 0f);
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
                                let _e84 = textureSampleLevel(XC, aa, vec2<f32>(_e81.x, 0f), 0f);
                                let _e87 = textureSampleLevel(XC, aa, vec2<f32>(_e81.y, 0f), 0f);
                                let _e90 = textureSampleLevel(XC, aa, vec2<f32>(_e81.z, 0f), 0f);
                                let _e93 = textureSampleLevel(XC, aa, vec2<f32>(_e81.w, 0f), 0f);
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
    let _e136 = l4_1;
    let _e138 = vec2<u32>(floor(_e136));
    let _e140 = d3_1[1u];
    let _e142 = d3_1[0u];
    let _e173 = u32(((abs(_e135) * 1024f) + 0.5f));
    let _e175 = n.a2_;
    let _e177 = (_e175 | (262144u - _e173));
    let _e180 = atomicMax((&P0_.c2_[(_e142 + (((((_e138.y >> bitcast<u32>(5u)) * (_e140 << bitcast<u32>(5u))) + ((_e138.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e138.x & 28u) << bitcast<u32>(5u)) + ((_e138.y & 28u) << bitcast<u32>(2i)))) + (((_e138.y & 3u) << bitcast<u32>(2i)) + (_e138.x & 3u))))]), _e177);
    if (_e180 >= _e175) {
        let _e185 = atomicAdd((&P0_.c2_[(_e142 + (((((_e138.y >> bitcast<u32>(5u)) * (_e140 << bitcast<u32>(5u))) + ((_e138.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e138.x & 28u) << bitcast<u32>(5u)) + ((_e138.y & 28u) << bitcast<u32>(2i)))) + (((_e138.y & 3u) << bitcast<u32>(2i)) + (_e138.x & 3u))))]), ((_e180 - max(_e180, _e177)) - _e173));
    }
    return;
}

@fragment
fn main(@location(2) O: vec4<f32>, @location(8) l4_: vec2<f32>, @location(7) @interpolate(flat, either) d3_: vec2<u32>, @location(0) f1_: vec4<f32>, @location(3) @interpolate(flat, either) B0_: f32, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @location(5) L0_: vec4<f32>, @location(6) @interpolate(flat, either) e2_: f32) {
    O_1 = O;
    l4_1 = l4_;
    d3_1 = d3_;
    f1_1 = f1_;
    B0_1 = B0_;
    U1_1 = U1_;
    L0_1 = L0_;
    e2_1 = e2_;
    main_1();
}
