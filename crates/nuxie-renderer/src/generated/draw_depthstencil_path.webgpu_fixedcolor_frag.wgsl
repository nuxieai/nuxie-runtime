struct VB {
    xd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    F8_: vec4<i32>,
    zi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Di: f32,
    w6_: u32,
    i3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    wi: u32,
    Ba: u32,
    M8_: f32,
    N8_: f32,
}

@id(7) override ej: bool = true;
@id(15) override mj: bool = false;
@id(16) override nj: bool = false;
@id(2) override Zi: bool = true;
@id(8) override fj: bool = true;

var<private> P0_1: f32;
var<private> O0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var O8_: sampler;
var<private> U0_1: vec3<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Gi: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;

fn main_1() {
    var phi_779_: f32;
    var phi_780_: f32;
    var phi_781_: vec2<f32>;
    var phi_782_: bool;
    var phi_784_: vec4<f32>;
    var phi_801_: vec4<f32>;
    var phi_800_: vec4<f32>;
    var phi_412_: bool;
    var phi_785_: f32;
    var phi_797_: vec4<f32>;
    var phi_803_: vec4<f32>;
    var phi_804_: vec3<f32>;

    let _e42 = P0_1;
    let _e45 = (Zi && (u32(_e42) != 0u));
    let _e47 = O0_1[3u];
    if (_e47 >= 0f) {
        let _e125 = O0_1;
        phi_800_ = _e125;
    } else {
        let _e49 = O0_1;
        let _e51 = j.M8_;
        let _e53 = j.N8_;
        let _e56 = abs(_e49.z);
        let _e57 = floor(_e56);
        let _e60 = (((_e56 - _e57) * 8f) + 0.0009765625f);
        let _e62 = ((_e57 * _e51) + _e53);
        let _e63 = floor(_e60);
        switch bitcast<i32>(0u) {
            default: {
                if (_e63 == 4f) {
                    let _e81 = normalize(vec2<f32>(_e49.x, -(_e49.y)));
                    let _e84 = acos(clamp(_e81.x, -1f, 1f));
                    if (_e81.y >= 0f) {
                        phi_780_ = _e84;
                    } else {
                        phi_780_ = -(_e84);
                    }
                    let _e89 = phi_780_;
                    phi_781_ = vec2<f32>((_e89 * 0.15915507f), _e62);
                    break;
                } else {
                    if (_e63 == 2f) {
                        phi_779_ = _e49.x;
                    } else {
                        phi_779_ = length(_e49.xy);
                    }
                    let _e71 = phi_779_;
                    phi_781_ = vec2<f32>(((clamp(_e71, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e49.z < 0f))) + _e60), _e62);
                    break;
                }
            }
        }
        let _e93 = phi_781_;
        let _e94 = textureSampleLevel(XC, O8_, _e93, 0f);
        switch bitcast<i32>(0u) {
            default: {
                if mj {
                    phi_782_ = true;
                    break;
                }
                if nj {
                    phi_782_ = true;
                    break;
                }
                phi_782_ = false;
                break;
            }
        }
        let _e97 = phi_782_;
        phi_784_ = _e94;
        if _e97 {
            phi_784_ = vec4<f32>(_e94.x, _e94.y, _e94.z, (_e94.w * ((fract(_e49.w) * -2f) + 1.5f)));
        }
        let _e110 = phi_784_;
        phi_801_ = _e110;
        if !(_e45) {
            let _e114 = (_e110.xyz * _e110.w);
            phi_801_ = vec4<f32>(_e114.x, _e114.y, _e114.z, (_e110.w * (ceil(_e49.w) * -0.003921569f)));
        }
        let _e124 = phi_801_;
        phi_800_ = _e124;
    }
    let _e127 = phi_800_;
    phi_412_ = fj;
    if fj {
        let _e129 = U0_1[2u];
        phi_412_ = (_e129 > 0f);
    }
    let _e132 = phi_412_;
    phi_803_ = _e127;
    if _e132 {
        let _e134 = U0_1[2u];
        let _e136 = U0_1;
        let _e138 = textureSampleLevel(TB, U4_, _e136.xy, (_e134 - 1f));
        phi_797_ = _e138;
        if _e45 {
            if (_e138.w != 0f) {
                phi_785_ = (1f / _e138.w);
            } else {
                phi_785_ = 0f;
            }
            let _e144 = phi_785_;
            let _e145 = (_e138.xyz * _e144);
            phi_797_ = vec4<f32>(_e145.x, _e145.y, _e145.z, _e138.w);
        }
        let _e151 = phi_797_;
        phi_803_ = (_e127 * _e151);
    }
    let _e154 = phi_803_;
    let _e155 = _e154.xyz;
    let _e157 = gl_FragCoord_1;
    let _e159 = j.F3_;
    let _e161 = j.G3_;
    if (ej && (_e154.w != 0f)) {
        phi_804_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e157.x) + (0.00583715f * _e157.y))))) * _e159) + _e161)) + _e155);
    } else {
        phi_804_ = _e155;
    }
    let _e177 = phi_804_;
    let _e183 = vec4<f32>(_e177.x, _e154.y, _e154.z, _e154.w);
    let _e189 = vec4<f32>(_e183.x, _e177.y, _e183.z, _e183.w);
    Gi = vec4<f32>(_e189.x, _e189.y, _e177.z, _e189.w);
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) P0_: f32, @location(0) O0_: vec4<f32>, @location(2) U0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    P0_1 = P0_;
    O0_1 = O0_;
    U0_1 = U0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Gi;
    return _e9;
}
