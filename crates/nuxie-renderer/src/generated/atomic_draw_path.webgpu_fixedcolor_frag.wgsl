struct lf {
    g2_: array<vec2<u32>>,
}

struct i0Td {
    g2_: array<u32>,
}

struct mf {
    g2_: array<vec4<f32>>,
}

struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

struct z4Td {
    g2_: array<u32>,
}

struct z4Td_1 {
    g2_: array<atomic<u32>>,
}

@id(7) override Lh: bool = true;
@id(4) override Ih: bool = true;
@id(0) override Eh: bool = true;
@id(1) override Fh: bool = true;
@id(2) override Gh: bool = true;
@id(3) override Hh: bool = true;

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(3)
var<storage> CD: lf;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Td;
@group(0) @binding(4)
var<storage> PB: mf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(0) @binding(0)
var<uniform> j: AC;
var<private> M_1: vec4<f32>;
var<private> D0_1: u32;
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td_1;
var<private> E1_: vec4<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;

fn main_1() {
    var phi_809_: bool;
    var phi_822_: bool;
    var phi_1688_: f32;
    var phi_1696_: f32;
    var phi_1704_: f32;
    var phi_1703_: f32;
    var phi_1306_: bool;
    var phi_1707_: f32;
    var phi_1706_: f32;
    var phi_1708_: f32;
    var phi_1711_: f32;
    var phi_1710_: f32;
    var phi_1343_: bool;
    var phi_1713_: f32;
    var phi_1751_: u32;
    var phi_1712_: f32;
    var phi_1749_: vec4<f32>;
    var phi_1750_: u32;
    var phi_1747_: vec4<f32>;
    var phi_1767_: u32;
    var phi_1762_: vec4<f32>;
    var phi_1764_: vec3<f32>;

    let _e74 = gl_FragCoord_1;
    let _e75 = _e74.xy;
    let _e78 = bitcast<vec2<u32>>(vec2<i32>(floor(_e75)));
    let _e80 = j.q6_;
    let _e109 = bitcast<i32>((((((_e78.y >> bitcast<u32>(5u)) * (((_e80 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e78.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e78.x & 28u) << bitcast<u32>(5u)) + ((_e78.y & 28u) << bitcast<u32>(2i)))) + (((_e78.y & 3u) << bitcast<u32>(2i)) + (_e78.x & 3u))));
    phi_809_ = Hh;
    if Hh {
        let _e110 = M_1;
        phi_809_ = (_e110.x < -1.5f);
    }
    let _e114 = phi_809_;
    if _e114 {
        let _e115 = M_1;
        let _e119 = textureSampleLevel(XC, ga, vec2<f32>((3f + _e115.x), 0f), 0f);
        let _e125 = textureSampleLevel(XC, ga, vec2<f32>((1f - _e115.y), 0f), 0f);
        phi_1703_ = ((1f - _e119.x) - _e125.x);
    } else {
        phi_822_ = Hh;
        if Hh {
            let _e128 = M_1;
            phi_822_ = (_e128.y < -1.5f);
        }
        let _e132 = phi_822_;
        if _e132 {
            let _e133 = M_1;
            let _e136 = max(_e133.w, 0f);
            if (_e133.z >= 0f) {
                let _e139 = textureSampleLevel(XC, ga, vec2<f32>(_e136, 0f), 0f);
                phi_1688_ = _e139.x;
            } else {
                phi_1688_ = 0f;
            }
            let _e142 = phi_1688_;
            phi_1696_ = _e142;
            if (abs(_e133.z) < 1000f) {
                let _e149 = (-2f - _e133.y);
                let _e151 = ((_e149 - _e136) * 0.5984134f);
                let _e154 = (vec4(_e136) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e151));
                let _e160 = ((_e154 * -(_e133.z)) + vec4(((_e149 * _e133.z) + (abs(_e133.x) - 0.25f))));
                let _e163 = textureSampleLevel(XC, ga, vec2<f32>(_e160.x, 0f), 0f);
                let _e166 = textureSampleLevel(XC, ga, vec2<f32>(_e160.y, 0f), 0f);
                let _e169 = textureSampleLevel(XC, ga, vec2<f32>(_e160.z, 0f), 0f);
                let _e172 = textureSampleLevel(XC, ga, vec2<f32>(_e160.w, 0f), 0f);
                let _e178 = (_e154 * 5.0959306f);
                phi_1696_ = (_e142 + (dot(vec4<f32>(_e163.x, _e166.x, _e169.x, _e172.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e178) * (_e178 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e151));
            }
            let _e187 = phi_1696_;
            phi_1704_ = (_e187 * sign(_e133.x));
        } else {
            let _e192 = M_1[0u];
            let _e194 = M_1[1u];
            phi_1704_ = min(min(_e192, abs(_e194)), 1f);
        }
        let _e199 = phi_1704_;
        phi_1703_ = _e199;
    }
    let _e201 = phi_1703_;
    let _e205 = u32(round(((_e201 * 2048f) + 65536f)));
    let _e206 = D0_1;
    let _e209 = ((_e206 << bitcast<u32>(17u)) | _e205);
    let _e212 = atomicMax((&z4_.g2_[_e109]), _e209);
    let _e214 = (_e212 >> bitcast<u32>(17u));
    if (_e214 == _e206) {
        let _e216 = M_1;
        if (_e216.y < 0f) {
            let _e223 = atomicAdd((&z4_.g2_[_e109]), ((_e205 + (_e212 - max(_e209, _e212))) - 65536u));
        }
        phi_1767_ = 0u;
        phi_1762_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e227 = ((f32((_e212 & 131071u)) * 0.00048828125f) + -32f);
        let _e230 = CD.g2_[_e214];
        phi_1706_ = _e227;
        if ((_e230.x & 768u) != 0u) {
            let _e234 = abs(_e227);
            phi_1306_ = Ih;
            if Ih {
                phi_1306_ = ((_e230.x & 512u) != 0u);
            }
            let _e238 = phi_1306_;
            phi_1707_ = _e234;
            if _e238 {
                phi_1707_ = (1f - abs(((fract((_e234 * 0.5f)) * 2f) + -1f)));
            }
            let _e246 = phi_1707_;
            phi_1706_ = _e246;
        }
        let _e248 = phi_1706_;
        let _e249 = clamp(_e248, 0f, 1f);
        phi_1710_ = _e249;
        if Eh {
            let _e251 = (_e230.x >> bitcast<u32>(16u));
            phi_1711_ = _e249;
            if (_e251 != 0u) {
                let _e255 = i0_.g2_[_e109];
                if (_e251 == (_e255 >> bitcast<u32>(16i))) {
                    phi_1708_ = min(_e249, unpack2x16float(_e255).x);
                } else {
                    phi_1708_ = 0f;
                }
                let _e263 = phi_1708_;
                phi_1711_ = _e263;
            }
            let _e265 = phi_1711_;
            phi_1710_ = _e265;
        }
        let _e267 = phi_1710_;
        phi_1343_ = Fh;
        if Fh {
            phi_1343_ = ((_e230.x & 1024u) != 0u);
        }
        let _e271 = phi_1343_;
        phi_1713_ = _e267;
        if _e271 {
            let _e272 = (_e214 * 8u);
            let _e276 = PB.g2_[(_e272 + 2u)];
            let _e287 = PB.g2_[(_e272 + 3u)];
            let _e292 = _e287.zw;
            let _e294 = ((abs(((mat2x2<f32>(vec2<f32>(_e276.x, _e276.y), vec2<f32>(_e276.z, _e276.w)) * _e75) + _e287.xy)) * _e292) - _e292);
            phi_1713_ = min(_e267, clamp((min(_e294.x, _e294.y) + 0.5f), 0f, 1f));
        }
        let _e302 = phi_1713_;
        let _e303 = (_e230.x & 15u);
        if (_e303 <= 1u) {
            let _e313 = (Eh && (_e303 == 0u));
            phi_1751_ = 0u;
            if _e313 {
                phi_1751_ = (_e230.y | pack2x16float(vec2<f32>(_e302, 0f)));
            }
            let _e318 = phi_1751_;
            phi_1750_ = _e318;
            phi_1747_ = select(unpack4x8unorm(_e230.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e313));
        } else {
            let _e321 = (_e214 * 8u);
            let _e324 = PB.g2_[_e321];
            let _e335 = PB.g2_[(_e321 + 1u)];
            let _e338 = ((mat2x2<f32>(vec2<f32>(_e324.x, _e324.y), vec2<f32>(_e324.z, _e324.w)) * _e75) + _e335.xy);
            if (_e303 == 2u) {
                phi_1712_ = _e338.x;
            } else {
                phi_1712_ = length(_e338);
            }
            let _e343 = phi_1712_;
            let _e352 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e343, 0f, 1f) * _e335.z) + _e335.w), bitcast<f32>(_e230.y)), 0f);
            phi_1749_ = _e352;
            if !((Gh && (((_e230.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e356 = (_e352.xyz * _e352.w);
                let _e362 = vec4<f32>(_e356.x, _e352.y, _e352.z, _e352.w);
                let _e368 = vec4<f32>(_e362.x, _e356.y, _e362.z, _e362.w);
                phi_1749_ = vec4<f32>(_e368.x, _e368.y, _e356.z, _e368.w);
            }
            let _e376 = phi_1749_;
            phi_1750_ = 0u;
            phi_1747_ = _e376;
        }
        let _e378 = phi_1750_;
        let _e380 = phi_1747_;
        phi_1767_ = _e378;
        phi_1762_ = (_e380 * _e302);
    }
    let _e383 = phi_1767_;
    let _e385 = phi_1762_;
    let _e386 = _e385.xyz;
    let _e389 = j.F3_;
    let _e391 = j.G3_;
    if (Lh && (_e385.w != 0f)) {
        phi_1764_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e74.x) + (0.00583715f * _e74.y))))) * _e389) + _e391)) + _e386);
    } else {
        phi_1764_ = _e386;
    }
    let _e407 = phi_1764_;
    let _e413 = vec4<f32>(_e407.x, _e385.y, _e385.z, _e385.w);
    let _e419 = vec4<f32>(_e413.x, _e407.y, _e413.z, _e413.w);
    E1_ = vec4<f32>(_e419.x, _e419.y, _e407.z, _e419.w);
    if (_e383 != 0u) {
        i0_.g2_[_e109] = _e383;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) M: vec4<f32>, @location(1) @interpolate(flat, either) D0_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    M_1 = M;
    D0_1 = D0_;
    main_1();
    let _e7 = E1_;
    return _e7;
}
