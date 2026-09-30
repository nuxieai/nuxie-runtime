struct Kg {
    g2_: array<vec4<u32>>,
}

struct Jg {
    g2_: array<vec4<u32>>,
}

struct pf {
    g2_: array<vec2<u32>>,
}

struct SB {
    xc: f32,
    Hd: f32,
    Mf: f32,
    Nf: f32,
    r6_: u32,
    Rb: u32,
    yf: u32,
    zf: u32,
    X7_: vec4<i32>,
    jh: vec2<f32>,
    Id: vec2<f32>,
    f2_: u32,
    nh: f32,
    g6_: u32,
    W2_: f32,
    Jd: f32,
    sf: u32,
    F3_: f32,
    G3_: f32,
    Kd: f32,
    gh: u32,
    Qb: u32,
    dc: f32,
    ec: f32,
}

struct qf {
    g2_: array<vec4<f32>>,
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

@id(0) override Jh: bool = true;
@id(2) override Lh: bool = true;
@id(8) override Rh: bool = true;

@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> HD: Kg;
@group(0) @binding(2)
var<storage> OB: Jg;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> CD: pf;
@group(0) @binding(0)
var<uniform> j: SB;
var<private> Y1_: vec2<f32>;
var<private> g1_: f32;
@group(0) @binding(4)
var<storage> PB: qf;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ha: sampler;

fn main_1() {
    var phi_2159_: f32;
    var phi_2131_: i32;
    var phi_1379_: bool;
    var phi_2144_: i32;
    var phi_2136_: vec4<u32>;
    var phi_2143_: i32;
    var phi_2135_: vec4<u32>;
    var phi_2142_: i32;
    var phi_2140_: vec4<u32>;
    var phi_2139_: u32;
    var phi_2146_: vec2<i32>;
    var phi_2147_: vec4<u32>;
    var phi_2151_: f32;
    var phi_2222_: f32;
    var phi_2165_: f32;
    var phi_2221_: f32;
    var phi_2169_: f32;
    var phi_2166_: f32;
    var phi_2163_: f32;
    var phi_2173_: f32;
    var phi_2219_: f32;
    var phi_2172_: f32;
    var phi_2228_: f32;
    var phi_2225_: f32;
    var phi_2282_: f32;
    var phi_2254_: i32;
    var phi_2264_: f32;
    var phi_1691_: bool;
    var phi_2271_: f32;
    var phi_2292_: vec2<f32>;
    var phi_2291_: vec2<f32>;
    var phi_2290_: vec2<f32>;
    var phi_2308_: vec2<f32>;
    var phi_2293_: vec2<f32>;
    var phi_2341_: u32;
    var phi_2312_: vec2<f32>;
    var phi_2311_: bool;
    var local: u32;
    var phi_2370_: u32;
    var phi_2371_: f32;
    var phi_2372_: f32;
    var phi_2410_: f32;
    var local_1: u32;
    var phi_2408_: vec4<f32>;
    var phi_2409_: vec4<f32>;
    var phi_1091_: bool;
    var local_2: u32;
    var phi_2423_: vec4<f32>;

    let _e78 = gl_InstanceIndex_1;
    let _e79 = UB_1;
    let _e80 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e83 = i32(_e79.x);
            let _e86 = bitcast<i32>(_e79.w);
            let _e88 = (_e86 >> bitcast<u32>(2i));
            let _e89 = (_e86 & 3i);
            let _e91 = min(_e83, (_e88 - 1i));
            let _e93 = ((_e78 * _e88) + _e91);
            let _e98 = textureLoad(JC, vec2<i32>((_e93 & 2047i), (_e93 >> bitcast<u32>(11i))), 0i);
            let _e105 = HD.g2_[(max((_e98.w & 65535u), 1u) - 1u)];
            let _e107 = bitcast<vec2<f32>>(_e105.xy);
            let _e109 = (_e105.z & 65535u);
            let _e111 = (_e109 * 4u);
            let _e114 = OB.g2_[_e111];
            let _e115 = bitcast<vec4<f32>>(_e114);
            let _e122 = mat2x2<f32>(vec2<f32>(_e115.x, _e115.y), vec2<f32>(_e115.z, _e115.w));
            let _e126 = OB.g2_[(_e111 + 1u)];
            let _e130 = bitcast<f32>(_e126.z);
            let _e132 = bitcast<f32>(_e126.w);
            let _e133 = (_e98.w & 8388608u);
            phi_2159_ = _e79.y;
            phi_2131_ = _e83;
            local = _e109;
            local_1 = _e109;
            local_2 = _e109;
            if (_e133 != 0u) {
                phi_2159_ = _e80.y;
                phi_2131_ = i32(_e80.x);
            }
            let _e139 = phi_2159_;
            let _e141 = phi_2131_;
            phi_2142_ = _e93;
            phi_2140_ = _e98;
            phi_2139_ = _e98.w;
            if (_e141 != _e91) {
                let _e144 = ((_e93 + _e141) - _e91);
                let _e149 = textureLoad(JC, vec2<i32>((_e144 & 2047i), (_e144 >> bitcast<u32>(11i))), 0i);
                if ((_e149.w & 8454143u) != (_e98.w & 8454143u)) {
                    let _e154 = (_e130 == 0f);
                    phi_1379_ = _e154;
                    if !(_e154) {
                        phi_1379_ = (_e107.x != 0f);
                    }
                    let _e159 = phi_1379_;
                    phi_2144_ = _e93;
                    phi_2136_ = _e98;
                    if _e159 {
                        let _e160 = bitcast<i32>(_e105.w);
                        let _e165 = textureLoad(JC, vec2<i32>((_e160 & 2047i), (_e160 >> bitcast<u32>(11i))), 0i);
                        phi_2144_ = _e160;
                        phi_2136_ = _e165;
                    }
                    let _e167 = phi_2144_;
                    let _e169 = phi_2136_;
                    phi_2143_ = _e167;
                    phi_2135_ = _e169;
                } else {
                    phi_2143_ = _e144;
                    phi_2135_ = _e149;
                }
                let _e171 = phi_2143_;
                let _e173 = phi_2135_;
                phi_2142_ = _e171;
                phi_2140_ = _e173;
                phi_2139_ = ((_e173.w & 4286578687u) | _e133);
            }
            let _e178 = phi_2142_;
            let _e180 = phi_2140_;
            let _e182 = phi_2139_;
            let _e183 = (_e182 & 469762048u);
            if ((_e183 == 67108864u) && (_e89 == 0i)) {
                let _e189 = f32((_e180.z & 65535u));
                let _e192 = f32((_e180.z >> bitcast<u32>(16i)));
                let _e198 = vec2<i32>(i32((-1f - _e189)), i32(((_e192 - _e189) + 1f)));
                phi_2146_ = _e198;
                if ((_e182 & 8388608u) != 0u) {
                    phi_2146_ = -(_e198);
                }
                let _e203 = phi_2146_;
                let _e205 = (_e178 + _e203.x);
                let _e210 = textureLoad(JC, vec2<i32>((_e205 & 2047i), (_e205 >> bitcast<u32>(11i))), 0i);
                let _e212 = (_e178 + _e203.y);
                let _e217 = textureLoad(JC, vec2<i32>((_e212 & 2047i), (_e212 >> bitcast<u32>(11i))), 0i);
                phi_2147_ = _e217;
                if ((_e217.w & 8454143u) != (_e210.w & 8454143u)) {
                    let _e223 = bitcast<i32>(_e105.w);
                    let _e228 = textureLoad(JC, vec2<i32>((_e223 & 2047i), (_e223 >> bitcast<u32>(11i))), 0i);
                    phi_2147_ = _e228;
                }
                let _e230 = phi_2147_;
                let _e232 = bitcast<f32>(_e210.z);
                let _e234 = bitcast<f32>(_e230.z);
                let _e235 = (_e234 - _e232);
                phi_2151_ = _e235;
                if (abs(_e235) > 3.1415927f) {
                    phi_2151_ = (_e235 - (6.2831855f * sign(_e235)));
                }
                let _e242 = phi_2151_;
                let _e243 = (_e192 + -2f);
                let _e249 = clamp(round(((abs(_e242) * 0.31830987f) * _e243)), 1f, (_e192 + -3f));
                let _e250 = (_e243 - _e249);
                if (_e189 <= _e250) {
                    phi_2222_ = _e139;
                    if (_e189 == _e250) {
                        phi_2222_ = -(_e139);
                    }
                    let _e259 = phi_2222_;
                    phi_2221_ = _e259;
                    phi_2169_ = -(((3.1415927f * sign(_e242)) - _e242));
                    phi_2166_ = _e250;
                    phi_2163_ = _e189;
                } else {
                    let _e261 = (_e189 == (_e250 + 1f));
                    if _e261 {
                        phi_2165_ = 0f;
                    } else {
                        phi_2165_ = (_e189 - (_e250 + 2f));
                    }
                    let _e265 = phi_2165_;
                    phi_2221_ = select(_e139, 0f, _e261);
                    phi_2169_ = _e242;
                    phi_2166_ = select(_e249, 0f, _e261);
                    phi_2163_ = _e265;
                }
                let _e269 = phi_2221_;
                let _e271 = phi_2169_;
                let _e273 = phi_2166_;
                let _e275 = phi_2163_;
                if (_e275 == _e273) {
                    phi_2173_ = _e234;
                } else {
                    phi_2173_ = (_e232 + (_e271 * (_e275 / _e273)));
                }
                let _e281 = phi_2173_;
                phi_2219_ = _e269;
                phi_2172_ = _e281;
            } else {
                phi_2219_ = _e139;
                phi_2172_ = bitcast<f32>(_e180.z);
            }
            let _e285 = phi_2219_;
            let _e287 = phi_2172_;
            let _e291 = vec2<f32>(sin(_e287), -(cos(_e287)));
            let _e293 = bitcast<vec2<f32>>(_e180.xy);
            phi_2228_ = _e132;
            if (_e132 != 0f) {
                phi_2228_ = max(_e132, (1f / length((_e122 * _e291))));
            }
            let _e300 = phi_2228_;
            if (_e130 != 0f) {
                let _e304 = (_e285 * sign(determinant(_e122)));
                let _e306 = ((_e182 & 1048576u) != 0u);
                phi_2225_ = _e304;
                if _e306 {
                    phi_2225_ = min(_e304, 0f);
                }
                let _e309 = phi_2225_;
                phi_2282_ = _e309;
                if ((_e182 & 524288u) != 0u) {
                    phi_2282_ = max(_e309, 0f);
                }
                let _e314 = phi_2282_;
                let _e316 = select(0f, _e300, (_e300 != 0f));
                let _e320 = select(_e130, _e316, ((_e316 > _e130) && (_e300 == 0f)));
                let _e321 = (_e320 + _e316);
                let _e322 = (_e291 * _e321);
                phi_2290_ = _e322;
                if (_e183 > 134217728u) {
                    let _e324 = (_e182 & 4194304u);
                    let _e326 = select(2i, -2i, (_e324 == 0u));
                    phi_2254_ = _e326;
                    if ((_e182 & 8388608u) != 0u) {
                        phi_2254_ = -(_e326);
                    }
                    let _e331 = phi_2254_;
                    let _e332 = (_e178 + _e331);
                    let _e337 = textureLoad(JC, vec2<i32>((_e332 & 2047i), (_e332 >> bitcast<u32>(11i))), 0i);
                    let _e341 = abs((bitcast<f32>(_e337.z) - _e287));
                    phi_2264_ = _e341;
                    if (_e341 > 3.1415927f) {
                        phi_2264_ = (6.2831855f - _e341);
                    }
                    let _e345 = phi_2264_;
                    let _e350 = ((_e345 * select(0.5f, -0.5f, ((_e324 != 0u) == _e306))) + _e287);
                    let _e354 = vec2<f32>(sin(_e350), -(cos(_e350)));
                    let _e355 = (_e122 * _e354);
                    let _e365 = cos((_e345 * 0.5f));
                    let _e366 = (_e183 == 335544320u);
                    phi_1691_ = _e366;
                    if !(_e366) {
                        phi_1691_ = ((_e183 == 268435456u) && (_e365 >= 0.25f));
                    }
                    let _e372 = phi_1691_;
                    if _e372 {
                        phi_2271_ = (_e320 * (1f / max(_e365, select(0.25f, 1f, ((_e182 & 33554432u) != 0u)))));
                    } else {
                        phi_2271_ = ((_e320 * _e365) + (((abs(_e355.x) + abs(_e355.y)) * (1f / dot(_e355, _e355))) * 0.5f));
                    }
                    let _e383 = phi_2271_;
                    phi_2291_ = _e322;
                    if ((_e182 & 2097152u) != 0u) {
                        if (_e321 <= ((_e383 * _e365) + (_e316 * 0.125f))) {
                            phi_2292_ = (_e354 * (_e321 * (1f / _e365)));
                        } else {
                            let _e393 = (_e354 * _e383);
                            phi_2292_ = (vec2<f32>(dot(_e322, _e322), dot(_e393, _e393)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e322, _e393)));
                        }
                        let _e401 = phi_2292_;
                        phi_2291_ = _e401;
                    }
                    let _e403 = phi_2291_;
                    phi_2290_ = _e403;
                }
                let _e405 = phi_2290_;
                if (_e89 != 0i) {
                    phi_2341_ = u32();
                    phi_2312_ = vec2<f32>();
                    phi_2311_ = false;
                    break;
                }
                phi_2308_ = (_e122 * (_e405 * _e314));
                phi_2293_ = _e293;
            } else {
                if (((_e182 & 2147483648u) != 0u) && (_e89 != 1i)) {
                    phi_2341_ = u32();
                    phi_2312_ = vec2<f32>();
                    phi_2311_ = false;
                    break;
                }
                phi_2308_ = vec2<f32>(0f, 0f);
                phi_2293_ = select(_e293, _e107, vec2((_e89 == 2i)));
            }
            let _e417 = phi_2308_;
            let _e419 = phi_2293_;
            let _e426 = OB.g2_[(_e111 + 2u)];
            phi_2341_ = _e426.x;
            phi_2312_ = (((_e122 * _e419) + _e417) + bitcast<vec2<f32>>(_e126.xy));
            phi_2311_ = true;
            break;
        }
    }
    let _e429 = phi_2341_;
    let _e431 = phi_2312_;
    let _e433 = phi_2311_;
    let _e436 = local;
    let _e438 = CD.g2_[_e436];
    let _e440 = (_e438.x & 15u);
    if Jh {
        let _e441 = (_e440 == 0u);
        if _e441 {
            phi_2370_ = _e438.y;
        } else {
            phi_2370_ = _e438.x;
        }
        let _e444 = phi_2370_;
        let _e446 = (_e444 >> bitcast<u32>(16i));
        let _e448 = j.g6_;
        if (_e446 == 0u) {
            phi_2371_ = 0f;
        } else {
            phi_2371_ = unpack2x16float(((_e446 + 1023u) * _e448)).x;
        }
        let _e455 = phi_2371_;
        phi_2372_ = _e455;
        if _e441 {
            phi_2372_ = -(_e455);
        }
        let _e458 = phi_2372_;
        Y1_[0u] = _e458;
    }
    if Lh {
        g1_ = f32(((_e438.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e440 == 1u) {
        X1_ = unpack4x8unorm(_e438.y);
    } else {
        if (Jh && (_e440 == 0u)) {
            let _e470 = (_e438.x >> bitcast<u32>(16i));
            let _e472 = j.g6_;
            if (_e470 == 0u) {
                phi_2410_ = 0f;
            } else {
                phi_2410_ = unpack2x16float(((_e470 + 1023u) * _e472)).x;
            }
            let _e479 = phi_2410_;
            Y1_[1u] = _e479;
        } else {
            let _e482 = local_1;
            let _e483 = (_e482 * 8u);
            let _e486 = PB.g2_[_e483];
            let _e497 = PB.g2_[(_e483 + 1u)];
            let _e506 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e438.y));
            let _e508 = ((mat2x2<f32>(vec2<f32>(_e486.x, _e486.y), vec2<f32>(_e486.z, _e486.w)) * _e431) + _e497.xy);
            if (_e497.z > 0.9f) {
                phi_2408_ = vec4<f32>(_e506.x, _e506.y, 2f, _e506.w);
            } else {
                phi_2408_ = vec4<f32>(_e506.x, _e506.y, _e497.w, _e506.w);
            }
            let _e523 = phi_2408_;
            if (f32(_e440) == 2f) {
                let _e530 = vec4<f32>(_e508.x, _e523.y, _e523.z, _e523.w);
                phi_2409_ = vec4<f32>(_e530.x, 0f, _e530.z, _e530.w);
            } else {
                let _e542 = vec4<f32>(_e523.x, _e523.y, -(_e523.z), _e523.w);
                let _e548 = vec4<f32>(_e508.x, _e542.y, _e542.z, _e542.w);
                phi_2409_ = vec4<f32>(_e548.x, _e508.y, _e548.z, _e548.w);
            }
            let _e556 = phi_2409_;
            X1_ = _e556;
            let _e558 = X1_[3u];
            X1_[3u] = -(_e558);
        }
    }
    phi_1091_ = Rh;
    if Rh {
        phi_1091_ = ((_e438.x & 2048u) != 0u);
    }
    let _e563 = phi_1091_;
    if _e563 {
        let _e565 = local_2;
        let _e566 = (_e565 * 8u);
        let _e570 = PB.g2_[(_e566 + 4u)];
        let _e581 = PB.g2_[(_e566 + 5u)];
        let _e584 = ((mat2x2<f32>(vec2<f32>(_e570.x, _e570.y), vec2<f32>(_e570.z, _e570.w)) * _e431) + _e581.xy);
        C2_ = vec3<f32>(_e584.x, _e584.y, (1f + _e581.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e433 {
        let _e591 = j.Mf;
        let _e593 = j.Nf;
        let _e601 = vec4<f32>(((_e431.x * _e591) - 1f), ((_e431.y * _e593) - sign(_e593)), 0f, 1f);
        phi_2423_ = vec4<f32>(_e601.x, _e601.y, (1f - (f32(_e429) * 0.000061035156f)), _e601.w);
    } else {
        let _e611 = j.W2_;
        phi_2423_ = vec4(_e611);
    }
    let _e614 = phi_2423_;
    unnamed.gl_Position = _e614;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
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
