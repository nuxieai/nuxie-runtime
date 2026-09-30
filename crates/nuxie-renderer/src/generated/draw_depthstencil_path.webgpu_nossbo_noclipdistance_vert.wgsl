struct BC {
    rc: f32,
    Bd: f32,
    Ff: f32,
    Gf: f32,
    q6_: u32,
    Ob: u32,
    rf: u32,
    sf: u32,
    V7_: vec4<i32>,
    ch: vec2<f32>,
    Cd: vec2<f32>,
    d2_: u32,
    gh: f32,
    f6_: u32,
    U2_: f32,
    Dd: f32,
    mf: u32,
    C3_: f32,
    D3_: f32,
    Ed: f32,
    Zg: u32,
    Nb: u32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Ch: bool = true;
@id(2) override Eh: bool = true;
@id(8) override Kh: bool = true;

@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(5)
var ID: texture_2d<u32>;
@group(0) @binding(2)
var PB: texture_2d<u32>;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
@group(0) @binding(3)
var DD: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> W1_: vec2<f32>;
var<private> g2_: f32;
@group(0) @binding(4)
var QB: texture_2d<f32>;
var<private> V1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ea: sampler;

fn main_1() {
    var phi_2283_: f32;
    var phi_2255_: i32;
    var phi_1488_: bool;
    var phi_2268_: i32;
    var phi_2260_: vec4<u32>;
    var phi_2267_: i32;
    var phi_2259_: vec4<u32>;
    var phi_2266_: i32;
    var phi_2264_: vec4<u32>;
    var phi_2263_: u32;
    var phi_2270_: vec2<i32>;
    var phi_2271_: vec4<u32>;
    var phi_2275_: f32;
    var phi_2346_: f32;
    var phi_2289_: f32;
    var phi_2345_: f32;
    var phi_2293_: f32;
    var phi_2290_: f32;
    var phi_2287_: f32;
    var phi_2297_: f32;
    var phi_2343_: f32;
    var phi_2296_: f32;
    var phi_2352_: f32;
    var phi_2349_: f32;
    var phi_2406_: f32;
    var phi_2378_: i32;
    var phi_2388_: f32;
    var phi_1800_: bool;
    var phi_2395_: f32;
    var phi_2416_: vec2<f32>;
    var phi_2415_: vec2<f32>;
    var phi_2414_: vec2<f32>;
    var phi_2432_: vec2<f32>;
    var phi_2417_: vec2<f32>;
    var phi_2465_: u32;
    var phi_2436_: vec2<f32>;
    var phi_2435_: bool;
    var local: u32;
    var local_1: u32;
    var phi_2494_: u32;
    var phi_2495_: f32;
    var phi_2496_: f32;
    var phi_2535_: vec4<f32>;
    var phi_2534_: f32;
    var local_2: u32;
    var phi_2532_: vec4<f32>;
    var phi_2533_: vec4<f32>;
    var phi_1165_: bool;
    var local_3: u32;
    var phi_2549_: vec4<f32>;

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
            let _e104 = (max((_e100.w & 65535u), 1u) - 1u);
            let _e111 = textureLoad(ID, vec2<i32>(bitcast<i32>((_e104 & 255u)), bitcast<i32>((_e104 >> bitcast<u32>(8i)))), 0i);
            let _e113 = bitcast<vec2<f32>>(_e111.xy);
            let _e115 = (_e111.z & 65535u);
            let _e117 = (_e115 * 4u);
            let _e124 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e117 & 255u)), bitcast<i32>((_e117 >> bitcast<u32>(8i)))), 0i);
            let _e125 = bitcast<vec4<f32>>(_e124);
            let _e132 = mat2x2<f32>(vec2<f32>(_e125.x, _e125.y), vec2<f32>(_e125.z, _e125.w));
            let _e133 = (_e117 + 1u);
            let _e140 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e133 & 255u)), bitcast<i32>((_e133 >> bitcast<u32>(8i)))), 0i);
            let _e144 = bitcast<f32>(_e140.z);
            let _e146 = bitcast<f32>(_e140.w);
            let _e147 = (_e100.w & 8388608u);
            phi_2283_ = _e81.y;
            phi_2255_ = _e85;
            local = _e111.z;
            local_1 = _e115;
            local_2 = _e115;
            local_3 = _e115;
            if (_e147 != 0u) {
                phi_2283_ = _e82.y;
                phi_2255_ = i32(_e82.x);
            }
            let _e153 = phi_2283_;
            let _e155 = phi_2255_;
            phi_2266_ = _e95;
            phi_2264_ = _e100;
            phi_2263_ = _e100.w;
            if (_e155 != _e93) {
                let _e158 = ((_e95 + _e155) - _e93);
                let _e163 = textureLoad(KC, vec2<i32>((_e158 & 2047i), (_e158 >> bitcast<u32>(11i))), 0i);
                if ((_e163.w & 8454143u) != (_e100.w & 8454143u)) {
                    let _e168 = (_e144 == 0f);
                    phi_1488_ = _e168;
                    if !(_e168) {
                        phi_1488_ = (_e113.x != 0f);
                    }
                    let _e173 = phi_1488_;
                    phi_2268_ = _e95;
                    phi_2260_ = _e100;
                    if _e173 {
                        let _e174 = bitcast<i32>(_e111.w);
                        let _e179 = textureLoad(KC, vec2<i32>((_e174 & 2047i), (_e174 >> bitcast<u32>(11i))), 0i);
                        phi_2268_ = _e174;
                        phi_2260_ = _e179;
                    }
                    let _e181 = phi_2268_;
                    let _e183 = phi_2260_;
                    phi_2267_ = _e181;
                    phi_2259_ = _e183;
                } else {
                    phi_2267_ = _e158;
                    phi_2259_ = _e163;
                }
                let _e185 = phi_2267_;
                let _e187 = phi_2259_;
                phi_2266_ = _e185;
                phi_2264_ = _e187;
                phi_2263_ = ((_e187.w & 4286578687u) | _e147);
            }
            let _e192 = phi_2266_;
            let _e194 = phi_2264_;
            let _e196 = phi_2263_;
            let _e197 = (_e196 & 469762048u);
            if ((_e197 == 67108864u) && (_e91 == 0i)) {
                let _e203 = f32((_e194.z & 65535u));
                let _e206 = f32((_e194.z >> bitcast<u32>(16i)));
                let _e212 = vec2<i32>(i32((-1f - _e203)), i32(((_e206 - _e203) + 1f)));
                phi_2270_ = _e212;
                if ((_e196 & 8388608u) != 0u) {
                    phi_2270_ = -(_e212);
                }
                let _e217 = phi_2270_;
                let _e219 = (_e192 + _e217.x);
                let _e224 = textureLoad(KC, vec2<i32>((_e219 & 2047i), (_e219 >> bitcast<u32>(11i))), 0i);
                let _e226 = (_e192 + _e217.y);
                let _e231 = textureLoad(KC, vec2<i32>((_e226 & 2047i), (_e226 >> bitcast<u32>(11i))), 0i);
                phi_2271_ = _e231;
                if ((_e231.w & 8454143u) != (_e224.w & 8454143u)) {
                    let _e237 = bitcast<i32>(_e111.w);
                    let _e242 = textureLoad(KC, vec2<i32>((_e237 & 2047i), (_e237 >> bitcast<u32>(11i))), 0i);
                    phi_2271_ = _e242;
                }
                let _e244 = phi_2271_;
                let _e246 = bitcast<f32>(_e224.z);
                let _e248 = bitcast<f32>(_e244.z);
                let _e249 = (_e248 - _e246);
                phi_2275_ = _e249;
                if (abs(_e249) > 3.1415927f) {
                    phi_2275_ = (_e249 - (6.2831855f * sign(_e249)));
                }
                let _e256 = phi_2275_;
                let _e257 = (_e206 + -2f);
                let _e263 = clamp(round(((abs(_e256) * 0.31830987f) * _e257)), 1f, (_e206 + -3f));
                let _e264 = (_e257 - _e263);
                if (_e203 <= _e264) {
                    phi_2346_ = _e153;
                    if (_e203 == _e264) {
                        phi_2346_ = -(_e153);
                    }
                    let _e273 = phi_2346_;
                    phi_2345_ = _e273;
                    phi_2293_ = -(((3.1415927f * sign(_e256)) - _e256));
                    phi_2290_ = _e264;
                    phi_2287_ = _e203;
                } else {
                    let _e275 = (_e203 == (_e264 + 1f));
                    if _e275 {
                        phi_2289_ = 0f;
                    } else {
                        phi_2289_ = (_e203 - (_e264 + 2f));
                    }
                    let _e279 = phi_2289_;
                    phi_2345_ = select(_e153, 0f, _e275);
                    phi_2293_ = _e256;
                    phi_2290_ = select(_e263, 0f, _e275);
                    phi_2287_ = _e279;
                }
                let _e283 = phi_2345_;
                let _e285 = phi_2293_;
                let _e287 = phi_2290_;
                let _e289 = phi_2287_;
                if (_e289 == _e287) {
                    phi_2297_ = _e248;
                } else {
                    phi_2297_ = (_e246 + (_e285 * (_e289 / _e287)));
                }
                let _e295 = phi_2297_;
                phi_2343_ = _e283;
                phi_2296_ = _e295;
            } else {
                phi_2343_ = _e153;
                phi_2296_ = bitcast<f32>(_e194.z);
            }
            let _e299 = phi_2343_;
            let _e301 = phi_2296_;
            let _e305 = vec2<f32>(sin(_e301), -(cos(_e301)));
            let _e307 = bitcast<vec2<f32>>(_e194.xy);
            phi_2352_ = _e146;
            if (_e146 != 0f) {
                phi_2352_ = max(_e146, (1f / length((_e132 * _e305))));
            }
            let _e314 = phi_2352_;
            if (_e144 != 0f) {
                let _e318 = (_e299 * sign(determinant(_e132)));
                let _e320 = ((_e196 & 1048576u) != 0u);
                phi_2349_ = _e318;
                if _e320 {
                    phi_2349_ = min(_e318, 0f);
                }
                let _e323 = phi_2349_;
                phi_2406_ = _e323;
                if ((_e196 & 524288u) != 0u) {
                    phi_2406_ = max(_e323, 0f);
                }
                let _e328 = phi_2406_;
                let _e330 = select(0f, _e314, (_e314 != 0f));
                let _e334 = select(_e144, _e330, ((_e330 > _e144) && (_e314 == 0f)));
                let _e335 = (_e334 + _e330);
                let _e336 = (_e305 * _e335);
                phi_2414_ = _e336;
                if (_e197 > 134217728u) {
                    let _e338 = (_e196 & 4194304u);
                    let _e340 = select(2i, -2i, (_e338 == 0u));
                    phi_2378_ = _e340;
                    if ((_e196 & 8388608u) != 0u) {
                        phi_2378_ = -(_e340);
                    }
                    let _e345 = phi_2378_;
                    let _e346 = (_e192 + _e345);
                    let _e351 = textureLoad(KC, vec2<i32>((_e346 & 2047i), (_e346 >> bitcast<u32>(11i))), 0i);
                    let _e355 = abs((bitcast<f32>(_e351.z) - _e301));
                    phi_2388_ = _e355;
                    if (_e355 > 3.1415927f) {
                        phi_2388_ = (6.2831855f - _e355);
                    }
                    let _e359 = phi_2388_;
                    let _e364 = ((_e359 * select(0.5f, -0.5f, ((_e338 != 0u) == _e320))) + _e301);
                    let _e368 = vec2<f32>(sin(_e364), -(cos(_e364)));
                    let _e369 = (_e132 * _e368);
                    let _e379 = cos((_e359 * 0.5f));
                    let _e380 = (_e197 == 335544320u);
                    phi_1800_ = _e380;
                    if !(_e380) {
                        phi_1800_ = ((_e197 == 268435456u) && (_e379 >= 0.25f));
                    }
                    let _e386 = phi_1800_;
                    if _e386 {
                        phi_2395_ = (_e334 * (1f / max(_e379, select(0.25f, 1f, ((_e196 & 33554432u) != 0u)))));
                    } else {
                        phi_2395_ = ((_e334 * _e379) + (((abs(_e369.x) + abs(_e369.y)) * (1f / dot(_e369, _e369))) * 0.5f));
                    }
                    let _e397 = phi_2395_;
                    phi_2415_ = _e336;
                    if ((_e196 & 2097152u) != 0u) {
                        if (_e335 <= ((_e397 * _e379) + (_e330 * 0.125f))) {
                            phi_2416_ = (_e368 * (_e335 * (1f / _e379)));
                        } else {
                            let _e407 = (_e368 * _e397);
                            phi_2416_ = (vec2<f32>(dot(_e336, _e336), dot(_e407, _e407)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e336, _e407)));
                        }
                        let _e415 = phi_2416_;
                        phi_2415_ = _e415;
                    }
                    let _e417 = phi_2415_;
                    phi_2414_ = _e417;
                }
                let _e419 = phi_2414_;
                if (_e91 != 0i) {
                    phi_2465_ = u32();
                    phi_2436_ = vec2<f32>();
                    phi_2435_ = false;
                    break;
                }
                phi_2432_ = (_e132 * (_e419 * _e328));
                phi_2417_ = _e307;
            } else {
                if (((_e196 & 2147483648u) != 0u) && (_e91 != 1i)) {
                    phi_2465_ = u32();
                    phi_2436_ = vec2<f32>();
                    phi_2435_ = false;
                    break;
                }
                phi_2432_ = vec2<f32>(0f, 0f);
                phi_2417_ = select(_e307, _e113, vec2((_e91 == 2i)));
            }
            let _e431 = phi_2432_;
            let _e433 = phi_2417_;
            let _e437 = (_e117 + 2u);
            let _e444 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e437 & 255u)), bitcast<i32>((_e437 >> bitcast<u32>(8i)))), 0i);
            phi_2465_ = _e444.x;
            phi_2436_ = (((_e132 * _e433) + _e431) + bitcast<vec2<f32>>(_e140.xy));
            phi_2435_ = true;
            break;
        }
    }
    let _e447 = phi_2465_;
    let _e449 = phi_2436_;
    let _e451 = phi_2435_;
    let _e453 = local;
    let _e457 = local_1;
    let _e462 = textureLoad(DD, vec2<i32>(bitcast<i32>((_e453 & 255u)), bitcast<i32>((_e457 >> bitcast<u32>(8i)))), 0i);
    let _e464 = (_e462.x & 15u);
    if Ch {
        let _e465 = (_e464 == 0u);
        if _e465 {
            phi_2494_ = _e462.y;
        } else {
            phi_2494_ = _e462.x;
        }
        let _e468 = phi_2494_;
        let _e470 = (_e468 >> bitcast<u32>(16i));
        let _e472 = l.f6_;
        if (_e470 == 0u) {
            phi_2495_ = 0f;
        } else {
            phi_2495_ = unpack2x16float(((_e470 + 1023u) * _e472)).x;
        }
        let _e479 = phi_2495_;
        phi_2496_ = _e479;
        if _e465 {
            phi_2496_ = -(_e479);
        }
        let _e482 = phi_2496_;
        W1_[0u] = _e482;
    }
    if Eh {
        g2_ = f32(((_e462.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e464 == 1u) {
        let _e490 = unpack4x8unorm(_e462.y);
        if Eh {
            phi_2535_ = _e490;
        } else {
            let _e493 = (_e490.xyz * _e490.w);
            let _e499 = vec4<f32>(_e493.x, _e490.y, _e490.z, _e490.w);
            let _e505 = vec4<f32>(_e499.x, _e493.y, _e499.z, _e499.w);
            phi_2535_ = vec4<f32>(_e505.x, _e505.y, _e493.z, _e505.w);
        }
        let _e513 = phi_2535_;
        V1_ = _e513;
    } else {
        if (Ch && (_e464 == 0u)) {
            let _e517 = (_e462.x >> bitcast<u32>(16i));
            let _e519 = l.f6_;
            if (_e517 == 0u) {
                phi_2534_ = 0f;
            } else {
                phi_2534_ = unpack2x16float(((_e517 + 1023u) * _e519)).x;
            }
            let _e526 = phi_2534_;
            W1_[1u] = _e526;
        } else {
            let _e529 = local_2;
            let _e530 = (_e529 * 8u);
            let _e537 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e530 & 255u)), bitcast<i32>((_e530 >> bitcast<u32>(8i)))), 0i);
            let _e545 = (_e530 + 1u);
            let _e552 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e545 & 255u)), bitcast<i32>((_e545 >> bitcast<u32>(8i)))), 0i);
            let _e561 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e462.y));
            let _e563 = ((mat2x2<f32>(vec2<f32>(_e537.x, _e537.y), vec2<f32>(_e537.z, _e537.w)) * _e449) + _e552.xy);
            if (_e552.z > 0.9f) {
                phi_2532_ = vec4<f32>(_e561.x, _e561.y, 2f, _e561.w);
            } else {
                phi_2532_ = vec4<f32>(_e561.x, _e561.y, _e552.w, _e561.w);
            }
            let _e578 = phi_2532_;
            if (f32(_e464) == 2f) {
                let _e585 = vec4<f32>(_e563.x, _e578.y, _e578.z, _e578.w);
                phi_2533_ = vec4<f32>(_e585.x, 0f, _e585.z, _e585.w);
            } else {
                let _e597 = vec4<f32>(_e578.x, _e578.y, -(_e578.z), _e578.w);
                let _e603 = vec4<f32>(_e563.x, _e597.y, _e597.z, _e597.w);
                phi_2533_ = vec4<f32>(_e603.x, _e563.y, _e603.z, _e603.w);
            }
            let _e611 = phi_2533_;
            V1_ = _e611;
            let _e613 = V1_[3u];
            V1_[3u] = -(_e613);
        }
    }
    phi_1165_ = Kh;
    if Kh {
        phi_1165_ = ((_e462.x & 2048u) != 0u);
    }
    let _e618 = phi_1165_;
    if _e618 {
        let _e620 = local_3;
        let _e621 = (_e620 * 8u);
        let _e622 = (_e621 + 4u);
        let _e629 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e622 & 255u)), bitcast<i32>((_e622 >> bitcast<u32>(8i)))), 0i);
        let _e637 = (_e621 + 5u);
        let _e644 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e637 & 255u)), bitcast<i32>((_e637 >> bitcast<u32>(8i)))), 0i);
        let _e647 = ((mat2x2<f32>(vec2<f32>(_e629.x, _e629.y), vec2<f32>(_e629.z, _e629.w)) * _e449) + _e644.xy);
        C2_ = vec3<f32>(_e647.x, _e647.y, (1f + _e644.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e451 {
        let _e654 = l.Ff;
        let _e656 = l.Gf;
        let _e664 = vec4<f32>(((_e449.x * _e654) - 1f), ((_e449.y * _e656) - sign(_e656)), 0f, 1f);
        phi_2549_ = vec4<f32>(_e664.x, _e664.y, (1f - (f32(_e447) * 0.000061035156f)), _e664.w);
    } else {
        let _e674 = l.U2_;
        phi_2549_ = vec4(_e674);
    }
    let _e677 = phi_2549_;
    unnamed.gl_Position = _e677;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e16 = W1_;
    let _e17 = g2_;
    let _e18 = V1_;
    let _e19 = C2_;
    let _e20 = unnamed.gl_Position;
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
