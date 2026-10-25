struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(16) override zj: bool = false;
@id(17) override Aj: bool = false;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;

var<private> S0_1: f32;
var<private> N0_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
var<private> O0_1: vec3<f32>;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ri: vec4<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;

fn main_1() {
    var phi_999_: f32;
    var phi_1000_: f32;
    var phi_1001_: vec2<f32>;
    var phi_1002_: bool;
    var phi_1004_: vec4<f32>;
    var phi_1021_: vec4<f32>;
    var phi_1020_: vec4<f32>;
    var phi_502_: bool;
    var phi_1005_: f32;
    var phi_1017_: vec4<f32>;
    var phi_1023_: vec4<f32>;
    var phi_1024_: vec3<f32>;
    var phi_563_: bool;
    var phi_1040_: f32;
    var phi_1055_: vec4<f32>;

    let _e48 = S0_1;
    let _e51 = (lj && (u32(_e48) != 0u));
    let _e53 = N0_1[3u];
    if (_e53 >= 0f) {
        let _e132 = N0_1;
        phi_1020_ = _e132;
    } else {
        let _e55 = N0_1;
        let _e57 = j.Q8_;
        let _e59 = j.R8_;
        let _e62 = abs(_e55.z);
        let _e63 = floor(_e62);
        let _e66 = (((_e62 - _e63) * 8f) + 0.0009765625f);
        let _e68 = ((_e63 * _e57) + _e59);
        let _e69 = floor(_e66);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e69 == 4f)) {
                    let _e88 = normalize(vec2<f32>(_e55.x, -(_e55.y)));
                    let _e91 = acos(clamp(_e88.x, -1f, 1f));
                    if (_e88.y >= 0f) {
                        phi_1000_ = _e91;
                    } else {
                        phi_1000_ = -(_e91);
                    }
                    let _e96 = phi_1000_;
                    phi_1001_ = vec2<f32>((_e96 * 0.15915494f), _e68);
                    break;
                } else {
                    if (_e69 == 2f) {
                        phi_999_ = _e55.x;
                    } else {
                        phi_999_ = length(_e55.xy);
                    }
                    let _e78 = phi_999_;
                    phi_1001_ = vec2<f32>(((clamp(_e78, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e55.z < 0f))) + _e66), _e68);
                    break;
                }
            }
        }
        let _e100 = phi_1001_;
        let _e101 = textureSampleLevel(ZC, S8_, _e100, 0f);
        switch bitcast<i32>(0u) {
            default: {
                if zj {
                    phi_1002_ = true;
                    break;
                }
                if Aj {
                    phi_1002_ = true;
                    break;
                }
                phi_1002_ = false;
                break;
            }
        }
        let _e104 = phi_1002_;
        phi_1004_ = _e101;
        if _e104 {
            phi_1004_ = vec4<f32>(_e101.x, _e101.y, _e101.z, (_e101.w * ((fract(_e55.w) * -2f) + 1.5f)));
        }
        let _e117 = phi_1004_;
        phi_1021_ = _e117;
        if !(_e51) {
            let _e121 = (_e117.xyz * _e117.w);
            phi_1021_ = vec4<f32>(_e121.x, _e121.y, _e121.z, (_e117.w * (ceil(_e55.w) * -0.003921569f)));
        }
        let _e131 = phi_1021_;
        phi_1020_ = _e131;
    }
    let _e134 = phi_1020_;
    phi_502_ = nj;
    if nj {
        let _e136 = O0_1[2u];
        phi_502_ = (_e136 > 0f);
    }
    let _e139 = phi_502_;
    phi_1023_ = _e134;
    if _e139 {
        let _e141 = O0_1[2u];
        let _e143 = O0_1;
        let _e145 = textureSampleLevel(RB, v4_, _e143.xy, (_e141 - 1f));
        phi_1017_ = _e145;
        if _e51 {
            if (_e145.w != 0f) {
                phi_1005_ = (1f / _e145.w);
            } else {
                phi_1005_ = 0f;
            }
            let _e151 = phi_1005_;
            let _e152 = (_e145.xyz * _e151);
            phi_1017_ = vec4<f32>(_e152.x, _e152.y, _e152.z, _e145.w);
        }
        let _e158 = phi_1017_;
        phi_1023_ = (_e134 * _e158);
    }
    let _e161 = phi_1023_;
    let _e162 = _e161.xyz;
    let _e164 = gl_FragCoord_1;
    let _e166 = j.H3_;
    let _e168 = j.I3_;
    if (rj && (_e161.w != 0f)) {
        phi_1024_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e164.x) + (0.00583715f * _e164.y))))) * _e166) + _e168)) + _e162);
    } else {
        phi_1024_ = _e162;
    }
    let _e184 = phi_1024_;
    let _e190 = vec4<f32>(_e184.x, _e161.y, _e161.z, _e161.w);
    let _e196 = vec4<f32>(_e190.x, _e184.y, _e190.z, _e190.w);
    phi_563_ = nj;
    if nj {
        let _e204 = O0_1[2u];
        phi_563_ = (_e204 < 0f);
    }
    let _e207 = phi_563_;
    phi_1055_ = vec4<f32>(_e196.x, _e196.y, _e184.z, _e196.w);
    if _e207 {
        let _e208 = O0_1;
        let _e210 = textureSampleLevel(RB, v4_, _e208.xy, 0f);
        let _e212 = O0_1[2u];
        let _e214 = u32((-1f - _e212));
        switch bitcast<i32>(0u) {
            default: {
                let _e217 = dot(_e210.xyz, vec3<f32>(0.3f, 0.59f, 0.11f));
                if (_e214 == 0u) {
                    phi_1040_ = _e210.w;
                    break;
                }
                if (_e214 == 1u) {
                    phi_1040_ = (1f - _e210.w);
                    break;
                }
                if (_e214 == 2u) {
                    phi_1040_ = _e217;
                    break;
                }
                phi_1040_ = (1f - _e217);
                break;
            }
        }
        let _e226 = phi_1040_;
        phi_1055_ = vec4(mix(1f, _e226, _e53));
    }
    let _e230 = phi_1055_;
    Ri = _e230;
    return;
}

@fragment
fn main(@location(1) @interpolate(flat, either) S0_: f32, @location(0) N0_: vec4<f32>, @location(2) O0_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    S0_1 = S0_;
    N0_1 = N0_;
    O0_1 = O0_;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e9 = Ri;
    return _e9;
}
