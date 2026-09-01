enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override eh: bool = true;
@id(2) override gh: bool = true;
@id(1) override fh: bool = true;
@id(8) override mh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(5)
var FD: texture_2d<u32>;
@group(0) @binding(2)
var QB: texture_2d<u32>;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
@group(0) @binding(3)
var BD: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> U1_: vec2<f32>;
var<private> e2_: f32;
@group(0) @binding(4)
var RB: texture_2d<f32>;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_2288_: f32;
    var phi_2260_: i32;
    var phi_1507_: bool;
    var phi_2273_: i32;
    var phi_2265_: vec4<u32>;
    var phi_2272_: i32;
    var phi_2264_: vec4<u32>;
    var phi_2271_: i32;
    var phi_2269_: vec4<u32>;
    var phi_2268_: u32;
    var phi_2275_: vec2<i32>;
    var phi_2276_: vec4<u32>;
    var phi_2280_: f32;
    var phi_2351_: f32;
    var phi_2294_: f32;
    var phi_2350_: f32;
    var phi_2298_: f32;
    var phi_2295_: f32;
    var phi_2292_: f32;
    var phi_2302_: f32;
    var phi_2348_: f32;
    var phi_2301_: f32;
    var phi_2357_: f32;
    var phi_2354_: f32;
    var phi_2411_: f32;
    var phi_2383_: i32;
    var phi_2393_: f32;
    var phi_1819_: bool;
    var phi_2400_: f32;
    var phi_2421_: vec2<f32>;
    var phi_2420_: vec2<f32>;
    var phi_2419_: vec2<f32>;
    var phi_2437_: vec2<f32>;
    var phi_2422_: vec2<f32>;
    var phi_2470_: u32;
    var phi_2441_: vec2<f32>;
    var phi_2440_: bool;
    var local: u32;
    var local_1: u32;
    var phi_2499_: u32;
    var phi_2500_: f32;
    var phi_2501_: f32;
    var local_2: u32;
    var phi_2503_: vec4<f32>;
    var phi_2502_: f32;
    var local_3: u32;
    var phi_1188_: bool;
    var local_4: u32;
    var phi_2520_: vec4<f32>;

    let _e81 = gl_InstanceIndex_1;
    let _e82 = VB_1;
    let _e83 = WB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e86 = i32(_e82.x);
            let _e89 = bitcast<i32>(_e82.w);
            let _e91 = (_e89 >> bitcast<u32>(2i));
            let _e92 = (_e89 & 3i);
            let _e94 = min(_e86, (_e91 - 1i));
            let _e96 = ((_e81 * _e91) + _e94);
            let _e101 = textureLoad(MC, vec2<i32>((_e96 & 2047i), (_e96 >> bitcast<u32>(11i))), 0i);
            let _e105 = (max((_e101.w & 65535u), 1u) - 1u);
            let _e112 = textureLoad(FD, vec2<i32>(bitcast<i32>((_e105 & 255u)), bitcast<i32>((_e105 >> bitcast<u32>(8i)))), 0i);
            let _e114 = bitcast<vec2<f32>>(_e112.xy);
            let _e116 = (_e112.z & 65535u);
            let _e118 = (_e116 * 4u);
            let _e125 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e118 & 255u)), bitcast<i32>((_e118 >> bitcast<u32>(8i)))), 0i);
            let _e126 = bitcast<vec4<f32>>(_e125);
            let _e133 = mat2x2<f32>(vec2<f32>(_e126.x, _e126.y), vec2<f32>(_e126.z, _e126.w));
            let _e134 = (_e118 + 1u);
            let _e141 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e134 & 255u)), bitcast<i32>((_e134 >> bitcast<u32>(8i)))), 0i);
            let _e145 = bitcast<f32>(_e141.z);
            let _e147 = bitcast<f32>(_e141.w);
            let _e148 = (_e101.w & 8388608u);
            phi_2288_ = _e82.y;
            phi_2260_ = _e86;
            local = _e112.z;
            local_1 = _e116;
            local_2 = _e116;
            local_3 = _e116;
            local_4 = _e116;
            if (_e148 != 0u) {
                phi_2288_ = _e83.y;
                phi_2260_ = i32(_e83.x);
            }
            let _e154 = phi_2288_;
            let _e156 = phi_2260_;
            phi_2271_ = _e96;
            phi_2269_ = _e101;
            phi_2268_ = _e101.w;
            if (_e156 != _e94) {
                let _e159 = ((_e96 + _e156) - _e94);
                let _e164 = textureLoad(MC, vec2<i32>((_e159 & 2047i), (_e159 >> bitcast<u32>(11i))), 0i);
                if ((_e164.w & 8454143u) != (_e101.w & 8454143u)) {
                    let _e169 = (_e145 == 0f);
                    phi_1507_ = _e169;
                    if !(_e169) {
                        phi_1507_ = (_e114.x != 0f);
                    }
                    let _e174 = phi_1507_;
                    phi_2273_ = _e96;
                    phi_2265_ = _e101;
                    if _e174 {
                        let _e175 = bitcast<i32>(_e112.w);
                        let _e180 = textureLoad(MC, vec2<i32>((_e175 & 2047i), (_e175 >> bitcast<u32>(11i))), 0i);
                        phi_2273_ = _e175;
                        phi_2265_ = _e180;
                    }
                    let _e182 = phi_2273_;
                    let _e184 = phi_2265_;
                    phi_2272_ = _e182;
                    phi_2264_ = _e184;
                } else {
                    phi_2272_ = _e159;
                    phi_2264_ = _e164;
                }
                let _e186 = phi_2272_;
                let _e188 = phi_2264_;
                phi_2271_ = _e186;
                phi_2269_ = _e188;
                phi_2268_ = ((_e188.w & 4286578687u) | _e148);
            }
            let _e193 = phi_2271_;
            let _e195 = phi_2269_;
            let _e197 = phi_2268_;
            let _e198 = (_e197 & 469762048u);
            if ((_e198 == 67108864u) && (_e92 == 0i)) {
                let _e204 = f32((_e195.z & 65535u));
                let _e207 = f32((_e195.z >> bitcast<u32>(16i)));
                let _e213 = vec2<i32>(i32((-1f - _e204)), i32(((_e207 - _e204) + 1f)));
                phi_2275_ = _e213;
                if ((_e197 & 8388608u) != 0u) {
                    phi_2275_ = -(_e213);
                }
                let _e218 = phi_2275_;
                let _e220 = (_e193 + _e218.x);
                let _e225 = textureLoad(MC, vec2<i32>((_e220 & 2047i), (_e220 >> bitcast<u32>(11i))), 0i);
                let _e227 = (_e193 + _e218.y);
                let _e232 = textureLoad(MC, vec2<i32>((_e227 & 2047i), (_e227 >> bitcast<u32>(11i))), 0i);
                phi_2276_ = _e232;
                if ((_e232.w & 8454143u) != (_e225.w & 8454143u)) {
                    let _e238 = bitcast<i32>(_e112.w);
                    let _e243 = textureLoad(MC, vec2<i32>((_e238 & 2047i), (_e238 >> bitcast<u32>(11i))), 0i);
                    phi_2276_ = _e243;
                }
                let _e245 = phi_2276_;
                let _e247 = bitcast<f32>(_e225.z);
                let _e249 = bitcast<f32>(_e245.z);
                let _e250 = (_e249 - _e247);
                phi_2280_ = _e250;
                if (abs(_e250) > 3.1415927f) {
                    phi_2280_ = (_e250 - (6.2831855f * sign(_e250)));
                }
                let _e257 = phi_2280_;
                let _e258 = (_e207 + -2f);
                let _e264 = clamp(round(((abs(_e257) * 0.31830987f) * _e258)), 1f, (_e207 + -3f));
                let _e265 = (_e258 - _e264);
                if (_e204 <= _e265) {
                    phi_2351_ = _e154;
                    if (_e204 == _e265) {
                        phi_2351_ = -(_e154);
                    }
                    let _e274 = phi_2351_;
                    phi_2350_ = _e274;
                    phi_2298_ = -(((3.1415927f * sign(_e257)) - _e257));
                    phi_2295_ = _e265;
                    phi_2292_ = _e204;
                } else {
                    let _e276 = (_e204 == (_e265 + 1f));
                    if _e276 {
                        phi_2294_ = 0f;
                    } else {
                        phi_2294_ = (_e204 - (_e265 + 2f));
                    }
                    let _e280 = phi_2294_;
                    phi_2350_ = select(_e154, 0f, _e276);
                    phi_2298_ = _e257;
                    phi_2295_ = select(_e264, 0f, _e276);
                    phi_2292_ = _e280;
                }
                let _e284 = phi_2350_;
                let _e286 = phi_2298_;
                let _e288 = phi_2295_;
                let _e290 = phi_2292_;
                if (_e290 == _e288) {
                    phi_2302_ = _e249;
                } else {
                    phi_2302_ = (_e247 + (_e286 * (_e290 / _e288)));
                }
                let _e296 = phi_2302_;
                phi_2348_ = _e284;
                phi_2301_ = _e296;
            } else {
                phi_2348_ = _e154;
                phi_2301_ = bitcast<f32>(_e195.z);
            }
            let _e300 = phi_2348_;
            let _e302 = phi_2301_;
            let _e306 = vec2<f32>(sin(_e302), -(cos(_e302)));
            let _e308 = bitcast<vec2<f32>>(_e195.xy);
            phi_2357_ = _e147;
            if (_e147 != 0f) {
                phi_2357_ = max(_e147, (1f / length((_e133 * _e306))));
            }
            let _e315 = phi_2357_;
            if (_e145 != 0f) {
                let _e319 = (_e300 * sign(determinant(_e133)));
                let _e321 = ((_e197 & 1048576u) != 0u);
                phi_2354_ = _e319;
                if _e321 {
                    phi_2354_ = min(_e319, 0f);
                }
                let _e324 = phi_2354_;
                phi_2411_ = _e324;
                if ((_e197 & 524288u) != 0u) {
                    phi_2411_ = max(_e324, 0f);
                }
                let _e329 = phi_2411_;
                let _e331 = select(0f, _e315, (_e315 != 0f));
                let _e335 = select(_e145, _e331, ((_e331 > _e145) && (_e315 == 0f)));
                let _e336 = (_e335 + _e331);
                let _e337 = (_e306 * _e336);
                phi_2419_ = _e337;
                if (_e198 > 134217728u) {
                    let _e339 = (_e197 & 4194304u);
                    let _e341 = select(2i, -2i, (_e339 == 0u));
                    phi_2383_ = _e341;
                    if ((_e197 & 8388608u) != 0u) {
                        phi_2383_ = -(_e341);
                    }
                    let _e346 = phi_2383_;
                    let _e347 = (_e193 + _e346);
                    let _e352 = textureLoad(MC, vec2<i32>((_e347 & 2047i), (_e347 >> bitcast<u32>(11i))), 0i);
                    let _e356 = abs((bitcast<f32>(_e352.z) - _e302));
                    phi_2393_ = _e356;
                    if (_e356 > 3.1415927f) {
                        phi_2393_ = (6.2831855f - _e356);
                    }
                    let _e360 = phi_2393_;
                    let _e365 = ((_e360 * select(0.5f, -0.5f, ((_e339 != 0u) == _e321))) + _e302);
                    let _e369 = vec2<f32>(sin(_e365), -(cos(_e365)));
                    let _e370 = (_e133 * _e369);
                    let _e380 = cos((_e360 * 0.5f));
                    let _e381 = (_e198 == 335544320u);
                    phi_1819_ = _e381;
                    if !(_e381) {
                        phi_1819_ = ((_e198 == 268435456u) && (_e380 >= 0.25f));
                    }
                    let _e387 = phi_1819_;
                    if _e387 {
                        phi_2400_ = (_e335 * (1f / max(_e380, select(0.25f, 1f, ((_e197 & 33554432u) != 0u)))));
                    } else {
                        phi_2400_ = ((_e335 * _e380) + (((abs(_e370.x) + abs(_e370.y)) * (1f / dot(_e370, _e370))) * 0.5f));
                    }
                    let _e398 = phi_2400_;
                    phi_2420_ = _e337;
                    if ((_e197 & 2097152u) != 0u) {
                        if (_e336 <= ((_e398 * _e380) + (_e331 * 0.125f))) {
                            phi_2421_ = (_e369 * (_e336 * (1f / _e380)));
                        } else {
                            let _e408 = (_e369 * _e398);
                            phi_2421_ = (vec2<f32>(dot(_e337, _e337), dot(_e408, _e408)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e337, _e408)));
                        }
                        let _e416 = phi_2421_;
                        phi_2420_ = _e416;
                    }
                    let _e418 = phi_2420_;
                    phi_2419_ = _e418;
                }
                let _e420 = phi_2419_;
                if (_e92 != 0i) {
                    phi_2470_ = u32();
                    phi_2441_ = vec2<f32>();
                    phi_2440_ = false;
                    break;
                }
                phi_2437_ = (_e133 * (_e420 * _e329));
                phi_2422_ = _e308;
            } else {
                if (((_e197 & 2147483648u) != 0u) && (_e92 != 1i)) {
                    phi_2470_ = u32();
                    phi_2441_ = vec2<f32>();
                    phi_2440_ = false;
                    break;
                }
                phi_2437_ = vec2<f32>(0f, 0f);
                phi_2422_ = select(_e308, _e114, vec2((_e92 == 2i)));
            }
            let _e432 = phi_2437_;
            let _e434 = phi_2422_;
            let _e438 = (_e118 + 2u);
            let _e445 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e438 & 255u)), bitcast<i32>((_e438 >> bitcast<u32>(8i)))), 0i);
            phi_2470_ = _e445.x;
            phi_2441_ = (((_e133 * _e434) + _e432) + bitcast<vec2<f32>>(_e141.xy));
            phi_2440_ = true;
            break;
        }
    }
    let _e448 = phi_2470_;
    let _e450 = phi_2441_;
    let _e452 = phi_2440_;
    let _e454 = local;
    let _e458 = local_1;
    let _e463 = textureLoad(BD, vec2<i32>(bitcast<i32>((_e454 & 255u)), bitcast<i32>((_e458 >> bitcast<u32>(8i)))), 0i);
    let _e465 = (_e463.x & 15u);
    if eh {
        let _e466 = (_e465 == 0u);
        if _e466 {
            phi_2499_ = _e463.y;
        } else {
            phi_2499_ = _e463.x;
        }
        let _e469 = phi_2499_;
        let _e471 = (_e469 >> bitcast<u32>(16i));
        let _e473 = m.c6_;
        if (_e471 == 0u) {
            phi_2500_ = 0f;
        } else {
            phi_2500_ = unpack2x16float(((_e471 + 1023u) * _e473)).x;
        }
        let _e480 = phi_2500_;
        phi_2501_ = _e480;
        if _e466 {
            phi_2501_ = -(_e480);
        }
        let _e483 = phi_2501_;
        U1_[0u] = _e483;
    }
    if gh {
        e2_ = f32(((_e463.x >> bitcast<u32>(4i)) & 15u));
    }
    if fh {
        let _e490 = local_2;
        let _e491 = (_e490 * 8u);
        let _e492 = (_e491 + 2u);
        let _e499 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e492 & 255u)), bitcast<i32>((_e492 >> bitcast<u32>(8i)))), 0i);
        let _e507 = (_e491 + 3u);
        let _e514 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e507 & 255u)), bitcast<i32>((_e507 >> bitcast<u32>(8i)))), 0i);
        if any((_e499 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e519 = ((mat2x2<f32>(vec2<f32>(_e499.x, _e499.y), vec2<f32>(_e499.z, _e499.w)) * _e450) + _e514.xy);
            unnamed.gl_ClipDistance[0i] = (_e519.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e519.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e519.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e519.y);
        } else {
            let _e535 = (_e514.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e535;
            unnamed.gl_ClipDistance[2i] = _e535;
            unnamed.gl_ClipDistance[1i] = _e535;
            unnamed.gl_ClipDistance[0i] = _e535;
        }
    }
    if (_e465 == 1u) {
        let _e546 = unpack4x8unorm(_e463.y);
        if gh {
            phi_2503_ = _e546;
        } else {
            let _e549 = (_e546.xyz * _e546.w);
            let _e555 = vec4<f32>(_e549.x, _e546.y, _e546.z, _e546.w);
            let _e561 = vec4<f32>(_e555.x, _e549.y, _e555.z, _e555.w);
            phi_2503_ = vec4<f32>(_e561.x, _e561.y, _e549.z, _e561.w);
        }
        let _e569 = phi_2503_;
        f1_ = _e569;
    } else {
        if (eh && (_e465 == 0u)) {
            let _e573 = (_e463.x >> bitcast<u32>(16i));
            let _e575 = m.c6_;
            if (_e573 == 0u) {
                phi_2502_ = 0f;
            } else {
                phi_2502_ = unpack2x16float(((_e573 + 1023u) * _e575)).x;
            }
            let _e582 = phi_2502_;
            U1_[1u] = _e582;
        } else {
            let _e585 = local_3;
            let _e586 = (_e585 * 8u);
            let _e593 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e586 & 255u)), bitcast<i32>((_e586 >> bitcast<u32>(8i)))), 0i);
            let _e601 = (_e586 + 1u);
            let _e608 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e601 & 255u)), bitcast<i32>((_e601 >> bitcast<u32>(8i)))), 0i);
            let _e611 = ((mat2x2<f32>(vec2<f32>(_e593.x, _e593.y), vec2<f32>(_e593.z, _e593.w)) * _e450) + _e608.xy);
            let _e612 = (_e465 == 2u);
            if (_e612 || (_e465 == 3u)) {
                f1_[3u] = -(bitcast<f32>(_e463.y));
                if (_e608.z > 0.9f) {
                    f1_[2u] = 2f;
                } else {
                    f1_[2u] = _e608.w;
                }
                if _e612 {
                    f1_[1u] = 0f;
                    f1_[0u] = _e611.x;
                } else {
                    let _e628 = f1_[2u];
                    f1_[2u] = -(_e628);
                    f1_[0u] = _e611.x;
                    f1_[1u] = _e611.y;
                }
            }
        }
    }
    phi_1188_ = mh;
    if mh {
        phi_1188_ = ((_e463.x & 2048u) != 0u);
    }
    let _e637 = phi_1188_;
    if _e637 {
        let _e639 = local_4;
        let _e640 = (_e639 * 8u);
        let _e641 = (_e640 + 4u);
        let _e648 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e641 & 255u)), bitcast<i32>((_e641 >> bitcast<u32>(8i)))), 0i);
        let _e656 = (_e640 + 5u);
        let _e663 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e656 & 255u)), bitcast<i32>((_e656 >> bitcast<u32>(8i)))), 0i);
        let _e666 = ((mat2x2<f32>(vec2<f32>(_e648.x, _e648.y), vec2<f32>(_e648.z, _e648.w)) * _e450) + _e663.xy);
        A2_ = vec3<f32>(_e666.x, _e666.y, (1f + _e663.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e452 {
        let _e673 = m.jf;
        let _e675 = m.kf;
        let _e683 = vec4<f32>(((_e450.x * _e673) - 1f), ((_e450.y * _e675) - sign(_e675)), 0f, 1f);
        phi_2520_ = vec4<f32>(_e683.x, _e683.y, (1f - (f32(_e448) * 0.000061035156f)), _e683.w);
    } else {
        let _e693 = m.R2_;
        phi_2520_ = vec4(_e693);
    }
    let _e696 = phi_2520_;
    unnamed.gl_Position = _e696;
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
    let _e19 = U1_;
    let _e20 = e2_;
    let _e21 = f1_;
    let _e22 = A2_;
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
