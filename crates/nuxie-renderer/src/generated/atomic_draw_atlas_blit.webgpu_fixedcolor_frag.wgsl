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
var<private> F0_1: u32;
@group(0) @binding(10)
var GD: texture_2d<f32>;
@group(3) @binding(10)
var ma: sampler;
var<private> K2_1: vec2<f32>;
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
    var phi_839_: bool;
    var phi_1219_: f32;
    var phi_1218_: f32;
    var phi_1220_: f32;
    var phi_1223_: f32;
    var phi_1222_: f32;
    var phi_876_: bool;
    var phi_1225_: f32;
    var phi_1251_: u32;
    var phi_1224_: f32;
    var phi_1249_: vec4<f32>;
    var phi_1250_: u32;
    var phi_1247_: vec4<f32>;
    var phi_1262_: vec3<f32>;

    let _e61 = gl_FragCoord_1;
    let _e62 = _e61.xy;
    let _e65 = bitcast<vec2<u32>>(vec2<i32>(floor(_e62)));
    let _e67 = j.z6_;
    let _e96 = bitcast<i32>((((((_e65.y >> bitcast<u32>(5u)) * (((_e67 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e65.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e65.x & 28u) << bitcast<u32>(5u)) + ((_e65.y & 28u) << bitcast<u32>(2i)))) + (((_e65.y & 3u) << bitcast<u32>(2i)) + (_e65.x & 3u))));
    let _e99 = K4_.j2_[_e96];
    let _e101 = (_e99 >> bitcast<u32>(17u));
    let _e102 = F0_1;
    let _e106 = K2_1;
    let _e107 = textureSampleLevel(GD, ma, _e106, 0f);
    K4_.j2_[_e96] = (((_e102 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e107.x, 0f, 1f) * 2048f)))));
    let _e118 = ((f32((_e99 & 131071u)) * 0.00048828125f) + -32f);
    let _e121 = XC.j2_[_e101];
    phi_1218_ = _e118;
    if ((_e121.x & 768u) != 0u) {
        let _e125 = abs(_e118);
        phi_839_ = fi;
        if fi {
            phi_839_ = ((_e121.x & 512u) != 0u);
        }
        let _e129 = phi_839_;
        phi_1219_ = _e125;
        if _e129 {
            phi_1219_ = (1f - abs(((fract((_e125 * 0.5f)) * 2f) + -1f)));
        }
        let _e137 = phi_1219_;
        phi_1218_ = _e137;
    }
    let _e139 = phi_1218_;
    let _e140 = clamp(_e139, 0f, 1f);
    phi_1222_ = _e140;
    if bi {
        let _e142 = (_e121.x >> bitcast<u32>(16u));
        phi_1223_ = _e140;
        if (_e142 != 0u) {
            let _e146 = m0_.j2_[_e96];
            if (_e142 == (_e146 >> bitcast<u32>(16i))) {
                phi_1220_ = min(_e140, unpack2x16float(_e146).x);
            } else {
                phi_1220_ = 0f;
            }
            let _e154 = phi_1220_;
            phi_1223_ = _e154;
        }
        let _e156 = phi_1223_;
        phi_1222_ = _e156;
    }
    let _e158 = phi_1222_;
    phi_876_ = ci;
    if ci {
        phi_876_ = ((_e121.x & 1024u) != 0u);
    }
    let _e162 = phi_876_;
    phi_1225_ = _e158;
    if _e162 {
        let _e163 = (_e101 * 8u);
        let _e167 = JB.j2_[(_e163 + 2u)];
        let _e178 = JB.j2_[(_e163 + 3u)];
        let _e183 = _e178.zw;
        let _e185 = ((abs(((mat2x2<f32>(vec2<f32>(_e167.x, _e167.y), vec2<f32>(_e167.z, _e167.w)) * _e62) + _e178.xy)) * _e183) - _e183);
        phi_1225_ = min(_e158, clamp((min(_e185.x, _e185.y) + 0.5f), 0f, 1f));
    }
    let _e193 = phi_1225_;
    let _e194 = (_e121.x & 15u);
    if (_e194 <= 1u) {
        let _e204 = (bi && (_e194 == 0u));
        phi_1251_ = 0u;
        if _e204 {
            phi_1251_ = (_e121.y | pack2x16float(vec2<f32>(_e193, 0f)));
        }
        let _e209 = phi_1251_;
        phi_1250_ = _e209;
        phi_1247_ = select(unpack4x8unorm(_e121.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e204));
    } else {
        let _e212 = (_e101 * 8u);
        let _e215 = JB.j2_[_e212];
        let _e226 = JB.j2_[(_e212 + 1u)];
        let _e229 = ((mat2x2<f32>(vec2<f32>(_e215.x, _e215.y), vec2<f32>(_e215.z, _e215.w)) * _e62) + _e226.xy);
        if (_e194 == 2u) {
            phi_1224_ = _e229.x;
        } else {
            phi_1224_ = length(_e229);
        }
        let _e234 = phi_1224_;
        let _e241 = bitcast<f32>(_e121.y);
        let _e244 = j.wc;
        let _e247 = j.xc;
        let _e250 = textureSampleLevel(FD, ha, vec2<f32>(((clamp(_e234, 0f, 1f) * _e226.z) + _e226.w), ((floor(_e241) * _e244) + _e247)), 0f);
        phi_1249_ = _e250;
        if !((di && (((_e121.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e254 = (_e250.xyz * _e250.w);
            phi_1249_ = vec4<f32>(_e254.x, _e254.y, _e254.z, (_e250.w * (fract(_e241) * 1.0039216f)));
        }
        let _e263 = phi_1249_;
        phi_1250_ = 0u;
        phi_1247_ = _e263;
    }
    let _e265 = phi_1250_;
    let _e267 = phi_1247_;
    let _e268 = (_e267 * _e193);
    let _e269 = _e268.xyz;
    let _e272 = j.M3_;
    let _e274 = j.N3_;
    if (ii && (_e268.w != 0f)) {
        phi_1262_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e61.x) + (0.00583715f * _e61.y))))) * _e272) + _e274)) + _e269);
    } else {
        phi_1262_ = _e269;
    }
    let _e290 = phi_1262_;
    let _e296 = vec4<f32>(_e290.x, _e268.y, _e268.z, _e268.w);
    let _e302 = vec4<f32>(_e296.x, _e290.y, _e296.z, _e296.w);
    J1_ = vec4<f32>(_e302.x, _e302.y, _e290.z, _e302.w);
    if (_e265 != 0u) {
        m0_.j2_[_e96] = _e265;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) F0_: u32, @location(0) K2_: vec2<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    F0_1 = F0_;
    K2_1 = K2_;
    main_1();
    let _e7 = J1_;
    return _e7;
}
