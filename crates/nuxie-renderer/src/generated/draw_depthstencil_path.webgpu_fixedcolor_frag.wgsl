struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

@id(7) override dj: bool = true;
@id(15) override lj: bool = false;
@id(16) override mj: bool = false;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;

var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
var<private> U0_1: vec3<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Fi: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;

fn main_1() {
    var phi_678_: f32;
    var phi_679_: bool;
    var phi_681_: vec4<f32>;
    var phi_696_: vec4<f32>;
    var phi_695_: vec4<f32>;
    var phi_365_: bool;
    var phi_682_: f32;
    var phi_692_: vec4<f32>;
    var phi_698_: vec4<f32>;
    var phi_699_: vec3<f32>;

    let _e39 = P0_1;
    let _e42 = (Yi && (u32(_e39) != 0u));
    let _e44 = O0_1[3u];
    if (_e44 >= 0f) {
        let _e46 = O0_1;
        phi_695_ = _e46;
    } else {
        let _e47 = O0_1;
        let _e49 = j.L8_;
        let _e51 = j.M8_;
        let _e54 = abs(_e47.z);
        let _e55 = floor(_e54);
        let _e58 = (((_e54 - _e55) * 8f) + 0.0009765625f);
        if (floor(_e58) == 2f) {
            phi_678_ = _e47.x;
        } else {
            phi_678_ = length(_e47.xy);
        }
        let _e67 = phi_678_;
        let _e73 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e67, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e47.z < 0f))) + _e58), ((_e55 * _e49) + _e51)), 0f);
        switch bitcast<i32>(0u) {
            default: {
                if lj {
                    phi_679_ = true;
                    break;
                }
                if mj {
                    phi_679_ = true;
                    break;
                }
                phi_679_ = false;
                break;
            }
        }
        let _e76 = phi_679_;
        phi_681_ = _e73;
        if _e76 {
            phi_681_ = vec4<f32>(_e73.x, _e73.y, _e73.z, (_e73.w * ((fract(_e47.w) * -2f) + 1.5f)));
        }
        let _e89 = phi_681_;
        phi_696_ = _e89;
        if !(_e42) {
            let _e93 = (_e89.xyz * _e89.w);
            phi_696_ = vec4<f32>(_e93.x, _e93.y, _e93.z, (_e89.w * (ceil(_e47.w) * -0.003921569f)));
        }
        let _e103 = phi_696_;
        phi_695_ = _e103;
    }
    let _e105 = phi_695_;
    phi_365_ = ej;
    if ej {
        let _e107 = U0_1[2u];
        phi_365_ = (_e107 > 0f);
    }
    let _e110 = phi_365_;
    phi_698_ = _e105;
    if _e110 {
        let _e112 = U0_1[2u];
        let _e114 = U0_1;
        let _e116 = textureSampleLevel(TB, U4_, _e114.xy, (_e112 - 1f));
        phi_692_ = _e116;
        if _e42 {
            if (_e116.w != 0f) {
                phi_682_ = (1f / _e116.w);
            } else {
                phi_682_ = 0f;
            }
            let _e122 = phi_682_;
            let _e123 = (_e116.xyz * _e122);
            phi_692_ = vec4<f32>(_e123.x, _e123.y, _e123.z, _e116.w);
        }
        let _e129 = phi_692_;
        phi_698_ = (_e105 * _e129);
    }
    let _e132 = phi_698_;
    let _e133 = _e132.xyz;
    let _e135 = gl_FragCoord_1;
    let _e137 = j.F3_;
    let _e139 = j.G3_;
    if (dj && (_e132.w != 0f)) {
        phi_699_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e135.x) + (0.00583715f * _e135.y))))) * _e137) + _e139)) + _e133);
    } else {
        phi_699_ = _e133;
    }
    let _e155 = phi_699_;
    let _e161 = vec4<f32>(_e155.x, _e132.y, _e132.z, _e132.w);
    let _e167 = vec4<f32>(_e161.x, _e155.y, _e161.z, _e161.w);
    Fi = vec4<f32>(_e167.x, _e167.y, _e155.z, _e167.w);
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(2) U0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    P0_1 = P0_;
    O0_1 = O0_;
    U0_1 = U0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Fi;
    return _e9;
}
