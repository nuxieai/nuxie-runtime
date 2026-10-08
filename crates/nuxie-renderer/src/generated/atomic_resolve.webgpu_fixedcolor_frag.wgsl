struct gg {
    v2_: array<vec2<u32>>,
}

struct m0Me {
    v2_: array<u32>,
}

struct hg {
    v2_: array<vec4<f32>>,
}

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

struct P4Me {
    v2_: array<u32>,
}

@id(7) override bj: bool = true;
@id(4) override Yi: bool = true;
@id(0) override Ui: bool = true;
@id(1) override Vi: bool = true;
@id(2) override Wi: bool = true;

@group(0) @binding(3)
var<storage> WC: gg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Me;
@group(0) @binding(4)
var<storage> JB: hg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var H8_: sampler;
@group(2) @binding(3)
var<storage, read_write> P4_: P4Me;
var<private> N1_: vec4<f32>;
@group(3) @binding(9)
var Ta: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;

fn main_1() {
    var phi_725_: bool;
    var phi_1069_: f32;
    var phi_1068_: f32;
    var phi_1070_: f32;
    var phi_1073_: f32;
    var phi_1072_: f32;
    var phi_762_: bool;
    var phi_1086_: f32;
    var phi_1074_: f32;
    var phi_1091_: vec4<f32>;
    var phi_1089_: vec4<f32>;
    var phi_1092_: vec3<f32>;

    let _e55 = gl_FragCoord_1;
    let _e56 = _e55.xy;
    let _e59 = bitcast<vec2<u32>>(vec2<i32>(floor(_e56)));
    let _e61 = j.L6_;
    let _e90 = bitcast<i32>((((((_e59.y >> bitcast<u32>(5u)) * (((_e61 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e59.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e59.x & 28u) << bitcast<u32>(5u)) + ((_e59.y & 28u) << bitcast<u32>(2i)))) + (((_e59.y & 3u) << bitcast<u32>(2i)) + (_e59.x & 3u))));
    let _e93 = P4_.v2_[_e90];
    let _e97 = ((f32((_e93 & 131071u)) * 0.00048828125f) + -32f);
    let _e99 = (_e93 >> bitcast<u32>(17u));
    let _e102 = WC.v2_[_e99];
    phi_1068_ = _e97;
    if ((_e102.x & 768u) != 0u) {
        let _e106 = abs(_e97);
        phi_725_ = Yi;
        if Yi {
            phi_725_ = ((_e102.x & 512u) != 0u);
        }
        let _e110 = phi_725_;
        phi_1069_ = _e106;
        if _e110 {
            phi_1069_ = (1f - abs(((fract((_e106 * 0.5f)) * 2f) + -1f)));
        }
        let _e118 = phi_1069_;
        phi_1068_ = _e118;
    }
    let _e120 = phi_1068_;
    let _e121 = clamp(_e120, 0f, 1f);
    phi_1072_ = _e121;
    if Ui {
        let _e123 = (_e102.x >> bitcast<u32>(16u));
        phi_1073_ = _e121;
        if (_e123 != 0u) {
            let _e127 = m0_.v2_[_e90];
            if (_e123 == (_e127 >> bitcast<u32>(16i))) {
                phi_1070_ = min(_e121, unpack2x16float(_e127).x);
            } else {
                phi_1070_ = 0f;
            }
            let _e135 = phi_1070_;
            phi_1073_ = _e135;
        }
        let _e137 = phi_1073_;
        phi_1072_ = _e137;
    }
    let _e139 = phi_1072_;
    phi_762_ = Vi;
    if Vi {
        phi_762_ = ((_e102.x & 1024u) != 0u);
    }
    let _e143 = phi_762_;
    phi_1086_ = _e139;
    if _e143 {
        let _e144 = (_e99 * 8u);
        let _e148 = JB.v2_[(_e144 + 2u)];
        let _e159 = JB.v2_[(_e144 + 3u)];
        let _e164 = _e159.zw;
        let _e166 = ((abs(((mat2x2<f32>(vec2<f32>(_e148.x, _e148.y), vec2<f32>(_e148.z, _e148.w)) * _e56) + _e159.xy)) * _e164) - _e164);
        phi_1086_ = min(_e139, clamp((min(_e166.x, _e166.y) + 0.5f), 0f, 1f));
    }
    let _e174 = phi_1086_;
    let _e175 = (_e102.x & 15u);
    if (_e175 <= 1u) {
        phi_1089_ = select(unpack4x8unorm(_e102.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((Ui && (_e175 == 0u))));
    } else {
        let _e188 = (_e99 * 8u);
        let _e191 = JB.v2_[_e188];
        let _e202 = JB.v2_[(_e188 + 1u)];
        let _e205 = ((mat2x2<f32>(vec2<f32>(_e191.x, _e191.y), vec2<f32>(_e191.z, _e191.w)) * _e56) + _e202.xy);
        if (_e175 == 2u) {
            phi_1074_ = _e205.x;
        } else {
            phi_1074_ = length(_e205);
        }
        let _e210 = phi_1074_;
        let _e217 = bitcast<f32>(_e102.y);
        let _e220 = j.ad;
        let _e223 = j.g7_;
        let _e226 = textureSampleLevel(YC, H8_, vec2<f32>(((clamp(_e210, 0f, 1f) * _e202.z) + _e202.w), ((floor(_e217) * _e220) + _e223)), 0f);
        phi_1091_ = _e226;
        if !((Wi && (((_e102.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e230 = (_e226.xyz * _e226.w);
            phi_1091_ = vec4<f32>(_e230.x, _e230.y, _e230.z, (_e226.w * (fract(_e217) * 1.0039216f)));
        }
        let _e239 = phi_1091_;
        phi_1089_ = _e239;
    }
    let _e241 = phi_1089_;
    let _e242 = (_e241 * _e174);
    let _e243 = _e242.xyz;
    let _e246 = j.E3_;
    let _e248 = j.F3_;
    if (bj && (_e242.w != 0f)) {
        phi_1092_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e55.x) + (0.00583715f * _e55.y))))) * _e246) + _e248)) + _e243);
    } else {
        phi_1092_ = _e243;
    }
    let _e264 = phi_1092_;
    let _e270 = vec4<f32>(_e264.x, _e242.y, _e242.z, _e242.w);
    let _e276 = vec4<f32>(_e270.x, _e264.y, _e270.z, _e270.w);
    N1_ = vec4<f32>(_e276.x, _e276.y, _e264.z, _e276.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = N1_;
    return _e3;
}
