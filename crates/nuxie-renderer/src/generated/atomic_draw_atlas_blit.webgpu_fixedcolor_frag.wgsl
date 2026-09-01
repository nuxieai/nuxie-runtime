struct Me {
    c2_: array<vec2<u32>>,
}

struct h0Dd {
    c2_: array<u32>,
}

struct Ne {
    c2_: array<vec4<f32>>,
}

struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

struct v4Dd {
    c2_: array<u32>,
}

@id(7) override lh: bool = true;
@id(4) override ih: bool = true;
@id(0) override eh: bool = true;
@id(1) override fh: bool = true;

@group(0) @binding(3)
var<storage> BD: Me;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Dd;
@group(0) @binding(4)
var<storage> RB: Ne;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(3) @binding(8)
var Ob: sampler;
@group(0) @binding(0)
var<uniform> m: DC;
@group(2) @binding(3)
var<storage, read_write> v4_: v4Dd;
var<private> B0_1: u32;
@group(0) @binding(10)
var CD: texture_2d<f32>;
@group(3) @binding(10)
var Q9_: sampler;
var<private> D2_1: vec2<f32>;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var JC: texture_2d<f32>;
@group(1) @binding(13)
var U5_: sampler;

fn main_1() {
    var phi_797_: bool;
    var phi_1139_: f32;
    var phi_1138_: f32;
    var phi_1140_: f32;
    var phi_1143_: f32;
    var phi_1142_: f32;
    var phi_834_: bool;
    var phi_1145_: f32;
    var phi_1169_: u32;
    var phi_1144_: f32;
    var phi_1168_: u32;
    var phi_1166_: vec4<f32>;
    var phi_1179_: vec3<f32>;

    let _e57 = gl_FragCoord_1;
    let _e58 = _e57.xy;
    let _e61 = bitcast<vec2<u32>>(vec2<i32>(floor(_e58)));
    let _e63 = m.o6_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e95 = v4_.c2_[_e92];
    let _e97 = (_e95 >> bitcast<u32>(17u));
    let _e98 = B0_1;
    let _e102 = D2_1;
    let _e103 = textureSampleLevel(CD, Q9_, _e102, 0f);
    v4_.c2_[_e92] = (((_e98 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e103.x, 0f, 1f) * 2048f)))));
    let _e114 = ((f32((_e95 & 131071u)) * 0.00048828125f) + -32f);
    let _e117 = BD.c2_[_e97];
    phi_1138_ = _e114;
    if ((_e117.x & 768u) != 0u) {
        let _e121 = abs(_e114);
        phi_797_ = ih;
        if ih {
            phi_797_ = ((_e117.x & 512u) != 0u);
        }
        let _e125 = phi_797_;
        phi_1139_ = _e121;
        if _e125 {
            phi_1139_ = (1f - abs(((fract((_e121 * 0.5f)) * 2f) + -1f)));
        }
        let _e133 = phi_1139_;
        phi_1138_ = _e133;
    }
    let _e135 = phi_1138_;
    let _e136 = clamp(_e135, 0f, 1f);
    phi_1142_ = _e136;
    if eh {
        let _e138 = (_e117.x >> bitcast<u32>(16u));
        phi_1143_ = _e136;
        if (_e138 != 0u) {
            let _e142 = h0_.c2_[_e92];
            if (_e138 == (_e142 >> bitcast<u32>(16i))) {
                phi_1140_ = min(_e136, unpack2x16float(_e142).x);
            } else {
                phi_1140_ = 0f;
            }
            let _e150 = phi_1140_;
            phi_1143_ = _e150;
        }
        let _e152 = phi_1143_;
        phi_1142_ = _e152;
    }
    let _e154 = phi_1142_;
    phi_834_ = fh;
    if fh {
        phi_834_ = ((_e117.x & 1024u) != 0u);
    }
    let _e158 = phi_834_;
    phi_1145_ = _e154;
    if _e158 {
        let _e159 = (_e97 * 8u);
        let _e163 = RB.c2_[(_e159 + 2u)];
        let _e174 = RB.c2_[(_e159 + 3u)];
        let _e179 = _e174.zw;
        let _e181 = ((abs(((mat2x2<f32>(vec2<f32>(_e163.x, _e163.y), vec2<f32>(_e163.z, _e163.w)) * _e58) + _e174.xy)) * _e179) - _e179);
        phi_1145_ = min(_e154, clamp((min(_e181.x, _e181.y) + 0.5f), 0f, 1f));
    }
    let _e189 = phi_1145_;
    let _e190 = (_e117.x & 15u);
    if (_e190 <= 1u) {
        let _e195 = (eh && (_e190 == 0u));
        phi_1169_ = 0u;
        if _e195 {
            phi_1169_ = (_e117.y | pack2x16float(vec2<f32>(_e189, 0f)));
        }
        let _e200 = phi_1169_;
        phi_1168_ = _e200;
        phi_1166_ = select(unpack4x8unorm(_e117.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e195));
    } else {
        let _e203 = (_e97 * 8u);
        let _e206 = RB.c2_[_e203];
        let _e217 = RB.c2_[(_e203 + 1u)];
        let _e220 = ((mat2x2<f32>(vec2<f32>(_e206.x, _e206.y), vec2<f32>(_e206.z, _e206.w)) * _e58) + _e217.xy);
        if (_e190 == 2u) {
            phi_1144_ = _e220.x;
        } else {
            phi_1144_ = length(_e220);
        }
        let _e225 = phi_1144_;
        let _e234 = textureSampleLevel(MD, Ob, vec2<f32>(((clamp(_e225, 0f, 1f) * _e217.z) + _e217.w), bitcast<f32>(_e117.y)), 0f);
        phi_1168_ = 0u;
        phi_1166_ = _e234;
    }
    let _e236 = phi_1168_;
    let _e238 = phi_1166_;
    let _e240 = (_e238.w * _e189);
    let _e242 = (_e238.xyz * _e240);
    let _e246 = vec4<f32>(_e242.x, _e242.y, _e242.z, _e240);
    let _e247 = _e246.xyz;
    let _e249 = m.B3_;
    let _e251 = m.C3_;
    if (lh && (_e240 != 0f)) {
        phi_1179_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e249) + _e251)) + _e247);
    } else {
        phi_1179_ = _e247;
    }
    let _e267 = phi_1179_;
    let _e273 = vec4<f32>(_e267.x, _e246.y, _e246.z, _e246.w);
    let _e279 = vec4<f32>(_e273.x, _e267.y, _e273.z, _e273.w);
    C1_ = vec4<f32>(_e279.x, _e279.y, _e267.z, _e279.w);
    if (_e236 != 0u) {
        h0_.c2_[_e92] = _e236;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) B0_: u32, @location(0) D2_: vec2<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    B0_1 = B0_;
    D2_1 = D2_;
    main_1();
    let _e7 = C1_;
    return _e7;
}
