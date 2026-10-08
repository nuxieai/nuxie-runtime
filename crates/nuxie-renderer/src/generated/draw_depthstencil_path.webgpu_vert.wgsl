enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct hi {
    v2_: array<vec4<u32>>,
}

struct gi {
    v2_: array<vec4<u32>>,
}

struct kg {
    v2_: array<vec4<f32>>,
}

struct jg {
    v2_: array<vec2<u32>>,
}

struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
}

@id(15) override I7_: bool = false;
@id(16) override fe: bool = false;
@id(1) override Zi: bool = true;
@id(2) override aj: bool = true;
@id(8) override gj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> CD: hi;
@group(0) @binding(2)
var<storage> KB: gi;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
@group(0) @binding(3)
var<storage> WC: jg;
var<private> Q0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Va: sampler;

fn main_1() {
    var phi_1283_: bool;
    var phi_1284_: i32;
    var phi_1340_: f32;
    var phi_1293_: f32;
    var phi_1290_: bool;
    var phi_1288_: i32;
    var phi_1287_: i32;
    var phi_1285_: i32;
    var phi_1332_: f32;
    var phi_1331_: f32;
    var phi_1296_: bool;
    var phi_1330_: f32;
    var phi_1295_: bool;
    var phi_1301_: i32;
    var phi_1306_: bool;
    var phi_1308_: vec4<u32>;
    var phi_1307_: vec4<u32>;
    var phi_1348_: u32;
    var phi_1325_: vec4<u32>;
    var phi_1314_: bool;
    var phi_1350_: f32;
    var phi_1362_: f32;
    var phi_1354_: f32;
    var phi_671_: bool;
    var phi_1357_: f32;
    var phi_1372_: vec2<f32>;
    var phi_1371_: vec2<f32>;
    var phi_1382_: f32;
    var phi_1390_: vec2<f32>;
    var phi_1379_: f32;
    var phi_1366_: vec2<f32>;
    var phi_1389_: vec2<f32>;
    var phi_1378_: f32;
    var phi_1375_: f32;
    var phi_1365_: vec2<f32>;
    var phi_1401_: vec2<f32>;
    var phi_1326_: vec2<f32>;
    var phi_1400_: vec2<f32>;
    var phi_1451_: bool;
    var phi_1449_: vec4<f32>;
    var phi_1450_: vec4<f32>;
    var phi_891_: bool;
    var phi_1462_: bool;
    var phi_1472_: u32;
    var phi_1471_: u32;

    let _e77 = gl_VertexIndex_1;
    let _e82 = ((_e77 & 536870912i) != 0i);
    let _e83 = (_e77 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1283_ = true;
                break;
            }
            if fe {
                phi_1283_ = true;
                break;
            }
            phi_1283_ = false;
            break;
        }
    }
    let _e86 = phi_1283_;
    if _e86 {
        let _e87 = select(5i, 6i, _e82);
        let _e93 = (_e83 & ((1i << bitcast<u32>(_e87)) - 1i));
        let _e94 = select(1i, 2i, _e82);
        let _e100 = (_e93 & ((1i << bitcast<u32>(_e94)) - 1i));
        phi_1284_ = _e100;
        if (I7_ && !(_e82)) {
            phi_1284_ = (_e100 + 1i);
        }
        let _e105 = phi_1284_;
        phi_1340_ = select(1f, 0f, ((_e105 == 0i) || (_e105 == 3i)));
        phi_1293_ = select(-1f, 1f, (_e105 < 2i));
        phi_1290_ = false;
        phi_1288_ = (_e83 >> bitcast<u32>(_e87));
        phi_1287_ = 8i;
        phi_1285_ = (_e93 >> bitcast<u32>(_e94));
    } else {
        let _e112 = select(4i, 5i, _e82);
        let _e118 = (_e83 & ((1i << bitcast<u32>(_e112)) - 1i));
        let _e122 = (!(_e82) && (_e118 == 9i));
        phi_1340_ = 1f;
        phi_1293_ = 0f;
        phi_1290_ = _e122;
        phi_1288_ = (_e83 >> bitcast<u32>(_e112));
        phi_1287_ = select(8i, 17i, _e82);
        phi_1285_ = select(_e118, 0i, _e122);
    }
    let _e125 = phi_1340_;
    let _e127 = phi_1293_;
    let _e129 = phi_1290_;
    let _e131 = phi_1288_;
    let _e133 = phi_1287_;
    let _e135 = phi_1285_;
    let _e137 = min(_e135, (_e133 - 1i));
    let _e139 = ((_e131 * _e133) + _e137);
    let _e144 = textureLoad(UB, vec2<i32>((_e139 & 2047i), (_e139 >> bitcast<u32>(11i))), 0i);
    let _e151 = CD.v2_[(max((_e144.w & 65535u), 1u) - 1u)];
    let _e153 = bitcast<vec2<f32>>(_e151.xy);
    let _e155 = (_e151.z & 65535u);
    let _e157 = (_e155 * 4u);
    let _e160 = KB.v2_[_e157];
    let _e161 = bitcast<vec4<f32>>(_e160);
    let _e168 = mat2x2<f32>(vec2<f32>(_e161.x, _e161.y), vec2<f32>(_e161.z, _e161.w));
    let _e172 = KB.v2_[(_e157 + 1u)];
    let _e174 = bitcast<vec2<f32>>(_e172.xy);
    let _e180 = KB.v2_[(_e157 + 2u)];
    let _e182 = (_e144.w & 8388608u);
    if I7_ {
        phi_1330_ = _e127;
        phi_1295_ = false;
    } else {
        if fe {
            let _e183 = (_e182 != 0u);
            phi_1332_ = _e127;
            if _e183 {
                phi_1332_ = -(_e127);
            }
            let _e186 = phi_1332_;
            phi_1331_ = _e186;
            phi_1296_ = _e183;
        } else {
            phi_1331_ = _e127;
            phi_1296_ = (((_e182 != 0u) && !(_e82)) && !(_e129));
        }
        let _e193 = phi_1331_;
        let _e195 = phi_1296_;
        phi_1330_ = _e193;
        phi_1295_ = _e195;
    }
    let _e197 = phi_1330_;
    let _e199 = phi_1295_;
    phi_1301_ = _e135;
    if _e199 {
        phi_1301_ = (_e135 - 1i);
    }
    let _e202 = phi_1301_;
    phi_1348_ = _e144.w;
    phi_1325_ = _e144;
    if (_e202 != _e137) {
        let _e205 = ((_e139 + _e202) - _e137);
        let _e210 = textureLoad(UB, vec2<i32>((_e205 & 2047i), (_e205 >> bitcast<u32>(11i))), 0i);
        if ((_e210.w & 8454143u) != (_e144.w & 8454143u)) {
            if I7_ {
                phi_1306_ = (_e153.x != 0f);
            } else {
                phi_1306_ = true;
            }
            let _e218 = phi_1306_;
            phi_1308_ = _e144;
            if _e218 {
                let _e219 = bitcast<i32>(_e151.w);
                let _e224 = textureLoad(UB, vec2<i32>((_e219 & 2047i), (_e219 >> bitcast<u32>(11i))), 0i);
                phi_1308_ = _e224;
            }
            let _e226 = phi_1308_;
            phi_1307_ = _e226;
        } else {
            phi_1307_ = _e210;
        }
        let _e228 = phi_1307_;
        phi_1348_ = ((_e228.w & 4286578687u) | _e182);
        phi_1325_ = _e228;
    }
    let _e233 = phi_1348_;
    let _e235 = phi_1325_;
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1314_ = true;
                break;
            }
            if fe {
                phi_1314_ = true;
                break;
            }
            phi_1314_ = false;
            break;
        }
    }
    let _e238 = phi_1314_;
    if _e238 {
        let _e241 = (_e197 * sign(determinant(_e168)));
        let _e244 = (f32(_e235.z) * 0.0000000014629181f);
        let _e248 = vec2<f32>(sin(_e244), -(cos(_e244)));
        let _e252 = select(0f, _e241, (_e125 == 0f));
        phi_1389_ = _e248;
        phi_1378_ = _e252;
        phi_1375_ = _e241;
        phi_1365_ = _e248;
        if I7_ {
            let _e254 = ((_e233 & 1048576u) != 0u);
            phi_1350_ = _e241;
            if _e254 {
                phi_1350_ = min(_e241, 0f);
            }
            let _e257 = phi_1350_;
            phi_1362_ = _e257;
            if ((_e233 & 524288u) != 0u) {
                phi_1362_ = max(_e257, 0f);
            }
            let _e262 = phi_1362_;
            let _e263 = (_e233 & 469762048u);
            phi_1390_ = _e248;
            phi_1379_ = _e252;
            phi_1366_ = _e248;
            if (_e263 > 134217728u) {
                let _e268 = f32((_e235.z & 65535u));
                let _e269 = (_e268 * 0.000015259022f);
                let _e273 = sqrt(max((1f - (_e269 * _e269)), 0f));
                phi_1354_ = _e273;
                if (((_e233 & 4194304u) != 0u) == _e254) {
                    phi_1354_ = -(_e273);
                }
                let _e277 = phi_1354_;
                let _e282 = (mat2x2<f32>(vec2<f32>(_e269, _e277), vec2<f32>(-(_e277), _e269)) * _e248);
                let _e283 = (_e263 == 201326592u);
                phi_671_ = _e283;
                if !(_e283) {
                    phi_671_ = ((_e263 != 335544320u) && (_e269 < 0.25f));
                }
                let _e289 = phi_671_;
                let _e291 = ((_e233 & 2097152u) != 0u);
                if (_e263 == 335544320u) {
                    phi_1371_ = (_e248 + _e282);
                } else {
                    phi_1372_ = _e248;
                    if (_e291 || !(_e289)) {
                        if _e289 {
                            phi_1357_ = _e269;
                        } else {
                            phi_1357_ = (65535f / _e268);
                        }
                        let _e298 = phi_1357_;
                        phi_1372_ = (_e282 * _e298);
                    }
                    let _e301 = phi_1372_;
                    phi_1371_ = _e301;
                }
                let _e303 = phi_1371_;
                phi_1382_ = _e252;
                if (!(_e82) && _e289) {
                    phi_1382_ = (0.5f * _e262);
                }
                let _e311 = phi_1382_;
                phi_1390_ = select(_e248, _e282, vec2((_e289 || _e291)));
                phi_1379_ = _e311;
                phi_1366_ = _e303;
            }
            let _e313 = phi_1390_;
            let _e315 = phi_1379_;
            let _e317 = phi_1366_;
            phi_1389_ = _e313;
            phi_1378_ = _e315;
            phi_1375_ = _e262;
            phi_1365_ = _e317;
        }
        let _e319 = phi_1389_;
        let _e321 = phi_1378_;
        let _e323 = phi_1375_;
        let _e325 = phi_1365_;
        let _e330 = ((_e168 * (bitcast<vec2<f32>>(_e235.xy) + (_e325 * (_e323 * bitcast<f32>(_e172.z))))) + _e174);
        phi_1401_ = _e330;
        if (_e321 != 0f) {
            phi_1401_ = (_e330 + (sign((_e319 * _naga_inverse_2x2_f32(_e168))) * _e321));
        }
        let _e338 = phi_1401_;
        phi_1400_ = _e338;
    } else {
        if _e129 {
            phi_1326_ = _e153;
        } else {
            phi_1326_ = bitcast<vec2<f32>>(_e235.xy);
        }
        let _e342 = phi_1326_;
        phi_1400_ = ((_e168 * _e342) + _e174);
    }
    let _e346 = phi_1400_;
    if Zi {
        let _e347 = (_e155 * 8u);
        let _e351 = JB.v2_[(_e347 + 2u)];
        let _e362 = JB.v2_[(_e347 + 3u)];
        if any((_e351 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e367 = ((mat2x2<f32>(vec2<f32>(_e351.x, _e351.y), vec2<f32>(_e351.z, _e351.w)) * _e346) + _e362.xy);
            unnamed.gl_ClipDistance[0i] = (_e367.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e367.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e367.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e367.y);
        } else {
            let _e383 = (_e362.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e383;
            unnamed.gl_ClipDistance[2i] = _e383;
            unnamed.gl_ClipDistance[1i] = _e383;
            unnamed.gl_ClipDistance[0i] = _e383;
        }
    }
    if ((_e77 & 268435456i) != 0i) {
        P0_ = vec4<f32>(0f, 0f, 0f, 0f);
        V0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e394 = WC.v2_[_e155];
        let _e396 = (_e394.x & 15u);
        phi_1451_ = false;
        if aj {
            let _e399 = ((_e394.x >> bitcast<u32>(4i)) & 15u);
            Q0_ = f32(_e399);
            phi_1451_ = (_e399 != 0u);
        }
        let _e403 = phi_1451_;
        if (_e396 == 1u) {
            P0_ = unpack4x8unorm(_e394.y);
            if _e403 {
                let _e408 = P0_[3u];
                P0_[3u] = (_e408 * _e125);
            } else {
                let _e410 = P0_;
                P0_ = (_e410 * _e125);
            }
        } else {
            let _e412 = (_e155 * 8u);
            let _e415 = JB.v2_[_e412];
            let _e426 = JB.v2_[(_e412 + 1u)];
            let _e428 = bitcast<f32>(_e394.y);
            let _e431 = ((mat2x2<f32>(vec2<f32>(_e415.x, _e415.y), vec2<f32>(_e415.z, _e415.w)) * _e346) + _e426.xy);
            let _e437 = vec4<f32>(_e431.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e443 = vec4<f32>(_e437.x, _e431.y, _e437.z, _e437.w);
            let _e448 = vec4<f32>(_e443.x, _e443.y, _e125, _e443.w);
            phi_1449_ = _e448;
            if (_e396 != 2u) {
                phi_1449_ = vec4<f32>(_e448.x, _e448.y, (_e125 + 2f), _e448.w);
            }
            let _e457 = phi_1449_;
            phi_1450_ = _e457;
            if (_e426.z > 0.9f) {
                phi_1450_ = vec4<f32>(_e457.x, _e457.y, -(_e457.z), _e457.w);
            }
            let _e468 = phi_1450_;
            P0_ = vec4<f32>(_e468.x, _e468.y, _e468.z, -(bitcast<f32>(((((_e396 << bitcast<u32>(28i)) | ((u32(_e428) - 1u) << bitcast<u32>(17i))) | (u32((_e426.w * 512f)) << bitcast<u32>(8i))) | u32((fract(_e428) * 256f))))));
        }
        phi_891_ = gj;
        if gj {
            phi_891_ = ((_e394.x & 2048u) != 0u);
        }
        let _e496 = phi_891_;
        if _e496 {
            let _e497 = (_e155 * 8u);
            let _e501 = JB.v2_[(_e497 + 4u)];
            let _e512 = JB.v2_[(_e497 + 5u)];
            let _e515 = ((mat2x2<f32>(vec2<f32>(_e501.x, _e501.y), vec2<f32>(_e501.z, _e501.w)) * _e346) + _e512.xy);
            V0_ = vec3<f32>(_e515.x, _e515.y, (1f + _e512.z));
        } else {
            V0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e522 = j.Gg;
    let _e524 = j.Hg;
    let _e532 = vec4<f32>(((_e346.x * _e522) - 1f), ((_e346.y * _e524) - sign(_e524)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if I7_ {
                phi_1462_ = true;
                break;
            }
            if fe {
                phi_1462_ = true;
                break;
            }
            phi_1462_ = false;
            break;
        }
    }
    let _e535 = phi_1462_;
    if _e535 {
        let _e537 = u32((_e125 * 254f));
        phi_1472_ = _e537;
        if ((_e77 & 1073741824i) == 0i) {
            phi_1472_ = (_e537 + bitcast<u32>(1i));
        }
        let _e542 = phi_1472_;
        phi_1471_ = _e542;
    } else {
        phi_1471_ = 255u;
    }
    let _e544 = phi_1471_;
    unnamed.gl_Position = vec4<f32>(_e532.x, _e532.y, ((f32(((_e180.x << bitcast<u32>(8u)) | _e544)) * 0.000000059604645f) + 0.000000029802322f), _e532.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e9 = unnamed.gl_Position;
    let _e10 = unnamed.gl_ClipDistance;
    let _e11 = P0_;
    let _e12 = V0_;
    let _e13 = Q0_;
    return VertexOutput(_e9, _e10, _e11, _e12, _e13);
}

fn _naga_inverse_2x2_f32(m: mat2x2<f32>) -> mat2x2<f32> {
    var adj: mat2x2<f32>;
    adj[0][0] = m[1][1];
    adj[0][1] = -m[0][1];
    adj[1][0] = -m[1][0];
    adj[1][1] = m[0][0];

    let det: f32 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
    return adj * (1 / det);
}
