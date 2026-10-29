struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
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

@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(4)
var CD: texture_2d<u32>;
@group(0) @binding(2)
var JB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> ZB_1: vec4<f32>;
var<private> S: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(3)
var KB: texture_2d<f32>;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_2329_: f32;
    var phi_2266_: f32;
    var phi_2356_: f32;
    var phi_2292_: f32;
    var phi_2260_: i32;
    var phi_1410_: bool;
    var phi_2277_: i32;
    var phi_2268_: vec4<u32>;
    var phi_2276_: i32;
    var phi_2267_: vec4<u32>;
    var phi_2275_: i32;
    var phi_2273_: vec4<u32>;
    var phi_2272_: u32;
    var phi_2279_: vec2<i32>;
    var phi_2280_: vec4<u32>;
    var phi_2284_: f32;
    var phi_2367_: f32;
    var phi_2299_: f32;
    var phi_2366_: f32;
    var phi_2307_: f32;
    var phi_2300_: f32;
    var phi_2297_: f32;
    var phi_2311_: f32;
    var phi_2389_: f32;
    var phi_2379_: f32;
    var phi_2364_: f32;
    var phi_2310_: f32;
    var phi_2362_: f32;
    var phi_2448_: f32;
    var phi_2459_: f32;
    var phi_2451_: f32;
    var phi_2513_: f32;
    var phi_2487_: f32;
    var phi_1725_: bool;
    var phi_2492_: f32;
    var phi_2504_: vec2<f32>;
    var phi_2503_: vec2<f32>;
    var phi_2521_: vec4<f32>;
    var phi_2534_: vec2<f32>;
    var phi_2520_: vec4<f32>;
    var phi_2568_: vec4<f32>;
    var phi_2400_: f32;
    var phi_2399_: f32;
    var phi_2401_: f32;
    var phi_2405_: f32;
    var phi_2428_: f32;
    var phi_2426_: f32;
    var phi_2444_: vec4<f32>;
    var phi_2565_: vec2<f32>;
    var phi_2443_: vec4<f32>;
    var phi_2571_: vec4<f32>;
    var phi_2572_: bool;
    var phi_2566_: vec4<f32>;
    var phi_2560_: vec2<f32>;
    var phi_2536_: vec2<f32>;
    var phi_2604_: vec4<f32>;

    let _e74 = gl_InstanceIndex_1;
    let _e75 = YB_1;
    let _e76 = ZB_1;
    let _e78 = i32(_e75.x);
    let _e82 = bitcast<i32>(_e75.w);
    let _e84 = (_e82 >> bitcast<u32>(2i));
    let _e85 = (_e82 & 3i);
    let _e87 = min(_e78, (_e84 - 1i));
    let _e89 = ((_e74 * _e84) + _e87);
    let _e94 = textureLoad(VB, vec2<i32>((_e89 & 2047i), (_e89 >> bitcast<u32>(11i))), 0i);
    let _e98 = (max((_e94.w & 65535u), 1u) - 1u);
    let _e105 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e98 & 255u)), bitcast<i32>((_e98 >> bitcast<u32>(8i)))), 0i);
    let _e107 = bitcast<vec2<f32>>(_e105.xy);
    let _e111 = ((_e105.z & 65535u) * 4u);
    let _e118 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e111 & 255u)), bitcast<i32>((_e111 >> bitcast<u32>(8i)))), 0i);
    let _e119 = bitcast<vec4<f32>>(_e118);
    let _e126 = mat2x2<f32>(vec2<f32>(_e119.x, _e119.y), vec2<f32>(_e119.z, _e119.w));
    let _e127 = (_e111 + 1u);
    let _e134 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e127 & 255u)), bitcast<i32>((_e127 >> bitcast<u32>(8i)))), 0i);
    phi_2329_ = 0f;
    phi_2266_ = 0f;
    if ((_e134.z & 32768u) != 0u) {
        let _e140 = (_e111 + 2u);
        let _e147 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e140 & 255u)), bitcast<i32>((_e140 >> bitcast<u32>(8i)))), 0i);
        phi_2329_ = bitcast<f32>(_e147.y);
        phi_2266_ = bitcast<f32>(_e147.x);
    }
    let _e153 = phi_2329_;
    let _e155 = phi_2266_;
    let _e156 = (_e94.w & 8388608u);
    phi_2356_ = _e75.z;
    phi_2292_ = _e75.y;
    phi_2260_ = _e78;
    if (_e156 != 0u) {
        phi_2356_ = _e76.z;
        phi_2292_ = _e76.y;
        phi_2260_ = i32(_e76.x);
    }
    let _e163 = phi_2356_;
    let _e165 = phi_2292_;
    let _e167 = phi_2260_;
    phi_2275_ = _e89;
    phi_2273_ = _e94;
    phi_2272_ = _e94.w;
    if (_e167 != _e87) {
        let _e170 = ((_e89 + _e167) - _e87);
        let _e175 = textureLoad(VB, vec2<i32>((_e170 & 2047i), (_e170 >> bitcast<u32>(11i))), 0i);
        if ((_e175.w & 8454143u) != (_e94.w & 8454143u)) {
            let _e180 = (_e155 == 0f);
            phi_1410_ = _e180;
            if !(_e180) {
                phi_1410_ = (_e107.x != 0f);
            }
            let _e185 = phi_1410_;
            phi_2277_ = _e89;
            phi_2268_ = _e94;
            if _e185 {
                let _e186 = bitcast<i32>(_e105.w);
                let _e191 = textureLoad(VB, vec2<i32>((_e186 & 2047i), (_e186 >> bitcast<u32>(11i))), 0i);
                phi_2277_ = _e186;
                phi_2268_ = _e191;
            }
            let _e193 = phi_2277_;
            let _e195 = phi_2268_;
            phi_2276_ = _e193;
            phi_2267_ = _e195;
        } else {
            phi_2276_ = _e170;
            phi_2267_ = _e175;
        }
        let _e197 = phi_2276_;
        let _e199 = phi_2267_;
        phi_2275_ = _e197;
        phi_2273_ = _e199;
        phi_2272_ = ((_e199.w & 4286578687u) | _e156);
    }
    let _e204 = phi_2275_;
    let _e206 = phi_2273_;
    let _e208 = phi_2272_;
    let _e209 = (_e208 & 469762048u);
    let _e212 = ((_e209 == 67108864u) && (_e85 == 0i));
    if _e212 {
        let _e215 = f32((_e206.z & 65535u));
        let _e218 = f32((_e206.z >> bitcast<u32>(16i)));
        let _e224 = vec2<i32>(i32((-1f - _e215)), i32(((_e218 - _e215) + 1f)));
        phi_2279_ = _e224;
        if ((_e208 & 8388608u) != 0u) {
            phi_2279_ = -(_e224);
        }
        let _e229 = phi_2279_;
        let _e231 = (_e204 + _e229.x);
        let _e236 = textureLoad(VB, vec2<i32>((_e231 & 2047i), (_e231 >> bitcast<u32>(11i))), 0i);
        let _e238 = (_e204 + _e229.y);
        let _e243 = textureLoad(VB, vec2<i32>((_e238 & 2047i), (_e238 >> bitcast<u32>(11i))), 0i);
        phi_2280_ = _e243;
        if ((_e243.w & 8454143u) != (_e236.w & 8454143u)) {
            let _e249 = bitcast<i32>(_e105.w);
            let _e254 = textureLoad(VB, vec2<i32>((_e249 & 2047i), (_e249 >> bitcast<u32>(11i))), 0i);
            phi_2280_ = _e254;
        }
        let _e256 = phi_2280_;
        let _e259 = (f32(_e236.z) * 0.0000000014629181f);
        let _e262 = (f32(_e256.z) * 0.0000000014629181f);
        let _e263 = (_e262 - _e259);
        phi_2284_ = _e263;
        if (abs(_e263) > 3.1415927f) {
            phi_2284_ = (_e263 - (6.2831855f * sign(_e263)));
        }
        let _e270 = phi_2284_;
        let _e271 = (_e218 + -2f);
        let _e277 = clamp(round(((abs(_e270) * 0.31830987f) * _e271)), 1f, (_e218 + -3f));
        let _e278 = (_e271 - _e277);
        if (_e215 <= _e278) {
            phi_2367_ = _e165;
            if (_e215 == _e278) {
                phi_2367_ = -(_e165);
            }
            let _e287 = phi_2367_;
            phi_2366_ = _e287;
            phi_2307_ = -(((3.1415927f * sign(_e270)) - _e270));
            phi_2300_ = _e278;
            phi_2297_ = _e215;
        } else {
            let _e289 = (_e215 == (_e278 + 1f));
            if _e289 {
                phi_2299_ = 0f;
            } else {
                phi_2299_ = (_e215 - (_e278 + 2f));
            }
            let _e293 = phi_2299_;
            phi_2366_ = select(_e165, 0f, _e289);
            phi_2307_ = _e270;
            phi_2300_ = select(_e277, 0f, _e289);
            phi_2297_ = _e293;
        }
        let _e297 = phi_2366_;
        let _e299 = phi_2307_;
        let _e301 = phi_2300_;
        let _e303 = phi_2297_;
        if (_e303 == _e301) {
            phi_2311_ = _e262;
        } else {
            phi_2311_ = (_e259 + (_e299 * (_e303 / _e301)));
        }
        let _e309 = phi_2311_;
        phi_2389_ = _e259;
        phi_2379_ = _e299;
        phi_2364_ = _e297;
        phi_2310_ = _e309;
    } else {
        phi_2389_ = f32();
        phi_2379_ = f32();
        phi_2364_ = _e165;
        phi_2310_ = (f32(_e206.z) * 0.0000000014629181f);
    }
    let _e314 = phi_2389_;
    let _e316 = phi_2379_;
    let _e318 = phi_2364_;
    let _e320 = phi_2310_;
    let _e324 = vec2<f32>(sin(_e320), -(cos(_e320)));
    let _e326 = bitcast<vec2<f32>>(_e206.xy);
    phi_2362_ = _e153;
    if (_e153 != 0f) {
        phi_2362_ = max(_e153, (1f / length((_e126 * _e324))));
    }
    let _e333 = phi_2362_;
    if (_e155 != 0f) {
        let _e337 = (_e318 * sign(determinant(_e126)));
        let _e339 = ((_e208 & 1048576u) != 0u);
        phi_2448_ = _e337;
        if _e339 {
            phi_2448_ = min(_e337, 0f);
        }
        let _e342 = phi_2448_;
        phi_2459_ = _e342;
        if ((_e208 & 524288u) != 0u) {
            phi_2459_ = max(_e342, 0f);
        }
        let _e347 = phi_2459_;
        let _e348 = (_e333 != 0f);
        if _e348 {
            phi_2451_ = _e333;
        } else {
            let _e349 = (_e126 * _e324);
            phi_2451_ = (((abs(_e349.x) + abs(_e349.y)) * (1f / dot(_e349, _e349))) * 0.5f);
        }
        let _e360 = phi_2451_;
        let _e363 = ((_e360 > _e155) && (_e333 == 0f));
        phi_2513_ = 1f;
        if _e363 {
            phi_2513_ = (_e155 / _e360);
        }
        let _e366 = phi_2513_;
        let _e367 = select(_e155, _e360, _e363);
        let _e368 = (_e367 + _e360);
        let _e369 = (_e324 * _e368);
        let _e370 = (_e347 * _e368);
        let _e377 = (((vec2<f32>(_e370, -(_e370)) + vec2(_e367)) * (0.5f / _e360)) + vec2<f32>(0.5f, 0.5f));
        let _e380 = vec4<f32>(_e377.x, _e377.y, 0f, 0f);
        phi_2534_ = _e369;
        phi_2520_ = _e380;
        if (_e209 > 201326592u) {
            let _e386 = f32((_e206.z & 65535u));
            let _e387 = (_e386 * 0.000015259022f);
            let _e391 = sqrt(max((1f - (_e387 * _e387)), 0f));
            phi_2487_ = _e391;
            if (((_e208 & 4194304u) != 0u) == _e339) {
                phi_2487_ = -(_e391);
            }
            let _e395 = phi_2487_;
            let _e400 = (mat2x2<f32>(vec2<f32>(_e387, _e395), vec2<f32>(-(_e395), _e387)) * _e324);
            let _e401 = (_e126 * _e400);
            let _e409 = ((abs(_e401.x) + abs(_e401.y)) * (1f / dot(_e401, _e401)));
            let _e410 = (_e209 == 402653184u);
            phi_1725_ = _e410;
            if !(_e410) {
                phi_1725_ = ((_e209 == 335544320u) && (_e387 >= 0.25f));
            }
            let _e416 = phi_1725_;
            if _e416 {
                phi_2492_ = (_e367 * (1f / max(_e387, select(0.25f, 1f, ((_e208 & 33554432u) != 0u)))));
            } else {
                phi_2492_ = ((_e367 * _e387) + (_e409 * 0.5f));
            }
            let _e427 = phi_2492_;
            let _e429 = (_e427 + (_e409 * 0.5f));
            phi_2503_ = _e369;
            if ((_e208 & 2097152u) != 0u) {
                if (_e368 <= ((_e429 * _e387) + (_e360 * 0.125f))) {
                    phi_2504_ = (_e400 * (_e368 * (65535f / _e386)));
                } else {
                    let _e439 = (_e400 * _e429);
                    phi_2504_ = (vec2<f32>(dot(_e369, _e369), dot(_e439, _e439)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e369, _e439)));
                }
                let _e447 = phi_2504_;
                phi_2503_ = _e447;
            }
            let _e449 = phi_2503_;
            let _e454 = ((_e429 - dot((_e449 * abs(_e347)), _e400)) / _e409);
            if _e339 {
                phi_2521_ = vec4<f32>(_e380.x, _e454, _e380.z, _e380.w);
            } else {
                phi_2521_ = vec4<f32>(_e454, _e380.y, _e380.z, _e380.w);
            }
            let _e466 = phi_2521_;
            phi_2534_ = _e449;
            phi_2520_ = _e466;
        }
        let _e468 = phi_2534_;
        let _e470 = phi_2520_;
        let _e472 = (_e470.xy * _e366);
        let _e478 = vec4<f32>(_e472.x, _e470.y, _e470.z, _e470.w);
        let _e485 = vec4<f32>(_e478.x, max(_e472.y, 0.0001f), _e478.z, _e478.w);
        phi_2568_ = _e485;
        if _e348 {
            phi_2568_ = vec4<f32>((-2f - _e472.x), _e485.y, _e485.z, _e485.w);
        }
        let _e493 = phi_2568_;
        phi_2572_ = (_e85 != 0i);
        phi_2566_ = _e493;
        phi_2560_ = (_e126 * (_e468 * _e347));
        phi_2536_ = _e326;
    } else {
        let _e497 = vec4<f32>(_e163, -1f, 0f, 0f);
        if (_e333 != 0f) {
            let _e503 = vec4<f32>(_e497.x, -2f, _e497.z, _e497.w);
            let _e508 = vec4<f32>(_e503.x, _e503.y, 1000000f, _e503.w);
            phi_2444_ = vec4<f32>(_e508.x, _e508.y, _e508.z, _e163);
            if _e212 {
                phi_2400_ = _e316;
                phi_2399_ = _e314;
                if (_e316 < 0f) {
                    phi_2400_ = -(_e316);
                    phi_2399_ = (_e314 + _e316);
                }
                let _e518 = phi_2400_;
                let _e520 = phi_2399_;
                let _e522 = ((_e320 - _e520) + 1.5707964f);
                let _e528 = clamp(((_e522 - (floor((_e522 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e518);
                phi_2401_ = _e528;
                if (_e528 > (_e518 * 0.5f)) {
                    phi_2401_ = (_e518 - _e528);
                }
                let _e533 = phi_2401_;
                let _e540 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e533), cos(_e533)) * abs(_e318))) * 0.5f);
                if (abs((_e518 - 1.5707964f)) < 0.001f) {
                    phi_2428_ = 0f;
                    phi_2426_ = 0f;
                } else {
                    let _e544 = tan(_e518);
                    let _e549 = (sign((1.5707964f - _e518)) / max(abs(_e544), 0.000001f));
                    if (_e549 >= 0f) {
                        phi_2405_ = (_e540.y - ((1f - _e540.x) * _e544));
                    } else {
                        phi_2405_ = (_e540.y + (_e540.x * _e544));
                    }
                    let _e561 = phi_2405_;
                    phi_2428_ = _e561;
                    phi_2426_ = _e549;
                }
                let _e563 = phi_2428_;
                let _e565 = phi_2426_;
                phi_2444_ = vec4<f32>((max(_e540.x, 0f) + 0.25f), (-2f - _e540.y), _e565, _e563);
            }
            let _e573 = phi_2444_;
            phi_2565_ = (_e126 * (_e324 * (_e318 * _e333)));
            phi_2443_ = _e573;
        } else {
            phi_2565_ = (sign(((_e324 * _e318) * _naga_inverse_2x2_f32(_e126))) * 0.5f);
            phi_2443_ = _e497;
        }
        let _e583 = phi_2565_;
        let _e585 = phi_2443_;
        phi_2571_ = _e585;
        if (((_e208 & 8388608u) != 0u) != ((_e208 & 16777216u) != 0u)) {
            phi_2571_ = (_e585 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e593 = phi_2571_;
        phi_2572_ = (((_e208 & 2147483648u) != 0u) && (_e85 != 1i));
        phi_2566_ = _e593;
        phi_2560_ = _e583;
        phi_2536_ = select(_e326, _e107, vec2((_e85 == 2i)));
    }
    let _e602 = phi_2572_;
    let _e604 = phi_2566_;
    let _e606 = phi_2560_;
    let _e608 = phi_2536_;
    let _e614 = j.Gi;
    let _e617 = select(_e604.xy, vec2<f32>(1f, -1f), vec2((_e614 != 0u)));
    let _e623 = vec4<f32>(_e617.x, _e604.y, _e604.z, _e604.w);
    S = vec4<f32>(_e623.x, _e617.y, _e623.z, _e623.w);
    if !(_e602) {
        let _e631 = (_e111 + 3u);
        let _e638 = textureLoad(JB, vec2<i32>(bitcast<i32>((_e631 & 255u)), bitcast<i32>((_e631 >> bitcast<u32>(8i)))), 0i);
        let _e640 = bitcast<vec3<f32>>(_e638.xyz);
        let _e644 = (((((_e126 * _e608) + _e606) + bitcast<vec2<f32>>(_e134.xy)) * _e640.x) + _e640.yz);
        let _e647 = j.Qe[0u];
        let _e650 = j.Qe[1u];
        phi_2604_ = vec4<f32>(((_e644.x * _e647) - 1f), ((_e644.y * _e650) - sign(_e650)), 0f, 1f);
    } else {
        let _e660 = j.j3_;
        phi_2604_ = vec4(_e660);
    }
    let _e663 = phi_2604_;
    unnamed.gl_Position = _e663;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) YB: vec4<f32>, @location(1) ZB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    YB_1 = YB;
    ZB_1 = ZB;
    main_1();
    let _e13 = S;
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
