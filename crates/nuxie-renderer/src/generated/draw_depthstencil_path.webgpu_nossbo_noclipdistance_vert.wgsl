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

@id(0) override Eh: bool = true;
@id(2) override Gh: bool = true;
@id(8) override Mh: bool = true;

@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(5)
var HD: texture_2d<u32>;
@group(0) @binding(2)
var OB: texture_2d<u32>;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> TB_1: vec4<f32>;
var<private> UB_1: vec4<f32>;
@group(0) @binding(3)
var CD: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: AC;
var<private> Y1_: vec2<f32>;
var<private> g1_: f32;
@group(0) @binding(4)
var PB: texture_2d<f32>;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ga: sampler;

fn main_1() {
    var phi_2260_: f32;
    var phi_2232_: i32;
    var phi_1472_: bool;
    var phi_2245_: i32;
    var phi_2237_: vec4<u32>;
    var phi_2244_: i32;
    var phi_2236_: vec4<u32>;
    var phi_2243_: i32;
    var phi_2241_: vec4<u32>;
    var phi_2240_: u32;
    var phi_2247_: vec2<i32>;
    var phi_2248_: vec4<u32>;
    var phi_2252_: f32;
    var phi_2323_: f32;
    var phi_2266_: f32;
    var phi_2322_: f32;
    var phi_2270_: f32;
    var phi_2267_: f32;
    var phi_2264_: f32;
    var phi_2274_: f32;
    var phi_2320_: f32;
    var phi_2273_: f32;
    var phi_2329_: f32;
    var phi_2326_: f32;
    var phi_2383_: f32;
    var phi_2355_: i32;
    var phi_2365_: f32;
    var phi_1784_: bool;
    var phi_2372_: f32;
    var phi_2393_: vec2<f32>;
    var phi_2392_: vec2<f32>;
    var phi_2391_: vec2<f32>;
    var phi_2409_: vec2<f32>;
    var phi_2394_: vec2<f32>;
    var phi_2442_: u32;
    var phi_2413_: vec2<f32>;
    var phi_2412_: bool;
    var local: u32;
    var local_1: u32;
    var phi_2471_: u32;
    var phi_2472_: f32;
    var phi_2473_: f32;
    var phi_2511_: f32;
    var local_2: u32;
    var phi_2509_: vec4<f32>;
    var phi_2510_: vec4<f32>;
    var phi_1148_: bool;
    var local_3: u32;
    var phi_2524_: vec4<f32>;

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
            let _e104 = (max((_e100.w & 65535u), 1u) - 1u);
            let _e111 = textureLoad(HD, vec2<i32>(bitcast<i32>((_e104 & 255u)), bitcast<i32>((_e104 >> bitcast<u32>(8i)))), 0i);
            let _e113 = bitcast<vec2<f32>>(_e111.xy);
            let _e115 = (_e111.z & 65535u);
            let _e117 = (_e115 * 4u);
            let _e124 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e117 & 255u)), bitcast<i32>((_e117 >> bitcast<u32>(8i)))), 0i);
            let _e125 = bitcast<vec4<f32>>(_e124);
            let _e132 = mat2x2<f32>(vec2<f32>(_e125.x, _e125.y), vec2<f32>(_e125.z, _e125.w));
            let _e133 = (_e117 + 1u);
            let _e140 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e133 & 255u)), bitcast<i32>((_e133 >> bitcast<u32>(8i)))), 0i);
            let _e144 = bitcast<f32>(_e140.z);
            let _e146 = bitcast<f32>(_e140.w);
            let _e147 = (_e100.w & 8388608u);
            phi_2260_ = _e81.y;
            phi_2232_ = _e85;
            local = _e111.z;
            local_1 = _e115;
            local_2 = _e115;
            local_3 = _e115;
            if (_e147 != 0u) {
                phi_2260_ = _e82.y;
                phi_2232_ = i32(_e82.x);
            }
            let _e153 = phi_2260_;
            let _e155 = phi_2232_;
            phi_2243_ = _e95;
            phi_2241_ = _e100;
            phi_2240_ = _e100.w;
            if (_e155 != _e93) {
                let _e158 = ((_e95 + _e155) - _e93);
                let _e163 = textureLoad(JC, vec2<i32>((_e158 & 2047i), (_e158 >> bitcast<u32>(11i))), 0i);
                if ((_e163.w & 8454143u) != (_e100.w & 8454143u)) {
                    let _e168 = (_e144 == 0f);
                    phi_1472_ = _e168;
                    if !(_e168) {
                        phi_1472_ = (_e113.x != 0f);
                    }
                    let _e173 = phi_1472_;
                    phi_2245_ = _e95;
                    phi_2237_ = _e100;
                    if _e173 {
                        let _e174 = bitcast<i32>(_e111.w);
                        let _e179 = textureLoad(JC, vec2<i32>((_e174 & 2047i), (_e174 >> bitcast<u32>(11i))), 0i);
                        phi_2245_ = _e174;
                        phi_2237_ = _e179;
                    }
                    let _e181 = phi_2245_;
                    let _e183 = phi_2237_;
                    phi_2244_ = _e181;
                    phi_2236_ = _e183;
                } else {
                    phi_2244_ = _e158;
                    phi_2236_ = _e163;
                }
                let _e185 = phi_2244_;
                let _e187 = phi_2236_;
                phi_2243_ = _e185;
                phi_2241_ = _e187;
                phi_2240_ = ((_e187.w & 4286578687u) | _e147);
            }
            let _e192 = phi_2243_;
            let _e194 = phi_2241_;
            let _e196 = phi_2240_;
            let _e197 = (_e196 & 469762048u);
            if ((_e197 == 67108864u) && (_e91 == 0i)) {
                let _e203 = f32((_e194.z & 65535u));
                let _e206 = f32((_e194.z >> bitcast<u32>(16i)));
                let _e212 = vec2<i32>(i32((-1f - _e203)), i32(((_e206 - _e203) + 1f)));
                phi_2247_ = _e212;
                if ((_e196 & 8388608u) != 0u) {
                    phi_2247_ = -(_e212);
                }
                let _e217 = phi_2247_;
                let _e219 = (_e192 + _e217.x);
                let _e224 = textureLoad(JC, vec2<i32>((_e219 & 2047i), (_e219 >> bitcast<u32>(11i))), 0i);
                let _e226 = (_e192 + _e217.y);
                let _e231 = textureLoad(JC, vec2<i32>((_e226 & 2047i), (_e226 >> bitcast<u32>(11i))), 0i);
                phi_2248_ = _e231;
                if ((_e231.w & 8454143u) != (_e224.w & 8454143u)) {
                    let _e237 = bitcast<i32>(_e111.w);
                    let _e242 = textureLoad(JC, vec2<i32>((_e237 & 2047i), (_e237 >> bitcast<u32>(11i))), 0i);
                    phi_2248_ = _e242;
                }
                let _e244 = phi_2248_;
                let _e246 = bitcast<f32>(_e224.z);
                let _e248 = bitcast<f32>(_e244.z);
                let _e249 = (_e248 - _e246);
                phi_2252_ = _e249;
                if (abs(_e249) > 3.1415927f) {
                    phi_2252_ = (_e249 - (6.2831855f * sign(_e249)));
                }
                let _e256 = phi_2252_;
                let _e257 = (_e206 + -2f);
                let _e263 = clamp(round(((abs(_e256) * 0.31830987f) * _e257)), 1f, (_e206 + -3f));
                let _e264 = (_e257 - _e263);
                if (_e203 <= _e264) {
                    phi_2323_ = _e153;
                    if (_e203 == _e264) {
                        phi_2323_ = -(_e153);
                    }
                    let _e273 = phi_2323_;
                    phi_2322_ = _e273;
                    phi_2270_ = -(((3.1415927f * sign(_e256)) - _e256));
                    phi_2267_ = _e264;
                    phi_2264_ = _e203;
                } else {
                    let _e275 = (_e203 == (_e264 + 1f));
                    if _e275 {
                        phi_2266_ = 0f;
                    } else {
                        phi_2266_ = (_e203 - (_e264 + 2f));
                    }
                    let _e279 = phi_2266_;
                    phi_2322_ = select(_e153, 0f, _e275);
                    phi_2270_ = _e256;
                    phi_2267_ = select(_e263, 0f, _e275);
                    phi_2264_ = _e279;
                }
                let _e283 = phi_2322_;
                let _e285 = phi_2270_;
                let _e287 = phi_2267_;
                let _e289 = phi_2264_;
                if (_e289 == _e287) {
                    phi_2274_ = _e248;
                } else {
                    phi_2274_ = (_e246 + (_e285 * (_e289 / _e287)));
                }
                let _e295 = phi_2274_;
                phi_2320_ = _e283;
                phi_2273_ = _e295;
            } else {
                phi_2320_ = _e153;
                phi_2273_ = bitcast<f32>(_e194.z);
            }
            let _e299 = phi_2320_;
            let _e301 = phi_2273_;
            let _e305 = vec2<f32>(sin(_e301), -(cos(_e301)));
            let _e307 = bitcast<vec2<f32>>(_e194.xy);
            phi_2329_ = _e146;
            if (_e146 != 0f) {
                phi_2329_ = max(_e146, (1f / length((_e132 * _e305))));
            }
            let _e314 = phi_2329_;
            if (_e144 != 0f) {
                let _e318 = (_e299 * sign(determinant(_e132)));
                let _e320 = ((_e196 & 1048576u) != 0u);
                phi_2326_ = _e318;
                if _e320 {
                    phi_2326_ = min(_e318, 0f);
                }
                let _e323 = phi_2326_;
                phi_2383_ = _e323;
                if ((_e196 & 524288u) != 0u) {
                    phi_2383_ = max(_e323, 0f);
                }
                let _e328 = phi_2383_;
                let _e330 = select(0f, _e314, (_e314 != 0f));
                let _e334 = select(_e144, _e330, ((_e330 > _e144) && (_e314 == 0f)));
                let _e335 = (_e334 + _e330);
                let _e336 = (_e305 * _e335);
                phi_2391_ = _e336;
                if (_e197 > 134217728u) {
                    let _e338 = (_e196 & 4194304u);
                    let _e340 = select(2i, -2i, (_e338 == 0u));
                    phi_2355_ = _e340;
                    if ((_e196 & 8388608u) != 0u) {
                        phi_2355_ = -(_e340);
                    }
                    let _e345 = phi_2355_;
                    let _e346 = (_e192 + _e345);
                    let _e351 = textureLoad(JC, vec2<i32>((_e346 & 2047i), (_e346 >> bitcast<u32>(11i))), 0i);
                    let _e355 = abs((bitcast<f32>(_e351.z) - _e301));
                    phi_2365_ = _e355;
                    if (_e355 > 3.1415927f) {
                        phi_2365_ = (6.2831855f - _e355);
                    }
                    let _e359 = phi_2365_;
                    let _e364 = ((_e359 * select(0.5f, -0.5f, ((_e338 != 0u) == _e320))) + _e301);
                    let _e368 = vec2<f32>(sin(_e364), -(cos(_e364)));
                    let _e369 = (_e132 * _e368);
                    let _e379 = cos((_e359 * 0.5f));
                    let _e380 = (_e197 == 335544320u);
                    phi_1784_ = _e380;
                    if !(_e380) {
                        phi_1784_ = ((_e197 == 268435456u) && (_e379 >= 0.25f));
                    }
                    let _e386 = phi_1784_;
                    if _e386 {
                        phi_2372_ = (_e334 * (1f / max(_e379, select(0.25f, 1f, ((_e196 & 33554432u) != 0u)))));
                    } else {
                        phi_2372_ = ((_e334 * _e379) + (((abs(_e369.x) + abs(_e369.y)) * (1f / dot(_e369, _e369))) * 0.5f));
                    }
                    let _e397 = phi_2372_;
                    phi_2392_ = _e336;
                    if ((_e196 & 2097152u) != 0u) {
                        if (_e335 <= ((_e397 * _e379) + (_e330 * 0.125f))) {
                            phi_2393_ = (_e368 * (_e335 * (1f / _e379)));
                        } else {
                            let _e407 = (_e368 * _e397);
                            phi_2393_ = (vec2<f32>(dot(_e336, _e336), dot(_e407, _e407)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e336, _e407)));
                        }
                        let _e415 = phi_2393_;
                        phi_2392_ = _e415;
                    }
                    let _e417 = phi_2392_;
                    phi_2391_ = _e417;
                }
                let _e419 = phi_2391_;
                if (_e91 != 0i) {
                    phi_2442_ = u32();
                    phi_2413_ = vec2<f32>();
                    phi_2412_ = false;
                    break;
                }
                phi_2409_ = (_e132 * (_e419 * _e328));
                phi_2394_ = _e307;
            } else {
                if (((_e196 & 2147483648u) != 0u) && (_e91 != 1i)) {
                    phi_2442_ = u32();
                    phi_2413_ = vec2<f32>();
                    phi_2412_ = false;
                    break;
                }
                phi_2409_ = vec2<f32>(0f, 0f);
                phi_2394_ = select(_e307, _e113, vec2((_e91 == 2i)));
            }
            let _e431 = phi_2409_;
            let _e433 = phi_2394_;
            let _e437 = (_e117 + 2u);
            let _e444 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e437 & 255u)), bitcast<i32>((_e437 >> bitcast<u32>(8i)))), 0i);
            phi_2442_ = _e444.x;
            phi_2413_ = (((_e132 * _e433) + _e431) + bitcast<vec2<f32>>(_e140.xy));
            phi_2412_ = true;
            break;
        }
    }
    let _e447 = phi_2442_;
    let _e449 = phi_2413_;
    let _e451 = phi_2412_;
    let _e453 = local;
    let _e457 = local_1;
    let _e462 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e453 & 255u)), bitcast<i32>((_e457 >> bitcast<u32>(8i)))), 0i);
    let _e464 = (_e462.x & 15u);
    if Eh {
        let _e465 = (_e464 == 0u);
        if _e465 {
            phi_2471_ = _e462.y;
        } else {
            phi_2471_ = _e462.x;
        }
        let _e468 = phi_2471_;
        let _e470 = (_e468 >> bitcast<u32>(16i));
        let _e472 = j.f6_;
        if (_e470 == 0u) {
            phi_2472_ = 0f;
        } else {
            phi_2472_ = unpack2x16float(((_e470 + 1023u) * _e472)).x;
        }
        let _e479 = phi_2472_;
        phi_2473_ = _e479;
        if _e465 {
            phi_2473_ = -(_e479);
        }
        let _e482 = phi_2473_;
        Y1_[0u] = _e482;
    }
    if Gh {
        g1_ = f32(((_e462.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e464 == 1u) {
        X1_ = unpack4x8unorm(_e462.y);
    } else {
        if (Eh && (_e464 == 0u)) {
            let _e494 = (_e462.x >> bitcast<u32>(16i));
            let _e496 = j.f6_;
            if (_e494 == 0u) {
                phi_2511_ = 0f;
            } else {
                phi_2511_ = unpack2x16float(((_e494 + 1023u) * _e496)).x;
            }
            let _e503 = phi_2511_;
            Y1_[1u] = _e503;
        } else {
            let _e506 = local_2;
            let _e507 = (_e506 * 8u);
            let _e514 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e507 & 255u)), bitcast<i32>((_e507 >> bitcast<u32>(8i)))), 0i);
            let _e522 = (_e507 + 1u);
            let _e529 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e522 & 255u)), bitcast<i32>((_e522 >> bitcast<u32>(8i)))), 0i);
            let _e538 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e462.y));
            let _e540 = ((mat2x2<f32>(vec2<f32>(_e514.x, _e514.y), vec2<f32>(_e514.z, _e514.w)) * _e449) + _e529.xy);
            if (_e529.z > 0.9f) {
                phi_2509_ = vec4<f32>(_e538.x, _e538.y, 2f, _e538.w);
            } else {
                phi_2509_ = vec4<f32>(_e538.x, _e538.y, _e529.w, _e538.w);
            }
            let _e555 = phi_2509_;
            if (f32(_e464) == 2f) {
                let _e562 = vec4<f32>(_e540.x, _e555.y, _e555.z, _e555.w);
                phi_2510_ = vec4<f32>(_e562.x, 0f, _e562.z, _e562.w);
            } else {
                let _e574 = vec4<f32>(_e555.x, _e555.y, -(_e555.z), _e555.w);
                let _e580 = vec4<f32>(_e540.x, _e574.y, _e574.z, _e574.w);
                phi_2510_ = vec4<f32>(_e580.x, _e540.y, _e580.z, _e580.w);
            }
            let _e588 = phi_2510_;
            X1_ = _e588;
            let _e590 = X1_[3u];
            X1_[3u] = -(_e590);
        }
    }
    phi_1148_ = Mh;
    if Mh {
        phi_1148_ = ((_e462.x & 2048u) != 0u);
    }
    let _e595 = phi_1148_;
    if _e595 {
        let _e597 = local_3;
        let _e598 = (_e597 * 8u);
        let _e599 = (_e598 + 4u);
        let _e606 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e599 & 255u)), bitcast<i32>((_e599 >> bitcast<u32>(8i)))), 0i);
        let _e614 = (_e598 + 5u);
        let _e621 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e614 & 255u)), bitcast<i32>((_e614 >> bitcast<u32>(8i)))), 0i);
        let _e624 = ((mat2x2<f32>(vec2<f32>(_e606.x, _e606.y), vec2<f32>(_e606.z, _e606.w)) * _e449) + _e621.xy);
        C2_ = vec3<f32>(_e624.x, _e624.y, (1f + _e621.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e451 {
        let _e631 = j.Hf;
        let _e633 = j.If;
        let _e641 = vec4<f32>(((_e449.x * _e631) - 1f), ((_e449.y * _e633) - sign(_e633)), 0f, 1f);
        phi_2524_ = vec4<f32>(_e641.x, _e641.y, (1f - (f32(_e447) * 0.000061035156f)), _e641.w);
    } else {
        let _e651 = j.U2_;
        phi_2524_ = vec4(_e651);
    }
    let _e654 = phi_2524_;
    unnamed.gl_Position = _e654;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) TB: vec4<f32>, @location(1) UB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    TB_1 = TB;
    UB_1 = UB;
    main_1();
    let _e16 = Y1_;
    let _e17 = g1_;
    let _e18 = X1_;
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
