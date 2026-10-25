struct ug {
    v2_: array<vec2<u32>>,
}

struct m0cf {
    v2_: array<u32>,
}

struct vg {
    v2_: array<vec4<f32>>,
}

struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct W4cf {
    v2_: array<u32>,
}

struct W4cf_1 {
    v2_: array<atomic<u32>>,
}

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;
@id(3) override mj: bool = true;

@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(3)
var<storage> XC: ug;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(0) @binding(4)
var<storage> JB: vg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
var<private> T_1: vec4<f32>;
var<private> I0_1: u32;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf_1;
var<private> Q1_: vec4<f32>;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;

fn main_1() {
    var phi_937_: bool;
    var phi_950_: bool;
    var phi_1938_: f32;
    var phi_1946_: f32;
    var phi_1954_: f32;
    var phi_1953_: f32;
    var phi_1447_: bool;
    var phi_1957_: f32;
    var phi_1956_: f32;
    var phi_1958_: f32;
    var phi_1961_: f32;
    var phi_1960_: f32;
    var phi_1484_: bool;
    var phi_1965_: f32;
    var phi_1962_: f32;
    var phi_1963_: f32;
    var phi_1964_: vec2<f32>;
    var phi_2003_: vec4<f32>;
    var phi_2005_: u32;
    var phi_2004_: u32;
    var phi_2001_: vec4<f32>;
    var phi_2023_: u32;
    var phi_2018_: vec4<f32>;
    var phi_2020_: vec3<f32>;

    let _e82 = gl_FragCoord_1;
    let _e83 = _e82.xy;
    let _e86 = bitcast<vec2<u32>>(vec2<i32>(floor(_e83)));
    let _e88 = j.T6_;
    let _e117 = bitcast<i32>((((((_e86.y >> bitcast<u32>(5u)) * (((_e88 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e86.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e86.x & 28u) << bitcast<u32>(5u)) + ((_e86.y & 28u) << bitcast<u32>(2i)))) + (((_e86.y & 3u) << bitcast<u32>(2i)) + (_e86.x & 3u))));
    phi_937_ = mj;
    if mj {
        let _e118 = T_1;
        phi_937_ = (_e118.x < -1.5f);
    }
    let _e122 = phi_937_;
    if _e122 {
        let _e195 = T_1;
        let _e199 = textureSampleLevel(AD, eb, vec2<f32>((3f + _e195.x), 0f), 0f);
        let _e205 = textureSampleLevel(AD, eb, vec2<f32>((1f - _e195.y), 0f), 0f);
        phi_1953_ = ((1f - _e199.x) - _e205.x);
    } else {
        phi_950_ = mj;
        if mj {
            let _e123 = T_1;
            phi_950_ = (_e123.y < -1.5f);
        }
        let _e127 = phi_950_;
        if _e127 {
            let _e135 = T_1;
            let _e138 = max(_e135.w, 0f);
            if (_e135.z >= 0f) {
                let _e141 = textureSampleLevel(AD, eb, vec2<f32>(_e138, 0f), 0f);
                phi_1938_ = _e141.x;
            } else {
                phi_1938_ = 0f;
            }
            let _e144 = phi_1938_;
            phi_1946_ = _e144;
            if (abs(_e135.z) < 1000f) {
                let _e151 = (-2f - _e135.y);
                let _e153 = ((_e151 - _e138) * 0.5984134f);
                let _e156 = (vec4(_e138) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e153));
                let _e162 = ((_e156 * -(_e135.z)) + vec4(((_e151 * _e135.z) + (abs(_e135.x) - 0.25f))));
                let _e165 = textureSampleLevel(AD, eb, vec2<f32>(_e162.x, 0f), 0f);
                let _e168 = textureSampleLevel(AD, eb, vec2<f32>(_e162.y, 0f), 0f);
                let _e171 = textureSampleLevel(AD, eb, vec2<f32>(_e162.z, 0f), 0f);
                let _e174 = textureSampleLevel(AD, eb, vec2<f32>(_e162.w, 0f), 0f);
                let _e180 = (_e156 * 5.0959306f);
                phi_1946_ = (_e144 + (dot(vec4<f32>(_e165.x, _e168.x, _e171.x, _e174.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e180) * (_e180 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e153));
            }
            let _e189 = phi_1946_;
            phi_1954_ = (_e189 * sign(_e135.x));
        } else {
            let _e129 = T_1[0u];
            let _e131 = T_1[1u];
            phi_1954_ = min(min(_e129, abs(_e131)), 1f);
        }
        let _e194 = phi_1954_;
        phi_1953_ = _e194;
    }
    let _e209 = phi_1953_;
    let _e213 = u32(round(((_e209 * 2048f) + 65536f)));
    let _e214 = I0_1;
    let _e217 = ((_e214 << bitcast<u32>(17u)) | _e213);
    let _e220 = atomicMax((&W4_.v2_[_e117]), _e217);
    let _e222 = (_e220 >> bitcast<u32>(17u));
    if (_e222 == _e214) {
        let _e398 = T_1;
        if (_e398.y < 0f) {
            let _e405 = atomicAdd((&W4_.v2_[_e117]), ((_e213 + (_e220 - max(_e217, _e220))) - 65536u));
        }
        phi_2023_ = 0u;
        phi_2018_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e227 = ((f32((_e220 & 131071u)) * 0.00048828125f) + -32f);
        let _e230 = XC.v2_[_e222];
        phi_1956_ = _e227;
        if ((_e230.x & 768u) != 0u) {
            let _e234 = abs(_e227);
            phi_1447_ = oj;
            if oj {
                phi_1447_ = ((_e230.x & 512u) != 0u);
            }
            let _e238 = phi_1447_;
            phi_1957_ = _e234;
            if _e238 {
                phi_1957_ = (1f - abs(((fract((_e234 * 0.5f)) * 2f) + -1f)));
            }
            let _e246 = phi_1957_;
            phi_1956_ = _e246;
        }
        let _e248 = phi_1956_;
        let _e249 = clamp(_e248, 0f, 1f);
        phi_1960_ = _e249;
        if jj {
            let _e251 = (_e230.x >> bitcast<u32>(16u));
            phi_1961_ = _e249;
            if (_e251 != 0u) {
                let _e255 = m0_.v2_[_e117];
                if (_e251 == (_e255 >> bitcast<u32>(16i))) {
                    phi_1958_ = min(_e249, unpack2x16float(_e255).x);
                } else {
                    phi_1958_ = 0f;
                }
                let _e263 = phi_1958_;
                phi_1961_ = _e263;
            }
            let _e265 = phi_1961_;
            phi_1960_ = _e265;
        }
        let _e267 = phi_1960_;
        phi_1484_ = kj;
        if kj {
            phi_1484_ = ((_e230.x & 1024u) != 0u);
        }
        let _e271 = phi_1484_;
        phi_1965_ = _e267;
        if _e271 {
            let _e272 = (_e222 * 8u);
            let _e276 = JB.v2_[(_e272 + 2u)];
            let _e287 = JB.v2_[(_e272 + 3u)];
            let _e292 = _e287.zw;
            let _e294 = ((abs(((mat2x2<f32>(vec2<f32>(_e276.x, _e276.y), vec2<f32>(_e276.z, _e276.w)) * _e83) + _e287.xy)) * _e292) - _e292);
            phi_1965_ = min(_e267, clamp((min(_e294.x, _e294.y) + 0.5f), 0f, 1f));
        }
        let _e302 = phi_1965_;
        let _e303 = (_e230.x & 15u);
        if (_e303 <= 1u) {
            let _e385 = (jj && (_e303 == 0u));
            phi_2005_ = 0u;
            if _e385 {
                phi_2005_ = (_e230.y | pack2x16float(vec2<f32>(_e302, 0f)));
            }
            let _e390 = phi_2005_;
            phi_2004_ = _e390;
            phi_2001_ = select(unpack4x8unorm(_e230.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e385));
        } else {
            let _e310 = (_e222 * 8u);
            let _e313 = JB.v2_[_e310];
            let _e324 = JB.v2_[(_e310 + 1u)];
            let _e327 = ((mat2x2<f32>(vec2<f32>(_e313.x, _e313.y), vec2<f32>(_e313.z, _e313.w)) * _e83) + _e324.xy);
            let _e331 = f32(_e303);
            let _e333 = j.Q8_;
            let _e335 = j.R8_;
            let _e340 = ((_e324.w * _e333) + _e335);
            switch bitcast<i32>(0u) {
                default: {
                    if (sj && (_e331 == 4f)) {
                        let _e354 = normalize(_e327);
                        let _e357 = acos(clamp(_e354.x, -1f, 1f));
                        if (_e354.y >= 0f) {
                            phi_1963_ = _e357;
                        } else {
                            phi_1963_ = -(_e357);
                        }
                        let _e362 = phi_1963_;
                        phi_1964_ = vec2<f32>((_e362 * 0.15915494f), _e340);
                        break;
                    } else {
                        if (_e331 == 2f) {
                            phi_1962_ = _e327.x;
                        } else {
                            phi_1962_ = length(_e327);
                        }
                        let _e348 = phi_1962_;
                        phi_1964_ = vec2<f32>(((clamp(_e348, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e324.z < 0f))) + ((max(0f, _e324.z) * 0.001953125f) + 0.0009765625f)), _e340);
                        break;
                    }
                }
            }
            let _e366 = phi_1964_;
            let _e367 = textureSampleLevel(ZC, S8_, _e366, 0f);
            phi_2003_ = _e367;
            if !((lj && (((_e230.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e371 = (_e367.xyz * _e367.w);
                phi_2003_ = vec4<f32>(_e371.x, _e371.y, _e371.z, (_e367.w * abs(bitcast<f32>(_e230.y))));
            }
            let _e381 = phi_2003_;
            phi_2004_ = 0u;
            phi_2001_ = _e381;
        }
        let _e394 = phi_2004_;
        let _e396 = phi_2001_;
        phi_2023_ = _e394;
        phi_2018_ = (_e396 * _e302);
    }
    let _e407 = phi_2023_;
    let _e409 = phi_2018_;
    let _e410 = _e409.xyz;
    let _e413 = j.H3_;
    let _e415 = j.I3_;
    if (rj && (_e409.w != 0f)) {
        phi_2020_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e82.x) + (0.00583715f * _e82.y))))) * _e413) + _e415)) + _e410);
    } else {
        phi_2020_ = _e410;
    }
    let _e431 = phi_2020_;
    let _e437 = vec4<f32>(_e431.x, _e409.y, _e409.z, _e409.w);
    let _e443 = vec4<f32>(_e437.x, _e431.y, _e437.z, _e437.w);
    Q1_ = vec4<f32>(_e443.x, _e443.y, _e431.z, _e443.w);
    if (_e407 != 0u) {
        m0_.v2_[_e117] = _e407;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) T: vec4<f32>, @location(1) @interpolate(flat, either) I0_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    T_1 = T;
    I0_1 = I0_;
    main_1();
    let _e7 = Q1_;
    return _e7;
}
