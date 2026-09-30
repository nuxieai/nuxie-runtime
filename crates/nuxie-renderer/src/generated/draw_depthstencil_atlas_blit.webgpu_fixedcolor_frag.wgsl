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
@group(0) @binding(10)
var ED: texture_2d<f32>;
@group(3) @binding(10)
var T9_: sampler;
var<private> F2_1: vec2<f32>;
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
var<private> O3_1: f32;

fn main_1() {
    var phi_620_: f32;
    var phi_621_: f32;
    var phi_635_: vec4<f32>;
    var phi_634_: vec4<f32>;
    var phi_440_: bool;
    var phi_622_: f32;
    var phi_631_: vec4<f32>;
    var phi_637_: vec4<f32>;
    var phi_638_: vec3<f32>;

    let _e30 = F2_1;
    let _e31 = textureSampleLevel(ED, T9_, _e30, 0f);
    let _e34 = g1_1;
    let _e36 = C2_1;
    let _e37 = X1_1;
    let _e39 = (Gh && (u32(_e34) != 0u));
    if (_e37.w >= 0f) {
        phi_634_ = _e37;
    } else {
        if (_e37.z > 0f) {
            phi_620_ = _e37.x;
        } else {
            phi_620_ = length(_e37.xy);
        }
        let _e49 = phi_620_;
        let _e50 = clamp(_e49, 0f, 1f);
        let _e51 = abs(_e37.z);
        if (_e51 > 1f) {
            phi_621_ = ((0.9980469f * _e50) + 0.0009765625f);
        } else {
            phi_621_ = ((0.001953125f * _e50) + _e51);
        }
        let _e58 = phi_621_;
        let _e60 = textureSampleLevel(DD, P9_, vec2<f32>(_e58, -(_e37.w)), 0f);
        phi_635_ = _e60;
        if !(_e39) {
            let _e64 = (_e60.xyz * _e60.w);
            let _e70 = vec4<f32>(_e64.x, _e60.y, _e60.z, _e60.w);
            let _e76 = vec4<f32>(_e70.x, _e64.y, _e70.z, _e70.w);
            phi_635_ = vec4<f32>(_e76.x, _e76.y, _e64.z, _e76.w);
        }
        let _e84 = phi_635_;
        phi_634_ = _e84;
    }
    let _e86 = phi_634_;
    phi_440_ = Mh;
    if Mh {
        phi_440_ = (_e36.z > 0f);
    }
    let _e90 = phi_440_;
    phi_637_ = _e86;
    if _e90 {
        let _e94 = textureSampleLevel(GC, Y5_, _e36.xy, (_e36.z - 1f));
        phi_631_ = _e94;
        if _e39 {
            if (_e94.w != 0f) {
                phi_622_ = (1f / _e94.w);
            } else {
                phi_622_ = 0f;
            }
            let _e100 = phi_622_;
            let _e101 = (_e94.xyz * _e100);
            phi_631_ = vec4<f32>(_e101.x, _e101.y, _e101.z, _e94.w);
        }
        let _e107 = phi_631_;
        phi_637_ = (_e86 * _e107);
    }
    let _e110 = phi_637_;
    let _e111 = (_e110 * clamp(_e31.x, 0f, 1f));
    let _e112 = _e111.xyz;
    let _e114 = gl_FragCoord_1;
    let _e116 = j.F3_;
    let _e118 = j.G3_;
    if (Lh && (_e111.w != 0f)) {
        phi_638_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e114.x) + (0.00583715f * _e114.y))))) * _e116) + _e118)) + _e112);
    } else {
        phi_638_ = _e112;
    }
    let _e134 = phi_638_;
    let _e140 = vec4<f32>(_e134.x, _e111.y, _e111.z, _e111.w);
    let _e146 = vec4<f32>(_e140.x, _e134.y, _e140.z, _e140.w);
    lh = vec4<f32>(_e146.x, _e146.y, _e134.z, _e146.w);
    return;
}

@fragment
fn main(@location(1) F2_: vec2<f32>, @location(9) C2_: vec3<f32>, @location(6) @interpolate(flat, either) g1_: f32, @location(0) X1_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) O3_: f32) -> @location(0) vec4<f32> {
    F2_1 = F2_;
    C2_1 = C2_;
    g1_1 = g1_;
    X1_1 = X1_;
    gl_FragCoord_1 = gl_FragCoord;
    O3_1 = O3_;
    main_1();
    let _e13 = lh;
    return _e13;
}
