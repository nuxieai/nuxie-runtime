enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct ei {
    r2_: array<vec4<u32>>,
}

struct di {
    r2_: array<vec4<u32>>,
}

struct kg {
    r2_: array<vec4<f32>>,
}

struct jg {
    r2_: array<vec2<u32>>,
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

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
}

@id(15) override K7_: bool = false;
@id(16) override fe: bool = false;
@id(1) override Xi: bool = true;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> BD: ei;
@group(0) @binding(2)
var<storage> KB: di;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> O0_: vec4<f32>;
var<private> U0_: vec3<f32>;
@group(0) @binding(3)
var<storage> VC: jg;
var<private> P0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;

fn main_1() {
    var phi_1268_: bool;
    var phi_1269_: i32;
    var phi_1325_: f32;
    var phi_1278_: f32;
    var phi_1275_: bool;
    var phi_1273_: i32;
    var phi_1272_: i32;
    var phi_1270_: i32;
    var phi_1317_: f32;
    var phi_1316_: f32;
    var phi_1281_: bool;
    var phi_1315_: f32;
    var phi_1280_: bool;
    var phi_1286_: i32;
    var phi_1291_: bool;
    var phi_1293_: vec4<u32>;
    var phi_1292_: vec4<u32>;
    var phi_1333_: u32;
    var phi_1310_: vec4<u32>;
    var phi_1299_: bool;
    var phi_1335_: f32;
    var phi_1347_: f32;
    var phi_1339_: f32;
    var phi_661_: bool;
    var phi_1342_: f32;
    var phi_1357_: vec2<f32>;
    var phi_1356_: vec2<f32>;
    var phi_1367_: f32;
    var phi_1375_: vec2<f32>;
    var phi_1364_: f32;
    var phi_1351_: vec2<f32>;
    var phi_1374_: vec2<f32>;
    var phi_1363_: f32;
    var phi_1360_: f32;
    var phi_1350_: vec2<f32>;
    var phi_1386_: vec2<f32>;
    var phi_1311_: vec2<f32>;
    var phi_1385_: vec2<f32>;
    var phi_1436_: bool;
    var phi_1434_: f32;
    var phi_883_: bool;
    var phi_1446_: bool;
    var phi_1455_: u32;
    var phi_1454_: u32;

    let _e75 = gl_VertexIndex_1;
    let _e80 = ((_e75 & 536870912i) != 0i);
    let _e81 = (_e75 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1268_ = true;
                break;
            }
            if fe {
                phi_1268_ = true;
                break;
            }
            phi_1268_ = false;
            break;
        }
    }
    let _e84 = phi_1268_;
    if _e84 {
        let _e85 = select(5i, 6i, _e80);
        let _e91 = (_e81 & ((1i << bitcast<u32>(_e85)) - 1i));
        let _e92 = select(1i, 2i, _e80);
        let _e98 = (_e91 & ((1i << bitcast<u32>(_e92)) - 1i));
        phi_1269_ = _e98;
        if (K7_ && !(_e80)) {
            phi_1269_ = (_e98 + 1i);
        }
        let _e103 = phi_1269_;
        phi_1325_ = select(1f, 0f, ((_e103 == 0i) || (_e103 == 3i)));
        phi_1278_ = select(-1f, 1f, (_e103 < 2i));
        phi_1275_ = false;
        phi_1273_ = (_e81 >> bitcast<u32>(_e85));
        phi_1272_ = 8i;
        phi_1270_ = (_e91 >> bitcast<u32>(_e92));
    } else {
        let _e110 = select(4i, 5i, _e80);
        let _e116 = (_e81 & ((1i << bitcast<u32>(_e110)) - 1i));
        let _e120 = (!(_e80) && (_e116 == 9i));
        phi_1325_ = 1f;
        phi_1278_ = 0f;
        phi_1275_ = _e120;
        phi_1273_ = (_e81 >> bitcast<u32>(_e110));
        phi_1272_ = select(8i, 17i, _e80);
        phi_1270_ = select(_e116, 0i, _e120);
    }
    let _e123 = phi_1325_;
    let _e125 = phi_1278_;
    let _e127 = phi_1275_;
    let _e129 = phi_1273_;
    let _e131 = phi_1272_;
    let _e133 = phi_1270_;
    let _e135 = min(_e133, (_e131 - 1i));
    let _e137 = ((_e129 * _e131) + _e135);
    let _e142 = textureLoad(UB, vec2<i32>((_e137 & 2047i), (_e137 >> bitcast<u32>(11i))), 0i);
    let _e149 = BD.r2_[(max((_e142.w & 65535u), 1u) - 1u)];
    let _e151 = bitcast<vec2<f32>>(_e149.xy);
    let _e153 = (_e149.z & 65535u);
    let _e155 = (_e153 * 4u);
    let _e158 = KB.r2_[_e155];
    let _e159 = bitcast<vec4<f32>>(_e158);
    let _e166 = mat2x2<f32>(vec2<f32>(_e159.x, _e159.y), vec2<f32>(_e159.z, _e159.w));
    let _e170 = KB.r2_[(_e155 + 1u)];
    let _e172 = bitcast<vec2<f32>>(_e170.xy);
    let _e178 = KB.r2_[(_e155 + 2u)];
    let _e180 = (_e142.w & 8388608u);
    if K7_ {
        phi_1315_ = _e125;
        phi_1280_ = false;
    } else {
        if fe {
            let _e181 = (_e180 != 0u);
            phi_1317_ = _e125;
            if _e181 {
                phi_1317_ = -(_e125);
            }
            let _e184 = phi_1317_;
            phi_1316_ = _e184;
            phi_1281_ = _e181;
        } else {
            phi_1316_ = _e125;
            phi_1281_ = (((_e180 != 0u) && !(_e80)) && !(_e127));
        }
        let _e191 = phi_1316_;
        let _e193 = phi_1281_;
        phi_1315_ = _e191;
        phi_1280_ = _e193;
    }
    let _e195 = phi_1315_;
    let _e197 = phi_1280_;
    phi_1286_ = _e133;
    if _e197 {
        phi_1286_ = (_e133 - 1i);
    }
    let _e200 = phi_1286_;
    phi_1333_ = _e142.w;
    phi_1310_ = _e142;
    if (_e200 != _e135) {
        let _e203 = ((_e137 + _e200) - _e135);
        let _e208 = textureLoad(UB, vec2<i32>((_e203 & 2047i), (_e203 >> bitcast<u32>(11i))), 0i);
        if ((_e208.w & 8454143u) != (_e142.w & 8454143u)) {
            if K7_ {
                phi_1291_ = (_e151.x != 0f);
            } else {
                phi_1291_ = true;
            }
            let _e216 = phi_1291_;
            phi_1293_ = _e142;
            if _e216 {
                let _e217 = bitcast<i32>(_e149.w);
                let _e222 = textureLoad(UB, vec2<i32>((_e217 & 2047i), (_e217 >> bitcast<u32>(11i))), 0i);
                phi_1293_ = _e222;
            }
            let _e224 = phi_1293_;
            phi_1292_ = _e224;
        } else {
            phi_1292_ = _e208;
        }
        let _e226 = phi_1292_;
        phi_1333_ = ((_e226.w & 4286578687u) | _e180);
        phi_1310_ = _e226;
    }
    let _e231 = phi_1333_;
    let _e233 = phi_1310_;
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1299_ = true;
                break;
            }
            if fe {
                phi_1299_ = true;
                break;
            }
            phi_1299_ = false;
            break;
        }
    }
    let _e236 = phi_1299_;
    if _e236 {
        let _e239 = (_e195 * sign(determinant(_e166)));
        let _e242 = (f32(_e233.z) * 0.0000000014629181f);
        let _e246 = vec2<f32>(sin(_e242), -(cos(_e242)));
        let _e250 = select(0f, _e239, (_e123 == 0f));
        phi_1374_ = _e246;
        phi_1363_ = _e250;
        phi_1360_ = _e239;
        phi_1350_ = _e246;
        if K7_ {
            let _e252 = ((_e231 & 1048576u) != 0u);
            phi_1335_ = _e239;
            if _e252 {
                phi_1335_ = min(_e239, 0f);
            }
            let _e255 = phi_1335_;
            phi_1347_ = _e255;
            if ((_e231 & 524288u) != 0u) {
                phi_1347_ = max(_e255, 0f);
            }
            let _e260 = phi_1347_;
            let _e261 = (_e231 & 469762048u);
            phi_1375_ = _e246;
            phi_1364_ = _e250;
            phi_1351_ = _e246;
            if (_e261 > 134217728u) {
                let _e266 = f32((_e233.z & 65535u));
                let _e267 = (_e266 * 0.000015259022f);
                let _e271 = sqrt(max((1f - (_e267 * _e267)), 0f));
                phi_1339_ = _e271;
                if (((_e231 & 4194304u) != 0u) == _e252) {
                    phi_1339_ = -(_e271);
                }
                let _e275 = phi_1339_;
                let _e280 = (mat2x2<f32>(vec2<f32>(_e267, _e275), vec2<f32>(-(_e275), _e267)) * _e246);
                let _e281 = (_e261 == 201326592u);
                phi_661_ = _e281;
                if !(_e281) {
                    phi_661_ = ((_e261 != 335544320u) && (_e267 < 0.25f));
                }
                let _e287 = phi_661_;
                let _e289 = ((_e231 & 2097152u) != 0u);
                if (_e261 == 335544320u) {
                    phi_1356_ = (_e246 + _e280);
                } else {
                    phi_1357_ = _e246;
                    if (_e289 || !(_e287)) {
                        if _e287 {
                            phi_1342_ = _e267;
                        } else {
                            phi_1342_ = (65535f / _e266);
                        }
                        let _e296 = phi_1342_;
                        phi_1357_ = (_e280 * _e296);
                    }
                    let _e299 = phi_1357_;
                    phi_1356_ = _e299;
                }
                let _e301 = phi_1356_;
                phi_1367_ = _e250;
                if (!(_e80) && _e287) {
                    phi_1367_ = (0.5f * _e260);
                }
                let _e309 = phi_1367_;
                phi_1375_ = select(_e246, _e280, vec2((_e287 || _e289)));
                phi_1364_ = _e309;
                phi_1351_ = _e301;
            }
            let _e311 = phi_1375_;
            let _e313 = phi_1364_;
            let _e315 = phi_1351_;
            phi_1374_ = _e311;
            phi_1363_ = _e313;
            phi_1360_ = _e260;
            phi_1350_ = _e315;
        }
        let _e317 = phi_1374_;
        let _e319 = phi_1363_;
        let _e321 = phi_1360_;
        let _e323 = phi_1350_;
        let _e328 = ((_e166 * (bitcast<vec2<f32>>(_e233.xy) + (_e323 * (_e321 * bitcast<f32>(_e170.z))))) + _e172);
        phi_1386_ = _e328;
        if (_e319 != 0f) {
            phi_1386_ = (_e328 + (sign((_e317 * _naga_inverse_2x2_f32(_e166))) * _e319));
        }
        let _e336 = phi_1386_;
        phi_1385_ = _e336;
    } else {
        if _e127 {
            phi_1311_ = _e151;
        } else {
            phi_1311_ = bitcast<vec2<f32>>(_e233.xy);
        }
        let _e340 = phi_1311_;
        phi_1385_ = ((_e166 * _e340) + _e172);
    }
    let _e344 = phi_1385_;
    if Xi {
        let _e345 = (_e153 * 8u);
        let _e349 = JB.r2_[(_e345 + 2u)];
        let _e360 = JB.r2_[(_e345 + 3u)];
        if any((_e349 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e365 = ((mat2x2<f32>(vec2<f32>(_e349.x, _e349.y), vec2<f32>(_e349.z, _e349.w)) * _e344) + _e360.xy);
            unnamed.gl_ClipDistance[0i] = (_e365.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e365.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e365.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e365.y);
        } else {
            let _e381 = (_e360.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e381;
            unnamed.gl_ClipDistance[2i] = _e381;
            unnamed.gl_ClipDistance[1i] = _e381;
            unnamed.gl_ClipDistance[0i] = _e381;
        }
    }
    if ((_e75 & 268435456i) != 0i) {
        O0_ = vec4<f32>(0f, 0f, 0f, 0f);
        U0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e392 = VC.r2_[_e153];
        let _e394 = (_e392.x & 15u);
        phi_1436_ = false;
        if Yi {
            let _e397 = ((_e392.x >> bitcast<u32>(4i)) & 15u);
            P0_ = f32(_e397);
            phi_1436_ = (_e397 != 0u);
        }
        let _e401 = phi_1436_;
        if (_e394 == 1u) {
            O0_ = unpack4x8unorm(_e392.y);
            if _e401 {
                let _e406 = O0_[3u];
                O0_[3u] = (_e406 * _e123);
            } else {
                let _e408 = O0_;
                O0_ = (_e408 * _e123);
            }
        } else {
            let _e410 = (_e153 * 8u);
            let _e413 = JB.r2_[_e410];
            let _e424 = JB.r2_[(_e410 + 1u)];
            let _e429 = ((mat2x2<f32>(vec2<f32>(_e413.x, _e413.y), vec2<f32>(_e413.z, _e413.w)) * _e344) + _e424.xy);
            let _e440 = ((_e424.w + (f32(_e394) * 0.125f)) + (max(_e424.z, 0f) * 0.00024414063f));
            if (_e424.z < 0f) {
                phi_1434_ = -(_e440);
            } else {
                phi_1434_ = _e440;
            }
            let _e443 = phi_1434_;
            O0_ = vec4<f32>(_e429.x, _e429.y, _e443, (((_e123 * -0.5f) - 0.25f) - round((bitcast<f32>(_e392.y) * 255f))));
        }
        phi_883_ = ej;
        if ej {
            phi_883_ = ((_e392.x & 2048u) != 0u);
        }
        let _e453 = phi_883_;
        if _e453 {
            let _e454 = (_e153 * 8u);
            let _e458 = JB.r2_[(_e454 + 4u)];
            let _e469 = JB.r2_[(_e454 + 5u)];
            let _e472 = ((mat2x2<f32>(vec2<f32>(_e458.x, _e458.y), vec2<f32>(_e458.z, _e458.w)) * _e344) + _e469.xy);
            U0_ = vec3<f32>(_e472.x, _e472.y, (1f + _e469.z));
        } else {
            U0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e479 = j.Hg;
    let _e481 = j.Ig;
    let _e489 = vec4<f32>(((_e344.x * _e479) - 1f), ((_e344.y * _e481) - sign(_e481)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1446_ = true;
                break;
            }
            if fe {
                phi_1446_ = true;
                break;
            }
            phi_1446_ = false;
            break;
        }
    }
    let _e492 = phi_1446_;
    if _e492 {
        let _e494 = u32((_e123 * 254f));
        phi_1455_ = _e494;
        if ((_e75 & 1073741824i) == 0i) {
            phi_1455_ = (_e494 + bitcast<u32>(1i));
        }
        let _e499 = phi_1455_;
        phi_1454_ = _e499;
    } else {
        phi_1454_ = 255u;
    }
    let _e501 = phi_1454_;
    unnamed.gl_Position = vec4<f32>(_e489.x, _e489.y, ((f32(((_e178.x << bitcast<u32>(8u)) | _e501)) * 0.000000059604645f) + 0.000000029802322f), _e489.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e9 = unnamed.gl_Position;
    let _e10 = unnamed.gl_ClipDistance;
    let _e11 = O0_;
    let _e12 = U0_;
    let _e13 = P0_;
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
