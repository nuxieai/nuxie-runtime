struct Re {
    d2_: array<vec2<u32>>,
}

struct g0Id {
    d2_: array<u32>,
}

struct Se {
    d2_: array<vec4<f32>>,
}

struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
}

struct w4Id {
    d2_: array<u32>,
}

@id(7) override qh: bool = true;
@id(4) override nh: bool = true;
@id(0) override jh: bool = true;
@id(1) override kh: bool = true;

@group(0) @binding(3)
var<storage> AD: Re;
@group(2) @binding(1)
var<storage, read_write> g0_: g0Id;
@group(0) @binding(4)
var<storage> QB: Se;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var MD: texture_2d<f32>;
@group(3) @binding(8)
var Rb: sampler;
@group(0) @binding(0)
var<uniform> m: BC;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var V5_: sampler;
var<private> Y1_1: vec2<f32>;
var<private> V4_1: f32;
var<private> M0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> w4_: w4Id;
var<private> w3_1: u32;
var<private> I1_1: f32;
var<private> D1_: vec4<f32>;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> B1_1: u32;

fn main_1() {
    var phi_1254_: f32;
    var phi_852_: bool;
    var phi_1201_: f32;
    var phi_1200_: f32;
    var phi_1202_: f32;
    var phi_1205_: f32;
    var phi_1204_: f32;
    var phi_889_: bool;
    var phi_1207_: f32;
    var phi_1234_: u32;
    var phi_1206_: f32;
    var phi_1233_: u32;
    var phi_1231_: vec4<f32>;
    var phi_640_: bool;
    var phi_1245_: u32;
    var phi_1260_: f32;
    var phi_1261_: f32;
    var phi_1282_: vec3<f32>;

    let _e58 = gl_FragCoord_1;
    let _e59 = _e58.xy;
    let _e62 = bitcast<vec2<u32>>(vec2<i32>(floor(_e59)));
    let _e64 = m.p6_;
    let _e93 = bitcast<i32>((((((_e62.y >> bitcast<u32>(5u)) * (((_e64 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e62.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e62.x & 28u) << bitcast<u32>(5u)) + ((_e62.y & 28u) << bitcast<u32>(2i)))) + (((_e62.y & 3u) << bitcast<u32>(2i)) + (_e62.x & 3u))));
    let _e94 = Y1_1;
    let _e95 = textureSample(HC, V5_, _e94);
    let _e96 = V4_1;
    let _e97 = min(_e96, 1f);
    phi_1254_ = _e97;
    if kh {
        let _e98 = M0_1;
        let _e101 = min(_e98.xy, _e98.zw);
        phi_1254_ = clamp(min(_e101.x, _e101.y), 0f, _e97);
    }
    let _e107 = phi_1254_;
    let _e110 = w4_.d2_[_e93];
    let _e112 = (_e110 >> bitcast<u32>(17u));
    let _e116 = ((f32((_e110 & 131071u)) * 0.00048828125f) + -32f);
    let _e119 = AD.d2_[_e112];
    phi_1200_ = _e116;
    if ((_e119.x & 768u) != 0u) {
        let _e123 = abs(_e116);
        phi_852_ = nh;
        if nh {
            phi_852_ = ((_e119.x & 512u) != 0u);
        }
        let _e127 = phi_852_;
        phi_1201_ = _e123;
        if _e127 {
            phi_1201_ = (1f - abs(((fract((_e123 * 0.5f)) * 2f) + -1f)));
        }
        let _e135 = phi_1201_;
        phi_1200_ = _e135;
    }
    let _e137 = phi_1200_;
    let _e138 = clamp(_e137, 0f, 1f);
    phi_1204_ = _e138;
    if jh {
        let _e140 = (_e119.x >> bitcast<u32>(16u));
        phi_1205_ = _e138;
        if (_e140 != 0u) {
            let _e144 = g0_.d2_[_e93];
            if (_e140 == (_e144 >> bitcast<u32>(16i))) {
                phi_1202_ = min(_e138, unpack2x16float(_e144).x);
            } else {
                phi_1202_ = 0f;
            }
            let _e152 = phi_1202_;
            phi_1205_ = _e152;
        }
        let _e154 = phi_1205_;
        phi_1204_ = _e154;
    }
    let _e156 = phi_1204_;
    phi_889_ = kh;
    if kh {
        phi_889_ = ((_e119.x & 1024u) != 0u);
    }
    let _e160 = phi_889_;
    phi_1207_ = _e156;
    if _e160 {
        let _e161 = (_e112 * 8u);
        let _e165 = QB.d2_[(_e161 + 2u)];
        let _e176 = QB.d2_[(_e161 + 3u)];
        let _e181 = _e176.zw;
        let _e183 = ((abs(((mat2x2<f32>(vec2<f32>(_e165.x, _e165.y), vec2<f32>(_e165.z, _e165.w)) * _e59) + _e176.xy)) * _e181) - _e181);
        phi_1207_ = min(_e156, clamp((min(_e183.x, _e183.y) + 0.5f), 0f, 1f));
    }
    let _e191 = phi_1207_;
    let _e192 = (_e119.x & 15u);
    if (_e192 <= 1u) {
        let _e197 = (jh && (_e192 == 0u));
        phi_1234_ = 0u;
        if _e197 {
            phi_1234_ = (_e119.y | pack2x16float(vec2<f32>(_e191, 0f)));
        }
        let _e202 = phi_1234_;
        phi_1233_ = _e202;
        phi_1231_ = select(unpack4x8unorm(_e119.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e197));
    } else {
        let _e205 = (_e112 * 8u);
        let _e208 = QB.d2_[_e205];
        let _e219 = QB.d2_[(_e205 + 1u)];
        let _e222 = ((mat2x2<f32>(vec2<f32>(_e208.x, _e208.y), vec2<f32>(_e208.z, _e208.w)) * _e59) + _e219.xy);
        if (_e192 == 2u) {
            phi_1206_ = _e222.x;
        } else {
            phi_1206_ = length(_e222);
        }
        let _e227 = phi_1206_;
        let _e236 = textureSampleLevel(MD, Rb, vec2<f32>(((clamp(_e227, 0f, 1f) * _e219.z) + _e219.w), bitcast<f32>(_e119.y)), 0f);
        phi_1233_ = 0u;
        phi_1231_ = _e236;
    }
    let _e238 = phi_1233_;
    let _e240 = phi_1231_;
    let _e242 = (_e240.w * _e191);
    let _e244 = (_e240.xyz * _e242);
    phi_640_ = jh;
    if jh {
        let _e249 = w3_1;
        phi_640_ = (_e249 != 0u);
    }
    let _e252 = phi_640_;
    phi_1261_ = _e107;
    if _e252 {
        if (_e238 != 0u) {
            phi_1245_ = _e238;
        } else {
            let _e256 = g0_.d2_[_e93];
            phi_1245_ = _e256;
        }
        let _e258 = phi_1245_;
        let _e259 = w3_1;
        if (_e259 == (_e258 >> bitcast<u32>(16i))) {
            phi_1260_ = min(_e107, unpack2x16float(_e258).x);
        } else {
            phi_1260_ = 0f;
        }
        let _e267 = phi_1260_;
        phi_1261_ = _e267;
    }
    let _e269 = phi_1261_;
    let _e270 = I1_1;
    let _e272 = (_e95 * (_e269 * _e270));
    let _e276 = ((vec4<f32>(_e244.x, _e244.y, _e244.z, _e242) * (1f - _e272.w)) + _e272);
    let _e277 = _e276.xyz;
    let _e280 = m.A3_;
    let _e282 = m.B3_;
    if (qh && (_e276.w != 0f)) {
        phi_1282_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e58.x) + (0.00583715f * _e58.y))))) * _e280) + _e282)) + _e277);
    } else {
        phi_1282_ = _e277;
    }
    let _e298 = phi_1282_;
    let _e304 = vec4<f32>(_e298.x, _e276.y, _e276.z, _e276.w);
    let _e310 = vec4<f32>(_e304.x, _e298.y, _e304.z, _e304.w);
    D1_ = vec4<f32>(_e310.x, _e310.y, _e298.z, _e310.w);
    if (_e238 != 0u) {
        g0_.d2_[_e93] = _e238;
    }
    w4_.d2_[_e93] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) Y1_: vec2<f32>, @location(1) V4_: f32, @location(2) M0_: vec4<f32>, @location(4) @interpolate(flat, either) w3_: u32, @location(3) @interpolate(flat, either) I1_: f32, @location(5) @interpolate(flat, either) B1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    Y1_1 = Y1_;
    V4_1 = V4_;
    M0_1 = M0_;
    w3_1 = w3_;
    I1_1 = I1_;
    B1_1 = B1_;
    main_1();
    let _e15 = D1_;
    return _e15;
}
