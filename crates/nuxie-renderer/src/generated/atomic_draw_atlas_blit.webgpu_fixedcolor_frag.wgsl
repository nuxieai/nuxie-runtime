struct jf {
    e2_: array<vec2<u32>>,
}

struct h0Rd {
    e2_: array<u32>,
}

struct kf {
    e2_: array<vec4<f32>>,
}

struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct x4Rd {
    e2_: array<u32>,
}

@id(7) override Jh: bool = true;
@id(4) override Gh: bool = true;
@id(0) override Ch: bool = true;
@id(1) override Dh: bool = true;

@group(0) @binding(3)
var<storage> DD: jf;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Rd;
@group(0) @binding(4)
var<storage> QB: kf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(0) @binding(0)
var<uniform> l: BC;
@group(2) @binding(3)
var<storage, read_write> x4_: x4Rd;
var<private> C0_1: u32;
@group(0) @binding(10)
var FD: texture_2d<f32>;
@group(3) @binding(10)
var R9_: sampler;
var<private> F2_1: vec2<f32>;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var X5_: sampler;

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
    let _e63 = l.q6_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e95 = x4_.e2_[_e92];
    let _e97 = (_e95 >> bitcast<u32>(17u));
    let _e98 = C0_1;
    let _e102 = F2_1;
    let _e103 = textureSampleLevel(FD, R9_, _e102, 0f);
    x4_.e2_[_e92] = (((_e98 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e103.x, 0f, 1f) * 2048f)))));
    let _e114 = ((f32((_e95 & 131071u)) * 0.00048828125f) + -32f);
    let _e117 = DD.e2_[_e97];
    phi_1138_ = _e114;
    if ((_e117.x & 768u) != 0u) {
        let _e121 = abs(_e114);
        phi_797_ = Gh;
        if Gh {
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
    if Ch {
        let _e138 = (_e117.x >> bitcast<u32>(16u));
        phi_1143_ = _e136;
        if (_e138 != 0u) {
            let _e142 = h0_.e2_[_e92];
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
    phi_834_ = Dh;
    if Dh {
        phi_834_ = ((_e117.x & 1024u) != 0u);
    }
    let _e158 = phi_834_;
    phi_1145_ = _e154;
    if _e158 {
        let _e159 = (_e97 * 8u);
        let _e163 = QB.e2_[(_e159 + 2u)];
        let _e174 = QB.e2_[(_e159 + 3u)];
        let _e179 = _e174.zw;
        let _e181 = ((abs(((mat2x2<f32>(vec2<f32>(_e163.x, _e163.y), vec2<f32>(_e163.z, _e163.w)) * _e58) + _e174.xy)) * _e179) - _e179);
        phi_1145_ = min(_e154, clamp((min(_e181.x, _e181.y) + 0.5f), 0f, 1f));
    }
    let _e189 = phi_1145_;
    let _e190 = (_e117.x & 15u);
    if (_e190 <= 1u) {
        let _e195 = (Ch && (_e190 == 0u));
        phi_1169_ = 0u;
        if _e195 {
            phi_1169_ = (_e117.y | pack2x16float(vec2<f32>(_e189, 0f)));
        }
        let _e200 = phi_1169_;
        phi_1168_ = _e200;
        phi_1166_ = select(unpack4x8unorm(_e117.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e195));
    } else {
        let _e203 = (_e97 * 8u);
        let _e206 = QB.e2_[_e203];
        let _e217 = QB.e2_[(_e203 + 1u)];
        let _e220 = ((mat2x2<f32>(vec2<f32>(_e206.x, _e206.y), vec2<f32>(_e206.z, _e206.w)) * _e58) + _e217.xy);
        if (_e190 == 2u) {
            phi_1144_ = _e220.x;
        } else {
            phi_1144_ = length(_e220);
        }
        let _e225 = phi_1144_;
        let _e234 = textureSampleLevel(ED, N9_, vec2<f32>(((clamp(_e225, 0f, 1f) * _e217.z) + _e217.w), bitcast<f32>(_e117.y)), 0f);
        phi_1168_ = 0u;
        phi_1166_ = _e234;
    }
    let _e236 = phi_1168_;
    let _e238 = phi_1166_;
    let _e240 = (_e238.w * _e189);
    let _e242 = (_e238.xyz * _e240);
    let _e246 = vec4<f32>(_e242.x, _e242.y, _e242.z, _e240);
    let _e247 = _e246.xyz;
    let _e249 = l.C3_;
    let _e251 = l.D3_;
    if (Jh && (_e240 != 0f)) {
        phi_1179_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e249) + _e251)) + _e247);
    } else {
        phi_1179_ = _e247;
    }
    let _e267 = phi_1179_;
    let _e273 = vec4<f32>(_e267.x, _e246.y, _e246.z, _e246.w);
    let _e279 = vec4<f32>(_e273.x, _e267.y, _e273.z, _e273.w);
    C1_ = vec4<f32>(_e279.x, _e279.y, _e267.z, _e279.w);
    if (_e236 != 0u) {
        h0_.e2_[_e92] = _e236;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) C0_: u32, @location(0) F2_: vec2<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    C0_1 = C0_;
    F2_1 = F2_;
    main_1();
    let _e7 = C1_;
    return _e7;
}
