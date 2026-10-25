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

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;

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
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;
var<private> n2_1: vec2<f32>;
var<private> x5_1: f32;
var<private> W0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> V3_1: u32;
var<private> y5_1: vec4<f32>;
var<private> Y1_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> N1_1: u32;

fn main_1() {
    var phi_1773_: f32;
    var phi_1098_: bool;
    var phi_1711_: f32;
    var phi_1710_: f32;
    var phi_1712_: f32;
    var phi_1715_: f32;
    var phi_1714_: f32;
    var phi_1135_: bool;
    var phi_1719_: f32;
    var phi_1716_: f32;
    var phi_1717_: f32;
    var phi_1718_: vec2<f32>;
    var phi_1748_: vec4<f32>;
    var phi_1750_: u32;
    var phi_1749_: u32;
    var phi_1746_: vec4<f32>;
    var phi_834_: bool;
    var phi_1764_: u32;
    var phi_1782_: f32;
    var phi_1829_: f32;
    var phi_1802_: f32;
    var phi_1803_: f32;
    var phi_1804_: vec2<f32>;
    var phi_1827_: vec4<f32>;
    var phi_1841_: vec3<f32>;

    let _e69 = gl_FragCoord_1;
    let _e70 = _e69.xy;
    let _e73 = bitcast<vec2<u32>>(vec2<i32>(floor(_e70)));
    let _e75 = j.T6_;
    let _e104 = bitcast<i32>((((((_e73.y >> bitcast<u32>(5u)) * (((_e75 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e73.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e73.x & 28u) << bitcast<u32>(5u)) + ((_e73.y & 28u) << bitcast<u32>(2i)))) + (((_e73.y & 3u) << bitcast<u32>(2i)) + (_e73.x & 3u))));
    let _e105 = n2_1;
    let _e106 = textureSample(RB, v4_, _e105);
    let _e107 = x5_1;
    let _e108 = min(_e107, 1f);
    phi_1773_ = _e108;
    if kj {
        let _e109 = W0_1;
        let _e112 = min(_e109.xy, _e109.zw);
        phi_1773_ = clamp(min(_e112.x, _e112.y), 0f, _e108);
    }
    let _e118 = phi_1773_;
    let _e121 = W4_.v2_[_e104];
    let _e123 = (_e121 >> bitcast<u32>(17u));
    let _e127 = ((f32((_e121 & 131071u)) * 0.00048828125f) + -32f);
    let _e130 = XC.v2_[_e123];
    phi_1710_ = _e127;
    if ((_e130.x & 768u) != 0u) {
        let _e134 = abs(_e127);
        phi_1098_ = oj;
        if oj {
            phi_1098_ = ((_e130.x & 512u) != 0u);
        }
        let _e138 = phi_1098_;
        phi_1711_ = _e134;
        if _e138 {
            phi_1711_ = (1f - abs(((fract((_e134 * 0.5f)) * 2f) + -1f)));
        }
        let _e146 = phi_1711_;
        phi_1710_ = _e146;
    }
    let _e148 = phi_1710_;
    let _e149 = clamp(_e148, 0f, 1f);
    phi_1714_ = _e149;
    if jj {
        let _e151 = (_e130.x >> bitcast<u32>(16u));
        phi_1715_ = _e149;
        if (_e151 != 0u) {
            let _e155 = m0_.v2_[_e104];
            if (_e151 == (_e155 >> bitcast<u32>(16i))) {
                phi_1712_ = min(_e149, unpack2x16float(_e155).x);
            } else {
                phi_1712_ = 0f;
            }
            let _e163 = phi_1712_;
            phi_1715_ = _e163;
        }
        let _e165 = phi_1715_;
        phi_1714_ = _e165;
    }
    let _e167 = phi_1714_;
    phi_1135_ = kj;
    if kj {
        phi_1135_ = ((_e130.x & 1024u) != 0u);
    }
    let _e171 = phi_1135_;
    phi_1719_ = _e167;
    if _e171 {
        let _e172 = (_e123 * 8u);
        let _e176 = JB.v2_[(_e172 + 2u)];
        let _e187 = JB.v2_[(_e172 + 3u)];
        let _e192 = _e187.zw;
        let _e194 = ((abs(((mat2x2<f32>(vec2<f32>(_e176.x, _e176.y), vec2<f32>(_e176.z, _e176.w)) * _e70) + _e187.xy)) * _e192) - _e192);
        phi_1719_ = min(_e167, clamp((min(_e194.x, _e194.y) + 0.5f), 0f, 1f));
    }
    let _e202 = phi_1719_;
    let _e203 = (_e130.x & 15u);
    if (_e203 <= 1u) {
        let _e285 = (jj && (_e203 == 0u));
        phi_1750_ = 0u;
        if _e285 {
            phi_1750_ = (_e130.y | pack2x16float(vec2<f32>(_e202, 0f)));
        }
        let _e290 = phi_1750_;
        phi_1749_ = _e290;
        phi_1746_ = select(unpack4x8unorm(_e130.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e285));
    } else {
        let _e210 = (_e123 * 8u);
        let _e213 = JB.v2_[_e210];
        let _e224 = JB.v2_[(_e210 + 1u)];
        let _e227 = ((mat2x2<f32>(vec2<f32>(_e213.x, _e213.y), vec2<f32>(_e213.z, _e213.w)) * _e70) + _e224.xy);
        let _e231 = f32(_e203);
        let _e233 = j.Q8_;
        let _e235 = j.R8_;
        let _e240 = ((_e224.w * _e233) + _e235);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e231 == 4f)) {
                    let _e254 = normalize(_e227);
                    let _e257 = acos(clamp(_e254.x, -1f, 1f));
                    if (_e254.y >= 0f) {
                        phi_1717_ = _e257;
                    } else {
                        phi_1717_ = -(_e257);
                    }
                    let _e262 = phi_1717_;
                    phi_1718_ = vec2<f32>((_e262 * 0.15915494f), _e240);
                    break;
                } else {
                    if (_e231 == 2f) {
                        phi_1716_ = _e227.x;
                    } else {
                        phi_1716_ = length(_e227);
                    }
                    let _e248 = phi_1716_;
                    phi_1718_ = vec2<f32>(((clamp(_e248, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e224.z < 0f))) + ((max(0f, _e224.z) * 0.001953125f) + 0.0009765625f)), _e240);
                    break;
                }
            }
        }
        let _e266 = phi_1718_;
        let _e267 = textureSampleLevel(ZC, S8_, _e266, 0f);
        phi_1748_ = _e267;
        if !((lj && (((_e130.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e271 = (_e267.xyz * _e267.w);
            phi_1748_ = vec4<f32>(_e271.x, _e271.y, _e271.z, (_e267.w * abs(bitcast<f32>(_e130.y))));
        }
        let _e281 = phi_1748_;
        phi_1749_ = 0u;
        phi_1746_ = _e281;
    }
    let _e294 = phi_1749_;
    let _e296 = phi_1746_;
    phi_834_ = jj;
    if jj {
        let _e298 = V3_1;
        phi_834_ = (_e298 != 0u);
    }
    let _e301 = phi_834_;
    phi_1829_ = _e118;
    if _e301 {
        if (_e294 != 0u) {
            phi_1764_ = _e294;
        } else {
            let _e305 = m0_.v2_[_e104];
            phi_1764_ = _e305;
        }
        let _e307 = phi_1764_;
        let _e308 = V3_1;
        if (_e308 == (_e307 >> bitcast<u32>(16i))) {
            phi_1782_ = min(_e118, unpack2x16float(_e307).x);
        } else {
            phi_1782_ = 0f;
        }
        let _e316 = phi_1782_;
        phi_1829_ = _e316;
    }
    let _e318 = phi_1829_;
    let _e320 = y5_1[3u];
    phi_1827_ = _e106;
    if (_e320 != 0f) {
        let _e322 = y5_1;
        let _e324 = j.Q8_;
        let _e326 = j.R8_;
        let _e329 = abs(_e322.z);
        let _e330 = floor(_e329);
        let _e333 = (((_e329 - _e330) * 8f) + 0.0009765625f);
        let _e335 = ((_e330 * _e324) + _e326);
        let _e336 = floor(_e333);
        let _e337 = _e322.xy;
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e336 == 4f)) {
                    let _e351 = normalize(_e337);
                    let _e354 = acos(clamp(_e351.x, -1f, 1f));
                    if (_e351.y >= 0f) {
                        phi_1803_ = _e354;
                    } else {
                        phi_1803_ = -(_e354);
                    }
                    let _e359 = phi_1803_;
                    phi_1804_ = vec2<f32>((_e359 * 0.15915494f), _e335);
                    break;
                } else {
                    if (_e336 == 2f) {
                        phi_1802_ = _e322.x;
                    } else {
                        phi_1802_ = length(_e337);
                    }
                    let _e345 = phi_1802_;
                    phi_1804_ = vec2<f32>(((clamp(_e345, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e322.z < 0f))) + _e333), _e335);
                    break;
                }
            }
        }
        let _e363 = phi_1804_;
        let _e364 = textureSampleLevel(ZC, S8_, _e363, 0f);
        let _e367 = (_e364.xyz * _e364.w);
        let _e373 = vec4<f32>(_e367.x, _e364.y, _e364.z, _e364.w);
        let _e379 = vec4<f32>(_e373.x, _e367.y, _e373.z, _e373.w);
        phi_1827_ = (_e106 * vec4<f32>(_e379.x, _e379.y, _e367.z, _e379.w));
    }
    let _e388 = phi_1827_;
    let _e389 = Y1_1;
    let _e391 = ((_e388 * _e389) * _e318);
    let _e395 = (((_e296 * _e202) * (1f - _e391.w)) + _e391);
    let _e396 = _e395.xyz;
    let _e399 = j.H3_;
    let _e401 = j.I3_;
    if (rj && (_e395.w != 0f)) {
        phi_1841_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e69.x) + (0.00583715f * _e69.y))))) * _e399) + _e401)) + _e396);
    } else {
        phi_1841_ = _e396;
    }
    let _e417 = phi_1841_;
    let _e423 = vec4<f32>(_e417.x, _e395.y, _e395.z, _e395.w);
    let _e429 = vec4<f32>(_e423.x, _e417.y, _e423.z, _e423.w);
    Q1_ = vec4<f32>(_e429.x, _e429.y, _e417.z, _e429.w);
    if (_e294 != 0u) {
        m0_.v2_[_e104] = _e294;
    }
    W4_.v2_[_e104] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) n2_: vec2<f32>, @location(1) x5_: f32, @location(3) W0_: vec4<f32>, @location(5) @interpolate(flat, either) V3_: u32, @location(2) y5_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec4<f32>, @location(6) @interpolate(flat, either) N1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    n2_1 = n2_;
    x5_1 = x5_;
    W0_1 = W0_;
    V3_1 = V3_;
    y5_1 = y5_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
    let _e17 = Q1_;
    return _e17;
}
