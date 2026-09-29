struct BC {
    qc: f32,
    Ad: f32,
    Ef: f32,
    Ff: f32,
    q6_: u32,
    Nb: u32,
    qf: u32,
    rf: u32,
    V7_: vec4<i32>,
    bh: vec2<f32>,
    Bd: vec2<f32>,
    d2_: u32,
    fh: f32,
    e6_: u32,
    T2_: f32,
    Cd: f32,
    lf: u32,
    B3_: f32,
    C3_: f32,
    Dd: f32,
    Yg: u32,
}

@id(7) override Ih: bool = true;
@id(2) override Dh: bool = true;
@id(8) override Jh: bool = true;

@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(10)
var FD: texture_2d<f32>;
@group(3) @binding(10)
var R9_: sampler;
var<private> F2_1: vec2<f32>;
var<private> V1_1: vec4<f32>;
var<private> C2_1: vec3<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> n: BC;
var<private> ih: vec4<f32>;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> K3_1: f32;
var<private> g2_1: f32;

fn main_1() {
    var phi_643_: vec4<f32>;
    var phi_627_: f32;
    var phi_628_: f32;
    var phi_644_: vec4<f32>;
    var phi_642_: vec4<f32>;
    var phi_442_: bool;
    var phi_629_: f32;
    var phi_639_: vec4<f32>;
    var phi_646_: vec4<f32>;
    var phi_647_: vec3<f32>;

    let _e29 = F2_1;
    let _e30 = textureSampleLevel(FD, R9_, _e29, 0f);
    let _e32 = clamp(_e30.x, 0f, 1f);
    let _e33 = V1_1;
    let _e34 = C2_1;
    if (_e33.w >= 0f) {
        if Dh {
            phi_643_ = vec4<f32>(_e33.x, _e33.y, _e33.z, (_e33.w * _e32));
        } else {
            phi_643_ = (_e33 * _e32);
        }
        let _e46 = phi_643_;
        phi_642_ = _e46;
    } else {
        if (_e33.z > 0f) {
            phi_627_ = _e33.x;
        } else {
            phi_627_ = length(_e33.xy);
        }
        let _e54 = phi_627_;
        let _e55 = clamp(_e54, 0f, 1f);
        let _e56 = abs(_e33.z);
        if (_e56 > 1f) {
            phi_628_ = ((0.9980469f * _e55) + 0.0009765625f);
        } else {
            phi_628_ = ((0.001953125f * _e55) + _e56);
        }
        let _e63 = phi_628_;
        let _e65 = textureSampleLevel(ED, N9_, vec2<f32>(_e63, -(_e33.w)), 0f);
        let _e67 = (_e65.w * _e32);
        let _e72 = vec4<f32>(_e65.x, _e65.y, _e65.z, _e67);
        if Dh {
            phi_644_ = _e72;
        } else {
            let _e74 = (_e72.xyz * _e67);
            phi_644_ = vec4<f32>(_e74.x, _e74.y, _e74.z, _e67);
        }
        let _e80 = phi_644_;
        phi_642_ = _e80;
    }
    let _e82 = phi_642_;
    phi_442_ = Jh;
    if Jh {
        phi_442_ = (_e34.z > 0f);
    }
    let _e86 = phi_442_;
    phi_646_ = _e82;
    if _e86 {
        let _e90 = textureSampleLevel(HC, W5_, _e34.xy, (_e34.z - 1f));
        phi_639_ = _e90;
        if Dh {
            if (_e90.w != 0f) {
                phi_629_ = (1f / _e90.w);
            } else {
                phi_629_ = 0f;
            }
            let _e96 = phi_629_;
            let _e97 = (_e90.xyz * _e96);
            phi_639_ = vec4<f32>(_e97.x, _e97.y, _e97.z, _e90.w);
        }
        let _e103 = phi_639_;
        phi_646_ = (_e82 * _e103);
    }
    let _e106 = phi_646_;
    let _e107 = _e106.xyz;
    let _e109 = gl_FragCoord_1;
    let _e111 = n.B3_;
    let _e113 = n.C3_;
    if (Ih && (_e106.w != 0f)) {
        phi_647_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e109.x) + (0.00583715f * _e109.y))))) * _e111) + _e113)) + _e107);
    } else {
        phi_647_ = _e107;
    }
    let _e129 = phi_647_;
    let _e135 = vec4<f32>(_e129.x, _e106.y, _e106.z, _e106.w);
    let _e141 = vec4<f32>(_e135.x, _e129.y, _e135.z, _e135.w);
    ih = vec4<f32>(_e141.x, _e141.y, _e129.z, _e141.w);
    return;
}

@fragment
fn main(@location(1) F2_: vec2<f32>, @location(0) V1_: vec4<f32>, @location(9) C2_: vec3<f32>, @builtin(position) gl_FragCoord: vec4<f32>, @location(4) @interpolate(flat, either) K3_: f32, @location(6) @interpolate(flat, either) g2_: f32) -> @location(0) vec4<f32> {
    F2_1 = F2_;
    V1_1 = V1_;
    C2_1 = C2_;
    gl_FragCoord_1 = gl_FragCoord;
    K3_1 = K3_;
    g2_1 = g2_;
    main_1();
    let _e13 = ih;
    return _e13;
}
