enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
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
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ga: sampler;

fn main_1() {
    var phi_2348_: f32;
    var phi_2320_: i32;
    var phi_1515_: bool;
    var phi_2333_: i32;
    var phi_2325_: vec4<u32>;
    var phi_2332_: i32;
    var phi_2324_: vec4<u32>;
    var phi_2331_: i32;
    var phi_2329_: vec4<u32>;
    var phi_2328_: u32;
    var phi_2335_: vec2<i32>;
    var phi_2336_: vec4<u32>;
    var phi_2340_: f32;
    var phi_2411_: f32;
    var phi_2354_: f32;
    var phi_2410_: f32;
    var phi_2358_: f32;
    var phi_2355_: f32;
    var phi_2352_: f32;
    var phi_2362_: f32;
    var phi_2408_: f32;
    var phi_2361_: f32;
    var phi_2417_: f32;
    var phi_2414_: f32;
    var phi_2471_: f32;
    var phi_2443_: i32;
    var phi_2453_: f32;
    var phi_1827_: bool;
    var phi_2460_: f32;
    var phi_2481_: vec2<f32>;
    var phi_2480_: vec2<f32>;
    var phi_2479_: vec2<f32>;
    var phi_2497_: vec2<f32>;
    var phi_2482_: vec2<f32>;
    var phi_2530_: u32;
    var phi_2501_: vec2<f32>;
    var phi_2500_: bool;
    var local: u32;
    var local_1: u32;
    var phi_2559_: u32;
    var phi_2560_: f32;
    var phi_2561_: f32;
    var local_2: u32;
    var phi_2601_: f32;
    var local_3: u32;
    var phi_2599_: vec4<f32>;
    var phi_2600_: vec4<f32>;
    var phi_1195_: bool;
    var local_4: u32;
    var phi_2616_: vec4<f32>;

    let _e82 = gl_InstanceIndex_1;
    let _e83 = TB_1;
    let _e84 = UB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e87 = i32(_e83.x);
            let _e90 = bitcast<i32>(_e83.w);
            let _e92 = (_e90 >> bitcast<u32>(2i));
            let _e93 = (_e90 & 3i);
            let _e95 = min(_e87, (_e92 - 1i));
            let _e97 = ((_e82 * _e92) + _e95);
            let _e102 = textureLoad(JC, vec2<i32>((_e97 & 2047i), (_e97 >> bitcast<u32>(11i))), 0i);
            let _e106 = (max((_e102.w & 65535u), 1u) - 1u);
            let _e113 = textureLoad(HD, vec2<i32>(bitcast<i32>((_e106 & 255u)), bitcast<i32>((_e106 >> bitcast<u32>(8i)))), 0i);
            let _e115 = bitcast<vec2<f32>>(_e113.xy);
            let _e117 = (_e113.z & 65535u);
            let _e119 = (_e117 * 4u);
            let _e126 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e119 & 255u)), bitcast<i32>((_e119 >> bitcast<u32>(8i)))), 0i);
            let _e127 = bitcast<vec4<f32>>(_e126);
            let _e134 = mat2x2<f32>(vec2<f32>(_e127.x, _e127.y), vec2<f32>(_e127.z, _e127.w));
            let _e135 = (_e119 + 1u);
            let _e142 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e135 & 255u)), bitcast<i32>((_e135 >> bitcast<u32>(8i)))), 0i);
            let _e146 = bitcast<f32>(_e142.z);
            let _e148 = bitcast<f32>(_e142.w);
            let _e149 = (_e102.w & 8388608u);
            phi_2348_ = _e83.y;
            phi_2320_ = _e87;
            local = _e113.z;
            local_1 = _e117;
            local_2 = _e117;
            local_3 = _e117;
            local_4 = _e117;
            if (_e149 != 0u) {
                phi_2348_ = _e84.y;
                phi_2320_ = i32(_e84.x);
            }
            let _e155 = phi_2348_;
            let _e157 = phi_2320_;
            phi_2331_ = _e97;
            phi_2329_ = _e102;
            phi_2328_ = _e102.w;
            if (_e157 != _e95) {
                let _e160 = ((_e97 + _e157) - _e95);
                let _e165 = textureLoad(JC, vec2<i32>((_e160 & 2047i), (_e160 >> bitcast<u32>(11i))), 0i);
                if ((_e165.w & 8454143u) != (_e102.w & 8454143u)) {
                    let _e170 = (_e146 == 0f);
                    phi_1515_ = _e170;
                    if !(_e170) {
                        phi_1515_ = (_e115.x != 0f);
                    }
                    let _e175 = phi_1515_;
                    phi_2333_ = _e97;
                    phi_2325_ = _e102;
                    if _e175 {
                        let _e176 = bitcast<i32>(_e113.w);
                        let _e181 = textureLoad(JC, vec2<i32>((_e176 & 2047i), (_e176 >> bitcast<u32>(11i))), 0i);
                        phi_2333_ = _e176;
                        phi_2325_ = _e181;
                    }
                    let _e183 = phi_2333_;
                    let _e185 = phi_2325_;
                    phi_2332_ = _e183;
                    phi_2324_ = _e185;
                } else {
                    phi_2332_ = _e160;
                    phi_2324_ = _e165;
                }
                let _e187 = phi_2332_;
                let _e189 = phi_2324_;
                phi_2331_ = _e187;
                phi_2329_ = _e189;
                phi_2328_ = ((_e189.w & 4286578687u) | _e149);
            }
            let _e194 = phi_2331_;
            let _e196 = phi_2329_;
            let _e198 = phi_2328_;
            let _e199 = (_e198 & 469762048u);
            if ((_e199 == 67108864u) && (_e93 == 0i)) {
                let _e205 = f32((_e196.z & 65535u));
                let _e208 = f32((_e196.z >> bitcast<u32>(16i)));
                let _e214 = vec2<i32>(i32((-1f - _e205)), i32(((_e208 - _e205) + 1f)));
                phi_2335_ = _e214;
                if ((_e198 & 8388608u) != 0u) {
                    phi_2335_ = -(_e214);
                }
                let _e219 = phi_2335_;
                let _e221 = (_e194 + _e219.x);
                let _e226 = textureLoad(JC, vec2<i32>((_e221 & 2047i), (_e221 >> bitcast<u32>(11i))), 0i);
                let _e228 = (_e194 + _e219.y);
                let _e233 = textureLoad(JC, vec2<i32>((_e228 & 2047i), (_e228 >> bitcast<u32>(11i))), 0i);
                phi_2336_ = _e233;
                if ((_e233.w & 8454143u) != (_e226.w & 8454143u)) {
                    let _e239 = bitcast<i32>(_e113.w);
                    let _e244 = textureLoad(JC, vec2<i32>((_e239 & 2047i), (_e239 >> bitcast<u32>(11i))), 0i);
                    phi_2336_ = _e244;
                }
                let _e246 = phi_2336_;
                let _e248 = bitcast<f32>(_e226.z);
                let _e250 = bitcast<f32>(_e246.z);
                let _e251 = (_e250 - _e248);
                phi_2340_ = _e251;
                if (abs(_e251) > 3.1415927f) {
                    phi_2340_ = (_e251 - (6.2831855f * sign(_e251)));
                }
                let _e258 = phi_2340_;
                let _e259 = (_e208 + -2f);
                let _e265 = clamp(round(((abs(_e258) * 0.31830987f) * _e259)), 1f, (_e208 + -3f));
                let _e266 = (_e259 - _e265);
                if (_e205 <= _e266) {
                    phi_2411_ = _e155;
                    if (_e205 == _e266) {
                        phi_2411_ = -(_e155);
                    }
                    let _e275 = phi_2411_;
                    phi_2410_ = _e275;
                    phi_2358_ = -(((3.1415927f * sign(_e258)) - _e258));
                    phi_2355_ = _e266;
                    phi_2352_ = _e205;
                } else {
                    let _e277 = (_e205 == (_e266 + 1f));
                    if _e277 {
                        phi_2354_ = 0f;
                    } else {
                        phi_2354_ = (_e205 - (_e266 + 2f));
                    }
                    let _e281 = phi_2354_;
                    phi_2410_ = select(_e155, 0f, _e277);
                    phi_2358_ = _e258;
                    phi_2355_ = select(_e265, 0f, _e277);
                    phi_2352_ = _e281;
                }
                let _e285 = phi_2410_;
                let _e287 = phi_2358_;
                let _e289 = phi_2355_;
                let _e291 = phi_2352_;
                if (_e291 == _e289) {
                    phi_2362_ = _e250;
                } else {
                    phi_2362_ = (_e248 + (_e287 * (_e291 / _e289)));
                }
                let _e297 = phi_2362_;
                phi_2408_ = _e285;
                phi_2361_ = _e297;
            } else {
                phi_2408_ = _e155;
                phi_2361_ = bitcast<f32>(_e196.z);
            }
            let _e301 = phi_2408_;
            let _e303 = phi_2361_;
            let _e307 = vec2<f32>(sin(_e303), -(cos(_e303)));
            let _e309 = bitcast<vec2<f32>>(_e196.xy);
            phi_2417_ = _e148;
            if (_e148 != 0f) {
                phi_2417_ = max(_e148, (1f / length((_e134 * _e307))));
            }
            let _e316 = phi_2417_;
            if (_e146 != 0f) {
                let _e320 = (_e301 * sign(determinant(_e134)));
                let _e322 = ((_e198 & 1048576u) != 0u);
                phi_2414_ = _e320;
                if _e322 {
                    phi_2414_ = min(_e320, 0f);
                }
                let _e325 = phi_2414_;
                phi_2471_ = _e325;
                if ((_e198 & 524288u) != 0u) {
                    phi_2471_ = max(_e325, 0f);
                }
                let _e330 = phi_2471_;
                let _e332 = select(0f, _e316, (_e316 != 0f));
                let _e336 = select(_e146, _e332, ((_e332 > _e146) && (_e316 == 0f)));
                let _e337 = (_e336 + _e332);
                let _e338 = (_e307 * _e337);
                phi_2479_ = _e338;
                if (_e199 > 134217728u) {
                    let _e340 = (_e198 & 4194304u);
                    let _e342 = select(2i, -2i, (_e340 == 0u));
                    phi_2443_ = _e342;
                    if ((_e198 & 8388608u) != 0u) {
                        phi_2443_ = -(_e342);
                    }
                    let _e347 = phi_2443_;
                    let _e348 = (_e194 + _e347);
                    let _e353 = textureLoad(JC, vec2<i32>((_e348 & 2047i), (_e348 >> bitcast<u32>(11i))), 0i);
                    let _e357 = abs((bitcast<f32>(_e353.z) - _e303));
                    phi_2453_ = _e357;
                    if (_e357 > 3.1415927f) {
                        phi_2453_ = (6.2831855f - _e357);
                    }
                    let _e361 = phi_2453_;
                    let _e366 = ((_e361 * select(0.5f, -0.5f, ((_e340 != 0u) == _e322))) + _e303);
                    let _e370 = vec2<f32>(sin(_e366), -(cos(_e366)));
                    let _e371 = (_e134 * _e370);
                    let _e381 = cos((_e361 * 0.5f));
                    let _e382 = (_e199 == 335544320u);
                    phi_1827_ = _e382;
                    if !(_e382) {
                        phi_1827_ = ((_e199 == 268435456u) && (_e381 >= 0.25f));
                    }
                    let _e388 = phi_1827_;
                    if _e388 {
                        phi_2460_ = (_e336 * (1f / max(_e381, select(0.25f, 1f, ((_e198 & 33554432u) != 0u)))));
                    } else {
                        phi_2460_ = ((_e336 * _e381) + (((abs(_e371.x) + abs(_e371.y)) * (1f / dot(_e371, _e371))) * 0.5f));
                    }
                    let _e399 = phi_2460_;
                    phi_2480_ = _e338;
                    if ((_e198 & 2097152u) != 0u) {
                        if (_e337 <= ((_e399 * _e381) + (_e332 * 0.125f))) {
                            phi_2481_ = (_e370 * (_e337 * (1f / _e381)));
                        } else {
                            let _e409 = (_e370 * _e399);
                            phi_2481_ = (vec2<f32>(dot(_e338, _e338), dot(_e409, _e409)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e338, _e409)));
                        }
                        let _e417 = phi_2481_;
                        phi_2480_ = _e417;
                    }
                    let _e419 = phi_2480_;
                    phi_2479_ = _e419;
                }
                let _e421 = phi_2479_;
                if (_e93 != 0i) {
                    phi_2530_ = u32();
                    phi_2501_ = vec2<f32>();
                    phi_2500_ = false;
                    break;
                }
                phi_2497_ = (_e134 * (_e421 * _e330));
                phi_2482_ = _e309;
            } else {
                if (((_e198 & 2147483648u) != 0u) && (_e93 != 1i)) {
                    phi_2530_ = u32();
                    phi_2501_ = vec2<f32>();
                    phi_2500_ = false;
                    break;
                }
                phi_2497_ = vec2<f32>(0f, 0f);
                phi_2482_ = select(_e309, _e115, vec2((_e93 == 2i)));
            }
            let _e433 = phi_2497_;
            let _e435 = phi_2482_;
            let _e439 = (_e119 + 2u);
            let _e446 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e439 & 255u)), bitcast<i32>((_e439 >> bitcast<u32>(8i)))), 0i);
            phi_2530_ = _e446.x;
            phi_2501_ = (((_e134 * _e435) + _e433) + bitcast<vec2<f32>>(_e142.xy));
            phi_2500_ = true;
            break;
        }
    }
    let _e449 = phi_2530_;
    let _e451 = phi_2501_;
    let _e453 = phi_2500_;
    let _e455 = local;
    let _e459 = local_1;
    let _e464 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e455 & 255u)), bitcast<i32>((_e459 >> bitcast<u32>(8i)))), 0i);
    let _e466 = (_e464.x & 15u);
    if Eh {
        let _e467 = (_e466 == 0u);
        if _e467 {
            phi_2559_ = _e464.y;
        } else {
            phi_2559_ = _e464.x;
        }
        let _e470 = phi_2559_;
        let _e472 = (_e470 >> bitcast<u32>(16i));
        let _e474 = j.f6_;
        if (_e472 == 0u) {
            phi_2560_ = 0f;
        } else {
            phi_2560_ = unpack2x16float(((_e472 + 1023u) * _e474)).x;
        }
        let _e481 = phi_2560_;
        phi_2561_ = _e481;
        if _e467 {
            phi_2561_ = -(_e481);
        }
        let _e484 = phi_2561_;
        Y1_[0u] = _e484;
    }
    if Gh {
        g1_ = f32(((_e464.x >> bitcast<u32>(4i)) & 15u));
    }
    if Fh {
        let _e491 = local_2;
        let _e492 = (_e491 * 8u);
        let _e493 = (_e492 + 2u);
        let _e500 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e493 & 255u)), bitcast<i32>((_e493 >> bitcast<u32>(8i)))), 0i);
        let _e508 = (_e492 + 3u);
        let _e515 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e508 & 255u)), bitcast<i32>((_e508 >> bitcast<u32>(8i)))), 0i);
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
        X1_ = unpack4x8unorm(_e464.y);
    } else {
        if (Eh && (_e466 == 0u)) {
            let _e551 = (_e464.x >> bitcast<u32>(16i));
            let _e553 = j.f6_;
            if (_e551 == 0u) {
                phi_2601_ = 0f;
            } else {
                phi_2601_ = unpack2x16float(((_e551 + 1023u) * _e553)).x;
            }
            let _e560 = phi_2601_;
            Y1_[1u] = _e560;
        } else {
            let _e563 = local_3;
            let _e564 = (_e563 * 8u);
            let _e571 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e564 & 255u)), bitcast<i32>((_e564 >> bitcast<u32>(8i)))), 0i);
            let _e579 = (_e564 + 1u);
            let _e586 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e579 & 255u)), bitcast<i32>((_e579 >> bitcast<u32>(8i)))), 0i);
            let _e595 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e464.y));
            let _e597 = ((mat2x2<f32>(vec2<f32>(_e571.x, _e571.y), vec2<f32>(_e571.z, _e571.w)) * _e451) + _e586.xy);
            if (_e586.z > 0.9f) {
                phi_2599_ = vec4<f32>(_e595.x, _e595.y, 2f, _e595.w);
            } else {
                phi_2599_ = vec4<f32>(_e595.x, _e595.y, _e586.w, _e595.w);
            }
            let _e612 = phi_2599_;
            if (f32(_e466) == 2f) {
                let _e619 = vec4<f32>(_e597.x, _e612.y, _e612.z, _e612.w);
                phi_2600_ = vec4<f32>(_e619.x, 0f, _e619.z, _e619.w);
            } else {
                let _e631 = vec4<f32>(_e612.x, _e612.y, -(_e612.z), _e612.w);
                let _e637 = vec4<f32>(_e597.x, _e631.y, _e631.z, _e631.w);
                phi_2600_ = vec4<f32>(_e637.x, _e597.y, _e637.z, _e637.w);
            }
            let _e645 = phi_2600_;
            X1_ = _e645;
            let _e647 = X1_[3u];
            X1_[3u] = -(_e647);
        }
    }
    phi_1195_ = Mh;
    if Mh {
        phi_1195_ = ((_e464.x & 2048u) != 0u);
    }
    let _e652 = phi_1195_;
    if _e652 {
        let _e654 = local_4;
        let _e655 = (_e654 * 8u);
        let _e656 = (_e655 + 4u);
        let _e663 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e656 & 255u)), bitcast<i32>((_e656 >> bitcast<u32>(8i)))), 0i);
        let _e671 = (_e655 + 5u);
        let _e678 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e671 & 255u)), bitcast<i32>((_e671 >> bitcast<u32>(8i)))), 0i);
        let _e681 = ((mat2x2<f32>(vec2<f32>(_e663.x, _e663.y), vec2<f32>(_e663.z, _e663.w)) * _e451) + _e678.xy);
        C2_ = vec3<f32>(_e681.x, _e681.y, (1f + _e678.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e453 {
        let _e688 = j.Hf;
        let _e690 = j.If;
        let _e698 = vec4<f32>(((_e451.x * _e688) - 1f), ((_e451.y * _e690) - sign(_e690)), 0f, 1f);
        phi_2616_ = vec4<f32>(_e698.x, _e698.y, (1f - (f32(_e449) * 0.000061035156f)), _e698.w);
    } else {
        let _e708 = j.U2_;
        phi_2616_ = vec4(_e708);
    }
    let _e711 = phi_2616_;
    unnamed.gl_Position = _e711;
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
