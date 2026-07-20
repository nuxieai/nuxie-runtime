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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(5)
var ED: texture_2d<u32>;
@group(0) @binding(2)
var PB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
var<private> O: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(3)
var AD: texture_2d<u32>;
@group(0) @binding(4)
var RB: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_2331_: f32;
    var phi_2269_: f32;
    var phi_2241_: i32;
    var phi_1383_: bool;
    var phi_2254_: i32;
    var phi_2246_: vec4<u32>;
    var phi_2253_: i32;
    var phi_2245_: vec4<u32>;
    var phi_2252_: i32;
    var phi_2250_: vec4<u32>;
    var phi_2249_: u32;
    var phi_2256_: vec2<i32>;
    var phi_2257_: vec4<u32>;
    var phi_2261_: f32;
    var phi_2341_: f32;
    var phi_2275_: f32;
    var phi_2340_: f32;
    var phi_2283_: f32;
    var phi_2276_: f32;
    var phi_2273_: f32;
    var phi_2287_: f32;
    var phi_2362_: f32;
    var phi_2353_: f32;
    var phi_2338_: f32;
    var phi_2286_: f32;
    var phi_2336_: f32;
    var phi_2419_: f32;
    var phi_2430_: f32;
    var phi_2422_: f32;
    var phi_2511_: f32;
    var phi_2468_: i32;
    var phi_2477_: f32;
    var phi_1722_: bool;
    var phi_2484_: f32;
    var phi_2500_: vec2<f32>;
    var phi_2499_: vec2<f32>;
    var phi_2521_: vec4<f32>;
    var phi_2536_: vec2<f32>;
    var phi_2520_: vec4<f32>;
    var phi_2567_: vec4<f32>;
    var phi_2372_: f32;
    var phi_2371_: f32;
    var phi_2373_: f32;
    var phi_2377_: f32;
    var phi_2399_: f32;
    var phi_2397_: f32;
    var phi_2415_: vec4<f32>;
    var phi_2565_: vec2<f32>;
    var phi_2414_: vec4<f32>;
    var phi_2569_: vec4<f32>;
    var phi_2566_: vec4<f32>;
    var phi_2562_: vec2<f32>;
    var phi_2538_: vec2<f32>;
    var phi_2609_: vec4<f32>;
    var phi_2571_: vec2<f32>;
    var phi_2570_: bool;
    var local: u32;
    var phi_2610_: vec4<f32>;

    let _e73 = gl_InstanceIndex_1;
    let _e74 = UB_1;
    let _e75 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e78 = i32(_e74.x);
            let _e82 = bitcast<i32>(_e74.w);
            let _e84 = (_e82 >> bitcast<u32>(2i));
            let _e85 = (_e82 & 3i);
            let _e87 = min(_e78, (_e84 - 1i));
            let _e89 = ((_e73 * _e84) + _e87);
            let _e94 = textureLoad(LC, vec2<i32>((_e89 & 2047i), (_e89 >> bitcast<u32>(11i))), 0i);
            let _e98 = (max((_e94.w & 65535u), 1u) - 1u);
            let _e105 = textureLoad(ED, vec2<i32>(bitcast<i32>((_e98 & 127u)), bitcast<i32>((_e98 >> bitcast<u32>(7i)))), 0i);
            let _e107 = bitcast<vec2<f32>>(_e105.xy);
            let _e111 = ((_e105.z & 65535u) * 4u);
            let _e118 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e111 & 127u)), bitcast<i32>((_e111 >> bitcast<u32>(7i)))), 0i);
            let _e119 = bitcast<vec4<f32>>(_e118);
            let _e126 = mat2x2<f32>(vec2<f32>(_e119.x, _e119.y), vec2<f32>(_e119.z, _e119.w));
            let _e127 = (_e111 + 1u);
            let _e134 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e127 & 127u)), bitcast<i32>((_e127 >> bitcast<u32>(7i)))), 0i);
            let _e138 = bitcast<f32>(_e134.z);
            let _e140 = bitcast<f32>(_e134.w);
            let _e141 = (_e94.w & 8388608u);
            phi_2331_ = _e74.z;
            phi_2269_ = _e74.y;
            phi_2241_ = _e78;
            local = _e111;
            if (_e141 != 0u) {
                phi_2331_ = _e75.z;
                phi_2269_ = _e75.y;
                phi_2241_ = i32(_e75.x);
            }
            let _e148 = phi_2331_;
            let _e150 = phi_2269_;
            let _e152 = phi_2241_;
            phi_2252_ = _e89;
            phi_2250_ = _e94;
            phi_2249_ = _e94.w;
            if (_e152 != _e87) {
                let _e155 = ((_e89 + _e152) - _e87);
                let _e160 = textureLoad(LC, vec2<i32>((_e155 & 2047i), (_e155 >> bitcast<u32>(11i))), 0i);
                if ((_e160.w & 8454143u) != (_e94.w & 8454143u)) {
                    let _e165 = (_e138 == 0f);
                    phi_1383_ = _e165;
                    if !(_e165) {
                        phi_1383_ = (_e107.x != 0f);
                    }
                    let _e170 = phi_1383_;
                    phi_2254_ = _e89;
                    phi_2246_ = _e94;
                    if _e170 {
                        let _e171 = bitcast<i32>(_e105.w);
                        let _e176 = textureLoad(LC, vec2<i32>((_e171 & 2047i), (_e171 >> bitcast<u32>(11i))), 0i);
                        phi_2254_ = _e171;
                        phi_2246_ = _e176;
                    }
                    let _e178 = phi_2254_;
                    let _e180 = phi_2246_;
                    phi_2253_ = _e178;
                    phi_2245_ = _e180;
                } else {
                    phi_2253_ = _e155;
                    phi_2245_ = _e160;
                }
                let _e182 = phi_2253_;
                let _e184 = phi_2245_;
                phi_2252_ = _e182;
                phi_2250_ = _e184;
                phi_2249_ = ((_e184.w & 4286578687u) | _e141);
            }
            let _e189 = phi_2252_;
            let _e191 = phi_2250_;
            let _e193 = phi_2249_;
            let _e194 = (_e193 & 469762048u);
            let _e197 = ((_e194 == 67108864u) && (_e85 == 0i));
            if _e197 {
                let _e200 = f32((_e191.z & 65535u));
                let _e203 = f32((_e191.z >> bitcast<u32>(16i)));
                let _e209 = vec2<i32>(i32((-1f - _e200)), i32(((_e203 - _e200) + 1f)));
                phi_2256_ = _e209;
                if ((_e193 & 8388608u) != 0u) {
                    phi_2256_ = -(_e209);
                }
                let _e214 = phi_2256_;
                let _e216 = (_e189 + _e214.x);
                let _e221 = textureLoad(LC, vec2<i32>((_e216 & 2047i), (_e216 >> bitcast<u32>(11i))), 0i);
                let _e223 = (_e189 + _e214.y);
                let _e228 = textureLoad(LC, vec2<i32>((_e223 & 2047i), (_e223 >> bitcast<u32>(11i))), 0i);
                phi_2257_ = _e228;
                if ((_e228.w & 8454143u) != (_e221.w & 8454143u)) {
                    let _e234 = bitcast<i32>(_e105.w);
                    let _e239 = textureLoad(LC, vec2<i32>((_e234 & 2047i), (_e234 >> bitcast<u32>(11i))), 0i);
                    phi_2257_ = _e239;
                }
                let _e241 = phi_2257_;
                let _e243 = bitcast<f32>(_e221.z);
                let _e245 = bitcast<f32>(_e241.z);
                let _e246 = (_e245 - _e243);
                phi_2261_ = _e246;
                if (abs(_e246) > 3.1415927f) {
                    phi_2261_ = (_e246 - (6.2831855f * sign(_e246)));
                }
                let _e253 = phi_2261_;
                let _e254 = (_e203 + -2f);
                let _e260 = clamp(round(((abs(_e253) * 0.31830987f) * _e254)), 1f, (_e203 + -3f));
                let _e261 = (_e254 - _e260);
                if (_e200 <= _e261) {
                    phi_2341_ = _e150;
                    if (_e200 == _e261) {
                        phi_2341_ = -(_e150);
                    }
                    let _e270 = phi_2341_;
                    phi_2340_ = _e270;
                    phi_2283_ = -(((3.1415927f * sign(_e253)) - _e253));
                    phi_2276_ = _e261;
                    phi_2273_ = _e200;
                } else {
                    let _e272 = (_e200 == (_e261 + 1f));
                    if _e272 {
                        phi_2275_ = 0f;
                    } else {
                        phi_2275_ = (_e200 - (_e261 + 2f));
                    }
                    let _e276 = phi_2275_;
                    phi_2340_ = select(_e150, 0f, _e272);
                    phi_2283_ = _e253;
                    phi_2276_ = select(_e260, 0f, _e272);
                    phi_2273_ = _e276;
                }
                let _e280 = phi_2340_;
                let _e282 = phi_2283_;
                let _e284 = phi_2276_;
                let _e286 = phi_2273_;
                if (_e286 == _e284) {
                    phi_2287_ = _e245;
                } else {
                    phi_2287_ = (_e243 + (_e282 * (_e286 / _e284)));
                }
                let _e292 = phi_2287_;
                phi_2362_ = _e243;
                phi_2353_ = _e282;
                phi_2338_ = _e280;
                phi_2286_ = _e292;
            } else {
                phi_2362_ = f32();
                phi_2353_ = f32();
                phi_2338_ = _e150;
                phi_2286_ = bitcast<f32>(_e191.z);
            }
            let _e296 = phi_2362_;
            let _e298 = phi_2353_;
            let _e300 = phi_2338_;
            let _e302 = phi_2286_;
            let _e306 = vec2<f32>(sin(_e302), -(cos(_e302)));
            let _e308 = bitcast<vec2<f32>>(_e191.xy);
            phi_2336_ = _e140;
            if (_e140 != 0f) {
                phi_2336_ = max(_e140, (1f / length((_e126 * _e306))));
            }
            let _e315 = phi_2336_;
            if (_e138 != 0f) {
                let _e319 = (_e300 * sign(determinant(_e126)));
                let _e321 = ((_e193 & 1048576u) != 0u);
                phi_2419_ = _e319;
                if _e321 {
                    phi_2419_ = min(_e319, 0f);
                }
                let _e324 = phi_2419_;
                phi_2430_ = _e324;
                if ((_e193 & 524288u) != 0u) {
                    phi_2430_ = max(_e324, 0f);
                }
                let _e329 = phi_2430_;
                let _e330 = (_e315 != 0f);
                if _e330 {
                    phi_2422_ = _e315;
                } else {
                    let _e331 = (_e126 * _e306);
                    phi_2422_ = (((abs(_e331.x) + abs(_e331.y)) * (1f / dot(_e331, _e331))) * 0.5f);
                }
                let _e342 = phi_2422_;
                let _e345 = ((_e342 > _e138) && (_e315 == 0f));
                phi_2511_ = 1f;
                if _e345 {
                    phi_2511_ = (_e138 / _e342);
                }
                let _e348 = phi_2511_;
                let _e349 = select(_e138, _e342, _e345);
                let _e350 = (_e349 + _e342);
                let _e351 = (_e306 * _e350);
                let _e352 = (_e329 * _e350);
                let _e359 = (((vec2<f32>(_e352, -(_e352)) + vec2(_e349)) * (0.5f / _e342)) + vec2<f32>(0.5f, 0.5f));
                let _e362 = vec4<f32>(_e359.x, _e359.y, 0f, 0f);
                phi_2536_ = _e351;
                phi_2520_ = _e362;
                if (_e194 > 134217728u) {
                    let _e364 = (_e193 & 4194304u);
                    let _e366 = select(2i, -2i, (_e364 == 0u));
                    phi_2468_ = _e366;
                    if ((_e193 & 8388608u) != 0u) {
                        phi_2468_ = -(_e366);
                    }
                    let _e371 = phi_2468_;
                    let _e372 = (_e189 + _e371);
                    let _e377 = textureLoad(LC, vec2<i32>((_e372 & 2047i), (_e372 >> bitcast<u32>(11i))), 0i);
                    let _e381 = abs((bitcast<f32>(_e377.z) - _e302));
                    phi_2477_ = _e381;
                    if (_e381 > 3.1415927f) {
                        phi_2477_ = (6.2831855f - _e381);
                    }
                    let _e385 = phi_2477_;
                    let _e390 = ((_e385 * select(0.5f, -0.5f, ((_e364 != 0u) == _e321))) + _e302);
                    let _e394 = vec2<f32>(sin(_e390), -(cos(_e390)));
                    let _e395 = (_e126 * _e394);
                    let _e403 = ((abs(_e395.x) + abs(_e395.y)) * (1f / dot(_e395, _e395)));
                    let _e405 = cos((_e385 * 0.5f));
                    let _e406 = (_e194 == 335544320u);
                    phi_1722_ = _e406;
                    if !(_e406) {
                        phi_1722_ = ((_e194 == 268435456u) && (_e405 >= 0.25f));
                    }
                    let _e412 = phi_1722_;
                    if _e412 {
                        phi_2484_ = (_e349 * (1f / max(_e405, select(0.25f, 1f, ((_e193 & 33554432u) != 0u)))));
                    } else {
                        phi_2484_ = ((_e349 * _e405) + (_e403 * 0.5f));
                    }
                    let _e423 = phi_2484_;
                    let _e425 = (_e423 + (_e403 * 0.5f));
                    phi_2499_ = _e351;
                    if ((_e193 & 2097152u) != 0u) {
                        if (_e350 <= ((_e425 * _e405) + (_e342 * 0.125f))) {
                            phi_2500_ = (_e394 * (_e350 * (1f / _e405)));
                        } else {
                            let _e435 = (_e394 * _e425);
                            phi_2500_ = (vec2<f32>(dot(_e351, _e351), dot(_e435, _e435)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e351, _e435)));
                        }
                        let _e443 = phi_2500_;
                        phi_2499_ = _e443;
                    }
                    let _e445 = phi_2499_;
                    let _e450 = ((_e425 - dot((_e445 * abs(_e329)), _e394)) / _e403);
                    if _e321 {
                        phi_2521_ = vec4<f32>(_e362.x, _e450, _e362.z, _e362.w);
                    } else {
                        phi_2521_ = vec4<f32>(_e450, _e362.y, _e362.z, _e362.w);
                    }
                    let _e462 = phi_2521_;
                    phi_2536_ = _e445;
                    phi_2520_ = _e462;
                }
                let _e464 = phi_2536_;
                let _e466 = phi_2520_;
                let _e468 = (_e466.xy * _e348);
                let _e474 = vec4<f32>(_e468.x, _e466.y, _e466.z, _e466.w);
                let _e481 = vec4<f32>(_e474.x, max(_e468.y, 0.0001f), _e474.z, _e474.w);
                phi_2567_ = _e481;
                if _e330 {
                    phi_2567_ = vec4<f32>((-2f - _e468.x), _e481.y, _e481.z, _e481.w);
                }
                let _e489 = phi_2567_;
                if (_e85 != 0i) {
                    phi_2609_ = _e489;
                    phi_2571_ = vec2<f32>();
                    phi_2570_ = false;
                    break;
                }
                phi_2566_ = _e489;
                phi_2562_ = (_e126 * (_e464 * _e329));
                phi_2538_ = _e308;
            } else {
                let _e493 = vec4<f32>(_e148, -1f, 0f, 0f);
                if (_e315 != 0f) {
                    let _e499 = vec4<f32>(_e493.x, -2f, _e493.z, _e493.w);
                    let _e504 = vec4<f32>(_e499.x, _e499.y, 1000000f, _e499.w);
                    phi_2415_ = vec4<f32>(_e504.x, _e504.y, _e504.z, _e148);
                    if _e197 {
                        phi_2372_ = _e298;
                        phi_2371_ = _e296;
                        if (_e298 < 0f) {
                            phi_2372_ = -(_e298);
                            phi_2371_ = (_e296 + _e298);
                        }
                        let _e514 = phi_2372_;
                        let _e516 = phi_2371_;
                        let _e518 = ((_e302 - _e516) + 1.5707964f);
                        let _e524 = clamp(((_e518 - (floor((_e518 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e514);
                        phi_2373_ = _e524;
                        if (_e524 > (_e514 * 0.5f)) {
                            phi_2373_ = (_e514 - _e524);
                        }
                        let _e529 = phi_2373_;
                        let _e536 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e529), cos(_e529)) * abs(_e300))) * 0.5f);
                        if (abs((_e514 - 1.5707964f)) < 0.001f) {
                            phi_2399_ = 0f;
                            phi_2397_ = 0f;
                        } else {
                            let _e540 = tan(_e514);
                            let _e545 = (sign((1.5707964f - _e514)) / max(abs(_e540), 0.000001f));
                            if (_e545 >= 0f) {
                                phi_2377_ = (_e536.y - ((1f - _e536.x) * _e540));
                            } else {
                                phi_2377_ = (_e536.y + (_e536.x * _e540));
                            }
                            let _e557 = phi_2377_;
                            phi_2399_ = _e557;
                            phi_2397_ = _e545;
                        }
                        let _e559 = phi_2399_;
                        let _e561 = phi_2397_;
                        phi_2415_ = vec4<f32>((max(_e536.x, 0f) + 0.25f), (-2f - _e536.y), _e561, _e559);
                    }
                    let _e569 = phi_2415_;
                    phi_2565_ = (_e126 * (_e306 * (_e300 * _e315)));
                    phi_2414_ = _e569;
                } else {
                    phi_2565_ = (sign(((_e306 * _e300) * _naga_inverse_2x2_f32(_e126))) * 0.5f);
                    phi_2414_ = _e493;
                }
                let _e579 = phi_2565_;
                let _e581 = phi_2414_;
                phi_2569_ = _e581;
                if (((_e193 & 8388608u) != 0u) != ((_e193 & 16777216u) != 0u)) {
                    phi_2569_ = (_e581 * vec4<f32>(-1f, 1f, 1f, 1f));
                }
                let _e589 = phi_2569_;
                if (((_e193 & 2147483648u) != 0u) && (_e85 != 1i)) {
                    phi_2609_ = _e589;
                    phi_2571_ = vec2<f32>();
                    phi_2570_ = false;
                    break;
                }
                phi_2566_ = _e589;
                phi_2562_ = _e579;
                phi_2538_ = select(_e308, _e107, vec2((_e85 == 2i)));
            }
            let _e598 = phi_2566_;
            let _e600 = phi_2562_;
            let _e602 = phi_2538_;
            let _e608 = m.ug;
            let _e611 = select(_e598.xy, vec2<f32>(1f, -1f), vec2((_e608 != 0u)));
            let _e617 = vec4<f32>(_e611.x, _e598.y, _e598.z, _e598.w);
            phi_2609_ = vec4<f32>(_e617.x, _e611.y, _e617.z, _e617.w);
            phi_2571_ = (((_e126 * _e602) + _e600) + bitcast<vec2<f32>>(_e134.xy));
            phi_2570_ = true;
            break;
        }
    }
    let _e625 = phi_2609_;
    let _e627 = phi_2571_;
    let _e629 = phi_2570_;
    O = _e625;
    if _e629 {
        let _e631 = local;
        let _e632 = (_e631 + 2u);
        let _e639 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e632 & 127u)), bitcast<i32>((_e632 >> bitcast<u32>(7i)))), 0i);
        let _e641 = bitcast<vec3<f32>>(_e639.yzw);
        let _e645 = ((_e627 * _e641.x) + _e641.yz);
        let _e648 = m.ld[0u];
        let _e651 = m.ld[1u];
        phi_2610_ = vec4<f32>(((_e645.x * _e648) - 1f), ((_e645.y * _e651) - sign(_e651)), 0f, 1f);
    } else {
        let _e661 = m.N2_;
        phi_2610_ = vec4(_e661);
    }
    let _e664 = phi_2610_;
    unnamed.gl_Position = _e664;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e13 = O;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e13, _e14);
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
