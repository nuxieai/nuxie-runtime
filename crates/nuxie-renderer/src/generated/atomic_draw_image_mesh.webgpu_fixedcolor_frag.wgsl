struct og {
    v2_: array<vec2<u32>>,
}

struct m0Ue {
    v2_: array<u32>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    H8_: vec4<i32>,
    Fi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ji: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Ci: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct T4Ue {
    v2_: array<u32>,
}

@id(7) override lj: bool = true;
@id(4) override ij: bool = true;
@id(0) override ej: bool = true;
@id(1) override fj: bool = true;
@id(2) override gj: bool = true;

@group(0) @binding(3)
var<storage> XC: og;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Ue;
@group(0) @binding(4)
var<storage> JB: pg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var Q8_: sampler;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;
var<private> n2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> T4_: T4Ue;
var<private> U3_1: u32;
var<private> Y1_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> N1_1: u32;

fn main_1() {
    var phi_1548_: f32;
    var phi_1009_: bool;
    var phi_1486_: f32;
    var phi_1485_: f32;
    var phi_1487_: f32;
    var phi_1490_: f32;
    var phi_1489_: f32;
    var phi_1046_: bool;
    var phi_1494_: f32;
    var phi_1491_: f32;
    var phi_1492_: f32;
    var phi_1493_: vec2<f32>;
    var phi_1523_: vec4<f32>;
    var phi_1525_: u32;
    var phi_1524_: u32;
    var phi_1521_: vec4<f32>;
    var phi_781_: bool;
    var phi_1539_: u32;
    var phi_1557_: f32;
    var phi_1577_: f32;
    var phi_1582_: vec3<f32>;

    let _e65 = gl_FragCoord_1;
    let _e66 = _e65.xy;
    let _e69 = bitcast<vec2<u32>>(vec2<i32>(floor(_e66)));
    let _e71 = j.T6_;
    let _e100 = bitcast<i32>((((((_e69.y >> bitcast<u32>(5u)) * (((_e71 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e69.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e69.x & 28u) << bitcast<u32>(5u)) + ((_e69.y & 28u) << bitcast<u32>(2i)))) + (((_e69.y & 3u) << bitcast<u32>(2i)) + (_e69.x & 3u))));
    let _e101 = n2_1;
    let _e102 = textureSample(UB, W4_, _e101);
    phi_1548_ = 1f;
    if fj {
        let _e103 = W0_1;
        let _e106 = min(_e103.xy, _e103.zw);
        phi_1548_ = clamp(min(_e106.x, _e106.y), 0f, 1f);
    }
    let _e112 = phi_1548_;
    let _e115 = T4_.v2_[_e100];
    let _e117 = (_e115 >> bitcast<u32>(17u));
    let _e121 = ((f32((_e115 & 131071u)) * 0.00048828125f) + -32f);
    let _e124 = XC.v2_[_e117];
    phi_1485_ = _e121;
    if ((_e124.x & 768u) != 0u) {
        let _e128 = abs(_e121);
        phi_1009_ = ij;
        if ij {
            phi_1009_ = ((_e124.x & 512u) != 0u);
        }
        let _e132 = phi_1009_;
        phi_1486_ = _e128;
        if _e132 {
            phi_1486_ = (1f - abs(((fract((_e128 * 0.5f)) * 2f) + -1f)));
        }
        let _e140 = phi_1486_;
        phi_1485_ = _e140;
    }
    let _e142 = phi_1485_;
    let _e143 = clamp(_e142, 0f, 1f);
    phi_1489_ = _e143;
    if ej {
        let _e145 = (_e124.x >> bitcast<u32>(16u));
        phi_1490_ = _e143;
        if (_e145 != 0u) {
            let _e149 = m0_.v2_[_e100];
            if (_e145 == (_e149 >> bitcast<u32>(16i))) {
                phi_1487_ = min(_e143, unpack2x16float(_e149).x);
            } else {
                phi_1487_ = 0f;
            }
            let _e157 = phi_1487_;
            phi_1490_ = _e157;
        }
        let _e159 = phi_1490_;
        phi_1489_ = _e159;
    }
    let _e161 = phi_1489_;
    phi_1046_ = fj;
    if fj {
        phi_1046_ = ((_e124.x & 1024u) != 0u);
    }
    let _e165 = phi_1046_;
    phi_1494_ = _e161;
    if _e165 {
        let _e166 = (_e117 * 8u);
        let _e170 = JB.v2_[(_e166 + 2u)];
        let _e181 = JB.v2_[(_e166 + 3u)];
        let _e186 = _e181.zw;
        let _e188 = ((abs(((mat2x2<f32>(vec2<f32>(_e170.x, _e170.y), vec2<f32>(_e170.z, _e170.w)) * _e66) + _e181.xy)) * _e186) - _e186);
        phi_1494_ = min(_e161, clamp((min(_e188.x, _e188.y) + 0.5f), 0f, 1f));
    }
    let _e196 = phi_1494_;
    let _e197 = (_e124.x & 15u);
    if (_e197 <= 1u) {
        let _e282 = (ej && (_e197 == 0u));
        phi_1525_ = 0u;
        if _e282 {
            phi_1525_ = (_e124.y | pack2x16float(vec2<f32>(_e196, 0f)));
        }
        let _e287 = phi_1525_;
        phi_1524_ = _e287;
        phi_1521_ = select(unpack4x8unorm(_e124.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e282));
    } else {
        let _e204 = (_e117 * 8u);
        let _e207 = JB.v2_[_e204];
        let _e218 = JB.v2_[(_e204 + 1u)];
        let _e221 = ((mat2x2<f32>(vec2<f32>(_e207.x, _e207.y), vec2<f32>(_e207.z, _e207.w)) * _e66) + _e218.xy);
        let _e225 = f32(_e197);
        let _e227 = j.O8_;
        let _e229 = j.P8_;
        let _e234 = ((_e218.w * _e227) + _e229);
        switch bitcast<i32>(0u) {
            default: {
                if (_e225 == 4f) {
                    let _e251 = normalize(vec2<f32>(_e221.x, -(_e221.y)));
                    let _e254 = acos(clamp(_e251.x, -1f, 1f));
                    if (_e251.y >= 0f) {
                        phi_1492_ = _e254;
                    } else {
                        phi_1492_ = -(_e254);
                    }
                    let _e259 = phi_1492_;
                    phi_1493_ = vec2<f32>((_e259 * 0.15915507f), _e234);
                    break;
                } else {
                    if (_e225 == 2f) {
                        phi_1491_ = _e221.x;
                    } else {
                        phi_1491_ = length(_e221);
                    }
                    let _e241 = phi_1491_;
                    phi_1493_ = vec2<f32>(((clamp(_e241, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e218.z < 0f))) + ((max(0f, _e218.z) * 0.001953125f) + 0.0009765625f)), _e234);
                    break;
                }
            }
        }
        let _e263 = phi_1493_;
        let _e264 = textureSampleLevel(ZC, Q8_, _e263, 0f);
        phi_1523_ = _e264;
        if !((gj && (((_e124.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e268 = (_e264.xyz * _e264.w);
            phi_1523_ = vec4<f32>(_e268.x, _e268.y, _e268.z, (_e264.w * abs(bitcast<f32>(_e124.y))));
        }
        let _e278 = phi_1523_;
        phi_1524_ = 0u;
        phi_1521_ = _e278;
    }
    let _e291 = phi_1524_;
    let _e293 = phi_1521_;
    phi_781_ = ej;
    if ej {
        let _e295 = U3_1;
        phi_781_ = (_e295 != 0u);
    }
    let _e298 = phi_781_;
    phi_1577_ = _e112;
    if _e298 {
        if (_e291 != 0u) {
            phi_1539_ = _e291;
        } else {
            let _e302 = m0_.v2_[_e100];
            phi_1539_ = _e302;
        }
        let _e304 = phi_1539_;
        let _e305 = U3_1;
        if (_e305 == (_e304 >> bitcast<u32>(16i))) {
            phi_1557_ = min(_e112, unpack2x16float(_e304).x);
        } else {
            phi_1557_ = 0f;
        }
        let _e313 = phi_1557_;
        phi_1577_ = _e313;
    }
    let _e315 = phi_1577_;
    let _e316 = Y1_1;
    let _e318 = ((_e102 * _e316) * _e315);
    let _e322 = (((_e293 * _e196) * (1f - _e318.w)) + _e318);
    let _e323 = _e322.xyz;
    let _e326 = j.G3_;
    let _e328 = j.H3_;
    if (lj && (_e322.w != 0f)) {
        phi_1582_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e65.x) + (0.00583715f * _e65.y))))) * _e326) + _e328)) + _e323);
    } else {
        phi_1582_ = _e323;
    }
    let _e344 = phi_1582_;
    let _e350 = vec4<f32>(_e344.x, _e322.y, _e322.z, _e322.w);
    let _e356 = vec4<f32>(_e350.x, _e344.y, _e350.z, _e350.w);
    Q1_ = vec4<f32>(_e356.x, _e356.y, _e344.z, _e356.w);
    if (_e291 != 0u) {
        m0_.v2_[_e100] = _e291;
    }
    T4_.v2_[_e100] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) n2_: vec2<f32>, @location(1) W0_: vec4<f32>, @location(4) @interpolate(flat, either) U3_: u32, @location(3) @interpolate(flat, either) Y1_: vec4<f32>, @location(5) @interpolate(flat, either) N1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    n2_1 = n2_;
    W0_1 = W0_;
    U3_1 = U3_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
    let _e13 = Q1_;
    return _e13;
}
