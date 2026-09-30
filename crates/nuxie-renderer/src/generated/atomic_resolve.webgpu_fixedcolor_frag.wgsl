struct Bf {
    j2_: array<vec2<u32>>,
}

struct m0ge {
    j2_: array<u32>,
}

struct Cf {
    j2_: array<vec4<f32>>,
}

struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct K4ge {
    j2_: array<u32>,
}

@id(7) override ii: bool = true;
@id(4) override fi: bool = true;
@id(0) override bi: bool = true;
@id(1) override ci: bool = true;
@id(2) override di: bool = true;

@group(0) @binding(3)
var<storage> XC: Bf;
@group(2) @binding(1)
var<storage, read_write> m0_: m0ge;
@group(0) @binding(4)
var<storage> JB: Cf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var FD: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(2) @binding(3)
var<storage, read_write> K4_: K4ge;
var<private> J1_: vec4<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var f6_: sampler;

fn main_1() {
    var phi_724_: bool;
    var phi_1068_: f32;
    var phi_1067_: f32;
    var phi_1069_: f32;
    var phi_1072_: f32;
    var phi_1071_: f32;
    var phi_761_: bool;
    var phi_1085_: f32;
    var phi_1073_: f32;
    var phi_1090_: vec4<f32>;
    var phi_1088_: vec4<f32>;
    var phi_1091_: vec3<f32>;

    let _e55 = gl_FragCoord_1;
    let _e56 = _e55.xy;
    let _e59 = bitcast<vec2<u32>>(vec2<i32>(floor(_e56)));
    let _e61 = j.z6_;
    let _e90 = bitcast<i32>((((((_e59.y >> bitcast<u32>(5u)) * (((_e61 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e59.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e59.x & 28u) << bitcast<u32>(5u)) + ((_e59.y & 28u) << bitcast<u32>(2i)))) + (((_e59.y & 3u) << bitcast<u32>(2i)) + (_e59.x & 3u))));
    let _e93 = K4_.j2_[_e90];
    let _e97 = ((f32((_e93 & 131071u)) * 0.00048828125f) + -32f);
    let _e99 = (_e93 >> bitcast<u32>(17u));
    let _e102 = XC.j2_[_e99];
    phi_1067_ = _e97;
    if ((_e102.x & 768u) != 0u) {
        let _e106 = abs(_e97);
        phi_724_ = fi;
        if fi {
            phi_724_ = ((_e102.x & 512u) != 0u);
        }
        let _e110 = phi_724_;
        phi_1068_ = _e106;
        if _e110 {
            phi_1068_ = (1f - abs(((fract((_e106 * 0.5f)) * 2f) + -1f)));
        }
        let _e118 = phi_1068_;
        phi_1067_ = _e118;
    }
    let _e120 = phi_1067_;
    let _e121 = clamp(_e120, 0f, 1f);
    phi_1071_ = _e121;
    if bi {
        let _e123 = (_e102.x >> bitcast<u32>(16u));
        phi_1072_ = _e121;
        if (_e123 != 0u) {
            let _e127 = m0_.j2_[_e90];
            if (_e123 == (_e127 >> bitcast<u32>(16i))) {
                phi_1069_ = min(_e121, unpack2x16float(_e127).x);
            } else {
                phi_1069_ = 0f;
            }
            let _e135 = phi_1069_;
            phi_1072_ = _e135;
        }
        let _e137 = phi_1072_;
        phi_1071_ = _e137;
    }
    let _e139 = phi_1071_;
    phi_761_ = ci;
    if ci {
        phi_761_ = ((_e102.x & 1024u) != 0u);
    }
    let _e143 = phi_761_;
    phi_1085_ = _e139;
    if _e143 {
        let _e144 = (_e99 * 8u);
        let _e148 = JB.j2_[(_e144 + 2u)];
        let _e159 = JB.j2_[(_e144 + 3u)];
        let _e164 = _e159.zw;
        let _e166 = ((abs(((mat2x2<f32>(vec2<f32>(_e148.x, _e148.y), vec2<f32>(_e148.z, _e148.w)) * _e56) + _e159.xy)) * _e164) - _e164);
        phi_1085_ = min(_e139, clamp((min(_e166.x, _e166.y) + 0.5f), 0f, 1f));
    }
    let _e174 = phi_1085_;
    let _e175 = (_e102.x & 15u);
    if (_e175 <= 1u) {
        phi_1088_ = select(unpack4x8unorm(_e102.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((bi && (_e175 == 0u))));
    } else {
        let _e188 = (_e99 * 8u);
        let _e191 = JB.j2_[_e188];
        let _e202 = JB.j2_[(_e188 + 1u)];
        let _e205 = ((mat2x2<f32>(vec2<f32>(_e191.x, _e191.y), vec2<f32>(_e191.z, _e191.w)) * _e56) + _e202.xy);
        if (_e175 == 2u) {
            phi_1073_ = _e205.x;
        } else {
            phi_1073_ = length(_e205);
        }
        let _e210 = phi_1073_;
        let _e217 = bitcast<f32>(_e102.y);
        let _e220 = j.wc;
        let _e223 = j.xc;
        let _e226 = textureSampleLevel(FD, ha, vec2<f32>(((clamp(_e210, 0f, 1f) * _e202.z) + _e202.w), ((floor(_e217) * _e220) + _e223)), 0f);
        phi_1090_ = _e226;
        if !((di && (((_e102.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e230 = (_e226.xyz * _e226.w);
            phi_1090_ = vec4<f32>(_e230.x, _e230.y, _e230.z, (_e226.w * (fract(_e217) * 1.0039216f)));
        }
        let _e239 = phi_1090_;
        phi_1088_ = _e239;
    }
    let _e241 = phi_1088_;
    let _e242 = (_e241 * _e174);
    let _e243 = _e242.xyz;
    let _e246 = j.M3_;
    let _e248 = j.N3_;
    if (ii && (_e242.w != 0f)) {
        phi_1091_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e55.x) + (0.00583715f * _e55.y))))) * _e246) + _e248)) + _e243);
    } else {
        phi_1091_ = _e243;
    }
    let _e264 = phi_1091_;
    let _e270 = vec4<f32>(_e264.x, _e242.y, _e242.z, _e242.w);
    let _e276 = vec4<f32>(_e270.x, _e264.y, _e270.z, _e270.w);
    J1_ = vec4<f32>(_e276.x, _e276.y, _e264.z, _e276.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = J1_;
    return _e3;
}
