struct hf {
    e2_: array<vec2<u32>>,
}

struct h0Qd {
    e2_: array<u32>,
}

struct jf {
    e2_: array<vec4<f32>>,
}

struct BC {
    qc: f32,
    Ad: f32,
    Ef: f32,
    Ff: f32,
    q6_: u32,
    Nb: u32,
    qf: u32,
    rf: u32,
    V7_: vec4<i32>,
    bh: vec2<f32>,
    Bd: vec2<f32>,
    d2_: u32,
    fh: f32,
    e6_: u32,
    T2_: f32,
    Cd: f32,
    lf: u32,
    B3_: f32,
    C3_: f32,
    Dd: f32,
    Yg: u32,
}

struct x4Qd {
    e2_: array<u32>,
}

@id(7) override Ih: bool = true;
@id(4) override Fh: bool = true;
@id(0) override Bh: bool = true;
@id(1) override Ch: bool = true;

@group(0) @binding(3)
var<storage> DD: hf;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Qd;
@group(0) @binding(4)
var<storage> QB: jf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(0) @binding(0)
var<uniform> n: BC;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;
var<private> Z1_1: vec2<f32>;
var<private> V4_1: f32;
var<private> M0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> x4_: x4Qd;
var<private> x3_1: u32;
var<private> M5_1: vec4<f32>;
var<private> H1_1: vec4<f32>;
var<private> C1_: vec4<f32>;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
var<private> A1_1: u32;

