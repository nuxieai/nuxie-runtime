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

@id(7) override Lh: bool = true;
@id(4) override Ih: bool = true;
@id(0) override Eh: bool = true;
@id(1) override Fh: bool = true;
@id(2) override Gh: bool = true;

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
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;
var<private> c2_1: vec2<f32>;
var<private> Y4_1: f32;
var<private> N0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td;
var<private> B3_1: u32;
var<private> P5_1: vec4<f32>;
var<private> J1_1: vec4<f32>;
var<private> E1_: vec4<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> C1_1: u32;

fn main_1() {
    var phi_1422_: f32;
    var phi_950_: bool;
    var phi_1366_: f32;
    var phi_1365_: f32;
    var phi_1367_: f32;
    var phi_1370_: f32;
    var phi_1369_: f32;
    var phi_987_: bool;
    var phi_1372_: f32;
    var phi_1401_: u32;
    var phi_1371_: f32;
    var phi_1399_: vec4<f32>;
    var phi_1400_: u32;
    var phi_1397_: vec4<f32>;
    var phi_705_: bool;
    var phi_1413_: u32;
    var phi_1429_: f32;
    var phi_1453_: f32;
    var phi_1430_: f32;
    var phi_1431_: f32;
    var phi_1451_: vec4<f32>;
    var phi_1463_: vec3<f32>;

    let _e63 = gl_FragCoord_1;
    let _e64 = _e63.xy;
    let _e67 = bitcast<vec2<u32>>(vec2<i32>(floor(_e64)));
    let _e69 = j.q6_;
    let _e98 = bitcast<i32>((((((_e67.y >> bitcast<u32>(5u)) * (((_e69 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e67.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e67.x & 28u) << bitcast<u32>(5u)) + ((_e67.y & 28u) << bitcast<u32>(2i)))) + (((_e67.y & 3u) << bitcast<u32>(2i)) + (_e67.x & 3u))));
    let _e99 = c2_1;
    let _e100 = textureSample(GC, Y5_, _e99);
    let _e101 = Y4_1;
    let _e102 = min(_e101, 1f);
    phi_1422_ = _e102;
    if Fh {
        let _e103 = N0_1;
        let _e106 = min(_e103.xy, _e103.zw);
        phi_1422_ = clamp(min(_e106.x, _e106.y), 0f, _e102);
    }
    let _e112 = phi_1422_;
    let _e115 = z4_.g2_[_e98];
    let _e117 = (_e115 >> bitcast<u32>(17u));
    let _e121 = ((f32((_e115 & 131071u)) * 0.00048828125f) + -32f);
    let _e124 = CD.g2_[_e117];
    phi_1365_ = _e121;
    if ((_e124.x & 768u) != 0u) {
        let _e128 = abs(_e121);
        phi_950_ = Ih;
        if Ih {
            phi_950_ = ((_e124.x & 512u) != 0u);
        }
        let _e132 = phi_950_;
        phi_1366_ = _e128;
        if _e132 {
            phi_1366_ = (1f - abs(((fract((_e128 * 0.5f)) * 2f) + -1f)));
        }
        let _e140 = phi_1366_;
        phi_1365_ = _e140;
    }
    let _e142 = phi_1365_;
    let _e143 = clamp(_e142, 0f, 1f);
    phi_1369_ = _e143;
    if Eh {
        let _e145 = (_e124.x >> bitcast<u32>(16u));
        phi_1370_ = _e143;
        if (_e145 != 0u) {
            let _e149 = i0_.g2_[_e98];
            if (_e145 == (_e149 >> bitcast<u32>(16i))) {
                phi_1367_ = min(_e143, unpack2x16float(_e149).x);
            } else {
                phi_1367_ = 0f;
            }
            let _e157 = phi_1367_;
            phi_1370_ = _e157;
        }
        let _e159 = phi_1370_;
        phi_1369_ = _e159;
    }
    let _e161 = phi_1369_;
    phi_987_ = Fh;
    if Fh {
        phi_987_ = ((_e124.x & 1024u) != 0u);
    }
    let _e165 = phi_987_;
    phi_1372_ = _e161;
    if _e165 {
        let _e166 = (_e117 * 8u);
        let _e170 = PB.g2_[(_e166 + 2u)];
        let _e181 = PB.g2_[(_e166 + 3u)];
        let _e186 = _e181.zw;
        let _e188 = ((abs(((mat2x2<f32>(vec2<f32>(_e170.x, _e170.y), vec2<f32>(_e170.z, _e170.w)) * _e64) + _e181.xy)) * _e186) - _e186);
        phi_1372_ = min(_e161, clamp((min(_e188.x, _e188.y) + 0.5f), 0f, 1f));
    }
    let _e196 = phi_1372_;
    let _e197 = (_e124.x & 15u);
    if (_e197 <= 1u) {
        let _e207 = (Eh && (_e197 == 0u));
        phi_1401_ = 0u;
        if _e207 {
            phi_1401_ = (_e124.y | pack2x16float(vec2<f32>(_e196, 0f)));
        }
        let _e212 = phi_1401_;
        phi_1400_ = _e212;
        phi_1397_ = select(unpack4x8unorm(_e124.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e207));
    } else {
        let _e215 = (_e117 * 8u);
        let _e218 = PB.g2_[_e215];
        let _e229 = PB.g2_[(_e215 + 1u)];
        let _e232 = ((mat2x2<f32>(vec2<f32>(_e218.x, _e218.y), vec2<f32>(_e218.z, _e218.w)) * _e64) + _e229.xy);
        if (_e197 == 2u) {
            phi_1371_ = _e232.x;
        } else {
            phi_1371_ = length(_e232);
        }
        let _e237 = phi_1371_;
        let _e246 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e237, 0f, 1f) * _e229.z) + _e229.w), bitcast<f32>(_e124.y)), 0f);
        phi_1399_ = _e246;
        if !((Gh && (((_e124.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e250 = (_e246.xyz * _e246.w);
            let _e256 = vec4<f32>(_e250.x, _e246.y, _e246.z, _e246.w);
            let _e262 = vec4<f32>(_e256.x, _e250.y, _e256.z, _e256.w);
            phi_1399_ = vec4<f32>(_e262.x, _e262.y, _e250.z, _e262.w);
        }
        let _e270 = phi_1399_;
        phi_1400_ = 0u;
        phi_1397_ = _e270;
    }
    let _e272 = phi_1400_;
    let _e274 = phi_1397_;
    phi_705_ = Eh;
    if Eh {
        let _e276 = B3_1;
        phi_705_ = (_e276 != 0u);
    }
    let _e279 = phi_705_;
    phi_1453_ = _e112;
    if _e279 {
        if (_e272 != 0u) {
            phi_1413_ = _e272;
        } else {
            let _e283 = i0_.g2_[_e98];
            phi_1413_ = _e283;
        }
        let _e285 = phi_1413_;
        let _e286 = B3_1;
        if (_e286 == (_e285 >> bitcast<u32>(16i))) {
            phi_1429_ = min(_e112, unpack2x16float(_e285).x);
        } else {
            phi_1429_ = 0f;
        }
        let _e294 = phi_1429_;
        phi_1453_ = _e294;
    }
    let _e296 = phi_1453_;
    let _e298 = P5_1[3u];
    phi_1451_ = _e100;
    if (_e298 != 0f) {
        let _e300 = P5_1;
        if (_e300.z > 0f) {
            phi_1430_ = _e300.x;
        } else {
            phi_1430_ = length(_e300.xy);
        }
        let _e307 = phi_1430_;
        let _e308 = clamp(_e307, 0f, 1f);
        let _e309 = abs(_e300.z);
        if (_e309 > 1f) {
            phi_1431_ = ((0.9980469f * _e308) + 0.0009765625f);
        } else {
            phi_1431_ = ((0.001953125f * _e308) + _e309);
        }
        let _e316 = phi_1431_;
        let _e319 = textureSampleLevel(DD, P9_, vec2<f32>(_e316, _e300.w), 0f);
        let _e322 = (_e319.xyz * _e319.w);
        let _e328 = vec4<f32>(_e322.x, _e319.y, _e319.z, _e319.w);
        let _e334 = vec4<f32>(_e328.x, _e322.y, _e328.z, _e328.w);
        phi_1451_ = (_e100 * vec4<f32>(_e334.x, _e334.y, _e322.z, _e334.w));
    }
    let _e343 = phi_1451_;
    let _e344 = J1_1;
    let _e346 = ((_e343 * _e344) * _e296);
    let _e350 = (((_e274 * _e196) * (1f - _e346.w)) + _e346);
    let _e351 = _e350.xyz;
    let _e354 = j.F3_;
    let _e356 = j.G3_;
    if (Lh && (_e350.w != 0f)) {
        phi_1463_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e63.x) + (0.00583715f * _e63.y))))) * _e354) + _e356)) + _e351);
    } else {
        phi_1463_ = _e351;
    }
    let _e372 = phi_1463_;
    let _e378 = vec4<f32>(_e372.x, _e350.y, _e350.z, _e350.w);
    let _e384 = vec4<f32>(_e378.x, _e372.y, _e378.z, _e378.w);
    E1_ = vec4<f32>(_e384.x, _e384.y, _e372.z, _e384.w);
    if (_e272 != 0u) {
        i0_.g2_[_e98] = _e272;
    }
    z4_.g2_[_e98] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) c2_: vec2<f32>, @location(1) Y4_: f32, @location(3) N0_: vec4<f32>, @location(5) @interpolate(flat, either) B3_: u32, @location(2) P5_: vec4<f32>, @location(4) @interpolate(flat, either) J1_: vec4<f32>, @location(6) @interpolate(flat, either) C1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    c2_1 = c2_;
    Y4_1 = Y4_;
    N0_1 = N0_;
    B3_1 = B3_;
    P5_1 = P5_;
    J1_1 = J1_;
    C1_1 = C1_;
    main_1();
    let _e17 = E1_;
    return _e17;
}
