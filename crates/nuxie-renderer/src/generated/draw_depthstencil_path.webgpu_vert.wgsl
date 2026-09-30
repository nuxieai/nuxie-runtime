enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct Fg {
    g2_: array<vec4<u32>>,
}

struct Eg {
    g2_: array<vec4<u32>>,
}

struct lf {
    g2_: array<vec2<u32>>,
}

struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

struct mf {
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

@id(0) override Eh: bool = true;
@id(2) override Gh: bool = true;
@id(1) override Fh: bool = true;
@id(8) override Mh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> HD: Fg;
@group(0) @binding(2)
var<storage> OB: Eg;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> TB_1: vec4<f32>;
var<private> UB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> CD: lf;
@group(0) @binding(0)
var<uniform> j: AC;
var<private> Y1_: vec2<f32>;
var<private> g1_: f32;
@group(0) @binding(4)
var<storage> PB: mf;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ga: sampler;

fn main_1() {
    var phi_2247_: f32;
    var phi_2219_: i32;
    var phi_1422_: bool;
    var phi_2232_: i32;
    var phi_2224_: vec4<u32>;
    var phi_2231_: i32;
    var phi_2223_: vec4<u32>;
    var phi_2230_: i32;
    var phi_2228_: vec4<u32>;
    var phi_2227_: u32;
    var phi_2234_: vec2<i32>;
    var phi_2235_: vec4<u32>;
    var phi_2239_: f32;
    var phi_2310_: f32;
    var phi_2253_: f32;
    var phi_2309_: f32;
    var phi_2257_: f32;
    var phi_2254_: f32;
    var phi_2251_: f32;
    var phi_2261_: f32;
    var phi_2307_: f32;
    var phi_2260_: f32;
    var phi_2316_: f32;
    var phi_2313_: f32;
    var phi_2370_: f32;
    var phi_2342_: i32;
    var phi_2352_: f32;
    var phi_1734_: bool;
    var phi_2359_: f32;
    var phi_2380_: vec2<f32>;
    var phi_2379_: vec2<f32>;
    var phi_2378_: vec2<f32>;
    var phi_2396_: vec2<f32>;
    var phi_2381_: vec2<f32>;
    var phi_2429_: u32;
    var phi_2400_: vec2<f32>;
    var phi_2399_: bool;
    var local: u32;
    var phi_2458_: u32;
    var phi_2459_: f32;
    var phi_2460_: f32;
    var local_1: u32;
    var phi_2500_: f32;
    var local_2: u32;
    var phi_2498_: vec4<f32>;
    var phi_2499_: vec4<f32>;
    var phi_1138_: bool;
    var local_3: u32;
    var phi_2515_: vec4<f32>;

    let _e80 = gl_InstanceIndex_1;
    let _e81 = TB_1;
    let _e82 = UB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e85 = i32(_e81.x);
            let _e88 = bitcast<i32>(_e81.w);
            let _e90 = (_e88 >> bitcast<u32>(2i));
            let _e91 = (_e88 & 3i);
            let _e93 = min(_e85, (_e90 - 1i));
            let _e95 = ((_e80 * _e90) + _e93);
            let _e100 = textureLoad(JC, vec2<i32>((_e95 & 2047i), (_e95 >> bitcast<u32>(11i))), 0i);
            let _e107 = HD.g2_[(max((_e100.w & 65535u), 1u) - 1u)];
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
            phi_2247_ = _e81.y;
            phi_2219_ = _e85;
            local = _e111;
            local_1 = _e111;
            local_2 = _e111;
            local_3 = _e111;
            if (_e135 != 0u) {
                phi_2247_ = _e82.y;
                phi_2219_ = i32(_e82.x);
            }
            let _e141 = phi_2247_;
            let _e143 = phi_2219_;
            phi_2230_ = _e95;
            phi_2228_ = _e100;
            phi_2227_ = _e100.w;
            if (_e143 != _e93) {
                let _e146 = ((_e95 + _e143) - _e93);
                let _e151 = textureLoad(JC, vec2<i32>((_e146 & 2047i), (_e146 >> bitcast<u32>(11i))), 0i);
                if ((_e151.w & 8454143u) != (_e100.w & 8454143u)) {
                    let _e156 = (_e132 == 0f);
                    phi_1422_ = _e156;
                    if !(_e156) {
                        phi_1422_ = (_e109.x != 0f);
                    }
                    let _e161 = phi_1422_;
                    phi_2232_ = _e95;
                    phi_2224_ = _e100;
                    if _e161 {
                        let _e162 = bitcast<i32>(_e107.w);
                        let _e167 = textureLoad(JC, vec2<i32>((_e162 & 2047i), (_e162 >> bitcast<u32>(11i))), 0i);
                        phi_2232_ = _e162;
                        phi_2224_ = _e167;
                    }
                    let _e169 = phi_2232_;
                    let _e171 = phi_2224_;
                    phi_2231_ = _e169;
                    phi_2223_ = _e171;
                } else {
                    phi_2231_ = _e146;
                    phi_2223_ = _e151;
                }
                let _e173 = phi_2231_;
                let _e175 = phi_2223_;
                phi_2230_ = _e173;
                phi_2228_ = _e175;
                phi_2227_ = ((_e175.w & 4286578687u) | _e135);
            }
            let _e180 = phi_2230_;
            let _e182 = phi_2228_;
            let _e184 = phi_2227_;
            let _e185 = (_e184 & 469762048u);
            if ((_e185 == 67108864u) && (_e91 == 0i)) {
                let _e191 = f32((_e182.z & 65535u));
                let _e194 = f32((_e182.z >> bitcast<u32>(16i)));
                let _e200 = vec2<i32>(i32((-1f - _e191)), i32(((_e194 - _e191) + 1f)));
                phi_2234_ = _e200;
                if ((_e184 & 8388608u) != 0u) {
                    phi_2234_ = -(_e200);
                }
                let _e205 = phi_2234_;
                let _e207 = (_e180 + _e205.x);
                let _e212 = textureLoad(JC, vec2<i32>((_e207 & 2047i), (_e207 >> bitcast<u32>(11i))), 0i);
                let _e214 = (_e180 + _e205.y);
                let _e219 = textureLoad(JC, vec2<i32>((_e214 & 2047i), (_e214 >> bitcast<u32>(11i))), 0i);
                phi_2235_ = _e219;
                if ((_e219.w & 8454143u) != (_e212.w & 8454143u)) {
                    let _e225 = bitcast<i32>(_e107.w);
                    let _e230 = textureLoad(JC, vec2<i32>((_e225 & 2047i), (_e225 >> bitcast<u32>(11i))), 0i);
                    phi_2235_ = _e230;
                }
                let _e232 = phi_2235_;
                let _e234 = bitcast<f32>(_e212.z);
                let _e236 = bitcast<f32>(_e232.z);
                let _e237 = (_e236 - _e234);
                phi_2239_ = _e237;
                if (abs(_e237) > 3.1415927f) {
                    phi_2239_ = (_e237 - (6.2831855f * sign(_e237)));
                }
                let _e244 = phi_2239_;
                let _e245 = (_e194 + -2f);
                let _e251 = clamp(round(((abs(_e244) * 0.31830987f) * _e245)), 1f, (_e194 + -3f));
                let _e252 = (_e245 - _e251);
                if (_e191 <= _e252) {
                    phi_2310_ = _e141;
                    if (_e191 == _e252) {
                        phi_2310_ = -(_e141);
                    }
                    let _e261 = phi_2310_;
                    phi_2309_ = _e261;
                    phi_2257_ = -(((3.1415927f * sign(_e244)) - _e244));
                    phi_2254_ = _e252;
                    phi_2251_ = _e191;
                } else {
                    let _e263 = (_e191 == (_e252 + 1f));
                    if _e263 {
                        phi_2253_ = 0f;
                    } else {
                        phi_2253_ = (_e191 - (_e252 + 2f));
                    }
                    let _e267 = phi_2253_;
                    phi_2309_ = select(_e141, 0f, _e263);
                    phi_2257_ = _e244;
                    phi_2254_ = select(_e251, 0f, _e263);
                    phi_2251_ = _e267;
                }
                let _e271 = phi_2309_;
                let _e273 = phi_2257_;
                let _e275 = phi_2254_;
                let _e277 = phi_2251_;
                if (_e277 == _e275) {
                    phi_2261_ = _e236;
                } else {
                    phi_2261_ = (_e234 + (_e273 * (_e277 / _e275)));
                }
                let _e283 = phi_2261_;
                phi_2307_ = _e271;
                phi_2260_ = _e283;
            } else {
                phi_2307_ = _e141;
                phi_2260_ = bitcast<f32>(_e182.z);
            }
            let _e287 = phi_2307_;
            let _e289 = phi_2260_;
            let _e293 = vec2<f32>(sin(_e289), -(cos(_e289)));
            let _e295 = bitcast<vec2<f32>>(_e182.xy);
            phi_2316_ = _e134;
            if (_e134 != 0f) {
                phi_2316_ = max(_e134, (1f / length((_e124 * _e293))));
            }
            let _e302 = phi_2316_;
            if (_e132 != 0f) {
                let _e306 = (_e287 * sign(determinant(_e124)));
                let _e308 = ((_e184 & 1048576u) != 0u);
                phi_2313_ = _e306;
                if _e308 {
                    phi_2313_ = min(_e306, 0f);
                }
                let _e311 = phi_2313_;
                phi_2370_ = _e311;
                if ((_e184 & 524288u) != 0u) {
                    phi_2370_ = max(_e311, 0f);
                }
                let _e316 = phi_2370_;
                let _e318 = select(0f, _e302, (_e302 != 0f));
                let _e322 = select(_e132, _e318, ((_e318 > _e132) && (_e302 == 0f)));
                let _e323 = (_e322 + _e318);
                let _e324 = (_e293 * _e323);
                phi_2378_ = _e324;
                if (_e185 > 134217728u) {
                    let _e326 = (_e184 & 4194304u);
                    let _e328 = select(2i, -2i, (_e326 == 0u));
                    phi_2342_ = _e328;
                    if ((_e184 & 8388608u) != 0u) {
                        phi_2342_ = -(_e328);
                    }
                    let _e333 = phi_2342_;
                    let _e334 = (_e180 + _e333);
                    let _e339 = textureLoad(JC, vec2<i32>((_e334 & 2047i), (_e334 >> bitcast<u32>(11i))), 0i);
                    let _e343 = abs((bitcast<f32>(_e339.z) - _e289));
                    phi_2352_ = _e343;
                    if (_e343 > 3.1415927f) {
                        phi_2352_ = (6.2831855f - _e343);
                    }
                    let _e347 = phi_2352_;
                    let _e352 = ((_e347 * select(0.5f, -0.5f, ((_e326 != 0u) == _e308))) + _e289);
                    let _e356 = vec2<f32>(sin(_e352), -(cos(_e352)));
                    let _e357 = (_e124 * _e356);
                    let _e367 = cos((_e347 * 0.5f));
                    let _e368 = (_e185 == 335544320u);
                    phi_1734_ = _e368;
                    if !(_e368) {
                        phi_1734_ = ((_e185 == 268435456u) && (_e367 >= 0.25f));
                    }
                    let _e374 = phi_1734_;
                    if _e374 {
                        phi_2359_ = (_e322 * (1f / max(_e367, select(0.25f, 1f, ((_e184 & 33554432u) != 0u)))));
                    } else {
                        phi_2359_ = ((_e322 * _e367) + (((abs(_e357.x) + abs(_e357.y)) * (1f / dot(_e357, _e357))) * 0.5f));
                    }
                    let _e385 = phi_2359_;
                    phi_2379_ = _e324;
                    if ((_e184 & 2097152u) != 0u) {
                        if (_e323 <= ((_e385 * _e367) + (_e318 * 0.125f))) {
                            phi_2380_ = (_e356 * (_e323 * (1f / _e367)));
                        } else {
                            let _e395 = (_e356 * _e385);
                            phi_2380_ = (vec2<f32>(dot(_e324, _e324), dot(_e395, _e395)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e324, _e395)));
                        }
                        let _e403 = phi_2380_;
                        phi_2379_ = _e403;
                    }
                    let _e405 = phi_2379_;
                    phi_2378_ = _e405;
                }
                let _e407 = phi_2378_;
                if (_e91 != 0i) {
                    phi_2429_ = u32();
                    phi_2400_ = vec2<f32>();
                    phi_2399_ = false;
                    break;
                }
                phi_2396_ = (_e124 * (_e407 * _e316));
                phi_2381_ = _e295;
            } else {
                if (((_e184 & 2147483648u) != 0u) && (_e91 != 1i)) {
                    phi_2429_ = u32();
                    phi_2400_ = vec2<f32>();
                    phi_2399_ = false;
                    break;
                }
                phi_2396_ = vec2<f32>(0f, 0f);
                phi_2381_ = select(_e295, _e109, vec2((_e91 == 2i)));
            }
            let _e419 = phi_2396_;
            let _e421 = phi_2381_;
            let _e428 = OB.g2_[(_e113 + 2u)];
            phi_2429_ = _e428.x;
            phi_2400_ = (((_e124 * _e421) + _e419) + bitcast<vec2<f32>>(_e128.xy));
            phi_2399_ = true;
            break;
        }
    }
    let _e431 = phi_2429_;
    let _e433 = phi_2400_;
    let _e435 = phi_2399_;
    let _e438 = local;
    let _e440 = CD.g2_[_e438];
    let _e442 = (_e440.x & 15u);
    if Eh {
        let _e443 = (_e442 == 0u);
        if _e443 {
            phi_2458_ = _e440.y;
        } else {
            phi_2458_ = _e440.x;
        }
        let _e446 = phi_2458_;
        let _e448 = (_e446 >> bitcast<u32>(16i));
        let _e450 = j.f6_;
        if (_e448 == 0u) {
            phi_2459_ = 0f;
        } else {
            phi_2459_ = unpack2x16float(((_e448 + 1023u) * _e450)).x;
        }
        let _e457 = phi_2459_;
        phi_2460_ = _e457;
        if _e443 {
            phi_2460_ = -(_e457);
        }
        let _e460 = phi_2460_;
        Y1_[0u] = _e460;
    }
    if Gh {
        g1_ = f32(((_e440.x >> bitcast<u32>(4i)) & 15u));
    }
    if Fh {
        let _e467 = local_1;
        let _e468 = (_e467 * 8u);
        let _e472 = PB.g2_[(_e468 + 2u)];
        let _e483 = PB.g2_[(_e468 + 3u)];
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
        X1_ = unpack4x8unorm(_e440.y);
    } else {
        if (Eh && (_e442 == 0u)) {
            let _e519 = (_e440.x >> bitcast<u32>(16i));
            let _e521 = j.f6_;
            if (_e519 == 0u) {
                phi_2500_ = 0f;
            } else {
                phi_2500_ = unpack2x16float(((_e519 + 1023u) * _e521)).x;
            }
            let _e528 = phi_2500_;
            Y1_[1u] = _e528;
        } else {
            let _e531 = local_2;
            let _e532 = (_e531 * 8u);
            let _e535 = PB.g2_[_e532];
            let _e546 = PB.g2_[(_e532 + 1u)];
            let _e555 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e440.y));
            let _e557 = ((mat2x2<f32>(vec2<f32>(_e535.x, _e535.y), vec2<f32>(_e535.z, _e535.w)) * _e433) + _e546.xy);
            if (_e546.z > 0.9f) {
                phi_2498_ = vec4<f32>(_e555.x, _e555.y, 2f, _e555.w);
            } else {
                phi_2498_ = vec4<f32>(_e555.x, _e555.y, _e546.w, _e555.w);
            }
            let _e572 = phi_2498_;
            if (f32(_e442) == 2f) {
                let _e579 = vec4<f32>(_e557.x, _e572.y, _e572.z, _e572.w);
                phi_2499_ = vec4<f32>(_e579.x, 0f, _e579.z, _e579.w);
            } else {
                let _e591 = vec4<f32>(_e572.x, _e572.y, -(_e572.z), _e572.w);
                let _e597 = vec4<f32>(_e557.x, _e591.y, _e591.z, _e591.w);
                phi_2499_ = vec4<f32>(_e597.x, _e557.y, _e597.z, _e597.w);
            }
            let _e605 = phi_2499_;
            X1_ = _e605;
            let _e607 = X1_[3u];
            X1_[3u] = -(_e607);
        }
    }
    phi_1138_ = Mh;
    if Mh {
        phi_1138_ = ((_e440.x & 2048u) != 0u);
    }
    let _e612 = phi_1138_;
    if _e612 {
        let _e614 = local_3;
        let _e615 = (_e614 * 8u);
        let _e619 = PB.g2_[(_e615 + 4u)];
        let _e630 = PB.g2_[(_e615 + 5u)];
        let _e633 = ((mat2x2<f32>(vec2<f32>(_e619.x, _e619.y), vec2<f32>(_e619.z, _e619.w)) * _e433) + _e630.xy);
        C2_ = vec3<f32>(_e633.x, _e633.y, (1f + _e630.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e435 {
        let _e640 = j.Hf;
        let _e642 = j.If;
        let _e650 = vec4<f32>(((_e433.x * _e640) - 1f), ((_e433.y * _e642) - sign(_e642)), 0f, 1f);
        phi_2515_ = vec4<f32>(_e650.x, _e650.y, (1f - (f32(_e431) * 0.000061035156f)), _e650.w);
    } else {
        let _e660 = j.U2_;
        phi_2515_ = vec4(_e660);
    }
    let _e663 = phi_2515_;
    unnamed.gl_Position = _e663;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) TB: vec4<f32>, @location(1) UB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    TB_1 = TB;
    UB_1 = UB;
    main_1();
    let _e17 = unnamed.gl_Position;
    let _e18 = unnamed.gl_ClipDistance;
    let _e19 = Y1_;
    let _e20 = g1_;
    let _e21 = X1_;
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
