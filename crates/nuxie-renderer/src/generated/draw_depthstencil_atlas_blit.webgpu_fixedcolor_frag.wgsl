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
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;

@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;
@group(0) @binding(10)
var JD: texture_2d<f32>;
@group(3) @binding(10)
var Ta: sampler;
var<private> U2_1: vec2<f32>;
var<private> O0_1: vec3<f32>;
var<private> S0_1: f32;
var<private> N0_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> Ri: vec4<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> i4_1: f32;

fn main_1() {
    var phi_909_: f32;
    var phi_910_: f32;
    var phi_911_: vec2<f32>;
    var phi_928_: vec4<f32>;
    var phi_927_: vec4<f32>;
    var phi_602_: bool;
    var phi_617_: bool;
    var phi_912_: f32;
    var phi_923_: vec4<f32>;
    var phi_930_: vec4<f32>;
    var phi_931_: vec4<f32>;
    var phi_932_: vec3<f32>;

    let _e39 = U2_1;
    let _e40 = textureSampleLevel(JD, Ta, _e39, 0f);
    let _e43 = S0_1;
    let _e45 = O0_1;
    let _e46 = N0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e49 = (lj && (u32(_e43) != 0u));
            if (_e46.w >= 0f) {
                phi_927_ = _e46;
            } else {
                let _e53 = j.Q8_;
                let _e55 = j.R8_;
                let _e58 = abs(_e46.z);
                let _e59 = floor(_e58);
                let _e62 = (((_e58 - _e59) * 8f) + 0.0009765625f);
                let _e64 = ((_e59 * _e53) + _e55);
                let _e65 = floor(_e62);
                switch bitcast<i32>(0u) {
                    default: {
                        if (sj && (_e65 == 4f)) {
                            let _e84 = normalize(vec2<f32>(_e46.x, -(_e46.y)));
                            let _e87 = acos(clamp(_e84.x, -1f, 1f));
                            if (_e84.y >= 0f) {
                                phi_910_ = _e87;
                            } else {
                                phi_910_ = -(_e87);
                            }
                            let _e92 = phi_910_;
                            phi_911_ = vec2<f32>((_e92 * 0.15915494f), _e64);
                            break;
                        } else {
                            if (_e65 == 2f) {
                                phi_909_ = _e46.x;
                            } else {
                                phi_909_ = length(_e46.xy);
                            }
                            let _e74 = phi_909_;
                            phi_911_ = vec2<f32>(((clamp(_e74, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e46.z < 0f))) + _e62), _e64);
                            break;
                        }
                    }
                }
                let _e96 = phi_911_;
                let _e97 = textureSampleLevel(ZC, S8_, _e96, 0f);
                phi_928_ = _e97;
                if !(_e49) {
                    let _e103 = (_e97.xyz * _e97.w);
                    phi_928_ = vec4<f32>(_e103.x, _e103.y, _e103.z, (_e97.w * (ceil(_e46.w) * -0.003921569f)));
                }
                let _e110 = phi_928_;
                phi_927_ = _e110;
            }
            let _e112 = phi_927_;
            phi_602_ = nj;
            if nj {
                phi_602_ = (_e45.z < 0f);
            }
            let _e116 = phi_602_;
            if _e116 {
                let _e118 = textureSampleLevel(RB, v4_, _e45.xy, 0f);
                phi_931_ = _e118;
                break;
            }
            phi_617_ = nj;
            if nj {
                phi_617_ = (_e45.z > 0f);
            }
            let _e122 = phi_617_;
            phi_930_ = _e112;
            if _e122 {
                let _e126 = textureSampleLevel(RB, v4_, _e45.xy, (_e45.z - 1f));
                phi_923_ = _e126;
                if _e49 {
                    if (_e126.w != 0f) {
                        phi_912_ = (1f / _e126.w);
                    } else {
                        phi_912_ = 0f;
                    }
                    let _e132 = phi_912_;
                    let _e133 = (_e126.xyz * _e132);
                    phi_923_ = vec4<f32>(_e133.x, _e133.y, _e133.z, _e126.w);
                }
                let _e139 = phi_923_;
                phi_930_ = (_e112 * _e139);
            }
            let _e142 = phi_930_;
            phi_931_ = _e142;
            break;
        }
    }
    let _e144 = phi_931_;
    let _e145 = (_e144 * clamp(_e40.x, 0f, 1f));
    let _e146 = _e145.xyz;
    let _e148 = gl_FragCoord_1;
    let _e150 = j.H3_;
    let _e152 = j.I3_;
    if (rj && (_e145.w != 0f)) {
        phi_932_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e148.x) + (0.00583715f * _e148.y))))) * _e150) + _e152)) + _e146);
    } else {
        phi_932_ = _e146;
    }
    let _e168 = phi_932_;
    let _e174 = vec4<f32>(_e168.x, _e145.y, _e145.z, _e145.w);
    let _e180 = vec4<f32>(_e174.x, _e168.y, _e174.z, _e174.w);
    Ri = vec4<f32>(_e180.x, _e180.y, _e168.z, _e180.w);
    return;
}

@fragment
fn main(@location(1) U2_: vec2<f32>, @location(9) O0_: vec3<f32>, @location(6) @interpolate(flat, either) S0_: f32, @location(0) N0_: vec4<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) i4_: f32) -> @location(0) vec4<f32> {
    U2_1 = U2_;
    O0_1 = O0_;
    S0_1 = S0_;
    N0_1 = N0_;
    gl_FragCoord_1 = gl_FragCoord;
    i4_1 = i4_;
    main_1();
    let _e13 = Ri;
    return _e13;
}
