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
    @location(4) @interpolate(flat) member: vec2<f32>,
    @location(6) @interpolate(flat) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Ug: bool = true;
@id(2) override Wg: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_2141_: f32;
    var phi_2113_: i32;
    var phi_1418_: bool;
    var phi_2126_: i32;
    var phi_2118_: vec4<u32>;
    var phi_2125_: i32;
    var phi_2117_: vec4<u32>;
    var phi_2124_: i32;
    var phi_2122_: vec4<u32>;
    var phi_2121_: u32;
    var phi_2128_: vec2<i32>;
    var phi_2129_: vec4<u32>;
    var phi_2133_: f32;
    var phi_2204_: f32;
    var phi_2147_: f32;
    var phi_2203_: f32;
    var phi_2151_: f32;
    var phi_2148_: f32;
    var phi_2145_: f32;
    var phi_2155_: f32;
    var phi_2201_: f32;
    var phi_2154_: f32;
    var phi_2210_: f32;
    var phi_2207_: f32;
    var phi_2264_: f32;
    var phi_2236_: i32;
    var phi_2246_: f32;
    var phi_1730_: bool;
    var phi_2253_: f32;
    var phi_2274_: vec2<f32>;
    var phi_2273_: vec2<f32>;
    var phi_2272_: vec2<f32>;
    var phi_2290_: vec2<f32>;
    var phi_2275_: vec2<f32>;
    var phi_2323_: u32;
    var phi_2294_: vec2<f32>;
    var phi_2293_: bool;
    var local: u32;
    var local_1: u32;
    var phi_2352_: u32;
    var phi_2353_: f32;
    var phi_2354_: f32;
    var phi_2356_: vec4<f32>;
    var phi_2355_: f32;
    var local_2: vec2<i32>;
    var local_3: vec2<i32>;
    var phi_2369_: vec4<f32>;

    let _e73 = gl_InstanceIndex_1;
    let _e74 = UB_1;
    let _e75 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e78 = i32(_e74.x);
            let _e81 = bitcast<i32>(_e74.w);
            let _e83 = (_e81 >> bitcast<u32>(2i));
            let _e84 = (_e81 & 3i);
            let _e86 = min(_e78, (_e83 - 1i));
            let _e88 = ((_e73 * _e83) + _e86);
            let _e93 = textureLoad(LC, vec2<i32>((_e88 & 2047i), (_e88 >> bitcast<u32>(11i))), 0i);
            let _e97 = (max((_e93.w & 65535u), 1u) - 1u);
            let _e104 = textureLoad(ED, vec2<i32>(bitcast<i32>((_e97 & 127u)), bitcast<i32>((_e97 >> bitcast<u32>(7i)))), 0i);
            let _e106 = bitcast<vec2<f32>>(_e104.xy);
            let _e108 = (_e104.z & 65535u);
            let _e110 = (_e108 * 4u);
            let _e116 = vec2<i32>(bitcast<i32>((_e110 & 127u)), bitcast<i32>((_e110 >> bitcast<u32>(7i))));
            let _e117 = textureLoad(PB, _e116, 0i);
            let _e118 = bitcast<vec4<f32>>(_e117);
            let _e125 = mat2x2<f32>(vec2<f32>(_e118.x, _e118.y), vec2<f32>(_e118.z, _e118.w));
            let _e126 = (_e110 + 1u);
            let _e132 = vec2<i32>(bitcast<i32>((_e126 & 127u)), bitcast<i32>((_e126 >> bitcast<u32>(7i))));
            let _e133 = textureLoad(PB, _e132, 0i);
            let _e137 = bitcast<f32>(_e133.z);
            let _e139 = bitcast<f32>(_e133.w);
            let _e140 = (_e93.w & 8388608u);
            phi_2141_ = _e74.y;
            phi_2113_ = _e78;
            local = _e104.z;
            local_1 = _e108;
            local_2 = _e116;
            local_3 = _e132;
            if (_e140 != 0u) {
                phi_2141_ = _e75.y;
                phi_2113_ = i32(_e75.x);
            }
            let _e146 = phi_2141_;
            let _e148 = phi_2113_;
            phi_2124_ = _e88;
            phi_2122_ = _e93;
            phi_2121_ = _e93.w;
            if (_e148 != _e86) {
                let _e151 = ((_e88 + _e148) - _e86);
                let _e156 = textureLoad(LC, vec2<i32>((_e151 & 2047i), (_e151 >> bitcast<u32>(11i))), 0i);
                if ((_e156.w & 8454143u) != (_e93.w & 8454143u)) {
                    let _e161 = (_e137 == 0f);
                    phi_1418_ = _e161;
                    if !(_e161) {
                        phi_1418_ = (_e106.x != 0f);
                    }
                    let _e166 = phi_1418_;
                    phi_2126_ = _e88;
                    phi_2118_ = _e93;
                    if _e166 {
                        let _e167 = bitcast<i32>(_e104.w);
                        let _e172 = textureLoad(LC, vec2<i32>((_e167 & 2047i), (_e167 >> bitcast<u32>(11i))), 0i);
                        phi_2126_ = _e167;
                        phi_2118_ = _e172;
                    }
                    let _e174 = phi_2126_;
                    let _e176 = phi_2118_;
                    phi_2125_ = _e174;
                    phi_2117_ = _e176;
                } else {
                    phi_2125_ = _e151;
                    phi_2117_ = _e156;
                }
                let _e178 = phi_2125_;
                let _e180 = phi_2117_;
                phi_2124_ = _e178;
                phi_2122_ = _e180;
                phi_2121_ = ((_e180.w & 4286578687u) | _e140);
            }
            let _e185 = phi_2124_;
            let _e187 = phi_2122_;
            let _e189 = phi_2121_;
            let _e190 = (_e189 & 469762048u);
            if ((_e190 == 67108864u) && (_e84 == 0i)) {
                let _e196 = f32((_e187.z & 65535u));
                let _e199 = f32((_e187.z >> bitcast<u32>(16i)));
                let _e205 = vec2<i32>(i32((-1f - _e196)), i32(((_e199 - _e196) + 1f)));
                phi_2128_ = _e205;
                if ((_e189 & 8388608u) != 0u) {
                    phi_2128_ = -(_e205);
                }
                let _e210 = phi_2128_;
                let _e212 = (_e185 + _e210.x);
                let _e217 = textureLoad(LC, vec2<i32>((_e212 & 2047i), (_e212 >> bitcast<u32>(11i))), 0i);
                let _e219 = (_e185 + _e210.y);
                let _e224 = textureLoad(LC, vec2<i32>((_e219 & 2047i), (_e219 >> bitcast<u32>(11i))), 0i);
                phi_2129_ = _e224;
                if ((_e224.w & 8454143u) != (_e217.w & 8454143u)) {
                    let _e230 = bitcast<i32>(_e104.w);
                    let _e235 = textureLoad(LC, vec2<i32>((_e230 & 2047i), (_e230 >> bitcast<u32>(11i))), 0i);
                    phi_2129_ = _e235;
                }
                let _e237 = phi_2129_;
                let _e239 = bitcast<f32>(_e217.z);
                let _e241 = bitcast<f32>(_e237.z);
                let _e242 = (_e241 - _e239);
                phi_2133_ = _e242;
                if (abs(_e242) > 3.1415927f) {
                    phi_2133_ = (_e242 - (6.2831855f * sign(_e242)));
                }
                let _e249 = phi_2133_;
                let _e250 = (_e199 + -2f);
                let _e256 = clamp(round(((abs(_e249) * 0.31830987f) * _e250)), 1f, (_e199 + -3f));
                let _e257 = (_e250 - _e256);
                if (_e196 <= _e257) {
                    phi_2204_ = _e146;
                    if (_e196 == _e257) {
                        phi_2204_ = -(_e146);
                    }
                    let _e266 = phi_2204_;
                    phi_2203_ = _e266;
                    phi_2151_ = -(((3.1415927f * sign(_e249)) - _e249));
                    phi_2148_ = _e257;
                    phi_2145_ = _e196;
                } else {
                    let _e268 = (_e196 == (_e257 + 1f));
                    if _e268 {
                        phi_2147_ = 0f;
                    } else {
                        phi_2147_ = (_e196 - (_e257 + 2f));
                    }
                    let _e272 = phi_2147_;
                    phi_2203_ = select(_e146, 0f, _e268);
                    phi_2151_ = _e249;
                    phi_2148_ = select(_e256, 0f, _e268);
                    phi_2145_ = _e272;
                }
                let _e276 = phi_2203_;
                let _e278 = phi_2151_;
                let _e280 = phi_2148_;
                let _e282 = phi_2145_;
                if (_e282 == _e280) {
                    phi_2155_ = _e241;
                } else {
                    phi_2155_ = (_e239 + (_e278 * (_e282 / _e280)));
                }
                let _e288 = phi_2155_;
                phi_2201_ = _e276;
                phi_2154_ = _e288;
            } else {
                phi_2201_ = _e146;
                phi_2154_ = bitcast<f32>(_e187.z);
            }
            let _e292 = phi_2201_;
            let _e294 = phi_2154_;
            let _e298 = vec2<f32>(sin(_e294), -(cos(_e294)));
            let _e300 = bitcast<vec2<f32>>(_e187.xy);
            phi_2210_ = _e139;
            if (_e139 != 0f) {
                phi_2210_ = max(_e139, (1f / length((_e125 * _e298))));
            }
            let _e307 = phi_2210_;
            if (_e137 != 0f) {
                let _e311 = (_e292 * sign(determinant(_e125)));
                let _e313 = ((_e189 & 1048576u) != 0u);
                phi_2207_ = _e311;
                if _e313 {
                    phi_2207_ = min(_e311, 0f);
                }
                let _e316 = phi_2207_;
                phi_2264_ = _e316;
                if ((_e189 & 524288u) != 0u) {
                    phi_2264_ = max(_e316, 0f);
                }
                let _e321 = phi_2264_;
                let _e323 = select(0f, _e307, (_e307 != 0f));
                let _e327 = select(_e137, _e323, ((_e323 > _e137) && (_e307 == 0f)));
                let _e328 = (_e327 + _e323);
                let _e329 = (_e298 * _e328);
                phi_2272_ = _e329;
                if (_e190 > 134217728u) {
                    let _e331 = (_e189 & 4194304u);
                    let _e333 = select(2i, -2i, (_e331 == 0u));
                    phi_2236_ = _e333;
                    if ((_e189 & 8388608u) != 0u) {
                        phi_2236_ = -(_e333);
                    }
                    let _e338 = phi_2236_;
                    let _e339 = (_e185 + _e338);
                    let _e344 = textureLoad(LC, vec2<i32>((_e339 & 2047i), (_e339 >> bitcast<u32>(11i))), 0i);
                    let _e348 = abs((bitcast<f32>(_e344.z) - _e294));
                    phi_2246_ = _e348;
                    if (_e348 > 3.1415927f) {
                        phi_2246_ = (6.2831855f - _e348);
                    }
                    let _e352 = phi_2246_;
                    let _e357 = ((_e352 * select(0.5f, -0.5f, ((_e331 != 0u) == _e313))) + _e294);
                    let _e361 = vec2<f32>(sin(_e357), -(cos(_e357)));
                    let _e362 = (_e125 * _e361);
                    let _e372 = cos((_e352 * 0.5f));
                    let _e373 = (_e190 == 335544320u);
                    phi_1730_ = _e373;
                    if !(_e373) {
                        phi_1730_ = ((_e190 == 268435456u) && (_e372 >= 0.25f));
                    }
                    let _e379 = phi_1730_;
                    if _e379 {
                        phi_2253_ = (_e327 * (1f / max(_e372, select(0.25f, 1f, ((_e189 & 33554432u) != 0u)))));
                    } else {
                        phi_2253_ = ((_e327 * _e372) + (((abs(_e362.x) + abs(_e362.y)) * (1f / dot(_e362, _e362))) * 0.5f));
                    }
                    let _e390 = phi_2253_;
                    phi_2273_ = _e329;
                    if ((_e189 & 2097152u) != 0u) {
                        if (_e328 <= ((_e390 * _e372) + (_e323 * 0.125f))) {
                            phi_2274_ = (_e361 * (_e328 * (1f / _e372)));
                        } else {
                            let _e400 = (_e361 * _e390);
                            phi_2274_ = (vec2<f32>(dot(_e329, _e329), dot(_e400, _e400)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e329, _e400)));
                        }
                        let _e408 = phi_2274_;
                        phi_2273_ = _e408;
                    }
                    let _e410 = phi_2273_;
                    phi_2272_ = _e410;
                }
                let _e412 = phi_2272_;
                if (_e84 != 0i) {
                    phi_2323_ = u32();
                    phi_2294_ = vec2<f32>();
                    phi_2293_ = false;
                    break;
                }
                phi_2290_ = (_e125 * (_e412 * _e321));
                phi_2275_ = _e300;
            } else {
                if (((_e189 & 2147483648u) != 0u) && (_e84 != 1i)) {
                    phi_2323_ = u32();
                    phi_2294_ = vec2<f32>();
                    phi_2293_ = false;
                    break;
                }
                phi_2290_ = vec2<f32>(0f, 0f);
                phi_2275_ = select(_e300, _e106, vec2((_e84 == 2i)));
            }
            let _e424 = phi_2290_;
            let _e426 = phi_2275_;
            let _e430 = (_e110 + 2u);
            let _e437 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e430 & 127u)), bitcast<i32>((_e430 >> bitcast<u32>(7i)))), 0i);
            phi_2323_ = _e437.x;
            phi_2294_ = (((_e125 * _e426) + _e424) + bitcast<vec2<f32>>(_e133.xy));
            phi_2293_ = true;
            break;
        }
    }
    let _e440 = phi_2323_;
    let _e442 = phi_2294_;
    let _e444 = phi_2293_;
    let _e446 = local;
    let _e450 = local_1;
    let _e455 = textureLoad(AD, vec2<i32>(bitcast<i32>((_e446 & 127u)), bitcast<i32>((_e450 >> bitcast<u32>(7i)))), 0i);
    let _e457 = (_e455.x & 15u);
    if Ug {
        let _e458 = (_e457 == 0u);
        if _e458 {
            phi_2352_ = _e455.y;
        } else {
            phi_2352_ = _e455.x;
        }
        let _e461 = phi_2352_;
        let _e463 = (_e461 >> bitcast<u32>(16i));
        let _e465 = m.Z5_;
        if (_e463 == 0u) {
            phi_2353_ = 0f;
        } else {
            phi_2353_ = unpack2x16float(((_e463 + 1023u) * _e465)).x;
        }
        let _e472 = phi_2353_;
        phi_2354_ = _e472;
        if _e458 {
            phi_2354_ = -(_e472);
        }
        let _e475 = phi_2354_;
        U1_[0u] = _e475;
    }
    if Wg {
        e2_ = f32(((_e455.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e457 == 1u) {
        let _e483 = unpack4x8unorm(_e455.y);
        if Wg {
            phi_2356_ = _e483;
        } else {
            let _e486 = (_e483.xyz * _e483.w);
            let _e492 = vec4<f32>(_e486.x, _e483.y, _e483.z, _e483.w);
            let _e498 = vec4<f32>(_e492.x, _e486.y, _e492.z, _e492.w);
            phi_2356_ = vec4<f32>(_e498.x, _e498.y, _e486.z, _e498.w);
        }
        let _e506 = phi_2356_;
        f1_ = _e506;
    } else {
        if (Ug && (_e457 == 0u)) {
            let _e510 = (_e455.x >> bitcast<u32>(16i));
            let _e512 = m.Z5_;
            if (_e510 == 0u) {
                phi_2355_ = 0f;
            } else {
                phi_2355_ = unpack2x16float(((_e510 + 1023u) * _e512)).x;
            }
            let _e519 = phi_2355_;
            U1_[1u] = _e519;
        } else {
            let _e522 = local_2;
            let _e523 = textureLoad(RB, _e522, 0i);
            let _e532 = local_3;
            let _e533 = textureLoad(RB, _e532, 0i);
            let _e536 = ((mat2x2<f32>(vec2<f32>(_e523.x, _e523.y), vec2<f32>(_e523.z, _e523.w)) * _e442) + _e533.xy);
            let _e537 = (_e457 == 2u);
            if (_e537 || (_e457 == 3u)) {
                f1_[3u] = -(bitcast<f32>(_e455.y));
                if (_e533.z > 0.9f) {
                    f1_[2u] = 2f;
                } else {
                    f1_[2u] = _e533.w;
                }
                if _e537 {
                    f1_[1u] = 0f;
                    f1_[0u] = _e536.x;
                } else {
                    let _e553 = f1_[2u];
                    f1_[2u] = -(_e553);
                    f1_[0u] = _e536.x;
                    f1_[1u] = _e536.y;
                }
            } else {
                f1_ = vec4<f32>(_e536.x, _e536.y, bitcast<f32>(_e455.y), (-2f - _e533.z));
            }
        }
    }
    if _e444 {
        let _e567 = m.bf;
        let _e569 = m.cf;
        let _e577 = vec4<f32>(((_e442.x * _e567) - 1f), ((_e442.y * _e569) - sign(_e569)), 0f, 1f);
        phi_2369_ = vec4<f32>(_e577.x, _e577.y, (1f - (f32(_e440) * 0.000061035156f)), _e577.w);
    } else {
        let _e587 = m.N2_;
        phi_2369_ = vec4(_e587);
    }
    let _e590 = phi_2369_;
    unnamed.gl_Position = _e590;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e15 = U1_;
    let _e16 = e2_;
    let _e17 = f1_;
    let _e18 = unnamed.gl_Position;
    return VertexOutput(_e15, _e16, _e17, _e18);
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