fn main_1() {
    var phi_1389_: f32;
    var phi_933_: bool;
    var phi_1336_: f32;
    var phi_1335_: f32;
    var phi_1337_: f32;
    var phi_1340_: f32;
    var phi_1339_: f32;
    var phi_970_: bool;
    var phi_1342_: f32;
    var phi_1369_: u32;
    var phi_1341_: f32;
    var phi_1368_: u32;
    var phi_1366_: vec4<f32>;
    var phi_690_: bool;
    var phi_1380_: u32;
    var phi_1395_: f32;
    var phi_1418_: f32;
    var phi_1396_: f32;
    var phi_1397_: f32;
    var phi_1416_: vec4<f32>;
    var phi_1428_: vec3<f32>;

    let _e62 = gl_FragCoord_1;
    let _e63 = _e62.xy;
    let _e66 = bitcast<vec2<u32>>(vec2<i32>(floor(_e63)));
    let _e68 = n.q6_;
    let _e97 = bitcast<i32>((((((_e66.y >> bitcast<u32>(5u)) * (((_e68 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e66.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e66.x & 28u) << bitcast<u32>(5u)) + ((_e66.y & 28u) << bitcast<u32>(2i)))) + (((_e66.y & 3u) << bitcast<u32>(2i)) + (_e66.x & 3u))));
    let _e98 = Z1_1;
    let _e99 = textureSample(HC, W5_, _e98);
    let _e100 = V4_1;
    let _e101 = min(_e100, 1f);
    phi_1389_ = _e101;
    if Ch {
        let _e102 = M0_1;
        let _e105 = min(_e102.xy, _e102.zw);
        phi_1389_ = clamp(min(_e105.x, _e105.y), 0f, _e101);
    }
    let _e111 = phi_1389_;
    let _e114 = x4_.e2_[_e97];
    let _e116 = (_e114 >> bitcast<u32>(17u));
    let _e120 = ((f32((_e114 & 131071u)) * 0.00048828125f) + -32f);
    let _e123 = DD.e2_[_e116];
    phi_1335_ = _e120;
    if ((_e123.x & 768u) != 0u) {
        let _e127 = abs(_e120);
        phi_933_ = Fh;
        if Fh {
            phi_933_ = ((_e123.x & 512u) != 0u);
        }
        let _e131 = phi_933_;
        phi_1336_ = _e127;
        if _e131 {
            phi_1336_ = (1f - abs(((fract((_e127 * 0.5f)) * 2f) + -1f)));
        }
        let _e139 = phi_1336_;
        phi_1335_ = _e139;
    }
    let _e141 = phi_1335_;
    let _e142 = clamp(_e141, 0f, 1f);
    phi_1339_ = _e142;
    if Bh {
        let _e144 = (_e123.x >> bitcast<u32>(16u));
        phi_1340_ = _e142;
        if (_e144 != 0u) {
            let _e148 = h0_.e2_[_e97];
            if (_e144 == (_e148 >> bitcast<u32>(16i))) {
                phi_1337_ = min(_e142, unpack2x16float(_e148).x);
            } else {
                phi_1337_ = 0f;
            }
            let _e156 = phi_1337_;
            phi_1340_ = _e156;
        }
        let _e158 = phi_1340_;
        phi_1339_ = _e158;
    }
    let _e160 = phi_1339_;
    phi_970_ = Ch;
    if Ch {
        phi_970_ = ((_e123.x & 1024u) != 0u);
    }
    let _e164 = phi_970_;
    phi_1342_ = _e160;
    if _e164 {
        let _e165 = (_e116 * 8u);
        let _e169 = QB.e2_[(_e165 + 2u)];
        let _e180 = QB.e2_[(_e165 + 3u)];
        let _e185 = _e180.zw;
        let _e187 = ((abs(((mat2x2<f32>(vec2<f32>(_e169.x, _e169.y), vec2<f32>(_e169.z, _e169.w)) * _e63) + _e180.xy)) * _e185) - _e185);
        phi_1342_ = min(_e160, clamp((min(_e187.x, _e187.y) + 0.5f), 0f, 1f));
    }
    let _e195 = phi_1342_;
    let _e196 = (_e123.x & 15u);
    if (_e196 <= 1u) {
        let _e201 = (Bh && (_e196 == 0u));
        phi_1369_ = 0u;
        if _e201 {
            phi_1369_ = (_e123.y | pack2x16float(vec2<f32>(_e195, 0f)));
        }
        let _e206 = phi_1369_;
        phi_1368_ = _e206;
        phi_1366_ = select(unpack4x8unorm(_e123.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e201));
    } else {
        let _e209 = (_e116 * 8u);
        let _e212 = QB.e2_[_e209];
        let _e223 = QB.e2_[(_e209 + 1u)];
        let _e226 = ((mat2x2<f32>(vec2<f32>(_e212.x, _e212.y), vec2<f32>(_e212.z, _e212.w)) * _e63) + _e223.xy);
        if (_e196 == 2u) {
            phi_1341_ = _e226.x;
        } else {
            phi_1341_ = length(_e226);
        }
        let _e231 = phi_1341_;
        let _e240 = textureSampleLevel(ED, N9_, vec2<f32>(((clamp(_e231, 0f, 1f) * _e223.z) + _e223.w), bitcast<f32>(_e123.y)), 0f);
        phi_1368_ = 0u;
        phi_1366_ = _e240;
    }
    let _e242 = phi_1368_;
    let _e244 = phi_1366_;
    let _e246 = (_e244.w * _e195);
    let _e248 = (_e244.xyz * _e246);
    phi_690_ = Bh;
    if Bh {
        let _e253 = x3_1;
        phi_690_ = (_e253 != 0u);
    }
    let _e256 = phi_690_;
    phi_1418_ = _e111;
    if _e256 {
        if (_e242 != 0u) {
            phi_1380_ = _e242;
        } else {
            let _e260 = h0_.e2_[_e97];
            phi_1380_ = _e260;
        }
        let _e262 = phi_1380_;
        let _e263 = x3_1;
        if (_e263 == (_e262 >> bitcast<u32>(16i))) {
            phi_1395_ = min(_e111, unpack2x16float(_e262).x);
        } else {
            phi_1395_ = 0f;
        }
        let _e271 = phi_1395_;
        phi_1418_ = _e271;
    }
    let _e273 = phi_1418_;
    let _e275 = M5_1[3u];
    phi_1416_ = _e99;
    if (_e275 != 0f) {
        let _e277 = M5_1;
        if (_e277.z > 0f) {
            phi_1396_ = _e277.x;
        } else {
            phi_1396_ = length(_e277.xy);
        }
        let _e284 = phi_1396_;
        let _e285 = clamp(_e284, 0f, 1f);
        let _e286 = abs(_e277.z);
        if (_e286 > 1f) {
            phi_1397_ = ((0.9980469f * _e285) + 0.0009765625f);
        } else {
            phi_1397_ = ((0.001953125f * _e285) + _e286);
        }
        let _e293 = phi_1397_;
        let _e296 = textureSampleLevel(ED, N9_, vec2<f32>(_e293, _e277.w), 0f);
        let _e299 = (_e296.xyz * _e296.w);
        let _e305 = vec4<f32>(_e299.x, _e296.y, _e296.z, _e296.w);
        let _e311 = vec4<f32>(_e305.x, _e299.y, _e305.z, _e305.w);
        phi_1416_ = (_e99 * vec4<f32>(_e311.x, _e311.y, _e299.z, _e311.w));
    }
    let _e320 = phi_1416_;
    let _e321 = H1_1;
    let _e323 = ((_e320 * _e321) * _e273);
    let _e327 = ((vec4<f32>(_e248.x, _e248.y, _e248.z, _e246) * (1f - _e323.w)) + _e323);
    let _e328 = _e327.xyz;
    let _e331 = n.B3_;
    let _e333 = n.C3_;
    if (Ih && (_e327.w != 0f)) {
        phi_1428_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e62.x) + (0.00583715f * _e62.y))))) * _e331) + _e333)) + _e328);
    } else {
        phi_1428_ = _e328;
    }
    let _e349 = phi_1428_;
    let _e355 = vec4<f32>(_e349.x, _e327.y, _e327.z, _e327.w);
    let _e361 = vec4<f32>(_e355.x, _e349.y, _e355.z, _e355.w);
    C1_ = vec4<f32>(_e361.x, _e361.y, _e349.z, _e361.w);
    if (_e242 != 0u) {
        h0_.e2_[_e97] = _e242;
    }
    x4_.e2_[_e97] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) Z1_: vec2<f32>, @location(1) V4_: f32, @location(3) M0_: vec4<f32>, @location(5) @interpolate(flat, either) x3_: u32, @location(2) M5_: vec4<f32>, @location(4) @interpolate(flat, either) H1_: vec4<f32>, @location(6) @interpolate(flat, either) A1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    Z1_1 = Z1_;
    V4_1 = V4_;
    M0_1 = M0_;
    x3_1 = x3_;
    M5_1 = M5_;
    H1_1 = H1_;
    A1_1 = A1_;
    main_1();
    let _e17 = C1_;
    return _e17;
}
