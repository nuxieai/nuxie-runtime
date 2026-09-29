enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct Cg {
    e2_: array<vec4<u32>>,
}

struct Bg {
    e2_: array<vec4<u32>>,
}

struct hf {
    e2_: array<vec2<u32>>,
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

struct jf {
    e2_: array<vec4<f32>>,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override Bh: bool = true;
@id(2) override Dh: bool = true;
@id(1) override Ch: bool = true;
@id(8) override Jh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ID: Cg;
@group(0) @binding(2)
var<storage> PB: Bg;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> DD: hf;
@group(0) @binding(0)
var<uniform> n: BC;
var<private> W1_: vec2<f32>;
var<private> g2_: f32;
@group(0) @binding(4)
var<storage> QB: jf;
var<private> V1_: vec4<f32>;
var<private> C2_: vec3<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ea: sampler;

fn main_1() {
    var phi_2270_: f32;
    var phi_2242_: i32;
    var phi_1438_: bool;
    var phi_2255_: i32;
    var phi_2247_: vec4<u32>;
    var phi_2254_: i32;
    var phi_2246_: vec4<u32>;
    var phi_2253_: i32;
    var phi_2251_: vec4<u32>;
    var phi_2250_: u32;
    var phi_2257_: vec2<i32>;
    var phi_2258_: vec4<u32>;
    var phi_2262_: f32;
    var phi_2333_: f32;
    var phi_2276_: f32;
    var phi_2332_: f32;
    var phi_2280_: f32;
    var phi_2277_: f32;
    var phi_2274_: f32;
    var phi_2284_: f32;
    var phi_2330_: f32;
    var phi_2283_: f32;
    var phi_2339_: f32;
    var phi_2336_: f32;
    var phi_2393_: f32;
    var phi_2365_: i32;
    var phi_2375_: f32;
    var phi_1750_: bool;
    var phi_2382_: f32;
    var phi_2403_: vec2<f32>;
    var phi_2402_: vec2<f32>;
    var phi_2401_: vec2<f32>;
    var phi_2419_: vec2<f32>;
    var phi_2404_: vec2<f32>;
    var phi_2452_: u32;
    var phi_2423_: vec2<f32>;
    var phi_2422_: bool;
    var local: u32;
    var phi_2481_: u32;
    var phi_2482_: f32;
    var phi_2483_: f32;
    var local_1: u32;
    var phi_2524_: vec4<f32>;
    var phi_2523_: f32;
    var local_2: u32;
    var phi_2521_: vec4<f32>;
    var phi_2522_: vec4<f32>;
    var phi_1155_: bool;
    var local_3: u32;
    var phi_2540_: vec4<f32>;

    let _e80 = gl_InstanceIndex_1;
    let _e81 = UB_1;
    let _e82 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e85 = i32(_e81.x);
            let _e88 = bitcast<i32>(_e81.w);
            let _e90 = (_e88 >> bitcast<u32>(2i));
            let _e91 = (_e88 & 3i);
            let _e93 = min(_e85, (_e90 - 1i));
            let _e95 = ((_e80 * _e90) + _e93);
            let _e100 = textureLoad(KC, vec2<i32>((_e95 & 2047i), (_e95 >> bitcast<u32>(11i))), 0i);
            let _e107 = ID.e2_[(max((_e100.w & 65535u), 1u) - 1u)];
            let _e109 = bitcast<vec2<f32>>(_e107.xy);
            let _e111 = (_e107.z & 65535u);
            let _e113 = (_e111 * 4u);
            let _e116 = PB.e2_[_e113];
            let _e117 = bitcast<vec4<f32>>(_e116);
            let _e124 = mat2x2<f32>(vec2<f32>(_e117.x, _e117.y), vec2<f32>(_e117.z, _e117.w));
            let _e128 = PB.e2_[(_e113 + 1u)];
            let _e132 = bitcast<f32>(_e128.z);
            let _e134 = bitcast<f32>(_e128.w);
            let _e135 = (_e100.w & 8388608u);
            phi_2270_ = _e81.y;
            phi_2242_ = _e85;
            local = _e111;
            local_1 = _e111;
            local_2 = _e111;
            local_3 = _e111;
            if (_e135 != 0u) {
                phi_2270_ = _e82.y;
                phi_2242_ = i32(_e82.x);
            }
            let _e141 = phi_2270_;
            let _e143 = phi_2242_;
            phi_2253_ = _e95;
            phi_2251_ = _e100;
            phi_2250_ = _e100.w;
            if (_e143 != _e93) {
                let _e146 = ((_e95 + _e143) - _e93);
                let _e151 = textureLoad(KC, vec2<i32>((_e146 & 2047i), (_e146 >> bitcast<u32>(11i))), 0i);
                if ((_e151.w & 8454143u) != (_e100.w & 8454143u)) {
                    let _e156 = (_e132 == 0f);
                    phi_1438_ = _e156;
                    if !(_e156) {
                        phi_1438_ = (_e109.x != 0f);
                    }
                    let _e161 = phi_1438_;
                    phi_2255_ = _e95;
                    phi_2247_ = _e100;
                    if _e161 {
                        let _e162 = bitcast<i32>(_e107.w);
                        let _e167 = textureLoad(KC, vec2<i32>((_e162 & 2047i), (_e162 >> bitcast<u32>(11i))), 0i);
                        phi_2255_ = _e162;
                        phi_2247_ = _e167;
                    }
                    let _e169 = phi_2255_;
                    let _e171 = phi_2247_;
                    phi_2254_ = _e169;
                    phi_2246_ = _e171;
                } else {
                    phi_2254_ = _e146;
                    phi_2246_ = _e151;
                }
                let _e173 = phi_2254_;
                let _e175 = phi_2246_;
                phi_2253_ = _e173;
                phi_2251_ = _e175;
                phi_2250_ = ((_e175.w & 4286578687u) | _e135);
            }
            let _e180 = phi_2253_;
            let _e182 = phi_2251_;
            let _e184 = phi_2250_;
            let _e185 = (_e184 & 469762048u);
            if ((_e185 == 67108864u) && (_e91 == 0i)) {
                let _e191 = f32((_e182.z & 65535u));
                let _e194 = f32((_e182.z >> bitcast<u32>(16i)));
                let _e200 = vec2<i32>(i32((-1f - _e191)), i32(((_e194 - _e191) + 1f)));
                phi_2257_ = _e200;
                if ((_e184 & 8388608u) != 0u) {
                    phi_2257_ = -(_e200);
                }
                let _e205 = phi_2257_;
                let _e207 = (_e180 + _e205.x);
                let _e212 = textureLoad(KC, vec2<i32>((_e207 & 2047i), (_e207 >> bitcast<u32>(11i))), 0i);
                let _e214 = (_e180 + _e205.y);
                let _e219 = textureLoad(KC, vec2<i32>((_e214 & 2047i), (_e214 >> bitcast<u32>(11i))), 0i);
                phi_2258_ = _e219;
                if ((_e219.w & 8454143u) != (_e212.w & 8454143u)) {
                    let _e225 = bitcast<i32>(_e107.w);
                    let _e230 = textureLoad(KC, vec2<i32>((_e225 & 2047i), (_e225 >> bitcast<u32>(11i))), 0i);
                    phi_2258_ = _e230;
                }
                let _e232 = phi_2258_;
                let _e234 = bitcast<f32>(_e212.z);
                let _e236 = bitcast<f32>(_e232.z);
                let _e237 = (_e236 - _e234);
                phi_2262_ = _e237;
                if (abs(_e237) > 3.1415927f) {
                    phi_2262_ = (_e237 - (6.2831855f * sign(_e237)));
                }
                let _e244 = phi_2262_;
                let _e245 = (_e194 + -2f);
                let _e251 = clamp(round(((abs(_e244) * 0.31830987f) * _e245)), 1f, (_e194 + -3f));
                let _e252 = (_e245 - _e251);
                if (_e191 <= _e252) {
                    phi_2333_ = _e141;
                    if (_e191 == _e252) {
                        phi_2333_ = -(_e141);
                    }
                    let _e261 = phi_2333_;
                    phi_2332_ = _e261;
                    phi_2280_ = -(((3.1415927f * sign(_e244)) - _e244));
                    phi_2277_ = _e252;
                    phi_2274_ = _e191;
                } else {
                    let _e263 = (_e191 == (_e252 + 1f));
                    if _e263 {
                        phi_2276_ = 0f;
                    } else {
                        phi_2276_ = (_e191 - (_e252 + 2f));
                    }
                    let _e267 = phi_2276_;
                    phi_2332_ = select(_e141, 0f, _e263);
                    phi_2280_ = _e244;
                    phi_2277_ = select(_e251, 0f, _e263);
                    phi_2274_ = _e267;
                }
                let _e271 = phi_2332_;
                let _e273 = phi_2280_;
                let _e275 = phi_2277_;
                let _e277 = phi_2274_;
                if (_e277 == _e275) {
                    phi_2284_ = _e236;
                } else {
                    phi_2284_ = (_e234 + (_e273 * (_e277 / _e275)));
                }
                let _e283 = phi_2284_;
                phi_2330_ = _e271;
                phi_2283_ = _e283;
            } else {
                phi_2330_ = _e141;
                phi_2283_ = bitcast<f32>(_e182.z);
            }
            let _e287 = phi_2330_;
            let _e289 = phi_2283_;
            let _e293 = vec2<f32>(sin(_e289), -(cos(_e289)));
            let _e295 = bitcast<vec2<f32>>(_e182.xy);
            phi_2339_ = _e134;
            if (_e134 != 0f) {
                phi_2339_ = max(_e134, (1f / length((_e124 * _e293))));
            }
            let _e302 = phi_2339_;
            if (_e132 != 0f) {
                let _e306 = (_e287 * sign(determinant(_e124)));
                let _e308 = ((_e184 & 1048576u) != 0u);
                phi_2336_ = _e306;
                if _e308 {
                    phi_2336_ = min(_e306, 0f);
                }
                let _e311 = phi_2336_;
                phi_2393_ = _e311;
                if ((_e184 & 524288u) != 0u) {
                    phi_2393_ = max(_e311, 0f);
                }
                let _e316 = phi_2393_;
                let _e318 = select(0f, _e302, (_e302 != 0f));
                let _e322 = select(_e132, _e318, ((_e318 > _e132) && (_e302 == 0f)));
                let _e323 = (_e322 + _e318);
                let _e324 = (_e293 * _e323);
                phi_2401_ = _e324;
                if (_e185 > 134217728u) {
                    let _e326 = (_e184 & 4194304u);
                    let _e328 = select(2i, -2i, (_e326 == 0u));
                    phi_2365_ = _e328;
                    if ((_e184 & 8388608u) != 0u) {
                        phi_2365_ = -(_e328);
                    }
                    let _e333 = phi_2365_;
                    let _e334 = (_e180 + _e333);
                    let _e339 = textureLoad(KC, vec2<i32>((_e334 & 2047i), (_e334 >> bitcast<u32>(11i))), 0i);
                    let _e343 = abs((bitcast<f32>(_e339.z) - _e289));
                    phi_2375_ = _e343;
                    if (_e343 > 3.1415927f) {
                        phi_2375_ = (6.2831855f - _e343);
                    }
                    let _e347 = phi_2375_;
                    let _e352 = ((_e347 * select(0.5f, -0.5f, ((_e326 != 0u) == _e308))) + _e289);
                    let _e356 = vec2<f32>(sin(_e352), -(cos(_e352)));
                    let _e357 = (_e124 * _e356);
                    let _e367 = cos((_e347 * 0.5f));
                    let _e368 = (_e185 == 335544320u);
                    phi_1750_ = _e368;
                    if !(_e368) {
                        phi_1750_ = ((_e185 == 268435456u) && (_e367 >= 0.25f));
                    }
                    let _e374 = phi_1750_;
                    if _e374 {
                        phi_2382_ = (_e322 * (1f / max(_e367, select(0.25f, 1f, ((_e184 & 33554432u) != 0u)))));
                    } else {
                        phi_2382_ = ((_e322 * _e367) + (((abs(_e357.x) + abs(_e357.y)) * (1f / dot(_e357, _e357))) * 0.5f));
                    }
                    let _e385 = phi_2382_;
                    phi_2402_ = _e324;
                    if ((_e184 & 2097152u) != 0u) {
                        if (_e323 <= ((_e385 * _e367) + (_e318 * 0.125f))) {
                            phi_2403_ = (_e356 * (_e323 * (1f / _e367)));
                        } else {
                            let _e395 = (_e356 * _e385);
                            phi_2403_ = (vec2<f32>(dot(_e324, _e324), dot(_e395, _e395)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e324, _e395)));
                        }
                        let _e403 = phi_2403_;
                        phi_2402_ = _e403;
                    }
                    let _e405 = phi_2402_;
                    phi_2401_ = _e405;
                }
                let _e407 = phi_2401_;
                if (_e91 != 0i) {
                    phi_2452_ = u32();
                    phi_2423_ = vec2<f32>();
                    phi_2422_ = false;
                    break;
                }
                phi_2419_ = (_e124 * (_e407 * _e316));
                phi_2404_ = _e295;
            } else {
                if (((_e184 & 2147483648u) != 0u) && (_e91 != 1i)) {
                    phi_2452_ = u32();
                    phi_2423_ = vec2<f32>();
                    phi_2422_ = false;
                    break;
                }
                phi_2419_ = vec2<f32>(0f, 0f);
                phi_2404_ = select(_e295, _e109, vec2((_e91 == 2i)));
            }
            let _e419 = phi_2419_;
            let _e421 = phi_2404_;
            let _e428 = PB.e2_[(_e113 + 2u)];
            phi_2452_ = _e428.x;
            phi_2423_ = (((_e124 * _e421) + _e419) + bitcast<vec2<f32>>(_e128.xy));
            phi_2422_ = true;
            break;
        }
    }
    let _e431 = phi_2452_;
    let _e433 = phi_2423_;
    let _e435 = phi_2422_;
    let _e438 = local;
    let _e440 = DD.e2_[_e438];
    let _e442 = (_e440.x & 15u);
    if Bh {
        let _e443 = (_e442 == 0u);
        if _e443 {
            phi_2481_ = _e440.y;
        } else {
            phi_2481_ = _e440.x;
        }
        let _e446 = phi_2481_;
        let _e448 = (_e446 >> bitcast<u32>(16i));
        let _e450 = n.e6_;
        if (_e448 == 0u) {
            phi_2482_ = 0f;
        } else {
            phi_2482_ = unpack2x16float(((_e448 + 1023u) * _e450)).x;
        }
        let _e457 = phi_2482_;
        phi_2483_ = _e457;
        if _e443 {
            phi_2483_ = -(_e457);
        }
        let _e460 = phi_2483_;
        W1_[0u] = _e460;
    }
    if Dh {
        g2_ = f32(((_e440.x >> bitcast<u32>(4i)) & 15u));
    }
    if Ch {
        let _e467 = local_1;
        let _e468 = (_e467 * 8u);
        let _e472 = QB.e2_[(_e468 + 2u)];
        let _e483 = QB.e2_[(_e468 + 3u)];
        if any((_e472 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e488 = ((mat2x2<f32>(vec2<f32>(_e472.x, _e472.y), vec2<f32>(_e472.z, _e472.w)) * _e433) + _e483.xy);
            unnamed.gl_ClipDistance[0i] = (_e488.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e488.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e488.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e488.y);
        } else {
            let _e504 = (_e483.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e504;
            unnamed.gl_ClipDistance[2i] = _e504;
            unnamed.gl_ClipDistance[1i] = _e504;
            unnamed.gl_ClipDistance[0i] = _e504;
        }
    }
    if (_e442 == 1u) {
        let _e515 = unpack4x8unorm(_e440.y);
        if Dh {
            phi_2524_ = _e515;
        } else {
            let _e518 = (_e515.xyz * _e515.w);
            let _e524 = vec4<f32>(_e518.x, _e515.y, _e515.z, _e515.w);
            let _e530 = vec4<f32>(_e524.x, _e518.y, _e524.z, _e524.w);
            phi_2524_ = vec4<f32>(_e530.x, _e530.y, _e518.z, _e530.w);
        }
        let _e538 = phi_2524_;
        V1_ = _e538;
    } else {
        if (Bh && (_e442 == 0u)) {
            let _e542 = (_e440.x >> bitcast<u32>(16i));
            let _e544 = n.e6_;
            if (_e542 == 0u) {
                phi_2523_ = 0f;
            } else {
                phi_2523_ = unpack2x16float(((_e542 + 1023u) * _e544)).x;
            }
            let _e551 = phi_2523_;
            W1_[1u] = _e551;
        } else {
            let _e554 = local_2;
            let _e555 = (_e554 * 8u);
            let _e558 = QB.e2_[_e555];
            let _e569 = QB.e2_[(_e555 + 1u)];
            let _e578 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e440.y));
            let _e580 = ((mat2x2<f32>(vec2<f32>(_e558.x, _e558.y), vec2<f32>(_e558.z, _e558.w)) * _e433) + _e569.xy);
            if (_e569.z > 0.9f) {
                phi_2521_ = vec4<f32>(_e578.x, _e578.y, 2f, _e578.w);
            } else {
                phi_2521_ = vec4<f32>(_e578.x, _e578.y, _e569.w, _e578.w);
            }
            let _e595 = phi_2521_;
            if (f32(_e442) == 2f) {
                let _e602 = vec4<f32>(_e580.x, _e595.y, _e595.z, _e595.w);
                phi_2522_ = vec4<f32>(_e602.x, 0f, _e602.z, _e602.w);
            } else {
                let _e614 = vec4<f32>(_e595.x, _e595.y, -(_e595.z), _e595.w);
                let _e620 = vec4<f32>(_e580.x, _e614.y, _e614.z, _e614.w);
                phi_2522_ = vec4<f32>(_e620.x, _e580.y, _e620.z, _e620.w);
            }
            let _e628 = phi_2522_;
            V1_ = _e628;
            let _e630 = V1_[3u];
            V1_[3u] = -(_e630);
        }
    }
    phi_1155_ = Jh;
    if Jh {
        phi_1155_ = ((_e440.x & 2048u) != 0u);
    }
    let _e635 = phi_1155_;
    if _e635 {
        let _e637 = local_3;
        let _e638 = (_e637 * 8u);
        let _e642 = QB.e2_[(_e638 + 4u)];
        let _e653 = QB.e2_[(_e638 + 5u)];
        let _e656 = ((mat2x2<f32>(vec2<f32>(_e642.x, _e642.y), vec2<f32>(_e642.z, _e642.w)) * _e433) + _e653.xy);
        C2_ = vec3<f32>(_e656.x, _e656.y, (1f + _e653.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e435 {
        let _e663 = n.Ef;
        let _e665 = n.Ff;
        let _e673 = vec4<f32>(((_e433.x * _e663) - 1f), ((_e433.y * _e665) - sign(_e665)), 0f, 1f);
        phi_2540_ = vec4<f32>(_e673.x, _e673.y, (1f - (f32(_e431) * 0.000061035156f)), _e673.w);
    } else {
        let _e683 = n.T2_;
        phi_2540_ = vec4(_e683);
    }
    let _e686 = phi_2540_;
    unnamed.gl_Position = _e686;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e17 = unnamed.gl_Position;
    let _e18 = unnamed.gl_ClipDistance;
    let _e19 = W1_;
    let _e20 = g2_;
    let _e21 = V1_;
    let _e22 = C2_;
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
