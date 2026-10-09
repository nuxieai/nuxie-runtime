struct jg {
    r2_: array<vec2<u32>>,
}

struct m0Pe {
    r2_: array<u32>,
}

struct kg {
    r2_: array<vec4<f32>>,
}

struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

struct R4Pe {
    r2_: array<u32>,
}

@id(7) override dj: bool = true;
@id(4) override aj: bool = true;
@id(0) override Wi: bool = true;
@id(1) override Xi: bool = true;
@id(2) override Yi: bool = true;

@group(0) @binding(3)
var<storage> VC: jg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var N8_: sampler;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;
var<private> l2_1: vec2<f32>;
var<private> V0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe;
var<private> S3_1: u32;
var<private> T1_1: vec4<f32>;
var<private> L1_: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> J1_1: u32;

fn main_1() {
    var phi_1440_: f32;
    var phi_956_: bool;
    var phi_1384_: f32;
    var phi_1383_: f32;
    var phi_1385_: f32;
    var phi_1388_: f32;
    var phi_1387_: f32;
    var phi_993_: bool;
    var phi_1390_: f32;
    var phi_1419_: u32;
    var phi_1389_: f32;
    var phi_1417_: vec4<f32>;
    var phi_1418_: u32;
    var phi_1415_: vec4<f32>;
    var phi_735_: bool;
    var phi_1431_: u32;
    var phi_1447_: f32;
    var phi_1465_: f32;
    var phi_1470_: vec3<f32>;

    let _e63 = gl_FragCoord_1;
    let _e64 = _e63.xy;
    let _e67 = bitcast<vec2<u32>>(vec2<i32>(floor(_e64)));
    let _e69 = j.P6_;
    let _e98 = bitcast<i32>((((((_e67.y >> bitcast<u32>(5u)) * (((_e69 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e67.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e67.x & 28u) << bitcast<u32>(5u)) + ((_e67.y & 28u) << bitcast<u32>(2i)))) + (((_e67.y & 3u) << bitcast<u32>(2i)) + (_e67.x & 3u))));
    let _e99 = l2_1;
    let _e100 = textureSample(TB, U4_, _e99);
    phi_1440_ = 1f;
    if Xi {
        let _e101 = V0_1;
        let _e104 = min(_e101.xy, _e101.zw);
        phi_1440_ = clamp(min(_e104.x, _e104.y), 0f, 1f);
    }
    let _e110 = phi_1440_;
    let _e113 = R4_.r2_[_e98];
    let _e115 = (_e113 >> bitcast<u32>(17u));
    let _e119 = ((f32((_e113 & 131071u)) * 0.00048828125f) + -32f);
    let _e122 = VC.r2_[_e115];
    phi_1383_ = _e119;
    if ((_e122.x & 768u) != 0u) {
        let _e126 = abs(_e119);
        phi_956_ = aj;
        if aj {
            phi_956_ = ((_e122.x & 512u) != 0u);
        }
        let _e130 = phi_956_;
        phi_1384_ = _e126;
        if _e130 {
            phi_1384_ = (1f - abs(((fract((_e126 * 0.5f)) * 2f) + -1f)));
        }
        let _e138 = phi_1384_;
        phi_1383_ = _e138;
    }
    let _e140 = phi_1383_;
    let _e141 = clamp(_e140, 0f, 1f);
    phi_1387_ = _e141;
    if Wi {
        let _e143 = (_e122.x >> bitcast<u32>(16u));
        phi_1388_ = _e141;
        if (_e143 != 0u) {
            let _e147 = m0_.r2_[_e98];
            if (_e143 == (_e147 >> bitcast<u32>(16i))) {
                phi_1385_ = min(_e141, unpack2x16float(_e147).x);
            } else {
                phi_1385_ = 0f;
            }
            let _e155 = phi_1385_;
            phi_1388_ = _e155;
        }
        let _e157 = phi_1388_;
        phi_1387_ = _e157;
    }
    let _e159 = phi_1387_;
    phi_993_ = Xi;
    if Xi {
        phi_993_ = ((_e122.x & 1024u) != 0u);
    }
    let _e163 = phi_993_;
    phi_1390_ = _e159;
    if _e163 {
        let _e164 = (_e115 * 8u);
        let _e168 = JB.r2_[(_e164 + 2u)];
        let _e179 = JB.r2_[(_e164 + 3u)];
        let _e184 = _e179.zw;
        let _e186 = ((abs(((mat2x2<f32>(vec2<f32>(_e168.x, _e168.y), vec2<f32>(_e168.z, _e168.w)) * _e64) + _e179.xy)) * _e184) - _e184);
        phi_1390_ = min(_e159, clamp((min(_e186.x, _e186.y) + 0.5f), 0f, 1f));
    }
    let _e194 = phi_1390_;
    let _e195 = (_e122.x & 15u);
    if (_e195 <= 1u) {
        let _e205 = (Wi && (_e195 == 0u));
        phi_1419_ = 0u;
        if _e205 {
            phi_1419_ = (_e122.y | pack2x16float(vec2<f32>(_e194, 0f)));
        }
        let _e210 = phi_1419_;
        phi_1418_ = _e210;
        phi_1415_ = select(unpack4x8unorm(_e122.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e205));
    } else {
        let _e213 = (_e115 * 8u);
        let _e216 = JB.r2_[_e213];
        let _e227 = JB.r2_[(_e213 + 1u)];
        let _e230 = ((mat2x2<f32>(vec2<f32>(_e216.x, _e216.y), vec2<f32>(_e216.z, _e216.w)) * _e64) + _e227.xy);
        let _e236 = j.L8_;
        let _e238 = j.M8_;
        if (f32(_e195) == 2f) {
            phi_1389_ = _e230.x;
        } else {
            phi_1389_ = length(_e230);
        }
        let _e248 = phi_1389_;
        let _e254 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e248, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e227.z < 0f))) + ((max(0f, _e227.z) * 0.001953125f) + 0.0009765625f)), ((_e227.w * _e236) + _e238)), 0f);
        phi_1417_ = _e254;
        if !((Yi && (((_e122.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e258 = (_e254.xyz * _e254.w);
            phi_1417_ = vec4<f32>(_e258.x, _e258.y, _e258.z, (_e254.w * abs(bitcast<f32>(_e122.y))));
        }
        let _e268 = phi_1417_;
        phi_1418_ = 0u;
        phi_1415_ = _e268;
    }
    let _e270 = phi_1418_;
    let _e272 = phi_1415_;
    phi_735_ = Wi;
    if Wi {
        let _e274 = S3_1;
        phi_735_ = (_e274 != 0u);
    }
    let _e277 = phi_735_;
    phi_1465_ = _e110;
    if _e277 {
        if (_e270 != 0u) {
            phi_1431_ = _e270;
        } else {
            let _e281 = m0_.r2_[_e98];
            phi_1431_ = _e281;
        }
        let _e283 = phi_1431_;
        let _e284 = S3_1;
        if (_e284 == (_e283 >> bitcast<u32>(16i))) {
            phi_1447_ = min(_e110, unpack2x16float(_e283).x);
        } else {
            phi_1447_ = 0f;
        }
        let _e292 = phi_1447_;
        phi_1465_ = _e292;
    }
    let _e294 = phi_1465_;
    let _e295 = T1_1;
    let _e297 = ((_e100 * _e295) * _e294);
    let _e301 = (((_e272 * _e194) * (1f - _e297.w)) + _e297);
    let _e302 = _e301.xyz;
    let _e305 = j.F3_;
    let _e307 = j.G3_;
    if (dj && (_e301.w != 0f)) {
        phi_1470_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e63.x) + (0.00583715f * _e63.y))))) * _e305) + _e307)) + _e302);
    } else {
        phi_1470_ = _e302;
    }
    let _e323 = phi_1470_;
    let _e329 = vec4<f32>(_e323.x, _e301.y, _e301.z, _e301.w);
    let _e335 = vec4<f32>(_e329.x, _e323.y, _e329.z, _e329.w);
    L1_ = vec4<f32>(_e335.x, _e335.y, _e323.z, _e335.w);
    if (_e270 != 0u) {
        m0_.r2_[_e98] = _e270;
    }
    R4_.r2_[_e98] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) l2_: vec2<f32>, @location(1) V0_: vec4<f32>, @location(4) @interpolate(flat, either) S3_: u32, @location(3) @interpolate(flat, either) T1_: vec4<f32>, @location(5) @interpolate(flat, either) J1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    l2_1 = l2_;
    V0_1 = V0_;
    S3_1 = S3_;
    T1_1 = T1_;
    J1_1 = J1_;
    main_1();
    let _e13 = L1_;
    return _e13;
}
