struct jd {
    x2_: array<vec4<u32>>,
}

struct m0cf {
    x2_: array<u32>,
}

struct ug {
    x2_: array<vec4<f32>>,
}

struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

struct W4cf {
    x2_: array<u32>,
}

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;

@group(0) @binding(2)
var<storage> JB: jd;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(0) @binding(3)
var<storage> KB: ug;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var Q8_: sampler;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(1) @binding(12)
var v4_: sampler;
var<private> o2_1: vec2<f32>;
var<private> w5_1: f32;
var<private> V0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> V3_1: u32;
var<private> x5_1: vec4<f32>;
var<private> Y1_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
var<private> N1_1: u32;

fn main_1() {
    var phi_1781_: f32;
    var phi_1106_: bool;
    var phi_1719_: f32;
    var phi_1718_: f32;
    var phi_1720_: f32;
    var phi_1723_: f32;
    var phi_1722_: f32;
    var phi_1143_: bool;
    var phi_1727_: f32;
    var phi_1724_: f32;
    var phi_1725_: f32;
    var phi_1726_: vec2<f32>;
    var phi_1756_: vec4<f32>;
    var phi_1758_: u32;
    var phi_1757_: u32;
    var phi_1754_: vec4<f32>;
    var phi_839_: bool;
    var phi_1772_: u32;
    var phi_1790_: f32;
    var phi_1837_: f32;
    var phi_1810_: f32;
    var phi_1811_: f32;
    var phi_1812_: vec2<f32>;
    var phi_1835_: vec4<f32>;
    var phi_1849_: vec3<f32>;

    let _e70 = gl_FragCoord_1;
    let _e71 = _e70.xy;
    let _e74 = bitcast<vec2<u32>>(vec2<i32>(floor(_e71)));
    let _e76 = j.R6_;
    let _e105 = bitcast<i32>((((((_e74.y >> bitcast<u32>(5u)) * (((_e76 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e74.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e74.x & 28u) << bitcast<u32>(5u)) + ((_e74.y & 28u) << bitcast<u32>(2i)))) + (((_e74.y & 3u) << bitcast<u32>(2i)) + (_e74.x & 3u))));
    let _e106 = o2_1;
    let _e107 = textureSample(RB, v4_, _e106);
    let _e108 = w5_1;
    let _e109 = min(_e108, 1f);
    phi_1781_ = _e109;
    if kj {
        let _e110 = V0_1;
        let _e113 = min(_e110.xy, _e110.zw);
        phi_1781_ = clamp(min(_e113.x, _e113.y), 0f, _e109);
    }
    let _e119 = phi_1781_;
    let _e122 = W4_.x2_[_e105];
    let _e124 = (_e122 >> bitcast<u32>(17u));
    let _e128 = ((f32((_e122 & 131071u)) * 0.00048828125f) + -32f);
    let _e133 = JB.x2_[((_e124 * 4u) + 1u)];
    phi_1718_ = _e128;
    if ((_e133.z & 768u) != 0u) {
        let _e137 = abs(_e128);
        phi_1106_ = oj;
        if oj {
            phi_1106_ = ((_e133.z & 512u) != 0u);
        }
        let _e141 = phi_1106_;
        phi_1719_ = _e137;
        if _e141 {
            phi_1719_ = (1f - abs(((fract((_e137 * 0.5f)) * 2f) + -1f)));
        }
        let _e149 = phi_1719_;
        phi_1718_ = _e149;
    }
    let _e151 = phi_1718_;
    let _e152 = clamp(_e151, 0f, 1f);
    phi_1722_ = _e152;
    if jj {
        let _e154 = (_e133.z >> bitcast<u32>(16u));
        phi_1723_ = _e152;
        if (_e154 != 0u) {
            let _e158 = m0_.x2_[_e105];
            if (_e154 == (_e158 >> bitcast<u32>(16i))) {
                phi_1720_ = min(_e152, unpack2x16float(_e158).x);
            } else {
                phi_1720_ = 0f;
            }
            let _e166 = phi_1720_;
            phi_1723_ = _e166;
        }
        let _e168 = phi_1723_;
        phi_1722_ = _e168;
    }
    let _e170 = phi_1722_;
    phi_1143_ = kj;
    if kj {
        phi_1143_ = ((_e133.z & 1024u) != 0u);
    }
    let _e174 = phi_1143_;
    phi_1727_ = _e170;
    if _e174 {
        let _e175 = (_e124 * 8u);
        let _e179 = KB.x2_[(_e175 + 2u)];
        let _e190 = KB.x2_[(_e175 + 3u)];
        let _e195 = _e190.zw;
        let _e197 = ((abs(((mat2x2<f32>(vec2<f32>(_e179.x, _e179.y), vec2<f32>(_e179.z, _e179.w)) * _e71) + _e190.xy)) * _e195) - _e195);
        phi_1727_ = min(_e170, clamp((min(_e197.x, _e197.y) + 0.5f), 0f, 1f));
    }
    let _e205 = phi_1727_;
    let _e206 = (_e133.z & 15u);
    if (_e206 <= 1u) {
        let _e288 = (jj && (_e206 == 0u));
        phi_1758_ = 0u;
        if _e288 {
            phi_1758_ = (_e133.w | pack2x16float(vec2<f32>(_e205, 0f)));
        }
        let _e293 = phi_1758_;
        phi_1757_ = _e293;
        phi_1754_ = select(unpack4x8unorm(_e133.w), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e288));
    } else {
        let _e213 = (_e124 * 8u);
        let _e216 = KB.x2_[_e213];
        let _e227 = KB.x2_[(_e213 + 1u)];
        let _e230 = ((mat2x2<f32>(vec2<f32>(_e216.x, _e216.y), vec2<f32>(_e216.z, _e216.w)) * _e71) + _e227.xy);
        let _e234 = f32(_e206);
        let _e236 = j.O8_;
        let _e238 = j.P8_;
        let _e243 = ((_e227.w * _e236) + _e238);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e234 == 4f)) {
                    let _e257 = normalize(_e230);
                    let _e260 = acos(clamp(_e257.x, -1f, 1f));
                    if (_e257.y >= 0f) {
                        phi_1725_ = _e260;
                    } else {
                        phi_1725_ = -(_e260);
                    }
                    let _e265 = phi_1725_;
                    phi_1726_ = vec2<f32>((_e265 * 0.15915494f), _e243);
                    break;
                } else {
                    if (_e234 == 2f) {
                        phi_1724_ = _e230.x;
                    } else {
                        phi_1724_ = length(_e230);
                    }
                    let _e251 = phi_1724_;
                    phi_1726_ = vec2<f32>(((clamp(_e251, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e227.z < 0f))) + ((max(0f, _e227.z) * 0.001953125f) + 0.0009765625f)), _e243);
                    break;
                }
            }
        }
        let _e269 = phi_1726_;
        let _e270 = textureSampleLevel(YC, Q8_, _e269, 0f);
        phi_1756_ = _e270;
        if !((lj && (((_e133.z >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e274 = (_e270.xyz * _e270.w);
            phi_1756_ = vec4<f32>(_e274.x, _e274.y, _e274.z, (_e270.w * abs(bitcast<f32>(_e133.w))));
        }
        let _e284 = phi_1756_;
        phi_1757_ = 0u;
        phi_1754_ = _e284;
    }
    let _e297 = phi_1757_;
    let _e299 = phi_1754_;
    phi_839_ = jj;
    if jj {
        let _e301 = V3_1;
        phi_839_ = (_e301 != 0u);
    }
    let _e304 = phi_839_;
    phi_1837_ = _e119;
    if _e304 {
        if (_e297 != 0u) {
            phi_1772_ = _e297;
        } else {
            let _e308 = m0_.x2_[_e105];
            phi_1772_ = _e308;
        }
        let _e310 = phi_1772_;
        let _e311 = V3_1;
        if (_e311 == (_e310 >> bitcast<u32>(16i))) {
            phi_1790_ = min(_e119, unpack2x16float(_e310).x);
        } else {
            phi_1790_ = 0f;
        }
        let _e319 = phi_1790_;
        phi_1837_ = _e319;
    }
    let _e321 = phi_1837_;
    let _e323 = x5_1[3u];
    phi_1835_ = _e107;
    if (_e323 != 0f) {
        let _e325 = x5_1;
        let _e327 = j.O8_;
        let _e329 = j.P8_;
        let _e332 = abs(_e325.z);
        let _e333 = floor(_e332);
        let _e336 = (((_e332 - _e333) * 8f) + 0.0009765625f);
        let _e338 = ((_e333 * _e327) + _e329);
        let _e339 = floor(_e336);
        let _e340 = _e325.xy;
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e339 == 4f)) {
                    let _e354 = normalize(_e340);
                    let _e357 = acos(clamp(_e354.x, -1f, 1f));
                    if (_e354.y >= 0f) {
                        phi_1811_ = _e357;
                    } else {
                        phi_1811_ = -(_e357);
                    }
                    let _e362 = phi_1811_;
                    phi_1812_ = vec2<f32>((_e362 * 0.15915494f), _e338);
                    break;
                } else {
                    if (_e339 == 2f) {
                        phi_1810_ = _e325.x;
                    } else {
                        phi_1810_ = length(_e340);
                    }
                    let _e348 = phi_1810_;
                    phi_1812_ = vec2<f32>(((clamp(_e348, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e325.z < 0f))) + _e336), _e338);
                    break;
                }
            }
        }
        let _e366 = phi_1812_;
        let _e367 = textureSampleLevel(YC, Q8_, _e366, 0f);
        let _e370 = (_e367.xyz * _e367.w);
        let _e376 = vec4<f32>(_e370.x, _e367.y, _e367.z, _e367.w);
        let _e382 = vec4<f32>(_e376.x, _e370.y, _e376.z, _e376.w);
        phi_1835_ = (_e107 * vec4<f32>(_e382.x, _e382.y, _e370.z, _e382.w));
    }
    let _e391 = phi_1835_;
    let _e392 = Y1_1;
    let _e394 = ((_e391 * _e392) * _e321);
    let _e398 = (((_e299 * _e205) * (1f - _e394.w)) + _e394);
    let _e399 = _e398.xyz;
    let _e402 = j.I3_;
    let _e404 = j.J3_;
    if (rj && (_e398.w != 0f)) {
        phi_1849_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e70.x) + (0.00583715f * _e70.y))))) * _e402) + _e404)) + _e399);
    } else {
        phi_1849_ = _e399;
    }
    let _e420 = phi_1849_;
    let _e426 = vec4<f32>(_e420.x, _e398.y, _e398.z, _e398.w);
    let _e432 = vec4<f32>(_e426.x, _e420.y, _e426.z, _e426.w);
    Q1_ = vec4<f32>(_e432.x, _e432.y, _e420.z, _e432.w);
    if (_e297 != 0u) {
        m0_.x2_[_e105] = _e297;
    }
    W4_.x2_[_e105] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) o2_: vec2<f32>, @location(1) w5_: f32, @location(3) V0_: vec4<f32>, @location(5) @interpolate(flat, either) V3_: u32, @location(2) x5_: vec4<f32>, @location(4) @interpolate(flat, either) Y1_: vec4<f32>, @location(6) @interpolate(flat, either) N1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    o2_1 = o2_;
    w5_1 = w5_;
    V0_1 = V0_;
    V3_1 = V3_;
    x5_1 = x5_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
    let _e17 = Q1_;
    return _e17;
}
