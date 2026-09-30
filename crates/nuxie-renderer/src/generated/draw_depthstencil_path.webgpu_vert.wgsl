enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct Jg {
    g2_: array<vec4<u32>>,
}

struct Ig {
    g2_: array<vec4<u32>>,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct TB {
    tc: f32,
    Bd: f32,
    Hf: f32,
    If: f32,
    o6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    T7_: vec4<i32>,
    hh: vec2<f32>,
    Cd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    X2_: f32,
    Dd: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Ed: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct lf {
    g2_: array<vec4<f32>>,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override Hh: bool = true;
@id(2) override Jh: bool = true;
@id(1) override Ih: bool = true;
@id(8) override Ph: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ID: Jg;
@group(0) @binding(2)
var<storage> OB: Ig;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> DD: kf;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> Y1_: vec2<f32>;
var<private> f1_: f32;
@group(0) @binding(4)
var<storage> PB: lf;
var<private> X1_: vec4<f32>;
var<private> D2_: vec3<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_2241_: f32;
    var phi_2213_: i32;
    var phi_1419_: bool;
    var phi_2226_: i32;
    var phi_2218_: vec4<u32>;
    var phi_2225_: i32;
    var phi_2217_: vec4<u32>;
    var phi_2224_: i32;
    var phi_2222_: vec4<u32>;
    var phi_2221_: u32;
    var phi_2228_: vec2<i32>;
    var phi_2229_: vec4<u32>;
    var phi_2233_: f32;
    var phi_2304_: f32;
    var phi_2247_: f32;
    var phi_2303_: f32;
    var phi_2251_: f32;
    var phi_2248_: f32;
    var phi_2245_: f32;
    var phi_2255_: f32;
    var phi_2301_: f32;
    var phi_2254_: f32;
    var phi_2310_: f32;
    var phi_2307_: f32;
    var phi_2339_: f32;
    var phi_2325_: f32;
    var phi_1707_: bool;
    var phi_2330_: f32;
    var phi_2347_: vec2<f32>;
    var phi_2346_: vec2<f32>;
    var phi_2345_: vec2<f32>;
    var phi_2368_: bool;
    var phi_2363_: vec2<f32>;
    var phi_2348_: vec2<f32>;
    var phi_2391_: u32;
    var phi_2392_: f32;
    var phi_2393_: f32;
    var phi_2432_: f32;
    var phi_2430_: vec4<f32>;
    var phi_2431_: vec4<f32>;
    var phi_1141_: bool;
    var phi_2447_: vec4<f32>;

    let _e81 = gl_InstanceIndex_1;
    let _e82 = VB_1;
    let _e83 = WB_1;
    let _e85 = i32(_e82.x);
    let _e88 = bitcast<i32>(_e82.w);
    let _e90 = (_e88 >> bitcast<u32>(2i));
    let _e91 = (_e88 & 3i);
    let _e93 = min(_e85, (_e90 - 1i));
    let _e95 = ((_e81 * _e90) + _e93);
    let _e100 = textureLoad(MC, vec2<i32>((_e95 & 2047i), (_e95 >> bitcast<u32>(11i))), 0i);
    let _e107 = ID.g2_[(max((_e100.w & 65535u), 1u) - 1u)];
    let _e109 = bitcast<vec2<f32>>(_e107.xy);
    let _e111 = (_e107.z & 65535u);
    let _e113 = (_e111 * 4u);
    let _e116 = OB.g2_[_e113];
    let _e117 = bitcast<vec4<f32>>(_e116);
    let _e124 = mat2x2<f32>(vec2<f32>(_e117.x, _e117.y), vec2<f32>(_e117.z, _e117.w));
    let _e128 = OB.g2_[(_e113 + 1u)];
    let _e132 = bitcast<f32>(_e128.z);
    let _e134 = bitcast<f32>(_e128.w);
    let _e135 = (_e100.w & 8388608u);
    phi_2241_ = _e82.y;
    phi_2213_ = _e85;
    if (_e135 != 0u) {
        phi_2241_ = _e83.y;
        phi_2213_ = i32(_e83.x);
    }
    let _e141 = phi_2241_;
    let _e143 = phi_2213_;
    phi_2224_ = _e95;
    phi_2222_ = _e100;
    phi_2221_ = _e100.w;
    if (_e143 != _e93) {
        let _e146 = ((_e95 + _e143) - _e93);
        let _e151 = textureLoad(MC, vec2<i32>((_e146 & 2047i), (_e146 >> bitcast<u32>(11i))), 0i);
        if ((_e151.w & 8454143u) != (_e100.w & 8454143u)) {
            let _e156 = (_e132 == 0f);
            phi_1419_ = _e156;
            if !(_e156) {
                phi_1419_ = (_e109.x != 0f);
            }
            let _e161 = phi_1419_;
            phi_2226_ = _e95;
            phi_2218_ = _e100;
            if _e161 {
                let _e162 = bitcast<i32>(_e107.w);
                let _e167 = textureLoad(MC, vec2<i32>((_e162 & 2047i), (_e162 >> bitcast<u32>(11i))), 0i);
                phi_2226_ = _e162;
                phi_2218_ = _e167;
            }
            let _e169 = phi_2226_;
            let _e171 = phi_2218_;
            phi_2225_ = _e169;
            phi_2217_ = _e171;
        } else {
            phi_2225_ = _e146;
            phi_2217_ = _e151;
        }
        let _e173 = phi_2225_;
        let _e175 = phi_2217_;
        phi_2224_ = _e173;
        phi_2222_ = _e175;
        phi_2221_ = ((_e175.w & 4286578687u) | _e135);
    }
    let _e180 = phi_2224_;
    let _e182 = phi_2222_;
    let _e184 = phi_2221_;
    let _e185 = (_e184 & 469762048u);
    if ((_e185 == 67108864u) && (_e91 == 0i)) {
        let _e191 = f32((_e182.z & 65535u));
        let _e194 = f32((_e182.z >> bitcast<u32>(16i)));
        let _e200 = vec2<i32>(i32((-1f - _e191)), i32(((_e194 - _e191) + 1f)));
        phi_2228_ = _e200;
        if ((_e184 & 8388608u) != 0u) {
            phi_2228_ = -(_e200);
        }
        let _e205 = phi_2228_;
        let _e207 = (_e180 + _e205.x);
        let _e212 = textureLoad(MC, vec2<i32>((_e207 & 2047i), (_e207 >> bitcast<u32>(11i))), 0i);
        let _e214 = (_e180 + _e205.y);
        let _e219 = textureLoad(MC, vec2<i32>((_e214 & 2047i), (_e214 >> bitcast<u32>(11i))), 0i);
        phi_2229_ = _e219;
        if ((_e219.w & 8454143u) != (_e212.w & 8454143u)) {
            let _e225 = bitcast<i32>(_e107.w);
            let _e230 = textureLoad(MC, vec2<i32>((_e225 & 2047i), (_e225 >> bitcast<u32>(11i))), 0i);
            phi_2229_ = _e230;
        }
        let _e232 = phi_2229_;
        let _e235 = (f32(_e212.z) * 0.0000000014629181f);
        let _e238 = (f32(_e232.z) * 0.0000000014629181f);
        let _e239 = (_e238 - _e235);
        phi_2233_ = _e239;
        if (abs(_e239) > 3.1415927f) {
            phi_2233_ = (_e239 - (6.2831855f * sign(_e239)));
        }
        let _e246 = phi_2233_;
        let _e247 = (_e194 + -2f);
        let _e253 = clamp(round(((abs(_e246) * 0.31830987f) * _e247)), 1f, (_e194 + -3f));
        let _e254 = (_e247 - _e253);
        if (_e191 <= _e254) {
            phi_2304_ = _e141;
            if (_e191 == _e254) {
                phi_2304_ = -(_e141);
            }
            let _e263 = phi_2304_;
            phi_2303_ = _e263;
            phi_2251_ = -(((3.1415927f * sign(_e246)) - _e246));
            phi_2248_ = _e254;
            phi_2245_ = _e191;
        } else {
            let _e265 = (_e191 == (_e254 + 1f));
            if _e265 {
                phi_2247_ = 0f;
            } else {
                phi_2247_ = (_e191 - (_e254 + 2f));
            }
            let _e269 = phi_2247_;
            phi_2303_ = select(_e141, 0f, _e265);
            phi_2251_ = _e246;
            phi_2248_ = select(_e253, 0f, _e265);
            phi_2245_ = _e269;
        }
        let _e273 = phi_2303_;
        let _e275 = phi_2251_;
        let _e277 = phi_2248_;
        let _e279 = phi_2245_;
        if (_e279 == _e277) {
            phi_2255_ = _e238;
        } else {
            phi_2255_ = (_e235 + (_e275 * (_e279 / _e277)));
        }
        let _e285 = phi_2255_;
        phi_2301_ = _e273;
        phi_2254_ = _e285;
    } else {
        phi_2301_ = _e141;
        phi_2254_ = (f32(_e182.z) * 0.0000000014629181f);
    }
    let _e290 = phi_2301_;
    let _e292 = phi_2254_;
    let _e296 = vec2<f32>(sin(_e292), -(cos(_e292)));
    let _e298 = bitcast<vec2<f32>>(_e182.xy);
    phi_2310_ = _e134;
    if (_e134 != 0f) {
        phi_2310_ = max(_e134, (1f / length((_e124 * _e296))));
    }
    let _e305 = phi_2310_;
    if (_e132 != 0f) {
        let _e309 = (_e290 * sign(determinant(_e124)));
        let _e311 = ((_e184 & 1048576u) != 0u);
        phi_2307_ = _e309;
        if _e311 {
            phi_2307_ = min(_e309, 0f);
        }
        let _e314 = phi_2307_;
        phi_2339_ = _e314;
        if ((_e184 & 524288u) != 0u) {
            phi_2339_ = max(_e314, 0f);
        }
        let _e319 = phi_2339_;
        let _e321 = select(0f, _e305, (_e305 != 0f));
        let _e325 = select(_e132, _e321, ((_e321 > _e132) && (_e305 == 0f)));
        let _e326 = (_e325 + _e321);
        let _e327 = (_e296 * _e326);
        phi_2345_ = _e327;
        if (_e185 > 134217728u) {
            let _e333 = f32((_e182.z & 65535u));
            let _e334 = (_e333 * 0.000015259022f);
            let _e338 = sqrt(max((1f - (_e334 * _e334)), 0f));
            phi_2325_ = _e338;
            if (((_e184 & 4194304u) != 0u) == _e311) {
                phi_2325_ = -(_e338);
            }
            let _e342 = phi_2325_;
            let _e347 = (mat2x2<f32>(vec2<f32>(_e334, _e342), vec2<f32>(-(_e342), _e334)) * _e296);
            let _e348 = (_e124 * _e347);
            let _e357 = (_e185 == 335544320u);
            phi_1707_ = _e357;
            if !(_e357) {
                phi_1707_ = ((_e185 == 268435456u) && (_e334 >= 0.25f));
            }
            let _e363 = phi_1707_;
            if _e363 {
                phi_2330_ = (_e325 * (1f / max(_e334, select(0.25f, 1f, ((_e184 & 33554432u) != 0u)))));
            } else {
                phi_2330_ = ((_e325 * _e334) + (((abs(_e348.x) + abs(_e348.y)) * (1f / dot(_e348, _e348))) * 0.5f));
            }
            let _e374 = phi_2330_;
            phi_2346_ = _e327;
            if ((_e184 & 2097152u) != 0u) {
                if (_e326 <= ((_e374 * _e334) + (_e321 * 0.125f))) {
                    phi_2347_ = (_e347 * (_e326 * (65535f / _e333)));
                } else {
                    let _e384 = (_e347 * _e374);
                    phi_2347_ = (vec2<f32>(dot(_e327, _e327), dot(_e384, _e384)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e327, _e384)));
                }
                let _e392 = phi_2347_;
                phi_2346_ = _e392;
            }
            let _e394 = phi_2346_;
            phi_2345_ = _e394;
        }
        let _e396 = phi_2345_;
        phi_2368_ = (_e91 != 0i);
        phi_2363_ = (_e124 * (_e396 * _e319));
        phi_2348_ = _e298;
    } else {
        phi_2368_ = (((_e184 & 2147483648u) != 0u) && (_e91 != 1i));
        phi_2363_ = vec2<f32>(0f, 0f);
        phi_2348_ = select(_e298, _e109, vec2((_e91 == 2i)));
    }
    let _e408 = phi_2368_;
    let _e410 = phi_2363_;
    let _e412 = phi_2348_;
    let _e415 = (((_e124 * _e412) + _e410) + bitcast<vec2<f32>>(_e128.xy));
    let _e419 = OB.g2_[(_e113 + 2u)];
    let _e423 = DD.g2_[_e111];
    let _e425 = (_e423.x & 15u);
    if Hh {
        let _e426 = (_e425 == 0u);
        if _e426 {
            phi_2391_ = _e423.y;
        } else {
            phi_2391_ = _e423.x;
        }
        let _e429 = phi_2391_;
        let _e431 = (_e429 >> bitcast<u32>(16i));
        let _e433 = j.c6_;
        if (_e431 == 0u) {
            phi_2392_ = 0f;
        } else {
            phi_2392_ = unpack2x16float(((_e431 + 1023u) * _e433)).x;
        }
        let _e440 = phi_2392_;
        phi_2393_ = _e440;
        if _e426 {
            phi_2393_ = -(_e440);
        }
        let _e443 = phi_2393_;
        Y1_[0u] = _e443;
    }
    if Jh {
        f1_ = f32(((_e423.x >> bitcast<u32>(4i)) & 15u));
    }
    if Ih {
        let _e449 = (_e111 * 8u);
        let _e453 = PB.g2_[(_e449 + 2u)];
        let _e464 = PB.g2_[(_e449 + 3u)];
        if any((_e453 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e469 = ((mat2x2<f32>(vec2<f32>(_e453.x, _e453.y), vec2<f32>(_e453.z, _e453.w)) * _e415) + _e464.xy);
            unnamed.gl_ClipDistance[0i] = (_e469.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e469.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e469.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e469.y);
        } else {
            let _e485 = (_e464.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e485;
            unnamed.gl_ClipDistance[2i] = _e485;
            unnamed.gl_ClipDistance[1i] = _e485;
            unnamed.gl_ClipDistance[0i] = _e485;
        }
    }
    if (_e425 == 1u) {
        X1_ = unpack4x8unorm(_e423.y);
    } else {
        if (Hh && (_e425 == 0u)) {
            let _e500 = (_e423.x >> bitcast<u32>(16i));
            let _e502 = j.c6_;
            if (_e500 == 0u) {
                phi_2432_ = 0f;
            } else {
                phi_2432_ = unpack2x16float(((_e500 + 1023u) * _e502)).x;
            }
            let _e509 = phi_2432_;
            Y1_[1u] = _e509;
        } else {
            let _e511 = (_e111 * 8u);
            let _e514 = PB.g2_[_e511];
            let _e525 = PB.g2_[(_e511 + 1u)];
            let _e534 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e423.y));
            let _e536 = ((mat2x2<f32>(vec2<f32>(_e514.x, _e514.y), vec2<f32>(_e514.z, _e514.w)) * _e415) + _e525.xy);
            if (_e525.z > 0.9f) {
                phi_2430_ = vec4<f32>(_e534.x, _e534.y, 2f, _e534.w);
            } else {
                phi_2430_ = vec4<f32>(_e534.x, _e534.y, _e525.w, _e534.w);
            }
            let _e551 = phi_2430_;
            if (f32(_e425) == 2f) {
                let _e558 = vec4<f32>(_e536.x, _e551.y, _e551.z, _e551.w);
                phi_2431_ = vec4<f32>(_e558.x, 0f, _e558.z, _e558.w);
            } else {
                let _e570 = vec4<f32>(_e551.x, _e551.y, -(_e551.z), _e551.w);
                let _e576 = vec4<f32>(_e536.x, _e570.y, _e570.z, _e570.w);
                phi_2431_ = vec4<f32>(_e576.x, _e536.y, _e576.z, _e576.w);
            }
            let _e584 = phi_2431_;
            X1_ = _e584;
            let _e586 = X1_[3u];
            X1_[3u] = -(_e586);
        }
    }
    phi_1141_ = Ph;
    if Ph {
        phi_1141_ = ((_e423.x & 2048u) != 0u);
    }
    let _e591 = phi_1141_;
    if _e591 {
        let _e592 = (_e111 * 8u);
        let _e596 = PB.g2_[(_e592 + 4u)];
        let _e607 = PB.g2_[(_e592 + 5u)];
        let _e610 = ((mat2x2<f32>(vec2<f32>(_e596.x, _e596.y), vec2<f32>(_e596.z, _e596.w)) * _e415) + _e607.xy);
        D2_ = vec3<f32>(_e610.x, _e610.y, (1f + _e607.z));
    } else {
        D2_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e408) {
        let _e618 = j.Hf;
        let _e620 = j.If;
        let _e628 = vec4<f32>(((_e415.x * _e618) - 1f), ((_e415.y * _e620) - sign(_e620)), 0f, 1f);
        phi_2447_ = vec4<f32>(_e628.x, _e628.y, ((f32(((_e419.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e628.w);
    } else {
        let _e641 = j.X2_;
        phi_2447_ = vec4(_e641);
    }
    let _e644 = phi_2447_;
    unnamed.gl_Position = _e644;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) VB: vec4<f32>, @location(1) WB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    VB_1 = VB;
    WB_1 = WB;
    main_1();
    let _e17 = unnamed.gl_Position;
    let _e18 = unnamed.gl_ClipDistance;
    let _e19 = Y1_;
    let _e20 = f1_;
    let _e21 = X1_;
    let _e22 = D2_;
    return VertexOutput(_e17, _e18, _e19, _e20, _e21, _e22);
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
