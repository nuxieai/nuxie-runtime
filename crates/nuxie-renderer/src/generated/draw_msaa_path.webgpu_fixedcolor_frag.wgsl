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

@id(7) override lh: bool = true;
@id(2) override gh: bool = true;
@id(8) override mh: bool = true;

@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(3) @binding(8)
var Ob: sampler;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var U5_: sampler;
var<private> f1_1: vec4<f32>;
var<private> A2_1: vec3<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> Pg: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> U1_1: vec2<f32>;
var<private> e2_1: f32;

fn main_1() {
    var phi_589_: vec4<f32>;
    var phi_573_: f32;
    var phi_574_: f32;
    var phi_590_: vec4<f32>;
    var phi_588_: vec4<f32>;
    var phi_434_: bool;
    var phi_575_: f32;
    var phi_585_: vec4<f32>;
    var phi_592_: vec4<f32>;
    var phi_593_: vec3<f32>;

    let _e26 = f1_1;
    let _e27 = A2_1;
    if (_e26.w >= 0f) {
        if gh {
            phi_589_ = vec4<f32>(_e26.x, _e26.y, _e26.z, _e26.w);
        } else {
            phi_589_ = (_e26 * 1f);
        }
        let _e38 = phi_589_;
        phi_588_ = _e38;
    } else {
        if (_e26.z > 0f) {
            phi_573_ = _e26.x;
        } else {
            phi_573_ = length(_e26.xy);
        }
        let _e45 = phi_573_;
        let _e46 = clamp(_e45, 0f, 1f);
        let _e47 = abs(_e26.z);
        if (_e47 > 1f) {
            phi_574_ = ((0.9980469f * _e46) + 0.0009765625f);
        } else {
            phi_574_ = ((0.001953125f * _e46) + _e47);
        }
        let _e54 = phi_574_;
        let _e57 = textureSampleLevel(MD, Ob, vec2<f32>(_e54, -(_e26.w)), 0f);
        let _e63 = vec4<f32>(_e57.x, _e57.y, _e57.z, _e57.w);
        if gh {
            phi_590_ = _e63;
        } else {
            let _e65 = (_e63.xyz * _e57.w);
            phi_590_ = vec4<f32>(_e65.x, _e65.y, _e65.z, _e57.w);
        }
        let _e71 = phi_590_;
        phi_588_ = _e71;
    }
    let _e73 = phi_588_;
    phi_434_ = mh;
    if mh {
        phi_434_ = (_e27.z > 0f);
    }
    let _e77 = phi_434_;
    phi_592_ = _e73;
    if _e77 {
        let _e81 = textureSampleLevel(JC, U5_, _e27.xy, (_e27.z - 1f));
        phi_585_ = _e81;
        if gh {
            if (_e81.w != 0f) {
                phi_575_ = (1f / _e81.w);
            } else {
                phi_575_ = 0f;
            }
            let _e87 = phi_575_;
            let _e88 = (_e81.xyz * _e87);
            phi_585_ = vec4<f32>(_e88.x, _e88.y, _e88.z, _e81.w);
        }
        let _e94 = phi_585_;
        phi_592_ = (_e73 * _e94);
    }
    let _e97 = phi_592_;
    let _e98 = _e97.xyz;
    let _e100 = gl_FragCoord_1;
    let _e102 = m.B3_;
    let _e104 = m.C3_;
    if (lh && (_e97.w != 0f)) {
        phi_593_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e100.x) + (0.00583715f * _e100.y))))) * _e102) + _e104)) + _e98);
    } else {
        phi_593_ = _e98;
    }
    let _e120 = phi_593_;
    let _e126 = vec4<f32>(_e120.x, _e97.y, _e97.z, _e97.w);
    let _e132 = vec4<f32>(_e126.x, _e120.y, _e126.z, _e126.w);
    Pg = vec4<f32>(_e132.x, _e132.y, _e120.z, _e132.w);
    return;
}

@fragment
fn main(@location(0) f1_: vec4<f32>, @location(9) A2_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) U1_: vec2<f32>, @location(6) @interpolate(flat, either) e2_: f32) -> @location(0) vec4<f32> {
    f1_1 = f1_;
    A2_1 = A2_;
    gl_FragCoord_1 = gl_FragCoord;
    U1_1 = U1_;
    e2_1 = e2_;
    main_1();
    let _e11 = Pg;
    return _e11;
}
