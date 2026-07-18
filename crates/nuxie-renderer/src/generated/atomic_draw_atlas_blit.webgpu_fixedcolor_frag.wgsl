struct Fe {
    c2_: array<vec2<u32>>,
}

struct h0xd {
    c2_: array<u32>,
}

struct Ge {
    c2_: array<vec4<f32>>,
}

struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct q4xd {
    c2_: array<u32>,
}

@id(7) override bh: bool = true;
@id(4) override Yg: bool = true;
@id(0) override Ug: bool = true;
@id(1) override Vg: bool = true;

@group(0) @binding(3)
var<storage> AD: Fe;
@group(2) @binding(1)
var<storage, read_write> h0_: h0xd;
@group(0) @binding(4)
var<storage> RB: Ge;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Jb: sampler;
@group(0) @binding(0)
var<uniform> m: CC;
@group(2) @binding(3)
var<storage, read_write> q4_: q4xd;
var<private> A0_1: u32;
@group(0) @binding(10)
var BD: texture_2d<f32>;
@group(3) @binding(10)
var P9_: sampler;
var<private> B2_1: vec2<f32>;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;

fn main_1() {
    var phi_788_: bool;
    var phi_1126_: f32;
    var phi_1125_: f32;
    var phi_1127_: f32;
    var phi_1130_: f32;
    var phi_1129_: f32;
    var phi_825_: bool;
    var phi_1132_: f32;
    var phi_1156_: u32;
    var phi_1131_: f32;
    var phi_1155_: u32;
    var phi_1153_: vec4<f32>;
    var phi_1166_: vec3<f32>;

    let _e57 = gl_FragCoord_1;
    let _e58 = _e57.xy;
    let _e61 = bitcast<vec2<u32>>(vec2<i32>(floor(_e58)));
    let _e63 = m.m6_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e95 = q4_.c2_[_e92];
    let _e97 = (_e95 >> bitcast<u32>(17u));
    let _e98 = A0_1;
    let _e102 = B2_1;
    let _e103 = textureSampleLevel(BD, P9_, _e102, 0f);
    q4_.c2_[_e92] = (((_e98 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e103.x, 0f, 1f) * 2048f)))));
    let _e114 = ((f32((_e95 & 131071u)) * 0.00048828125f) + -32f);
    let _e117 = AD.c2_[_e97];
    phi_1125_ = _e114;
    if ((_e117.x & 768u) != 0u) {
        let _e121 = abs(_e114);
        phi_788_ = Yg;
        if Yg {
            phi_788_ = ((_e117.x & 512u) != 0u);
        }
        let _e125 = phi_788_;
        phi_1126_ = _e121;
        if _e125 {
            phi_1126_ = (1f - abs(((fract((_e121 * 0.5f)) * 2f) + -1f)));
        }
        let _e133 = phi_1126_;
        phi_1125_ = _e133;
    }
    let _e135 = phi_1125_;
    let _e136 = clamp(_e135, 0f, 1f);
    phi_1129_ = _e136;
    if Ug {
        let _e138 = (_e117.x >> bitcast<u32>(16u));
        phi_1130_ = _e136;
        if (_e138 != 0u) {
            let _e142 = h0_.c2_[_e92];
            if (_e138 == (_e142 >> bitcast<u32>(16i))) {
                phi_1127_ = min(_e136, unpack2x16float(_e142).x);
            } else {
                phi_1127_ = 0f;
            }
            let _e150 = phi_1127_;
            phi_1130_ = _e150;
        }
        let _e152 = phi_1130_;
        phi_1129_ = _e152;
    }
    let _e154 = phi_1129_;
    phi_825_ = Vg;
    if Vg {
        phi_825_ = ((_e117.x & 1024u) != 0u);
    }
    let _e158 = phi_825_;
    phi_1132_ = _e154;
    if _e158 {
        let _e159 = (_e97 * 4u);
        let _e163 = RB.c2_[(_e159 + 2u)];
        let _e174 = RB.c2_[(_e159 + 3u)];
        let _e179 = _e174.zw;
        let _e181 = ((abs(((mat2x2<f32>(vec2<f32>(_e163.x, _e163.y), vec2<f32>(_e163.z, _e163.w)) * _e58) + _e174.xy)) * _e179) - _e179);
        phi_1132_ = min(_e154, clamp((min(_e181.x, _e181.y) + 0.5f), 0f, 1f));
    }
    let _e189 = phi_1132_;
    let _e190 = (_e117.x & 15u);
    if (_e190 <= 1u) {
        let _e195 = (Ug && (_e190 == 0u));
        phi_1156_ = 0u;
        if _e195 {
            phi_1156_ = (_e117.y | pack2x16float(vec2<f32>(_e189, 0f)));
        }
        let _e200 = phi_1156_;
        phi_1155_ = _e200;
        phi_1153_ = select(unpack4x8unorm(_e117.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e195));
    } else {
        let _e203 = (_e97 * 4u);
        let _e206 = RB.c2_[_e203];
        let _e217 = RB.c2_[(_e203 + 1u)];
        let _e220 = ((mat2x2<f32>(vec2<f32>(_e206.x, _e206.y), vec2<f32>(_e206.z, _e206.w)) * _e58) + _e217.xy);
        if (_e190 == 2u) {
            phi_1131_ = _e220.x;
        } else {
            phi_1131_ = length(_e220);
        }
        let _e225 = phi_1131_;
        let _e234 = textureSampleLevel(KD, Jb, vec2<f32>(((clamp(_e225, 0f, 1f) * _e217.z) + _e217.w), bitcast<f32>(_e117.y)), 0f);
        phi_1155_ = 0u;
        phi_1153_ = _e234;
    }
    let _e236 = phi_1155_;
    let _e238 = phi_1153_;
    let _e240 = (_e238.w * _e189);
    let _e242 = (_e238.xyz * _e240);
    let _e246 = vec4<f32>(_e242.x, _e242.y, _e242.z, _e240);
    let _e247 = _e246.xyz;
    let _e249 = m.y3_;
    let _e251 = m.z3_;
    if bh {
        phi_1166_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e249) + _e251)) + _e247);
    } else {
        phi_1166_ = _e247;
    }
    let _e265 = phi_1166_;
    let _e271 = vec4<f32>(_e265.x, _e246.y, _e246.z, _e246.w);
    let _e277 = vec4<f32>(_e271.x, _e265.y, _e271.z, _e271.w);
    C1_ = vec4<f32>(_e277.x, _e277.y, _e265.z, _e277.w);
    if (_e236 != 0u) {
        h0_.c2_[_e92] = _e236;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat) A0_: u32, @location(0) B2_: vec2<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    A0_1 = A0_;
    B2_1 = B2_;
    main_1();
    let _e7 = C1_;
    return _e7;
}
