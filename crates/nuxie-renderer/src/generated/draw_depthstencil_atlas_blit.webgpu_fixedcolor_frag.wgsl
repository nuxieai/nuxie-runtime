struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

@id(7) override Oh: bool = true;
@id(2) override Jh: bool = true;
@id(8) override Ph: bool = true;

@group(0) @binding(0)
var<uniform> j: TB;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var M9_: sampler;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
@group(0) @binding(10)
var ED: texture_2d<f32>;
@group(3) @binding(10)
var R9_: sampler;
var<private> F2_1: vec2<f32>;
var<private> C2_1: vec3<f32>;
var<private> g1_1: f32;
var<private> X1_1: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> oh: vec4<f32>;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> O3_1: f32;

fn main_1() {
    var phi_687_: f32;
    var phi_688_: f32;
    var phi_702_: vec4<f32>;
    var phi_701_: vec4<f32>;
    var phi_495_: bool;
    var phi_689_: f32;
    var phi_698_: vec4<f32>;
    var phi_704_: vec4<f32>;
    var phi_705_: vec3<f32>;

    let _e33 = F2_1;
    let _e34 = textureSampleLevel(ED, R9_, _e33, 0f);
    let _e37 = g1_1;
    let _e39 = C2_1;
    let _e40 = X1_1;
    let _e42 = (Jh && (u32(_e37) != 0u));
    if (_e40.w >= 0f) {
        phi_701_ = _e40;
    } else {
        let _e45 = -(_e40.w);
        let _e50 = j.Zb;
        let _e53 = j.ac;
        if (_e40.z > 0f) {
            phi_687_ = _e40.x;
        } else {
            phi_687_ = length(_e40.xy);
        }
        let _e61 = phi_687_;
        let _e62 = clamp(_e61, 0f, 1f);
        let _e63 = abs(_e40.z);
        if (_e63 > 1f) {
            phi_688_ = ((0.9980469f * _e62) + 0.0009765625f);
        } else {
            phi_688_ = ((0.001953125f * _e62) + _e63);
        }
        let _e70 = phi_688_;
        let _e72 = textureSampleLevel(DD, M9_, vec2<f32>(_e70, ((floor(_e45) * _e50) + _e53)), 0f);
        phi_702_ = _e72;
        if !(_e42) {
            let _e76 = (_e72.xyz * _e72.w);
            phi_702_ = vec4<f32>(_e76.x, _e76.y, _e76.z, (_e72.w * (fract(_e45) * 1.0039216f)));
        }
        let _e83 = phi_702_;
        phi_701_ = _e83;
    }
    let _e85 = phi_701_;
    phi_495_ = Ph;
    if Ph {
        phi_495_ = (_e39.z > 0f);
    }
    let _e89 = phi_495_;
    phi_704_ = _e85;
    if _e89 {
        let _e93 = textureSampleLevel(GC, W5_, _e39.xy, (_e39.z - 1f));
        phi_698_ = _e93;
        if _e42 {
            if (_e93.w != 0f) {
                phi_689_ = (1f / _e93.w);
            } else {
                phi_689_ = 0f;
            }
            let _e99 = phi_689_;
            let _e100 = (_e93.xyz * _e99);
            phi_698_ = vec4<f32>(_e100.x, _e100.y, _e100.z, _e93.w);
        }
        let _e106 = phi_698_;
        phi_704_ = (_e85 * _e106);
    }
    let _e109 = phi_704_;
    let _e110 = (_e109 * clamp(_e34.x, 0f, 1f));
    let _e111 = _e110.xyz;
    let _e113 = gl_FragCoord_1;
    let _e115 = j.F3_;
    let _e117 = j.G3_;
    if (Oh && (_e110.w != 0f)) {
        phi_705_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e113.x) + (0.00583715f * _e113.y))))) * _e115) + _e117)) + _e111);
    } else {
        phi_705_ = _e111;
    }
    let _e133 = phi_705_;
    let _e139 = vec4<f32>(_e133.x, _e110.y, _e110.z, _e110.w);
    let _e145 = vec4<f32>(_e139.x, _e133.y, _e139.z, _e139.w);
    oh = vec4<f32>(_e145.x, _e145.y, _e133.z, _e145.w);
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
    let _e13 = oh;
    return _e13;
}
