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

@id(7) override dh: bool = true;
@id(2) override Yg: bool = true;

@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Kb: sampler;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> Hg: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> U1_1: vec2<f32>;
var<private> e2_1: f32;

fn main_1() {
    var phi_588_: vec4<f32>;
    var phi_585_: f32;
    var phi_586_: f32;
    var phi_590_: vec4<f32>;
    var phi_581_: f32;
    var phi_591_: vec4<f32>;
    var phi_589_: vec4<f32>;
    var phi_587_: vec4<f32>;
    var phi_592_: vec3<f32>;

    let _e26 = f1_1;
    if (_e26.w >= 0f) {
        if Yg {
            phi_588_ = vec4<f32>(_e26.x, _e26.y, _e26.z, _e26.w);
        } else {
            phi_588_ = (_e26 * 1f);
        }
        let _e37 = phi_588_;
        phi_587_ = _e37;
    } else {
        if (_e26.w > -1f) {
            if (_e26.z > 0f) {
                phi_585_ = _e26.x;
            } else {
                phi_585_ = length(_e26.xy);
            }
            let _e45 = phi_585_;
            let _e46 = clamp(_e45, 0f, 1f);
            let _e47 = abs(_e26.z);
            if (_e47 > 1f) {
                phi_586_ = ((0.9980469f * _e46) + 0.0009765625f);
            } else {
                phi_586_ = ((0.001953125f * _e46) + _e47);
            }
            let _e54 = phi_586_;
            let _e57 = textureSampleLevel(KD, Kb, vec2<f32>(_e54, -(_e26.w)), 0f);
            let _e63 = vec4<f32>(_e57.x, _e57.y, _e57.z, _e57.w);
            if Yg {
                phi_590_ = _e63;
            } else {
                let _e65 = (_e63.xyz * _e57.w);
                phi_590_ = vec4<f32>(_e65.x, _e65.y, _e65.z, _e57.w);
            }
            let _e71 = phi_590_;
            phi_589_ = _e71;
        } else {
            let _e74 = textureSampleLevel(IC, S5_, _e26.xy, (-2f - _e26.w));
            if Yg {
                if (_e74.w != 0f) {
                    phi_581_ = (1f / _e74.w);
                } else {
                    phi_581_ = 0f;
                }
                let _e81 = phi_581_;
                let _e82 = (_e74.xyz * _e81);
                phi_591_ = vec4<f32>(_e82.x, _e82.y, _e82.z, (_e74.w * _e26.z));
            } else {
                phi_591_ = (_e74 * _e26.z);
            }
            let _e90 = phi_591_;
            phi_589_ = _e90;
        }
        let _e92 = phi_589_;
        phi_587_ = _e92;
    }
    let _e94 = phi_587_;
    let _e95 = _e94.xyz;
    let _e97 = gl_FragCoord_1;
    let _e99 = n.z3_;
    let _e101 = n.A3_;
    if (dh && (_e94.w != 0f)) {
        phi_592_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e97.x) + (0.00583715f * _e97.y))))) * _e99) + _e101)) + _e95);
    } else {
        phi_592_ = _e95;
    }
    let _e117 = phi_592_;
    let _e123 = vec4<f32>(_e117.x, _e94.y, _e94.z, _e94.w);
    let _e129 = vec4<f32>(_e123.x, _e117.y, _e123.z, _e123.w);
    Hg = vec4<f32>(_e129.x, _e129.y, _e117.z, _e129.w);
    return;
}

@fragment
fn main(@location(0) f1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @location(6) @interpolate(flat, either) e2_: f32) -> @location(0) vec4<f32> {
    f1_1 = f1_;
    gl_FragCoord_1 = gl_FragCoord;
    U1_1 = U1_;
    e2_1 = e2_;
    main_1();
    let _e9 = Hg;
    return _e9;
}
