struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

@id(7) override Lh: bool = true;
@id(2) override Gh: bool = true;
@id(8) override Mh: bool = true;

@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: AC;
var<private> lh: vec4<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> Y1_1: vec2<f32>;

fn main_1() {
    var phi_593_: f32;
    var phi_594_: f32;
    var phi_608_: vec4<f32>;
    var phi_607_: vec4<f32>;
    var phi_413_: bool;
    var phi_595_: f32;
    var phi_604_: vec4<f32>;
    var phi_610_: vec4<f32>;
    var phi_611_: vec3<f32>;

    let _e27 = g1_1;
    let _e29 = C2_1;
    let _e30 = X1_1;
    let _e32 = (Gh && (u32(_e27) != 0u));
    if (_e30.w >= 0f) {
        phi_607_ = _e30;
    } else {
        if (_e30.z > 0f) {
            phi_593_ = _e30.x;
        } else {
            phi_593_ = length(_e30.xy);
        }
        let _e42 = phi_593_;
        let _e43 = clamp(_e42, 0f, 1f);
        let _e44 = abs(_e30.z);
        if (_e44 > 1f) {
            phi_594_ = ((0.9980469f * _e43) + 0.0009765625f);
        } else {
            phi_594_ = ((0.001953125f * _e43) + _e44);
        }
        let _e51 = phi_594_;
        let _e53 = textureSampleLevel(DD, P9_, vec2<f32>(_e51, -(_e30.w)), 0f);
        phi_608_ = _e53;
        if !(_e32) {
            let _e57 = (_e53.xyz * _e53.w);
            let _e63 = vec4<f32>(_e57.x, _e53.y, _e53.z, _e53.w);
            let _e69 = vec4<f32>(_e63.x, _e57.y, _e63.z, _e63.w);
            phi_608_ = vec4<f32>(_e69.x, _e69.y, _e57.z, _e69.w);
        }
        let _e77 = phi_608_;
        phi_607_ = _e77;
    }
    let _e79 = phi_607_;
    phi_413_ = Mh;
    if Mh {
        phi_413_ = (_e29.z > 0f);
    }
    let _e83 = phi_413_;
    phi_610_ = _e79;
    if _e83 {
        let _e87 = textureSampleLevel(GC, Y5_, _e29.xy, (_e29.z - 1f));
        phi_604_ = _e87;
        if _e32 {
            if (_e87.w != 0f) {
                phi_595_ = (1f / _e87.w);
            } else {
                phi_595_ = 0f;
            }
            let _e93 = phi_595_;
            let _e94 = (_e87.xyz * _e93);
            phi_604_ = vec4<f32>(_e94.x, _e94.y, _e94.z, _e87.w);
        }
        let _e100 = phi_604_;
        phi_610_ = (_e79 * _e100);
    }
    let _e103 = phi_610_;
    let _e104 = (_e103 * 1f);
    let _e105 = _e104.xyz;
    let _e107 = gl_FragCoord_1;
    let _e109 = j.F3_;
    let _e111 = j.G3_;
    if (Lh && (_e104.w != 0f)) {
        phi_611_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e107.x) + (0.00583715f * _e107.y))))) * _e109) + _e111)) + _e105);
    } else {
        phi_611_ = _e105;
    }
    let _e127 = phi_611_;
    let _e133 = vec4<f32>(_e127.x, _e104.y, _e104.z, _e104.w);
    let _e139 = vec4<f32>(_e133.x, _e127.y, _e133.z, _e133.w);
    lh = vec4<f32>(_e139.x, _e139.y, _e127.z, _e139.w);
    return;
}

@fragment
fn main(@location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec2<f32>) -> @location(0) vec4<f32> {
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    gl_FragCoord_1 = gl_FragCoord;
    Y1_1 = Y1_;
    main_1();
    let _e11 = lh;
    return _e11;
}
