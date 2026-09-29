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
    var phi_1250_: f32;
    var phi_848_: bool;
    var phi_1197_: f32;
    var phi_1196_: f32;
    var phi_1198_: f32;
    var phi_1201_: f32;
    var phi_1200_: f32;
    var phi_885_: bool;
    var phi_1203_: f32;
    var phi_1230_: u32;
    var phi_1202_: f32;
    var phi_1229_: u32;
    var phi_1227_: vec4<f32>;
    var phi_635_: bool;
    var phi_1241_: u32;
    var phi_1256_: f32;
    var phi_1257_: f32;
    var phi_1278_: vec3<f32>;

    let _e57 = gl_FragCoord_1;
    let _e58 = _e57.xy;
    let _e61 = bitcast<vec2<u32>>(vec2<i32>(floor(_e58)));
    let _e63 = m.p6_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e93 = Y1_1;
    let _e94 = textureSample(HC, V5_, _e93);
    phi_1250_ = 1f;
    if kh {
        let _e95 = M0_1;
        let _e98 = min(_e95.xy, _e95.zw);
        phi_1250_ = clamp(min(_e98.x, _e98.y), 0f, 1f);
    }
    let _e104 = phi_1250_;
    let _e107 = w4_.d2_[_e92];
    let _e109 = (_e107 >> bitcast<u32>(17u));
    let _e113 = ((f32((_e107 & 131071u)) * 0.00048828125f) + -32f);
    let _e116 = AD.d2_[_e109];
    phi_1196_ = _e113;
    if ((_e116.x & 768u) != 0u) {
        let _e120 = abs(_e113);
        phi_848_ = nh;
        if nh {
            phi_848_ = ((_e116.x & 512u) != 0u);
        }
        let _e124 = phi_848_;
        phi_1197_ = _e120;
        if _e124 {
            phi_1197_ = (1f - abs(((fract((_e120 * 0.5f)) * 2f) + -1f)));
        }
        let _e132 = phi_1197_;
        phi_1196_ = _e132;
    }
    let _e134 = phi_1196_;
    let _e135 = clamp(_e134, 0f, 1f);
    phi_1200_ = _e135;
    if jh {
        let _e137 = (_e116.x >> bitcast<u32>(16u));
        phi_1201_ = _e135;
        if (_e137 != 0u) {
            let _e141 = g0_.d2_[_e92];
            if (_e137 == (_e141 >> bitcast<u32>(16i))) {
                phi_1198_ = min(_e135, unpack2x16float(_e141).x);
            } else {
                phi_1198_ = 0f;
            }
            let _e149 = phi_1198_;
            phi_1201_ = _e149;
        }
        let _e151 = phi_1201_;
        phi_1200_ = _e151;
    }
    let _e153 = phi_1200_;
    phi_885_ = kh;
    if kh {
        phi_885_ = ((_e116.x & 1024u) != 0u);
    }
    let _e157 = phi_885_;
    phi_1203_ = _e153;
    if _e157 {
        let _e158 = (_e109 * 8u);
        let _e162 = QB.d2_[(_e158 + 2u)];
        let _e173 = QB.d2_[(_e158 + 3u)];
        let _e178 = _e173.zw;
        let _e180 = ((abs(((mat2x2<f32>(vec2<f32>(_e162.x, _e162.y), vec2<f32>(_e162.z, _e162.w)) * _e58) + _e173.xy)) * _e178) - _e178);
        phi_1203_ = min(_e153, clamp((min(_e180.x, _e180.y) + 0.5f), 0f, 1f));
    }
    let _e188 = phi_1203_;
    let _e189 = (_e116.x & 15u);
    if (_e189 <= 1u) {
        let _e194 = (jh && (_e189 == 0u));
        phi_1230_ = 0u;
        if _e194 {
            phi_1230_ = (_e116.y | pack2x16float(vec2<f32>(_e188, 0f)));
        }
        let _e199 = phi_1230_;
        phi_1229_ = _e199;
        phi_1227_ = select(unpack4x8unorm(_e116.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e194));
    } else {
        let _e202 = (_e109 * 8u);
        let _e205 = QB.d2_[_e202];
        let _e216 = QB.d2_[(_e202 + 1u)];
        let _e219 = ((mat2x2<f32>(vec2<f32>(_e205.x, _e205.y), vec2<f32>(_e205.z, _e205.w)) * _e58) + _e216.xy);
        if (_e189 == 2u) {
            phi_1202_ = _e219.x;
        } else {
            phi_1202_ = length(_e219);
        }
        let _e224 = phi_1202_;
        let _e233 = textureSampleLevel(MD, Rb, vec2<f32>(((clamp(_e224, 0f, 1f) * _e216.z) + _e216.w), bitcast<f32>(_e116.y)), 0f);
        phi_1229_ = 0u;
        phi_1227_ = _e233;
    }
    let _e235 = phi_1229_;
    let _e237 = phi_1227_;
    let _e239 = (_e237.w * _e188);
    let _e241 = (_e237.xyz * _e239);
    phi_635_ = jh;
    if jh {
        let _e246 = w3_1;
        phi_635_ = (_e246 != 0u);
    }
    let _e249 = phi_635_;
    phi_1257_ = _e104;
    if _e249 {
        if (_e235 != 0u) {
            phi_1241_ = _e235;
        } else {
            let _e253 = g0_.d2_[_e92];
            phi_1241_ = _e253;
        }
        let _e255 = phi_1241_;
        let _e256 = w3_1;
        if (_e256 == (_e255 >> bitcast<u32>(16i))) {
            phi_1256_ = min(_e104, unpack2x16float(_e255).x);
        } else {
            phi_1256_ = 0f;
        }
        let _e264 = phi_1256_;
        phi_1257_ = _e264;
    }
    let _e266 = phi_1257_;
    let _e267 = I1_1;
    let _e269 = (_e94 * (_e266 * _e267));
    let _e273 = ((vec4<f32>(_e241.x, _e241.y, _e241.z, _e239) * (1f - _e269.w)) + _e269);
    let _e274 = _e273.xyz;
    let _e277 = m.A3_;
    let _e279 = m.B3_;
    if (qh && (_e273.w != 0f)) {
        phi_1278_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e277) + _e279)) + _e274);
    } else {
        phi_1278_ = _e274;
    }
    let _e295 = phi_1278_;
    let _e301 = vec4<f32>(_e295.x, _e273.y, _e273.z, _e273.w);
    let _e307 = vec4<f32>(_e301.x, _e295.y, _e301.z, _e301.w);
    D1_ = vec4<f32>(_e307.x, _e307.y, _e295.z, _e307.w);
    if (_e235 != 0u) {
        g0_.d2_[_e92] = _e235;
    }
    w4_.d2_[_e92] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) Y1_: vec2<f32>, @location(1) M0_: vec4<f32>, @location(4) @interpolate(flat, either) w3_: u32, @location(3) @interpolate(flat, either) I1_: f32, @location(5) @interpolate(flat, either) B1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    Y1_1 = Y1_;
    M0_1 = M0_;
    w3_1 = w3_;
    I1_1 = I1_;
    B1_1 = B1_;
    main_1();
    let _e13 = D1_;
    return _e13;
}
