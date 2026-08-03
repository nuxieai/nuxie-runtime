struct He {
    c2_: array<vec2<u32>>,
}

struct h0zd {
    c2_: array<u32>,
}

struct Ie {
    c2_: array<vec4<f32>>,
}

struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

struct q4zd {
    c2_: array<u32>,
}

struct q4zd_1 {
    c2_: array<atomic<u32>>,
}

@id(7) override dh: bool = true;
@id(4) override ah: bool = true;
@id(0) override Wg: bool = true;
@id(1) override Xg: bool = true;
@id(3) override Zg: bool = true;

@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(3)
var<storage> AD: He;
@group(2) @binding(1)
var<storage, read_write> h0_: h0zd;
@group(0) @binding(4)
var<storage> RB: Ie;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Kb: sampler;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> O_1: vec4<f32>;
var<private> B0_1: u32;
@group(2) @binding(3)
var<storage, read_write> q4_: q4zd_1;
var<private> C1_: vec4<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;

fn main_1() {
    var phi_794_: bool;
    var phi_807_: bool;
    var phi_1657_: f32;
    var phi_1665_: f32;
    var phi_1673_: f32;
    var phi_1672_: f32;
    var phi_1288_: bool;
    var phi_1676_: f32;
    var phi_1675_: f32;
    var phi_1677_: f32;
    var phi_1680_: f32;
    var phi_1679_: f32;
    var phi_1325_: bool;
    var phi_1682_: f32;
    var phi_1718_: u32;
    var phi_1681_: f32;
    var phi_1717_: u32;
    var phi_1715_: vec4<f32>;
    var phi_1733_: u32;
    var phi_1728_: vec4<f32>;
    var phi_1730_: vec3<f32>;

    let _e73 = gl_FragCoord_1;
    let _e74 = _e73.xy;
    let _e77 = bitcast<vec2<u32>>(vec2<i32>(floor(_e74)));
    let _e79 = n.m6_;
    let _e108 = bitcast<i32>((((((_e77.y >> bitcast<u32>(5u)) * (((_e79 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e77.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e77.x & 28u) << bitcast<u32>(5u)) + ((_e77.y & 28u) << bitcast<u32>(2i)))) + (((_e77.y & 3u) << bitcast<u32>(2i)) + (_e77.x & 3u))));
    phi_794_ = Zg;
    if Zg {
        let _e109 = O_1;
        phi_794_ = (_e109.x < -1.5f);
    }
    let _e113 = phi_794_;
    if _e113 {
        let _e114 = O_1;
        let _e118 = textureSampleLevel(XC, aa, vec2<f32>((3f + _e114.x), 0f), 0f);
        let _e124 = textureSampleLevel(XC, aa, vec2<f32>((1f - _e114.y), 0f), 0f);
        phi_1672_ = ((1f - _e118.x) - _e124.x);
    } else {
        phi_807_ = Zg;
        if Zg {
            let _e127 = O_1;
            phi_807_ = (_e127.y < -1.5f);
        }
        let _e131 = phi_807_;
        if _e131 {
            let _e132 = O_1;
            let _e135 = max(_e132.w, 0f);
            if (_e132.z >= 0f) {
                let _e138 = textureSampleLevel(XC, aa, vec2<f32>(_e135, 0f), 0f);
                phi_1657_ = _e138.x;
            } else {
                phi_1657_ = 0f;
            }
            let _e141 = phi_1657_;
            phi_1665_ = _e141;
            if (abs(_e132.z) < 1000f) {
                let _e148 = (-2f - _e132.y);
                let _e150 = ((_e148 - _e135) * 0.5984134f);
                let _e153 = (vec4(_e135) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e150));
                let _e159 = ((_e153 * -(_e132.z)) + vec4(((_e148 * _e132.z) + (abs(_e132.x) - 0.25f))));
                let _e162 = textureSampleLevel(XC, aa, vec2<f32>(_e159.x, 0f), 0f);
                let _e165 = textureSampleLevel(XC, aa, vec2<f32>(_e159.y, 0f), 0f);
                let _e168 = textureSampleLevel(XC, aa, vec2<f32>(_e159.z, 0f), 0f);
                let _e171 = textureSampleLevel(XC, aa, vec2<f32>(_e159.w, 0f), 0f);
                let _e177 = (_e153 * 5.0959306f);
                phi_1665_ = (_e141 + (dot(vec4<f32>(_e162.x, _e165.x, _e168.x, _e171.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e177) * (_e177 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e150));
            }
            let _e186 = phi_1665_;
            phi_1673_ = (_e186 * sign(_e132.x));
        } else {
            let _e191 = O_1[0u];
            let _e193 = O_1[1u];
            phi_1673_ = min(min(_e191, abs(_e193)), 1f);
        }
        let _e198 = phi_1673_;
        phi_1672_ = _e198;
    }
    let _e200 = phi_1672_;
    let _e204 = u32(round(((_e200 * 2048f) + 65536f)));
    let _e205 = B0_1;
    let _e208 = ((_e205 << bitcast<u32>(17u)) | _e204);
    let _e211 = atomicMax((&q4_.c2_[_e108]), _e208);
    let _e213 = (_e211 >> bitcast<u32>(17u));
    if (_e213 == _e205) {
        let _e215 = O_1;
        if (_e215.y < 0f) {
            let _e222 = atomicAdd((&q4_.c2_[_e108]), ((_e204 + (_e211 - max(_e208, _e211))) - 65536u));
        }
        phi_1733_ = 0u;
        phi_1728_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e226 = ((f32((_e211 & 131071u)) * 0.00048828125f) + -32f);
        let _e229 = AD.c2_[_e213];
        phi_1675_ = _e226;
        if ((_e229.x & 768u) != 0u) {
            let _e233 = abs(_e226);
            phi_1288_ = ah;
            if ah {
                phi_1288_ = ((_e229.x & 512u) != 0u);
            }
            let _e237 = phi_1288_;
            phi_1676_ = _e233;
            if _e237 {
                phi_1676_ = (1f - abs(((fract((_e233 * 0.5f)) * 2f) + -1f)));
            }
            let _e245 = phi_1676_;
            phi_1675_ = _e245;
        }
        let _e247 = phi_1675_;
        let _e248 = clamp(_e247, 0f, 1f);
        phi_1679_ = _e248;
        if Wg {
            let _e250 = (_e229.x >> bitcast<u32>(16u));
            phi_1680_ = _e248;
            if (_e250 != 0u) {
                let _e254 = h0_.c2_[_e108];
                if (_e250 == (_e254 >> bitcast<u32>(16i))) {
                    phi_1677_ = min(_e248, unpack2x16float(_e254).x);
                } else {
                    phi_1677_ = 0f;
                }
                let _e262 = phi_1677_;
                phi_1680_ = _e262;
            }
            let _e264 = phi_1680_;
            phi_1679_ = _e264;
        }
        let _e266 = phi_1679_;
        phi_1325_ = Xg;
        if Xg {
            phi_1325_ = ((_e229.x & 1024u) != 0u);
        }
        let _e270 = phi_1325_;
        phi_1682_ = _e266;
        if _e270 {
            let _e271 = (_e213 * 4u);
            let _e275 = RB.c2_[(_e271 + 2u)];
            let _e286 = RB.c2_[(_e271 + 3u)];
            let _e291 = _e286.zw;
            let _e293 = ((abs(((mat2x2<f32>(vec2<f32>(_e275.x, _e275.y), vec2<f32>(_e275.z, _e275.w)) * _e74) + _e286.xy)) * _e291) - _e291);
            phi_1682_ = min(_e266, clamp((min(_e293.x, _e293.y) + 0.5f), 0f, 1f));
        }
        let _e301 = phi_1682_;
        let _e302 = (_e229.x & 15u);
        if (_e302 <= 1u) {
            let _e307 = (Wg && (_e302 == 0u));
            phi_1718_ = 0u;
            if _e307 {
                phi_1718_ = (_e229.y | pack2x16float(vec2<f32>(_e301, 0f)));
            }
            let _e312 = phi_1718_;
            phi_1717_ = _e312;
            phi_1715_ = select(unpack4x8unorm(_e229.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e307));
        } else {
            let _e315 = (_e213 * 4u);
            let _e318 = RB.c2_[_e315];
            let _e329 = RB.c2_[(_e315 + 1u)];
            let _e332 = ((mat2x2<f32>(vec2<f32>(_e318.x, _e318.y), vec2<f32>(_e318.z, _e318.w)) * _e74) + _e329.xy);
            if (_e302 == 2u) {
                phi_1681_ = _e332.x;
            } else {
                phi_1681_ = length(_e332);
            }
            let _e337 = phi_1681_;
            let _e346 = textureSampleLevel(KD, Kb, vec2<f32>(((clamp(_e337, 0f, 1f) * _e329.z) + _e329.w), bitcast<f32>(_e229.y)), 0f);
            phi_1717_ = 0u;
            phi_1715_ = _e346;
        }
        let _e348 = phi_1717_;
        let _e350 = phi_1715_;
        let _e352 = (_e350.w * _e301);
        let _e354 = (_e350.xyz * _e352);
        phi_1733_ = _e348;
        phi_1728_ = vec4<f32>(_e354.x, _e354.y, _e354.z, _e352);
    }
    let _e360 = phi_1733_;
    let _e362 = phi_1728_;
    let _e363 = _e362.xyz;
    let _e366 = n.z3_;
    let _e368 = n.A3_;
    if (dh && (_e362.w != 0f)) {
        phi_1730_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e73.x) + (0.00583715f * _e73.y))))) * _e366) + _e368)) + _e363);
    } else {
        phi_1730_ = _e363;
    }
    let _e384 = phi_1730_;
    let _e390 = vec4<f32>(_e384.x, _e362.y, _e362.z, _e362.w);
    let _e396 = vec4<f32>(_e390.x, _e384.y, _e390.z, _e390.w);
    C1_ = vec4<f32>(_e396.x, _e396.y, _e384.z, _e396.w);
    if (_e360 != 0u) {
        h0_.c2_[_e108] = _e360;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) O: vec4<f32>, @location(1) @interpolate(flat, either) B0_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    O_1 = O;
    B0_1 = B0_;
    main_1();
    let _e7 = C1_;
    return _e7;
}
