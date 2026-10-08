struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

@id(7) override bj: bool = true;
@id(2) override Wi: bool = true;
@id(15) override jj: bool = false;
@id(8) override cj: bool = true;

var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var H8_: sampler;
var<private> V0_1: vec3<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ei: vec4<f32>;
@group(3) @binding(9)
var Ta: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;

fn main_1() {
    var phi_594_: f32;
    var phi_590_: f32;
    var phi_592_: vec2<f32>;
    var phi_595_: vec4<f32>;
    var phi_610_: vec4<f32>;
    var phi_609_: vec4<f32>;
    var phi_327_: bool;
    var phi_596_: f32;
    var phi_606_: vec4<f32>;
    var phi_612_: vec4<f32>;
    var phi_613_: vec3<f32>;

    let _e38 = P0_1;
    let _e41 = (Wi && (u32(_e38) != 0u));
    let _e43 = O0_1[3u];
    if (_e43 >= 0f) {
        let _e45 = O0_1;
        phi_609_ = _e45;
    } else {
        let _e46 = O0_1;
        let _e48 = j.Bb;
        let _e50 = j.g7_;
        let _e52 = bitcast<u32>(_e46.w);
        let _e56 = ((f32((_e52 & 268304384u)) * _e48) - _e50);
        let _e58 = abs(_e46.z);
        if (_e58 < 1.5f) {
            phi_594_ = _e58;
            phi_590_ = _e46.x;
        } else {
            phi_594_ = (_e58 - 2f);
            phi_590_ = length(_e46.xy);
        }
        let _e65 = phi_594_;
        let _e67 = phi_590_;
        let _e68 = clamp(_e67, 0f, 1f);
        if (_e46.z < 0f) {
            phi_592_ = vec2<f32>(((_e68 * 0.9980469f) + 0.0009765625f), _e56);
        } else {
            phi_592_ = vec2<f32>(((_e68 * 0.001953125f) + ((f32((_e52 & 130816u)) * 0.0000076293945f) + 0.0009765625f)), _e56);
        }
        let _e81 = phi_592_;
        let _e82 = textureSampleLevel(YC, H8_, _e81, 0f);
        phi_595_ = _e82;
        if jj {
            phi_595_ = vec4<f32>(_e82.x, _e82.y, _e82.z, (_e82.w * _e65));
        }
        let _e91 = phi_595_;
        phi_610_ = _e91;
        if !(_e41) {
            let _e95 = (_e91.xyz * _e91.w);
            phi_610_ = vec4<f32>(_e95.x, _e95.y, _e95.z, (_e91.w * (f32((_e52 & 255u)) * 0.003921569f)));
        }
        let _e105 = phi_610_;
        phi_609_ = _e105;
    }
    let _e107 = phi_609_;
    phi_327_ = cj;
    if cj {
        let _e109 = V0_1[2u];
        phi_327_ = (_e109 > 0f);
    }
    let _e112 = phi_327_;
    phi_612_ = _e107;
    if _e112 {
        let _e114 = V0_1[2u];
        let _e116 = V0_1;
        let _e118 = textureSampleLevel(TB, S4_, _e116.xy, (_e114 - 1f));
        phi_606_ = _e118;
        if _e41 {
            if (_e118.w != 0f) {
                phi_596_ = (1f / _e118.w);
            } else {
                phi_596_ = 0f;
            }
            let _e124 = phi_596_;
            let _e125 = (_e118.xyz * _e124);
            phi_606_ = vec4<f32>(_e125.x, _e125.y, _e125.z, _e118.w);
        }
        let _e131 = phi_606_;
        phi_612_ = (_e107 * _e131);
    }
    let _e134 = phi_612_;
    let _e135 = _e134.xyz;
    let _e137 = gl_FragCoord_1;
    let _e139 = j.E3_;
    let _e141 = j.F3_;
    if (bj && (_e134.w != 0f)) {
        phi_613_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e137.x) + (0.00583715f * _e137.y))))) * _e139) + _e141)) + _e135);
    } else {
        phi_613_ = _e135;
    }
    let _e157 = phi_613_;
    let _e163 = vec4<f32>(_e157.x, _e134.y, _e134.z, _e134.w);
    let _e169 = vec4<f32>(_e163.x, _e157.y, _e163.z, _e163.w);
    Ei = vec4<f32>(_e169.x, _e169.y, _e157.z, _e169.w);
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(2) V0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    P0_1 = P0_;
    O0_1 = O0_;
    V0_1 = V0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Ei;
    return _e9;
}
