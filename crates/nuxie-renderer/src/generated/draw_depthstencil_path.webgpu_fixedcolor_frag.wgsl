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

@id(7) override Jh: bool = true;
@id(2) override Eh: bool = true;
@id(8) override Kh: bool = true;

@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var X5_: sampler;
var<private> V1_1: vec4<f32>;
var<private> C2_1: vec3<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> jh: vec4<f32>;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> W1_1: vec2<f32>;
var<private> g2_1: f32;

fn main_1() {
    var phi_616_: vec4<f32>;
    var phi_600_: f32;
    var phi_601_: f32;
    var phi_617_: vec4<f32>;
    var phi_615_: vec4<f32>;
    var phi_415_: bool;
    var phi_602_: f32;
    var phi_612_: vec4<f32>;
    var phi_619_: vec4<f32>;
    var phi_620_: vec3<f32>;

    let _e26 = V1_1;
    let _e27 = C2_1;
    if (_e26.w >= 0f) {
        if Eh {
            phi_616_ = vec4<f32>(_e26.x, _e26.y, _e26.z, _e26.w);
        } else {
            phi_616_ = (_e26 * 1f);
        }
        let _e38 = phi_616_;
        phi_615_ = _e38;
    } else {
        if (_e26.z > 0f) {
            phi_600_ = _e26.x;
        } else {
            phi_600_ = length(_e26.xy);
        }
        let _e46 = phi_600_;
        let _e47 = clamp(_e46, 0f, 1f);
        let _e48 = abs(_e26.z);
        if (_e48 > 1f) {
            phi_601_ = ((0.9980469f * _e47) + 0.0009765625f);
        } else {
            phi_601_ = ((0.001953125f * _e47) + _e48);
        }
        let _e55 = phi_601_;
        let _e57 = textureSampleLevel(ED, N9_, vec2<f32>(_e55, -(_e26.w)), 0f);
        let _e63 = vec4<f32>(_e57.x, _e57.y, _e57.z, _e57.w);
        if Eh {
            phi_617_ = _e63;
        } else {
            let _e65 = (_e63.xyz * _e57.w);
            phi_617_ = vec4<f32>(_e65.x, _e65.y, _e65.z, _e57.w);
        }
        let _e71 = phi_617_;
        phi_615_ = _e71;
    }
    let _e73 = phi_615_;
    phi_415_ = Kh;
    if Kh {
        phi_415_ = (_e27.z > 0f);
    }
    let _e77 = phi_415_;
    phi_619_ = _e73;
    if _e77 {
        let _e81 = textureSampleLevel(HC, X5_, _e27.xy, (_e27.z - 1f));
        phi_612_ = _e81;
        if Eh {
            if (_e81.w != 0f) {
                phi_602_ = (1f / _e81.w);
            } else {
                phi_602_ = 0f;
            }
            let _e87 = phi_602_;
            let _e88 = (_e81.xyz * _e87);
            phi_612_ = vec4<f32>(_e88.x, _e88.y, _e88.z, _e81.w);
        }
        let _e94 = phi_612_;
        phi_619_ = (_e73 * _e94);
    }
    let _e97 = phi_619_;
    let _e98 = _e97.xyz;
    let _e100 = gl_FragCoord_1;
    let _e102 = l.C3_;
    let _e104 = l.D3_;
    if (Jh && (_e97.w != 0f)) {
        phi_620_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e100.x) + (0.00583715f * _e100.y))))) * _e102) + _e104)) + _e98);
    } else {
        phi_620_ = _e98;
    }
    let _e120 = phi_620_;
    let _e126 = vec4<f32>(_e120.x, _e97.y, _e97.z, _e97.w);
    let _e132 = vec4<f32>(_e126.x, _e120.y, _e126.z, _e126.w);
    jh = vec4<f32>(_e132.x, _e132.y, _e120.z, _e132.w);
    return;
}

@fragment
fn main(@location(0) V1_: vec4<f32>, @location(9) C2_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) W1_: vec2<f32>, @location(6) @interpolate(flat, either) g2_: f32) -> @location(0) vec4<f32> {
    V1_1 = V1_;
    C2_1 = C2_;
    gl_FragCoord_1 = gl_FragCoord;
    W1_1 = W1_;
    g2_1 = g2_;
    main_1();
    let _e11 = jh;
    return _e11;
}
