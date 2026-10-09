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
var<private> q5_1: f32;
var<private> V0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe;
var<private> S3_1: u32;
var<private> r5_1: vec4<f32>;
var<private> T1_1: vec4<f32>;
var<private> L1_: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> J1_1: u32;

fn main_1() {
    var phi_1631_: f32;
    var phi_1048_: bool;
    var phi_1575_: f32;
    var phi_1574_: f32;
    var phi_1576_: f32;
    var phi_1579_: f32;
    var phi_1578_: f32;
    var phi_1085_: bool;
    var phi_1581_: f32;
    var phi_1610_: u32;
    var phi_1580_: f32;
    var phi_1608_: vec4<f32>;
    var phi_1609_: u32;
    var phi_1606_: vec4<f32>;
    var phi_791_: bool;
    var phi_1622_: u32;
    var phi_1638_: f32;
    var phi_1677_: f32;
    var phi_1656_: f32;
    var phi_1675_: vec4<f32>;
    var phi_1685_: vec3<f32>;

    let _e66 = gl_FragCoord_1;
    let _e67 = _e66.xy;
    let _e70 = bitcast<vec2<u32>>(vec2<i32>(floor(_e67)));
    let _e72 = j.P6_;
    let _e101 = bitcast<i32>((((((_e70.y >> bitcast<u32>(5u)) * (((_e72 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e70.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e70.x & 28u) << bitcast<u32>(5u)) + ((_e70.y & 28u) << bitcast<u32>(2i)))) + (((_e70.y & 3u) << bitcast<u32>(2i)) + (_e70.x & 3u))));
    let _e102 = l2_1;
    let _e103 = textureSample(TB, U4_, _e102);
    let _e104 = q5_1;
    let _e105 = min(_e104, 1f);
    phi_1631_ = _e105;
    if Xi {
        let _e106 = V0_1;
        let _e109 = min(_e106.xy, _e106.zw);
        phi_1631_ = clamp(min(_e109.x, _e109.y), 0f, _e105);
    }
    let _e115 = phi_1631_;
    let _e118 = R4_.r2_[_e101];
    let _e120 = (_e118 >> bitcast<u32>(17u));
    let _e124 = ((f32((_e118 & 131071u)) * 0.00048828125f) + -32f);
    let _e127 = VC.r2_[_e120];
    phi_1574_ = _e124;
    if ((_e127.x & 768u) != 0u) {
        let _e131 = abs(_e124);
        phi_1048_ = aj;
        if aj {
            phi_1048_ = ((_e127.x & 512u) != 0u);
        }
        let _e135 = phi_1048_;
        phi_1575_ = _e131;
        if _e135 {
            phi_1575_ = (1f - abs(((fract((_e131 * 0.5f)) * 2f) + -1f)));
        }
        let _e143 = phi_1575_;
        phi_1574_ = _e143;
    }
    let _e145 = phi_1574_;
    let _e146 = clamp(_e145, 0f, 1f);
    phi_1578_ = _e146;
    if Wi {
        let _e148 = (_e127.x >> bitcast<u32>(16u));
        phi_1579_ = _e146;
        if (_e148 != 0u) {
            let _e152 = m0_.r2_[_e101];
            if (_e148 == (_e152 >> bitcast<u32>(16i))) {
                phi_1576_ = min(_e146, unpack2x16float(_e152).x);
            } else {
                phi_1576_ = 0f;
            }
            let _e160 = phi_1576_;
            phi_1579_ = _e160;
        }
        let _e162 = phi_1579_;
        phi_1578_ = _e162;
    }
    let _e164 = phi_1578_;
    phi_1085_ = Xi;
    if Xi {
        phi_1085_ = ((_e127.x & 1024u) != 0u);
    }
    let _e168 = phi_1085_;
    phi_1581_ = _e164;
    if _e168 {
        let _e169 = (_e120 * 8u);
        let _e173 = JB.r2_[(_e169 + 2u)];
        let _e184 = JB.r2_[(_e169 + 3u)];
        let _e189 = _e184.zw;
        let _e191 = ((abs(((mat2x2<f32>(vec2<f32>(_e173.x, _e173.y), vec2<f32>(_e173.z, _e173.w)) * _e67) + _e184.xy)) * _e189) - _e189);
        phi_1581_ = min(_e164, clamp((min(_e191.x, _e191.y) + 0.5f), 0f, 1f));
    }
    let _e199 = phi_1581_;
    let _e200 = (_e127.x & 15u);
    if (_e200 <= 1u) {
        let _e210 = (Wi && (_e200 == 0u));
        phi_1610_ = 0u;
        if _e210 {
            phi_1610_ = (_e127.y | pack2x16float(vec2<f32>(_e199, 0f)));
        }
        let _e215 = phi_1610_;
        phi_1609_ = _e215;
        phi_1606_ = select(unpack4x8unorm(_e127.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e210));
    } else {
        let _e218 = (_e120 * 8u);
        let _e221 = JB.r2_[_e218];
        let _e232 = JB.r2_[(_e218 + 1u)];
        let _e235 = ((mat2x2<f32>(vec2<f32>(_e221.x, _e221.y), vec2<f32>(_e221.z, _e221.w)) * _e67) + _e232.xy);
        let _e241 = j.L8_;
        let _e243 = j.M8_;
        if (f32(_e200) == 2f) {
            phi_1580_ = _e235.x;
        } else {
            phi_1580_ = length(_e235);
        }
        let _e253 = phi_1580_;
        let _e259 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e253, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e232.z < 0f))) + ((max(0f, _e232.z) * 0.001953125f) + 0.0009765625f)), ((_e232.w * _e241) + _e243)), 0f);
        phi_1608_ = _e259;
        if !((Yi && (((_e127.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e263 = (_e259.xyz * _e259.w);
            phi_1608_ = vec4<f32>(_e263.x, _e263.y, _e263.z, (_e259.w * abs(bitcast<f32>(_e127.y))));
        }
        let _e273 = phi_1608_;
        phi_1609_ = 0u;
        phi_1606_ = _e273;
    }
    let _e275 = phi_1609_;
    let _e277 = phi_1606_;
    phi_791_ = Wi;
    if Wi {
        let _e279 = S3_1;
        phi_791_ = (_e279 != 0u);
    }
    let _e282 = phi_791_;
    phi_1677_ = _e115;
    if _e282 {
        if (_e275 != 0u) {
            phi_1622_ = _e275;
        } else {
            let _e286 = m0_.r2_[_e101];
            phi_1622_ = _e286;
        }
        let _e288 = phi_1622_;
        let _e289 = S3_1;
        if (_e289 == (_e288 >> bitcast<u32>(16i))) {
            phi_1638_ = min(_e115, unpack2x16float(_e288).x);
        } else {
            phi_1638_ = 0f;
        }
        let _e297 = phi_1638_;
        phi_1677_ = _e297;
    }
    let _e299 = phi_1677_;
    let _e301 = r5_1[3u];
    phi_1675_ = _e103;
    if (_e301 != 0f) {
        let _e303 = r5_1;
        let _e305 = j.L8_;
        let _e307 = j.M8_;
        let _e310 = abs(_e303.z);
        let _e311 = floor(_e310);
        let _e314 = (((_e310 - _e311) * 8f) + 0.0009765625f);
        if (floor(_e314) == 2f) {
            phi_1656_ = _e303.x;
        } else {
            phi_1656_ = length(_e303.xy);
        }
        let _e323 = phi_1656_;
        let _e329 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e323, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e303.z < 0f))) + _e314), ((_e311 * _e305) + _e307)), 0f);
        let _e332 = (_e329.xyz * _e329.w);
        let _e338 = vec4<f32>(_e332.x, _e329.y, _e329.z, _e329.w);
        let _e344 = vec4<f32>(_e338.x, _e332.y, _e338.z, _e338.w);
        phi_1675_ = (_e103 * vec4<f32>(_e344.x, _e344.y, _e332.z, _e344.w));
    }
    let _e353 = phi_1675_;
    let _e354 = T1_1;
    let _e356 = ((_e353 * _e354) * _e299);
    let _e360 = (((_e277 * _e199) * (1f - _e356.w)) + _e356);
    let _e361 = _e360.xyz;
    let _e364 = j.F3_;
    let _e366 = j.G3_;
    if (dj && (_e360.w != 0f)) {
        phi_1685_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e66.x) + (0.00583715f * _e66.y))))) * _e364) + _e366)) + _e361);
    } else {
        phi_1685_ = _e361;
    }
    let _e382 = phi_1685_;
    let _e388 = vec4<f32>(_e382.x, _e360.y, _e360.z, _e360.w);
    let _e394 = vec4<f32>(_e388.x, _e382.y, _e388.z, _e388.w);
    L1_ = vec4<f32>(_e394.x, _e394.y, _e382.z, _e394.w);
    if (_e275 != 0u) {
        m0_.r2_[_e101] = _e275;
    }
    R4_.r2_[_e101] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) l2_: vec2<f32>, @location(1) q5_: f32, @location(3) V0_: vec4<f32>, @location(5) @interpolate(flat, either) S3_: u32, @location(2) r5_: vec4<f32>, @location(4) @interpolate(flat, either) T1_: vec4<f32>, @location(6) @interpolate(flat, either) J1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    l2_1 = l2_;
    q5_1 = q5_;
    V0_1 = V0_;
    S3_1 = S3_;
    r5_1 = r5_;
    T1_1 = T1_;
    J1_1 = J1_;
    main_1();
    let _e17 = L1_;
    return _e17;
}
