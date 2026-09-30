enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct BC {
    sc: f32,
    Cd: f32,
    Gf: f32,
    Hf: f32,
    o6_: u32,
    Pb: u32,
    sf: u32,
    tf: u32,
    V7_: vec4<i32>,
    dh: vec2<f32>,
    Dd: vec2<f32>,
    d2_: u32,
    hh: f32,
    d6_: u32,
    T2_: f32,
    Ed: f32,
    nf: u32,
    C3_: f32,
    D3_: f32,
    Fd: f32,
    ah: u32,
    Ob: u32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override Dh: bool = true;
@id(2) override Fh: bool = true;
@id(1) override Eh: bool = true;
@id(8) override Lh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
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
var<private> B2_: vec3<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var fa: sampler;

fn main_1() {
    var phi_2371_: f32;
    var phi_2343_: i32;
    var phi_1531_: bool;
    var phi_2356_: i32;
    var phi_2348_: vec4<u32>;
    var phi_2355_: i32;
    var phi_2347_: vec4<u32>;
    var phi_2354_: i32;
    var phi_2352_: vec4<u32>;
    var phi_2351_: u32;
    var phi_2358_: vec2<i32>;
    var phi_2359_: vec4<u32>;
    var phi_2363_: f32;
    var phi_2434_: f32;
    var phi_2377_: f32;
    var phi_2433_: f32;
    var phi_2381_: f32;
    var phi_2378_: f32;
    var phi_2375_: f32;
    var phi_2385_: f32;
    var phi_2431_: f32;
    var phi_2384_: f32;
    var phi_2440_: f32;
    var phi_2437_: f32;
    var phi_2494_: f32;
    var phi_2466_: i32;
    var phi_2476_: f32;
    var phi_1843_: bool;
    var phi_2483_: f32;
    var phi_2504_: vec2<f32>;
    var phi_2503_: vec2<f32>;
    var phi_2502_: vec2<f32>;
    var phi_2520_: vec2<f32>;
    var phi_2505_: vec2<f32>;
    var phi_2553_: u32;
    var phi_2524_: vec2<f32>;
    var phi_2523_: bool;
    var local: u32;
    var local_1: u32;
    var phi_2582_: u32;
    var phi_2583_: f32;
    var phi_2584_: f32;
    var local_2: u32;
    var phi_2625_: vec4<f32>;
    var phi_2624_: f32;
    var local_3: u32;
    var phi_2622_: vec4<f32>;
    var phi_2623_: vec4<f32>;
    var phi_1212_: bool;
    var local_4: u32;
    var phi_2641_: vec4<f32>;

    let _e82 = gl_InstanceIndex_1;
    let _e83 = UB_1;
    let _e84 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e87 = i32(_e83.x);
            let _e90 = bitcast<i32>(_e83.w);
            let _e92 = (_e90 >> bitcast<u32>(2i));
            let _e93 = (_e90 & 3i);
            let _e95 = min(_e87, (_e92 - 1i));
            let _e97 = ((_e82 * _e92) + _e95);
            let _e102 = textureLoad(KC, vec2<i32>((_e97 & 2047i), (_e97 >> bitcast<u32>(11i))), 0i);
            let _e106 = (max((_e102.w & 65535u), 1u) - 1u);
            let _e113 = textureLoad(ID, vec2<i32>(bitcast<i32>((_e106 & 255u)), bitcast<i32>((_e106 >> bitcast<u32>(8i)))), 0i);
            let _e115 = bitcast<vec2<f32>>(_e113.xy);
            let _e117 = (_e113.z & 65535u);
            let _e119 = (_e117 * 4u);
            let _e126 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e119 & 255u)), bitcast<i32>((_e119 >> bitcast<u32>(8i)))), 0i);
            let _e127 = bitcast<vec4<f32>>(_e126);
            let _e134 = mat2x2<f32>(vec2<f32>(_e127.x, _e127.y), vec2<f32>(_e127.z, _e127.w));
            let _e135 = (_e119 + 1u);
            let _e142 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e135 & 255u)), bitcast<i32>((_e135 >> bitcast<u32>(8i)))), 0i);
            let _e146 = bitcast<f32>(_e142.z);
            let _e148 = bitcast<f32>(_e142.w);
            let _e149 = (_e102.w & 8388608u);
            phi_2371_ = _e83.y;
            phi_2343_ = _e87;
            local = _e113.z;
            local_1 = _e117;
            local_2 = _e117;
            local_3 = _e117;
            local_4 = _e117;
            if (_e149 != 0u) {
                phi_2371_ = _e84.y;
                phi_2343_ = i32(_e84.x);
            }
            let _e155 = phi_2371_;
            let _e157 = phi_2343_;
            phi_2354_ = _e97;
            phi_2352_ = _e102;
            phi_2351_ = _e102.w;
            if (_e157 != _e95) {
                let _e160 = ((_e97 + _e157) - _e95);
                let _e165 = textureLoad(KC, vec2<i32>((_e160 & 2047i), (_e160 >> bitcast<u32>(11i))), 0i);
                if ((_e165.w & 8454143u) != (_e102.w & 8454143u)) {
                    let _e170 = (_e146 == 0f);
                    phi_1531_ = _e170;
                    if !(_e170) {
                        phi_1531_ = (_e115.x != 0f);
                    }
                    let _e175 = phi_1531_;
                    phi_2356_ = _e97;
                    phi_2348_ = _e102;
                    if _e175 {
                        let _e176 = bitcast<i32>(_e113.w);
                        let _e181 = textureLoad(KC, vec2<i32>((_e176 & 2047i), (_e176 >> bitcast<u32>(11i))), 0i);
                        phi_2356_ = _e176;
                        phi_2348_ = _e181;
                    }
                    let _e183 = phi_2356_;
                    let _e185 = phi_2348_;
                    phi_2355_ = _e183;
                    phi_2347_ = _e185;
                } else {
                    phi_2355_ = _e160;
                    phi_2347_ = _e165;
                }
                let _e187 = phi_2355_;
                let _e189 = phi_2347_;
                phi_2354_ = _e187;
                phi_2352_ = _e189;
                phi_2351_ = ((_e189.w & 4286578687u) | _e149);
            }
            let _e194 = phi_2354_;
            let _e196 = phi_2352_;
            let _e198 = phi_2351_;
            let _e199 = (_e198 & 469762048u);
            if ((_e199 == 67108864u) && (_e93 == 0i)) {
                let _e205 = f32((_e196.z & 65535u));
                let _e208 = f32((_e196.z >> bitcast<u32>(16i)));
                let _e214 = vec2<i32>(i32((-1f - _e205)), i32(((_e208 - _e205) + 1f)));
                phi_2358_ = _e214;
                if ((_e198 & 8388608u) != 0u) {
                    phi_2358_ = -(_e214);
                }
                let _e219 = phi_2358_;
                let _e221 = (_e194 + _e219.x);
                let _e226 = textureLoad(KC, vec2<i32>((_e221 & 2047i), (_e221 >> bitcast<u32>(11i))), 0i);
                let _e228 = (_e194 + _e219.y);
                let _e233 = textureLoad(KC, vec2<i32>((_e228 & 2047i), (_e228 >> bitcast<u32>(11i))), 0i);
                phi_2359_ = _e233;
                if ((_e233.w & 8454143u) != (_e226.w & 8454143u)) {
                    let _e239 = bitcast<i32>(_e113.w);
                    let _e244 = textureLoad(KC, vec2<i32>((_e239 & 2047i), (_e239 >> bitcast<u32>(11i))), 0i);
                    phi_2359_ = _e244;
                }
                let _e246 = phi_2359_;
                let _e248 = bitcast<f32>(_e226.z);
                let _e250 = bitcast<f32>(_e246.z);
                let _e251 = (_e250 - _e248);
                phi_2363_ = _e251;
                if (abs(_e251) > 3.1415927f) {
                    phi_2363_ = (_e251 - (6.2831855f * sign(_e251)));
                }
                let _e258 = phi_2363_;
                let _e259 = (_e208 + -2f);
                let _e265 = clamp(round(((abs(_e258) * 0.31830987f) * _e259)), 1f, (_e208 + -3f));
                let _e266 = (_e259 - _e265);
                if (_e205 <= _e266) {
                    phi_2434_ = _e155;
                    if (_e205 == _e266) {
                        phi_2434_ = -(_e155);
                    }
                    let _e275 = phi_2434_;
                    phi_2433_ = _e275;
                    phi_2381_ = -(((3.1415927f * sign(_e258)) - _e258));
                    phi_2378_ = _e266;
                    phi_2375_ = _e205;
                } else {
                    let _e277 = (_e205 == (_e266 + 1f));
                    if _e277 {
                        phi_2377_ = 0f;
                    } else {
                        phi_2377_ = (_e205 - (_e266 + 2f));
                    }
                    let _e281 = phi_2377_;
                    phi_2433_ = select(_e155, 0f, _e277);
                    phi_2381_ = _e258;
                    phi_2378_ = select(_e265, 0f, _e277);
                    phi_2375_ = _e281;
                }
                let _e285 = phi_2433_;
                let _e287 = phi_2381_;
                let _e289 = phi_2378_;
                let _e291 = phi_2375_;
                if (_e291 == _e289) {
                    phi_2385_ = _e250;
                } else {
                    phi_2385_ = (_e248 + (_e287 * (_e291 / _e289)));
                }
                let _e297 = phi_2385_;
                phi_2431_ = _e285;
                phi_2384_ = _e297;
            } else {
                phi_2431_ = _e155;
                phi_2384_ = bitcast<f32>(_e196.z);
            }
            let _e301 = phi_2431_;
            let _e303 = phi_2384_;
            let _e307 = vec2<f32>(sin(_e303), -(cos(_e303)));
            let _e309 = bitcast<vec2<f32>>(_e196.xy);
            phi_2440_ = _e148;
            if (_e148 != 0f) {
                phi_2440_ = max(_e148, (1f / length((_e134 * _e307))));
            }
            let _e316 = phi_2440_;
            if (_e146 != 0f) {
                let _e320 = (_e301 * sign(determinant(_e134)));
                let _e322 = ((_e198 & 1048576u) != 0u);
                phi_2437_ = _e320;
                if _e322 {
                    phi_2437_ = min(_e320, 0f);
                }
                let _e325 = phi_2437_;
                phi_2494_ = _e325;
                if ((_e198 & 524288u) != 0u) {
                    phi_2494_ = max(_e325, 0f);
                }
                let _e330 = phi_2494_;
                let _e332 = select(0f, _e316, (_e316 != 0f));
                let _e336 = select(_e146, _e332, ((_e332 > _e146) && (_e316 == 0f)));
                let _e337 = (_e336 + _e332);
                let _e338 = (_e307 * _e337);
                phi_2502_ = _e338;
                if (_e199 > 134217728u) {
                    let _e340 = (_e198 & 4194304u);
                    let _e342 = select(2i, -2i, (_e340 == 0u));
                    phi_2466_ = _e342;
                    if ((_e198 & 8388608u) != 0u) {
                        phi_2466_ = -(_e342);
                    }
                    let _e347 = phi_2466_;
                    let _e348 = (_e194 + _e347);
                    let _e353 = textureLoad(KC, vec2<i32>((_e348 & 2047i), (_e348 >> bitcast<u32>(11i))), 0i);
                    let _e357 = abs((bitcast<f32>(_e353.z) - _e303));
                    phi_2476_ = _e357;
                    if (_e357 > 3.1415927f) {
                        phi_2476_ = (6.2831855f - _e357);
                    }
                    let _e361 = phi_2476_;
                    let _e366 = ((_e361 * select(0.5f, -0.5f, ((_e340 != 0u) == _e322))) + _e303);
                    let _e370 = vec2<f32>(sin(_e366), -(cos(_e366)));
                    let _e371 = (_e134 * _e370);
                    let _e381 = cos((_e361 * 0.5f));
                    let _e382 = (_e199 == 335544320u);
                    phi_1843_ = _e382;
                    if !(_e382) {
                        phi_1843_ = ((_e199 == 268435456u) && (_e381 >= 0.25f));
                    }
                    let _e388 = phi_1843_;
                    if _e388 {
                        phi_2483_ = (_e336 * (1f / max(_e381, select(0.25f, 1f, ((_e198 & 33554432u) != 0u)))));
                    } else {
                        phi_2483_ = ((_e336 * _e381) + (((abs(_e371.x) + abs(_e371.y)) * (1f / dot(_e371, _e371))) * 0.5f));
                    }
                    let _e399 = phi_2483_;
                    phi_2503_ = _e338;
                    if ((_e198 & 2097152u) != 0u) {
                        if (_e337 <= ((_e399 * _e381) + (_e332 * 0.125f))) {
                            phi_2504_ = (_e370 * (_e337 * (1f / _e381)));
                        } else {
                            let _e409 = (_e370 * _e399);
                            phi_2504_ = (vec2<f32>(dot(_e338, _e338), dot(_e409, _e409)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e338, _e409)));
                        }
                        let _e417 = phi_2504_;
                        phi_2503_ = _e417;
                    }
                    let _e419 = phi_2503_;
                    phi_2502_ = _e419;
                }
                let _e421 = phi_2502_;
                if (_e93 != 0i) {
                    phi_2553_ = u32();
                    phi_2524_ = vec2<f32>();
                    phi_2523_ = false;
                    break;
                }
                phi_2520_ = (_e134 * (_e421 * _e330));
                phi_2505_ = _e309;
            } else {
                if (((_e198 & 2147483648u) != 0u) && (_e93 != 1i)) {
                    phi_2553_ = u32();
                    phi_2524_ = vec2<f32>();
                    phi_2523_ = false;
                    break;
                }
                phi_2520_ = vec2<f32>(0f, 0f);
                phi_2505_ = select(_e309, _e115, vec2((_e93 == 2i)));
            }
            let _e433 = phi_2520_;
            let _e435 = phi_2505_;
            let _e439 = (_e119 + 2u);
            let _e446 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e439 & 255u)), bitcast<i32>((_e439 >> bitcast<u32>(8i)))), 0i);
            phi_2553_ = _e446.x;
            phi_2524_ = (((_e134 * _e435) + _e433) + bitcast<vec2<f32>>(_e142.xy));
            phi_2523_ = true;
            break;
        }
    }
    let _e449 = phi_2553_;
    let _e451 = phi_2524_;
    let _e453 = phi_2523_;
    let _e455 = local;
    let _e459 = local_1;
    let _e464 = textureLoad(DD, vec2<i32>(bitcast<i32>((_e455 & 255u)), bitcast<i32>((_e459 >> bitcast<u32>(8i)))), 0i);
    let _e466 = (_e464.x & 15u);
    if Dh {
        let _e467 = (_e466 == 0u);
        if _e467 {
            phi_2582_ = _e464.y;
        } else {
            phi_2582_ = _e464.x;
        }
        let _e470 = phi_2582_;
        let _e472 = (_e470 >> bitcast<u32>(16i));
        let _e474 = l.d6_;
        if (_e472 == 0u) {
            phi_2583_ = 0f;
        } else {
            phi_2583_ = unpack2x16float(((_e472 + 1023u) * _e474)).x;
        }
        let _e481 = phi_2583_;
        phi_2584_ = _e481;
        if _e467 {
            phi_2584_ = -(_e481);
        }
        let _e484 = phi_2584_;
        W1_[0u] = _e484;
    }
    if Fh {
        g2_ = f32(((_e464.x >> bitcast<u32>(4i)) & 15u));
    }
    if Eh {
        let _e491 = local_2;
        let _e492 = (_e491 * 8u);
        let _e493 = (_e492 + 2u);
        let _e500 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e493 & 255u)), bitcast<i32>((_e493 >> bitcast<u32>(8i)))), 0i);
        let _e508 = (_e492 + 3u);
        let _e515 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e508 & 255u)), bitcast<i32>((_e508 >> bitcast<u32>(8i)))), 0i);
        if any((_e500 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e520 = ((mat2x2<f32>(vec2<f32>(_e500.x, _e500.y), vec2<f32>(_e500.z, _e500.w)) * _e451) + _e515.xy);
            unnamed.gl_ClipDistance[0i] = (_e520.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e520.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e520.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e520.y);
        } else {
            let _e536 = (_e515.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e536;
            unnamed.gl_ClipDistance[2i] = _e536;
            unnamed.gl_ClipDistance[1i] = _e536;
            unnamed.gl_ClipDistance[0i] = _e536;
        }
    }
    if (_e466 == 1u) {
        let _e547 = unpack4x8unorm(_e464.y);
        if Fh {
            phi_2625_ = _e547;
        } else {
            let _e550 = (_e547.xyz * _e547.w);
            let _e556 = vec4<f32>(_e550.x, _e547.y, _e547.z, _e547.w);
            let _e562 = vec4<f32>(_e556.x, _e550.y, _e556.z, _e556.w);
            phi_2625_ = vec4<f32>(_e562.x, _e562.y, _e550.z, _e562.w);
        }
        let _e570 = phi_2625_;
        V1_ = _e570;
    } else {
        if (Dh && (_e466 == 0u)) {
            let _e574 = (_e464.x >> bitcast<u32>(16i));
            let _e576 = l.d6_;
            if (_e574 == 0u) {
                phi_2624_ = 0f;
            } else {
                phi_2624_ = unpack2x16float(((_e574 + 1023u) * _e576)).x;
            }
            let _e583 = phi_2624_;
            W1_[1u] = _e583;
        } else {
            let _e586 = local_3;
            let _e587 = (_e586 * 8u);
            let _e594 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e587 & 255u)), bitcast<i32>((_e587 >> bitcast<u32>(8i)))), 0i);
            let _e602 = (_e587 + 1u);
            let _e609 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e602 & 255u)), bitcast<i32>((_e602 >> bitcast<u32>(8i)))), 0i);
            let _e618 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e464.y));
            let _e620 = ((mat2x2<f32>(vec2<f32>(_e594.x, _e594.y), vec2<f32>(_e594.z, _e594.w)) * _e451) + _e609.xy);
            if (_e609.z > 0.9f) {
                phi_2622_ = vec4<f32>(_e618.x, _e618.y, 2f, _e618.w);
            } else {
                phi_2622_ = vec4<f32>(_e618.x, _e618.y, _e609.w, _e618.w);
            }
            let _e635 = phi_2622_;
            if (f32(_e466) == 2f) {
                let _e642 = vec4<f32>(_e620.x, _e635.y, _e635.z, _e635.w);
                phi_2623_ = vec4<f32>(_e642.x, 0f, _e642.z, _e642.w);
            } else {
                let _e654 = vec4<f32>(_e635.x, _e635.y, -(_e635.z), _e635.w);
                let _e660 = vec4<f32>(_e620.x, _e654.y, _e654.z, _e654.w);
                phi_2623_ = vec4<f32>(_e660.x, _e620.y, _e660.z, _e660.w);
            }
            let _e668 = phi_2623_;
            V1_ = _e668;
            let _e670 = V1_[3u];
            V1_[3u] = -(_e670);
        }
    }
    phi_1212_ = Lh;
    if Lh {
        phi_1212_ = ((_e464.x & 2048u) != 0u);
    }
    let _e675 = phi_1212_;
    if _e675 {
        let _e677 = local_4;
        let _e678 = (_e677 * 8u);
        let _e679 = (_e678 + 4u);
        let _e686 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e679 & 255u)), bitcast<i32>((_e679 >> bitcast<u32>(8i)))), 0i);
        let _e694 = (_e678 + 5u);
        let _e701 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e694 & 255u)), bitcast<i32>((_e694 >> bitcast<u32>(8i)))), 0i);
        let _e704 = ((mat2x2<f32>(vec2<f32>(_e686.x, _e686.y), vec2<f32>(_e686.z, _e686.w)) * _e451) + _e701.xy);
        B2_ = vec3<f32>(_e704.x, _e704.y, (1f + _e701.z));
    } else {
        B2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e453 {
        let _e711 = l.Gf;
        let _e713 = l.Hf;
        let _e721 = vec4<f32>(((_e451.x * _e711) - 1f), ((_e451.y * _e713) - sign(_e713)), 0f, 1f);
        phi_2641_ = vec4<f32>(_e721.x, _e721.y, (1f - (f32(_e449) * 0.000061035156f)), _e721.w);
    } else {
        let _e731 = l.T2_;
        phi_2641_ = vec4(_e731);
    }
    let _e734 = phi_2641_;
    unnamed.gl_Position = _e734;
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
    let _e22 = B2_;
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
