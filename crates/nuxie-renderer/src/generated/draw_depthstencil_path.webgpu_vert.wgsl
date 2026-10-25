enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct mi {
    v2_: array<vec4<u32>>,
}

struct li {
    v2_: array<vec4<u32>>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct og {
    v2_: array<vec2<u32>>,
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
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
}

@id(16) override O7_: bool = false;
@id(17) override ke: bool = false;
@id(1) override gj: bool = true;
@id(2) override hj: bool = true;
@id(4) override jj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> DD: mi;
@group(0) @binding(2)
var<storage> KB: li;
@group(0) @binding(4)
var<storage> JB: pg;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
@group(0) @binding(3)
var<storage> XC: og;
var<private> Q0_: f32;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var db: sampler;

fn main_1() {
    var phi_1269_: bool;
    var phi_1270_: i32;
    var phi_1326_: f32;
    var phi_1279_: f32;
    var phi_1276_: bool;
    var phi_1274_: i32;
    var phi_1273_: i32;
    var phi_1271_: i32;
    var phi_1318_: f32;
    var phi_1317_: f32;
    var phi_1282_: bool;
    var phi_1316_: f32;
    var phi_1281_: bool;
    var phi_1287_: i32;
    var phi_1292_: bool;
    var phi_1294_: vec4<u32>;
    var phi_1293_: vec4<u32>;
    var phi_1334_: u32;
    var phi_1311_: vec4<u32>;
    var phi_1300_: bool;
    var phi_1336_: f32;
    var phi_1348_: f32;
    var phi_1340_: f32;
    var phi_661_: bool;
    var phi_1343_: f32;
    var phi_1358_: vec2<f32>;
    var phi_1357_: vec2<f32>;
    var phi_1368_: f32;
    var phi_1376_: vec2<f32>;
    var phi_1365_: f32;
    var phi_1352_: vec2<f32>;
    var phi_1375_: vec2<f32>;
    var phi_1364_: f32;
    var phi_1361_: f32;
    var phi_1351_: vec2<f32>;
    var phi_1387_: vec2<f32>;
    var phi_1312_: vec2<f32>;
    var phi_1386_: vec2<f32>;
    var phi_1437_: bool;
    var phi_1435_: f32;
    var phi_883_: bool;
    var phi_1447_: bool;
    var phi_1456_: u32;
    var phi_1455_: u32;

    let _e75 = gl_VertexIndex_1;
    let _e80 = ((_e75 & 536870912i) != 0i);
    let _e81 = (_e75 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1269_ = true;
                break;
            }
            if ke {
                phi_1269_ = true;
                break;
            }
            phi_1269_ = false;
            break;
        }
    }
    let _e84 = phi_1269_;
    if _e84 {
        let _e85 = select(5i, 6i, _e80);
        let _e91 = (_e81 & ((1i << bitcast<u32>(_e85)) - 1i));
        let _e92 = select(1i, 2i, _e80);
        let _e98 = (_e91 & ((1i << bitcast<u32>(_e92)) - 1i));
        phi_1270_ = _e98;
        if (O7_ && !(_e80)) {
            phi_1270_ = (_e98 + 1i);
        }
        let _e103 = phi_1270_;
        phi_1326_ = select(1f, 0f, ((_e103 == 0i) || (_e103 == 3i)));
        phi_1279_ = select(-1f, 1f, (_e103 < 2i));
        phi_1276_ = false;
        phi_1274_ = (_e81 >> bitcast<u32>(_e85));
        phi_1273_ = 8i;
        phi_1271_ = (_e91 >> bitcast<u32>(_e92));
    } else {
        let _e110 = select(4i, 5i, _e80);
        let _e116 = (_e81 & ((1i << bitcast<u32>(_e110)) - 1i));
        let _e120 = (!(_e80) && (_e116 == 9i));
        phi_1326_ = 1f;
        phi_1279_ = 0f;
        phi_1276_ = _e120;
        phi_1274_ = (_e81 >> bitcast<u32>(_e110));
        phi_1273_ = select(8i, 17i, _e80);
        phi_1271_ = select(_e116, 0i, _e120);
    }
    let _e123 = phi_1326_;
    let _e125 = phi_1279_;
    let _e127 = phi_1276_;
    let _e129 = phi_1274_;
    let _e131 = phi_1273_;
    let _e133 = phi_1271_;
    let _e135 = min(_e133, (_e131 - 1i));
    let _e137 = ((_e129 * _e131) + _e135);
    let _e142 = textureLoad(VB, vec2<i32>((_e137 & 2047i), (_e137 >> bitcast<u32>(11i))), 0i);
    let _e149 = DD.v2_[(max((_e142.w & 65535u), 1u) - 1u)];
    let _e151 = bitcast<vec2<f32>>(_e149.xy);
    let _e153 = (_e149.z & 65535u);
    let _e155 = (_e153 * 4u);
    let _e158 = KB.v2_[_e155];
    let _e159 = bitcast<vec4<f32>>(_e158);
    let _e166 = mat2x2<f32>(vec2<f32>(_e159.x, _e159.y), vec2<f32>(_e159.z, _e159.w));
    let _e170 = KB.v2_[(_e155 + 1u)];
    let _e172 = bitcast<vec2<f32>>(_e170.xy);
    let _e178 = KB.v2_[(_e155 + 2u)];
    let _e180 = (_e142.w & 8388608u);
    if O7_ {
        phi_1316_ = _e125;
        phi_1281_ = false;
    } else {
        if ke {
            let _e181 = (_e180 != 0u);
            phi_1318_ = _e125;
            if _e181 {
                phi_1318_ = -(_e125);
            }
            let _e184 = phi_1318_;
            phi_1317_ = _e184;
            phi_1282_ = _e181;
        } else {
            phi_1317_ = _e125;
            phi_1282_ = (((_e180 != 0u) && !(_e80)) && !(_e127));
        }
        let _e191 = phi_1317_;
        let _e193 = phi_1282_;
        phi_1316_ = _e191;
        phi_1281_ = _e193;
    }
    let _e195 = phi_1316_;
    let _e197 = phi_1281_;
    phi_1287_ = _e133;
    if _e197 {
        phi_1287_ = (_e133 - 1i);
    }
    let _e200 = phi_1287_;
    phi_1334_ = _e142.w;
    phi_1311_ = _e142;
    if (_e200 != _e135) {
        let _e203 = ((_e137 + _e200) - _e135);
        let _e208 = textureLoad(VB, vec2<i32>((_e203 & 2047i), (_e203 >> bitcast<u32>(11i))), 0i);
        if ((_e208.w & 8454143u) != (_e142.w & 8454143u)) {
            if O7_ {
                phi_1292_ = (_e151.x != 0f);
            } else {
                phi_1292_ = true;
            }
            let _e216 = phi_1292_;
            phi_1294_ = _e142;
            if _e216 {
                let _e217 = bitcast<i32>(_e149.w);
                let _e222 = textureLoad(VB, vec2<i32>((_e217 & 2047i), (_e217 >> bitcast<u32>(11i))), 0i);
                phi_1294_ = _e222;
            }
            let _e224 = phi_1294_;
            phi_1293_ = _e224;
        } else {
            phi_1293_ = _e208;
        }
        let _e226 = phi_1293_;
        phi_1334_ = ((_e226.w & 4286578687u) | _e180);
        phi_1311_ = _e226;
    }
    let _e231 = phi_1334_;
    let _e233 = phi_1311_;
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1300_ = true;
                break;
            }
            if ke {
                phi_1300_ = true;
                break;
            }
            phi_1300_ = false;
            break;
        }
    }
    let _e236 = phi_1300_;
    if _e236 {
        let _e239 = (_e195 * sign(determinant(_e166)));
        let _e242 = (f32(_e233.z) * 0.0000000014629181f);
        let _e246 = vec2<f32>(sin(_e242), -(cos(_e242)));
        let _e250 = select(0f, _e239, (_e123 == 0f));
        phi_1375_ = _e246;
        phi_1364_ = _e250;
        phi_1361_ = _e239;
        phi_1351_ = _e246;
        if O7_ {
            let _e252 = ((_e231 & 1048576u) != 0u);
            phi_1336_ = _e239;
            if _e252 {
                phi_1336_ = min(_e239, 0f);
            }
            let _e255 = phi_1336_;
            phi_1348_ = _e255;
            if ((_e231 & 524288u) != 0u) {
                phi_1348_ = max(_e255, 0f);
            }
            let _e260 = phi_1348_;
            let _e261 = (_e231 & 469762048u);
            phi_1376_ = _e246;
            phi_1365_ = _e250;
            phi_1352_ = _e246;
            if (_e261 > 201326592u) {
                let _e266 = f32((_e233.z & 65535u));
                let _e267 = (_e266 * 0.000015259022f);
                let _e271 = sqrt(max((1f - (_e267 * _e267)), 0f));
                phi_1340_ = _e271;
                if (((_e231 & 4194304u) != 0u) == _e252) {
                    phi_1340_ = -(_e271);
                }
                let _e275 = phi_1340_;
                let _e280 = (mat2x2<f32>(vec2<f32>(_e267, _e275), vec2<f32>(-(_e275), _e267)) * _e246);
                let _e281 = (_e261 == 268435456u);
                phi_661_ = _e281;
                if !(_e281) {
                    phi_661_ = ((_e261 != 402653184u) && (_e267 < 0.25f));
                }
                let _e287 = phi_661_;
                let _e289 = ((_e231 & 2097152u) != 0u);
                if (_e261 == 402653184u) {
                    phi_1357_ = (_e246 + _e280);
                } else {
                    phi_1358_ = _e246;
                    if (_e289 || !(_e287)) {
                        if _e287 {
                            phi_1343_ = _e267;
                        } else {
                            phi_1343_ = (65535f / _e266);
                        }
                        let _e296 = phi_1343_;
                        phi_1358_ = (_e280 * _e296);
                    }
                    let _e299 = phi_1358_;
                    phi_1357_ = _e299;
                }
                let _e301 = phi_1357_;
                phi_1368_ = _e250;
                if (!(_e80) && _e287) {
                    phi_1368_ = (0.5f * _e260);
                }
                let _e309 = phi_1368_;
                phi_1376_ = select(_e246, _e280, vec2((_e287 || _e289)));
                phi_1365_ = _e309;
                phi_1352_ = _e301;
            }
            let _e311 = phi_1376_;
            let _e313 = phi_1365_;
            let _e315 = phi_1352_;
            phi_1375_ = _e311;
            phi_1364_ = _e313;
            phi_1361_ = _e260;
            phi_1351_ = _e315;
        }
        let _e317 = phi_1375_;
        let _e319 = phi_1364_;
        let _e321 = phi_1361_;
        let _e323 = phi_1351_;
        let _e328 = ((_e166 * (bitcast<vec2<f32>>(_e233.xy) + (_e323 * (_e321 * bitcast<f32>(_e170.z))))) + _e172);
        phi_1387_ = _e328;
        if (_e319 != 0f) {
            phi_1387_ = (_e328 + (sign((_e317 * _naga_inverse_2x2_f32(_e166))) * _e319));
        }
        let _e336 = phi_1387_;
        phi_1386_ = _e336;
    } else {
        if _e127 {
            phi_1312_ = _e151;
        } else {
            phi_1312_ = bitcast<vec2<f32>>(_e233.xy);
        }
        let _e340 = phi_1312_;
        phi_1386_ = ((_e166 * _e340) + _e172);
    }
    let _e344 = phi_1386_;
    if gj {
        let _e345 = (_e153 * 8u);
        let _e349 = JB.v2_[(_e345 + 2u)];
        let _e360 = JB.v2_[(_e345 + 3u)];
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
        P0_ = vec4<f32>(0f, 0f, 0f, 0f);
        V0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e392 = XC.v2_[_e153];
        let _e394 = (_e392.x & 15u);
        phi_1437_ = false;
        if hj {
            let _e397 = ((_e392.x >> bitcast<u32>(4i)) & 15u);
            Q0_ = f32(_e397);
            phi_1437_ = (_e397 != 0u);
        }
        let _e401 = phi_1437_;
        if (_e394 == 1u) {
            P0_ = unpack4x8unorm(_e392.y);
            if _e401 {
                let _e406 = P0_[3u];
                P0_[3u] = (_e406 * _e123);
            } else {
                let _e408 = P0_;
                P0_ = (_e408 * _e123);
            }
        } else {
            let _e410 = (_e153 * 8u);
            let _e413 = JB.v2_[_e410];
            let _e424 = JB.v2_[(_e410 + 1u)];
            let _e429 = ((mat2x2<f32>(vec2<f32>(_e413.x, _e413.y), vec2<f32>(_e413.z, _e413.w)) * _e344) + _e424.xy);
            let _e440 = ((_e424.w + (f32(_e394) * 0.125f)) + (max(_e424.z, 0f) * 0.00024414063f));
            if (_e424.z < 0f) {
                phi_1435_ = -(_e440);
            } else {
                phi_1435_ = _e440;
            }
            let _e443 = phi_1435_;
            P0_ = vec4<f32>(_e429.x, _e429.y, _e443, (((_e123 * -0.5f) - 0.25f) - round((bitcast<f32>(_e392.y) * 255f))));
        }
        phi_883_ = jj;
        if jj {
            phi_883_ = ((_e392.x & 2048u) != 0u);
        }
        let _e453 = phi_883_;
        if _e453 {
            let _e454 = (_e153 * 8u);
            let _e458 = JB.v2_[(_e454 + 4u)];
            let _e469 = JB.v2_[(_e454 + 5u)];
            let _e472 = ((mat2x2<f32>(vec2<f32>(_e458.x, _e458.y), vec2<f32>(_e458.z, _e458.w)) * _e344) + _e469.xy);
            V0_ = vec3<f32>(_e472.x, _e472.y, (1f + _e469.z));
        } else {
            V0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e479 = j.Mg;
    let _e481 = j.Ng;
    let _e489 = vec4<f32>(((_e344.x * _e479) - 1f), ((_e344.y * _e481) - sign(_e481)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if O7_ {
                phi_1447_ = true;
                break;
            }
            if ke {
                phi_1447_ = true;
                break;
            }
            phi_1447_ = false;
            break;
        }
    }
    let _e492 = phi_1447_;
    if _e492 {
        let _e494 = u32((_e123 * 254f));
        phi_1456_ = _e494;
        if ((_e75 & 1073741824i) == 0i) {
            phi_1456_ = (_e494 + bitcast<u32>(1i));
        }
        let _e499 = phi_1456_;
        phi_1455_ = _e499;
    } else {
        phi_1455_ = 255u;
    }
    let _e501 = phi_1455_;
    unnamed.gl_Position = vec4<f32>(_e489.x, _e489.y, ((f32(((_e178.x << bitcast<u32>(8u)) | _e501)) * 0.000000059604645f) + 0.000000029802322f), _e489.w);
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
