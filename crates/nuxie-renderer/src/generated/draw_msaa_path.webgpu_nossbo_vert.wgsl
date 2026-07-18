enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
}

@id(0) override Ug: bool = true;
@id(2) override Wg: bool = true;
@id(1) override Vg: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(5)
var ED: texture_2d<u32>;
@group(0) @binding(2)
var PB: texture_2d<u32>;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
@group(0) @binding(3)
var AD: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> U1_: vec2<f32>;
var<private> e2_: f32;
@group(0) @binding(4)
var RB: texture_2d<f32>;
var<private> f1_: vec4<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_2229_: f32;
    var phi_2201_: i32;
    var phi_1461_: bool;
    var phi_2214_: i32;
    var phi_2206_: vec4<u32>;
    var phi_2213_: i32;
    var phi_2205_: vec4<u32>;
    var phi_2212_: i32;
    var phi_2210_: vec4<u32>;
    var phi_2209_: u32;
    var phi_2216_: vec2<i32>;
    var phi_2217_: vec4<u32>;
    var phi_2221_: f32;
    var phi_2292_: f32;
    var phi_2235_: f32;
    var phi_2291_: f32;
    var phi_2239_: f32;
    var phi_2236_: f32;
    var phi_2233_: f32;
    var phi_2243_: f32;
    var phi_2289_: f32;
    var phi_2242_: f32;
    var phi_2298_: f32;
    var phi_2295_: f32;
    var phi_2352_: f32;
    var phi_2324_: i32;
    var phi_2334_: f32;
    var phi_1773_: bool;
    var phi_2341_: f32;
    var phi_2362_: vec2<f32>;
    var phi_2361_: vec2<f32>;
    var phi_2360_: vec2<f32>;
    var phi_2378_: vec2<f32>;
    var phi_2363_: vec2<f32>;
    var phi_2411_: u32;
    var phi_2382_: vec2<f32>;
    var phi_2381_: bool;
    var local: u32;
    var local_1: u32;
    var phi_2440_: u32;
    var phi_2441_: f32;
    var phi_2442_: f32;
    var local_2: u32;
    var local_3: u32;
    var phi_2444_: vec4<f32>;
    var phi_2443_: f32;
    var local_4: vec2<i32>;
    var local_5: vec2<i32>;
    var phi_2459_: vec4<f32>;

    let _e75 = gl_InstanceIndex_1;
    let _e76 = UB_1;
    let _e77 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e80 = i32(_e76.x);
            let _e83 = bitcast<i32>(_e76.w);
            let _e85 = (_e83 >> bitcast<u32>(2i));
            let _e86 = (_e83 & 3i);
            let _e88 = min(_e80, (_e85 - 1i));
            let _e90 = ((_e75 * _e85) + _e88);
            let _e95 = textureLoad(LC, vec2<i32>((_e90 & 2047i), (_e90 >> bitcast<u32>(11i))), 0i);
            let _e99 = (max((_e95.w & 65535u), 1u) - 1u);
            let _e106 = textureLoad(ED, vec2<i32>(bitcast<i32>((_e99 & 127u)), bitcast<i32>((_e99 >> bitcast<u32>(7i)))), 0i);
            let _e108 = bitcast<vec2<f32>>(_e106.xy);
            let _e110 = (_e106.z & 65535u);
            let _e112 = (_e110 * 4u);
            let _e118 = vec2<i32>(bitcast<i32>((_e112 & 127u)), bitcast<i32>((_e112 >> bitcast<u32>(7i))));
            let _e119 = textureLoad(PB, _e118, 0i);
            let _e120 = bitcast<vec4<f32>>(_e119);
            let _e127 = mat2x2<f32>(vec2<f32>(_e120.x, _e120.y), vec2<f32>(_e120.z, _e120.w));
            let _e128 = (_e112 + 1u);
            let _e134 = vec2<i32>(bitcast<i32>((_e128 & 127u)), bitcast<i32>((_e128 >> bitcast<u32>(7i))));
            let _e135 = textureLoad(PB, _e134, 0i);
            let _e139 = bitcast<f32>(_e135.z);
            let _e141 = bitcast<f32>(_e135.w);
            let _e142 = (_e95.w & 8388608u);
            phi_2229_ = _e76.y;
            phi_2201_ = _e80;
            local = _e106.z;
            local_1 = _e110;
            local_2 = _e112;
            local_3 = _e112;
            local_4 = _e118;
            local_5 = _e134;
            if (_e142 != 0u) {
                phi_2229_ = _e77.y;
                phi_2201_ = i32(_e77.x);
            }
            let _e148 = phi_2229_;
            let _e150 = phi_2201_;
            phi_2212_ = _e90;
            phi_2210_ = _e95;
            phi_2209_ = _e95.w;
            if (_e150 != _e88) {
                let _e153 = ((_e90 + _e150) - _e88);
                let _e158 = textureLoad(LC, vec2<i32>((_e153 & 2047i), (_e153 >> bitcast<u32>(11i))), 0i);
                if ((_e158.w & 8454143u) != (_e95.w & 8454143u)) {
                    let _e163 = (_e139 == 0f);
                    phi_1461_ = _e163;
                    if !(_e163) {
                        phi_1461_ = (_e108.x != 0f);
                    }
                    let _e168 = phi_1461_;
                    phi_2214_ = _e90;
                    phi_2206_ = _e95;
                    if _e168 {
                        let _e169 = bitcast<i32>(_e106.w);
                        let _e174 = textureLoad(LC, vec2<i32>((_e169 & 2047i), (_e169 >> bitcast<u32>(11i))), 0i);
                        phi_2214_ = _e169;
                        phi_2206_ = _e174;
                    }
                    let _e176 = phi_2214_;
                    let _e178 = phi_2206_;
                    phi_2213_ = _e176;
                    phi_2205_ = _e178;
                } else {
                    phi_2213_ = _e153;
                    phi_2205_ = _e158;
                }
                let _e180 = phi_2213_;
                let _e182 = phi_2205_;
                phi_2212_ = _e180;
                phi_2210_ = _e182;
                phi_2209_ = ((_e182.w & 4286578687u) | _e142);
            }
            let _e187 = phi_2212_;
            let _e189 = phi_2210_;
            let _e191 = phi_2209_;
            let _e192 = (_e191 & 469762048u);
            if ((_e192 == 67108864u) && (_e86 == 0i)) {
                let _e198 = f32((_e189.z & 65535u));
                let _e201 = f32((_e189.z >> bitcast<u32>(16i)));
                let _e207 = vec2<i32>(i32((-1f - _e198)), i32(((_e201 - _e198) + 1f)));
                phi_2216_ = _e207;
                if ((_e191 & 8388608u) != 0u) {
                    phi_2216_ = -(_e207);
                }
                let _e212 = phi_2216_;
                let _e214 = (_e187 + _e212.x);
                let _e219 = textureLoad(LC, vec2<i32>((_e214 & 2047i), (_e214 >> bitcast<u32>(11i))), 0i);
                let _e221 = (_e187 + _e212.y);
                let _e226 = textureLoad(LC, vec2<i32>((_e221 & 2047i), (_e221 >> bitcast<u32>(11i))), 0i);
                phi_2217_ = _e226;
                if ((_e226.w & 8454143u) != (_e219.w & 8454143u)) {
                    let _e232 = bitcast<i32>(_e106.w);
                    let _e237 = textureLoad(LC, vec2<i32>((_e232 & 2047i), (_e232 >> bitcast<u32>(11i))), 0i);
                    phi_2217_ = _e237;
                }
                let _e239 = phi_2217_;
                let _e241 = bitcast<f32>(_e219.z);
                let _e243 = bitcast<f32>(_e239.z);
                let _e244 = (_e243 - _e241);
                phi_2221_ = _e244;
                if (abs(_e244) > 3.1415927f) {
                    phi_2221_ = (_e244 - (6.2831855f * sign(_e244)));
                }
                let _e251 = phi_2221_;
                let _e252 = (_e201 + -2f);
                let _e258 = clamp(round(((abs(_e251) * 0.31830987f) * _e252)), 1f, (_e201 + -3f));
                let _e259 = (_e252 - _e258);
                if (_e198 <= _e259) {
                    phi_2292_ = _e148;
                    if (_e198 == _e259) {
                        phi_2292_ = -(_e148);
                    }
                    let _e268 = phi_2292_;
                    phi_2291_ = _e268;
                    phi_2239_ = -(((3.1415927f * sign(_e251)) - _e251));
                    phi_2236_ = _e259;
                    phi_2233_ = _e198;
                } else {
                    let _e270 = (_e198 == (_e259 + 1f));
                    if _e270 {
                        phi_2235_ = 0f;
                    } else {
                        phi_2235_ = (_e198 - (_e259 + 2f));
                    }
                    let _e274 = phi_2235_;
                    phi_2291_ = select(_e148, 0f, _e270);
                    phi_2239_ = _e251;
                    phi_2236_ = select(_e258, 0f, _e270);
                    phi_2233_ = _e274;
                }
                let _e278 = phi_2291_;
                let _e280 = phi_2239_;
                let _e282 = phi_2236_;
                let _e284 = phi_2233_;
                if (_e284 == _e282) {
                    phi_2243_ = _e243;
                } else {
                    phi_2243_ = (_e241 + (_e280 * (_e284 / _e282)));
                }
                let _e290 = phi_2243_;
                phi_2289_ = _e278;
                phi_2242_ = _e290;
            } else {
                phi_2289_ = _e148;
                phi_2242_ = bitcast<f32>(_e189.z);
            }
            let _e294 = phi_2289_;
            let _e296 = phi_2242_;
            let _e300 = vec2<f32>(sin(_e296), -(cos(_e296)));
            let _e302 = bitcast<vec2<f32>>(_e189.xy);
            phi_2298_ = _e141;
            if (_e141 != 0f) {
                phi_2298_ = max(_e141, (1f / length((_e127 * _e300))));
            }
            let _e309 = phi_2298_;
            if (_e139 != 0f) {
                let _e313 = (_e294 * sign(determinant(_e127)));
                let _e315 = ((_e191 & 1048576u) != 0u);
                phi_2295_ = _e313;
                if _e315 {
                    phi_2295_ = min(_e313, 0f);
                }
                let _e318 = phi_2295_;
                phi_2352_ = _e318;
                if ((_e191 & 524288u) != 0u) {
                    phi_2352_ = max(_e318, 0f);
                }
                let _e323 = phi_2352_;
                let _e325 = select(0f, _e309, (_e309 != 0f));
                let _e329 = select(_e139, _e325, ((_e325 > _e139) && (_e309 == 0f)));
                let _e330 = (_e329 + _e325);
                let _e331 = (_e300 * _e330);
                phi_2360_ = _e331;
                if (_e192 > 134217728u) {
                    let _e333 = (_e191 & 4194304u);
                    let _e335 = select(2i, -2i, (_e333 == 0u));
                    phi_2324_ = _e335;
                    if ((_e191 & 8388608u) != 0u) {
                        phi_2324_ = -(_e335);
                    }
                    let _e340 = phi_2324_;
                    let _e341 = (_e187 + _e340);
                    let _e346 = textureLoad(LC, vec2<i32>((_e341 & 2047i), (_e341 >> bitcast<u32>(11i))), 0i);
                    let _e350 = abs((bitcast<f32>(_e346.z) - _e296));
                    phi_2334_ = _e350;
                    if (_e350 > 3.1415927f) {
                        phi_2334_ = (6.2831855f - _e350);
                    }
                    let _e354 = phi_2334_;
                    let _e359 = ((_e354 * select(0.5f, -0.5f, ((_e333 != 0u) == _e315))) + _e296);
                    let _e363 = vec2<f32>(sin(_e359), -(cos(_e359)));
                    let _e364 = (_e127 * _e363);
                    let _e374 = cos((_e354 * 0.5f));
                    let _e375 = (_e192 == 335544320u);
                    phi_1773_ = _e375;
                    if !(_e375) {
                        phi_1773_ = ((_e192 == 268435456u) && (_e374 >= 0.25f));
                    }
                    let _e381 = phi_1773_;
                    if _e381 {
                        phi_2341_ = (_e329 * (1f / max(_e374, select(0.25f, 1f, ((_e191 & 33554432u) != 0u)))));
                    } else {
                        phi_2341_ = ((_e329 * _e374) + (((abs(_e364.x) + abs(_e364.y)) * (1f / dot(_e364, _e364))) * 0.5f));
                    }
                    let _e392 = phi_2341_;
                    phi_2361_ = _e331;
                    if ((_e191 & 2097152u) != 0u) {
                        if (_e330 <= ((_e392 * _e374) + (_e325 * 0.125f))) {
                            phi_2362_ = (_e363 * (_e330 * (1f / _e374)));
                        } else {
                            let _e402 = (_e363 * _e392);
                            phi_2362_ = (vec2<f32>(dot(_e331, _e331), dot(_e402, _e402)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e331, _e402)));
                        }
                        let _e410 = phi_2362_;
                        phi_2361_ = _e410;
                    }
                    let _e412 = phi_2361_;
                    phi_2360_ = _e412;
                }
                let _e414 = phi_2360_;
                if (_e86 != 0i) {
                    phi_2411_ = u32();
                    phi_2382_ = vec2<f32>();
                    phi_2381_ = false;
                    break;
                }
                phi_2378_ = (_e127 * (_e414 * _e323));
                phi_2363_ = _e302;
            } else {
                if (((_e191 & 2147483648u) != 0u) && (_e86 != 1i)) {
                    phi_2411_ = u32();
                    phi_2382_ = vec2<f32>();
                    phi_2381_ = false;
                    break;
                }
                phi_2378_ = vec2<f32>(0f, 0f);
                phi_2363_ = select(_e302, _e108, vec2((_e86 == 2i)));
            }
            let _e426 = phi_2378_;
            let _e428 = phi_2363_;
            let _e432 = (_e112 + 2u);
            let _e439 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e432 & 127u)), bitcast<i32>((_e432 >> bitcast<u32>(7i)))), 0i);
            phi_2411_ = _e439.x;
            phi_2382_ = (((_e127 * _e428) + _e426) + bitcast<vec2<f32>>(_e135.xy));
            phi_2381_ = true;
            break;
        }
    }
    let _e442 = phi_2411_;
    let _e444 = phi_2382_;
    let _e446 = phi_2381_;
    let _e448 = local;
    let _e452 = local_1;
    let _e457 = textureLoad(AD, vec2<i32>(bitcast<i32>((_e448 & 127u)), bitcast<i32>((_e452 >> bitcast<u32>(7i)))), 0i);
    let _e459 = (_e457.x & 15u);
    if Ug {
        let _e460 = (_e459 == 0u);
        if _e460 {
            phi_2440_ = _e457.y;
        } else {
            phi_2440_ = _e457.x;
        }
        let _e463 = phi_2440_;
        let _e465 = (_e463 >> bitcast<u32>(16i));
        let _e467 = m.Z5_;
        if (_e465 == 0u) {
            phi_2441_ = 0f;
        } else {
            phi_2441_ = unpack2x16float(((_e465 + 1023u) * _e467)).x;
        }
        let _e474 = phi_2441_;
        phi_2442_ = _e474;
        if _e460 {
            phi_2442_ = -(_e474);
        }
        let _e477 = phi_2442_;
        U1_[0u] = _e477;
    }
    if Wg {
        e2_ = f32(((_e457.x >> bitcast<u32>(4i)) & 15u));
    }
    if Vg {
        let _e484 = local_2;
        let _e485 = (_e484 + 2u);
        let _e492 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e485 & 127u)), bitcast<i32>((_e485 >> bitcast<u32>(7i)))), 0i);
        let _e501 = local_3;
        let _e502 = (_e501 + 3u);
        let _e509 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e502 & 127u)), bitcast<i32>((_e502 >> bitcast<u32>(7i)))), 0i);
        if any((_e492 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e514 = ((mat2x2<f32>(vec2<f32>(_e492.x, _e492.y), vec2<f32>(_e492.z, _e492.w)) * _e444) + _e509.xy);
            unnamed.gl_ClipDistance[0i] = (_e514.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e514.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e514.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e514.y);
        } else {
            let _e530 = (_e509.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e530;
            unnamed.gl_ClipDistance[2i] = _e530;
            unnamed.gl_ClipDistance[1i] = _e530;
            unnamed.gl_ClipDistance[0i] = _e530;
        }
    }
    if (_e459 == 1u) {
        let _e541 = unpack4x8unorm(_e457.y);
        if Wg {
            phi_2444_ = _e541;
        } else {
            let _e544 = (_e541.xyz * _e541.w);
            let _e550 = vec4<f32>(_e544.x, _e541.y, _e541.z, _e541.w);
            let _e556 = vec4<f32>(_e550.x, _e544.y, _e550.z, _e550.w);
            phi_2444_ = vec4<f32>(_e556.x, _e556.y, _e544.z, _e556.w);
        }
        let _e564 = phi_2444_;
        f1_ = _e564;
    } else {
        if (Ug && (_e459 == 0u)) {
            let _e568 = (_e457.x >> bitcast<u32>(16i));
            let _e570 = m.Z5_;
            if (_e568 == 0u) {
                phi_2443_ = 0f;
            } else {
                phi_2443_ = unpack2x16float(((_e568 + 1023u) * _e570)).x;
            }
            let _e577 = phi_2443_;
            U1_[1u] = _e577;
        } else {
            let _e580 = local_4;
            let _e581 = textureLoad(RB, _e580, 0i);
            let _e590 = local_5;
            let _e591 = textureLoad(RB, _e590, 0i);
            let _e594 = ((mat2x2<f32>(vec2<f32>(_e581.x, _e581.y), vec2<f32>(_e581.z, _e581.w)) * _e444) + _e591.xy);
            let _e595 = (_e459 == 2u);
            if (_e595 || (_e459 == 3u)) {
                f1_[3u] = -(bitcast<f32>(_e457.y));
                if (_e591.z > 0.9f) {
                    f1_[2u] = 2f;
                } else {
                    f1_[2u] = _e591.w;
                }
                if _e595 {
                    f1_[1u] = 0f;
                    f1_[0u] = _e594.x;
                } else {
                    let _e611 = f1_[2u];
                    f1_[2u] = -(_e611);
                    f1_[0u] = _e594.x;
                    f1_[1u] = _e594.y;
                }
            } else {
                f1_ = vec4<f32>(_e594.x, _e594.y, bitcast<f32>(_e457.y), (-2f - _e591.z));
            }
        }
    }
    if _e446 {
        let _e625 = m.bf;
        let _e627 = m.cf;
        let _e635 = vec4<f32>(((_e444.x * _e625) - 1f), ((_e444.y * _e627) - sign(_e627)), 0f, 1f);
        phi_2459_ = vec4<f32>(_e635.x, _e635.y, (1f - (f32(_e442) * 0.000061035156f)), _e635.w);
    } else {
        let _e645 = m.N2_;
        phi_2459_ = vec4(_e645);
    }
    let _e648 = phi_2459_;
    unnamed.gl_Position = _e648;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e16 = unnamed.gl_Position;
    let _e17 = unnamed.gl_ClipDistance;
    let _e18 = U1_;
    let _e19 = e2_;
    let _e20 = f1_;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20);
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
