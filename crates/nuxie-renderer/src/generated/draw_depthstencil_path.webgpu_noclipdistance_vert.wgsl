struct qi {
    v2_: array<vec4<u32>>,
}

struct pi {
    v2_: array<vec4<u32>>,
}

struct vg {
    v2_: array<vec4<f32>>,
}

struct ug {
    v2_: array<vec2<u32>>,
}

struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(16) override N7_: bool = false;
@id(17) override qe: bool = false;
@id(2) override lj: bool = true;
@id(4) override nj: bool = true;

var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> DD: qi;
@group(0) @binding(2)
var<storage> KB: pi;
@group(0) @binding(4)
var<storage> JB: vg;
var<private> N0_: vec4<f32>;
var<private> O0_: vec3<f32>;
@group(0) @binding(3)
var<storage> XC: ug;
var<private> S0_: f32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var eb: sampler;

fn main_1() {
    var phi_1204_: bool;
    var phi_1205_: i32;
    var phi_1261_: f32;
    var phi_1214_: f32;
    var phi_1211_: bool;
    var phi_1209_: i32;
    var phi_1208_: i32;
    var phi_1206_: i32;
    var phi_1253_: f32;
    var phi_1252_: f32;
    var phi_1217_: bool;
    var phi_1251_: f32;
    var phi_1216_: bool;
    var phi_1222_: i32;
    var phi_1227_: bool;
    var phi_1229_: vec4<u32>;
    var phi_1228_: vec4<u32>;
    var phi_1269_: u32;
    var phi_1246_: vec4<u32>;
    var phi_1235_: bool;
    var phi_1271_: f32;
    var phi_1283_: f32;
    var phi_1275_: f32;
    var phi_612_: bool;
    var phi_1278_: f32;
    var phi_1293_: vec2<f32>;
    var phi_1292_: vec2<f32>;
    var phi_1303_: f32;
    var phi_1311_: vec2<f32>;
    var phi_1300_: f32;
    var phi_1287_: vec2<f32>;
    var phi_1310_: vec2<f32>;
    var phi_1299_: f32;
    var phi_1296_: f32;
    var phi_1286_: vec2<f32>;
    var phi_1322_: vec2<f32>;
    var phi_1247_: vec2<f32>;
    var phi_1321_: vec2<f32>;
    var phi_1368_: bool;
    var phi_1366_: f32;
    var phi_837_: bool;
    var phi_1369_: f32;
    var phi_1378_: bool;
    var phi_1388_: u32;
    var phi_1387_: u32;

    let _e77 = gl_VertexIndex_1;
    let _e82 = ((_e77 & 536870912i) != 0i);
    let _e83 = (_e77 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if N7_ {
                phi_1204_ = true;
                break;
            }
            if qe {
                phi_1204_ = true;
                break;
            }
            phi_1204_ = false;
            break;
        }
    }
    let _e86 = phi_1204_;
    if _e86 {
        let _e87 = select(5i, 6i, _e82);
        let _e93 = (_e83 & ((1i << bitcast<u32>(_e87)) - 1i));
        let _e94 = select(1i, 2i, _e82);
        let _e100 = (_e93 & ((1i << bitcast<u32>(_e94)) - 1i));
        phi_1205_ = _e100;
        if (N7_ && !(_e82)) {
            phi_1205_ = (_e100 + 1i);
        }
        let _e105 = phi_1205_;
        phi_1261_ = select(1f, 0f, ((_e105 == 0i) || (_e105 == 3i)));
        phi_1214_ = select(-1f, 1f, (_e105 < 2i));
        phi_1211_ = false;
        phi_1209_ = (_e83 >> bitcast<u32>(_e87));
        phi_1208_ = 8i;
        phi_1206_ = (_e93 >> bitcast<u32>(_e94));
    } else {
        let _e112 = select(4i, 5i, _e82);
        let _e118 = (_e83 & ((1i << bitcast<u32>(_e112)) - 1i));
        let _e122 = (!(_e82) && (_e118 == 9i));
        phi_1261_ = 1f;
        phi_1214_ = 0f;
        phi_1211_ = _e122;
        phi_1209_ = (_e83 >> bitcast<u32>(_e112));
        phi_1208_ = select(8i, 17i, _e82);
        phi_1206_ = select(_e118, 0i, _e122);
    }
    let _e125 = phi_1261_;
    let _e127 = phi_1214_;
    let _e129 = phi_1211_;
    let _e131 = phi_1209_;
    let _e133 = phi_1208_;
    let _e135 = phi_1206_;
    let _e137 = min(_e135, (_e133 - 1i));
    let _e139 = ((_e131 * _e133) + _e137);
    let _e144 = textureLoad(VB, vec2<i32>((_e139 & 2047i), (_e139 >> bitcast<u32>(11i))), 0i);
    let _e151 = DD.v2_[(max((_e144.w & 65535u), 1u) - 1u)];
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
    if N7_ {
        phi_1251_ = _e127;
        phi_1216_ = false;
    } else {
        if qe {
            let _e183 = (_e182 != 0u);
            phi_1253_ = _e127;
            if _e183 {
                phi_1253_ = -(_e127);
            }
            let _e186 = phi_1253_;
            phi_1252_ = _e186;
            phi_1217_ = _e183;
        } else {
            phi_1252_ = _e127;
            phi_1217_ = (((_e182 != 0u) && !(_e82)) && !(_e129));
        }
        let _e193 = phi_1252_;
        let _e195 = phi_1217_;
        phi_1251_ = _e193;
        phi_1216_ = _e195;
    }
    let _e197 = phi_1251_;
    let _e199 = phi_1216_;
    phi_1222_ = _e135;
    if _e199 {
        phi_1222_ = (_e135 - 1i);
    }
    let _e202 = phi_1222_;
    phi_1269_ = _e144.w;
    phi_1246_ = _e144;
    if (_e202 != _e137) {
        let _e205 = ((_e139 + _e202) - _e137);
        let _e210 = textureLoad(VB, vec2<i32>((_e205 & 2047i), (_e205 >> bitcast<u32>(11i))), 0i);
        if ((_e210.w & 8454143u) != (_e144.w & 8454143u)) {
            if N7_ {
                phi_1227_ = (_e153.x != 0f);
            } else {
                phi_1227_ = true;
            }
            let _e218 = phi_1227_;
            phi_1229_ = _e144;
            if _e218 {
                let _e219 = bitcast<i32>(_e151.w);
                let _e224 = textureLoad(VB, vec2<i32>((_e219 & 2047i), (_e219 >> bitcast<u32>(11i))), 0i);
                phi_1229_ = _e224;
            }
            let _e226 = phi_1229_;
            phi_1228_ = _e226;
        } else {
            phi_1228_ = _e210;
        }
        let _e228 = phi_1228_;
        phi_1269_ = ((_e228.w & 4286578687u) | _e182);
        phi_1246_ = _e228;
    }
    let _e233 = phi_1269_;
    let _e235 = phi_1246_;
    switch bitcast<i32>(0u) {
        default: {
            if N7_ {
                phi_1235_ = true;
                break;
            }
            if qe {
                phi_1235_ = true;
                break;
            }
            phi_1235_ = false;
            break;
        }
    }
    let _e238 = phi_1235_;
    if _e238 {
        let _e241 = (_e197 * sign(determinant(_e168)));
        let _e244 = (f32(_e235.z) * 0.0000000014629181f);
        let _e248 = vec2<f32>(sin(_e244), -(cos(_e244)));
        let _e252 = select(0f, _e241, (_e125 == 0f));
        phi_1310_ = _e248;
        phi_1299_ = _e252;
        phi_1296_ = _e241;
        phi_1286_ = _e248;
        if N7_ {
            let _e254 = ((_e233 & 1048576u) != 0u);
            phi_1271_ = _e241;
            if _e254 {
                phi_1271_ = min(_e241, 0f);
            }
            let _e257 = phi_1271_;
            phi_1283_ = _e257;
            if ((_e233 & 524288u) != 0u) {
                phi_1283_ = max(_e257, 0f);
            }
            let _e262 = phi_1283_;
            let _e263 = (_e233 & 469762048u);
            phi_1311_ = _e248;
            phi_1300_ = _e252;
            phi_1287_ = _e248;
            if (_e263 > 201326592u) {
                let _e268 = f32((_e235.z & 65535u));
                let _e269 = (_e268 * 0.000015259022f);
                let _e273 = sqrt(max((1f - (_e269 * _e269)), 0f));
                phi_1275_ = _e273;
                if (((_e233 & 4194304u) != 0u) == _e254) {
                    phi_1275_ = -(_e273);
                }
                let _e277 = phi_1275_;
                let _e282 = (mat2x2<f32>(vec2<f32>(_e269, _e277), vec2<f32>(-(_e277), _e269)) * _e248);
                let _e283 = (_e263 == 268435456u);
                phi_612_ = _e283;
                if !(_e283) {
                    phi_612_ = ((_e263 != 402653184u) && (_e269 < 0.25f));
                }
                let _e289 = phi_612_;
                let _e291 = ((_e233 & 2097152u) != 0u);
                if (_e263 == 402653184u) {
                    phi_1292_ = (_e248 + _e282);
                } else {
                    phi_1293_ = _e248;
                    if (_e291 || !(_e289)) {
                        if _e289 {
                            phi_1278_ = _e269;
                        } else {
                            phi_1278_ = (65535f / _e268);
                        }
                        let _e298 = phi_1278_;
                        phi_1293_ = (_e282 * _e298);
                    }
                    let _e301 = phi_1293_;
                    phi_1292_ = _e301;
                }
                let _e303 = phi_1292_;
                phi_1303_ = _e252;
                if (!(_e82) && _e289) {
                    phi_1303_ = (0.5f * _e262);
                }
                let _e311 = phi_1303_;
                phi_1311_ = select(_e248, _e282, vec2((_e289 || _e291)));
                phi_1300_ = _e311;
                phi_1287_ = _e303;
            }
            let _e313 = phi_1311_;
            let _e315 = phi_1300_;
            let _e317 = phi_1287_;
            phi_1310_ = _e313;
            phi_1299_ = _e315;
            phi_1296_ = _e262;
            phi_1286_ = _e317;
        }
        let _e319 = phi_1310_;
        let _e321 = phi_1299_;
        let _e323 = phi_1296_;
        let _e325 = phi_1286_;
        let _e330 = ((_e168 * (bitcast<vec2<f32>>(_e235.xy) + (_e325 * (_e323 * bitcast<f32>(_e172.z))))) + _e174);
        phi_1322_ = _e330;
        if (_e321 != 0f) {
            phi_1322_ = (_e330 + (sign((_e319 * _naga_inverse_2x2_f32(_e168))) * _e321));
        }
        let _e338 = phi_1322_;
        phi_1321_ = _e338;
    } else {
        if _e129 {
            phi_1247_ = _e153;
        } else {
            phi_1247_ = bitcast<vec2<f32>>(_e235.xy);
        }
        let _e342 = phi_1247_;
        phi_1321_ = ((_e168 * _e342) + _e174);
    }
    let _e346 = phi_1321_;
    if ((_e77 & 268435456i) != 0i) {
        N0_ = vec4<f32>(0f, 0f, 0f, 0f);
        O0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e349 = XC.v2_[_e155];
        let _e351 = (_e349.x & 15u);
        phi_1368_ = false;
        if lj {
            let _e354 = ((_e349.x >> bitcast<u32>(4i)) & 15u);
            S0_ = f32(_e354);
            phi_1368_ = (_e354 != 0u);
        }
        let _e358 = phi_1368_;
        if (_e351 == 1u) {
            N0_ = unpack4x8unorm(_e349.y);
            if _e358 {
                let _e363 = N0_[3u];
                N0_[3u] = (_e363 * _e125);
            } else {
                let _e365 = N0_;
                N0_ = (_e365 * _e125);
            }
        } else {
            let _e367 = (_e155 * 8u);
            let _e370 = JB.v2_[_e367];
            let _e381 = JB.v2_[(_e367 + 1u)];
            let _e386 = ((mat2x2<f32>(vec2<f32>(_e370.x, _e370.y), vec2<f32>(_e370.z, _e370.w)) * _e346) + _e381.xy);
            let _e397 = ((_e381.w + (f32(_e351) * 0.125f)) + (max(_e381.z, 0f) * 0.00024414063f));
            if (_e381.z < 0f) {
                phi_1366_ = -(_e397);
            } else {
                phi_1366_ = _e397;
            }
            let _e400 = phi_1366_;
            N0_ = vec4<f32>(_e386.x, _e386.y, _e400, (((_e125 * -0.5f) - 0.25f) - round((bitcast<f32>(_e349.y) * 255f))));
        }
        phi_837_ = nj;
        if nj {
            phi_837_ = ((_e349.x & 2048u) != 0u);
        }
        let _e410 = phi_837_;
        if _e410 {
            let _e411 = (_e155 * 8u);
            let _e415 = JB.v2_[(_e411 + 4u)];
            let _e426 = JB.v2_[(_e411 + 5u)];
            let _e429 = ((mat2x2<f32>(vec2<f32>(_e415.x, _e415.y), vec2<f32>(_e415.z, _e415.w)) * _e346) + _e426.xy);
            phi_1369_ = (1f + _e426.z);
            if ((_e349.x & 4096u) != 0u) {
                phi_1369_ = (-1f - f32(((_e349.x & 24576u) >> bitcast<u32>(13u))));
            }
            let _e440 = phi_1369_;
            O0_ = vec3<f32>(_e429.x, _e429.y, _e440);
        } else {
            O0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e445 = j.Sg;
    let _e447 = j.Tg;
    let _e455 = vec4<f32>(((_e346.x * _e445) - 1f), ((_e346.y * _e447) - sign(_e447)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if N7_ {
                phi_1378_ = true;
                break;
            }
            if qe {
                phi_1378_ = true;
                break;
            }
            phi_1378_ = false;
            break;
        }
    }
    let _e458 = phi_1378_;
    if _e458 {
        let _e460 = u32((_e125 * 254f));
        phi_1388_ = _e460;
        if ((_e77 & 1073741824i) == 0i) {
            phi_1388_ = (_e460 + bitcast<u32>(1i));
        }
        let _e465 = phi_1388_;
        phi_1387_ = _e465;
    } else {
        phi_1387_ = 255u;
    }
    let _e467 = phi_1387_;
    unnamed.gl_Position = vec4<f32>(_e455.x, _e455.y, ((f32(((_e180.x << bitcast<u32>(8u)) | _e467)) * 0.000000059604645f) + 0.000000029802322f), _e455.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e8 = N0_;
    let _e9 = O0_;
    let _e10 = S0_;
    let _e11 = unnamed.gl_Position;
    return VertexOutput(_e8, _e9, _e10, _e11);
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
