enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct nh {
    k2_: array<vec4<u32>>,
}

struct mh {
    k2_: array<vec4<u32>>,
}

struct Ef {
    k2_: array<vec2<u32>>,
}

struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct Ff {
    k2_: array<vec4<f32>>,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override ki: bool = true;
@id(2) override mi: bool = true;
@id(1) override li: bool = true;
@id(8) override si: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ZC: nh;
@group(0) @binding(2)
var<storage> LB: mh;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> WB_1: vec4<f32>;
var<private> XB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> WC: Ef;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> l1_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: Ff;
var<private> a1_: vec4<f32>;
var<private> r1_: vec3<f32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_2262_: f32;
    var phi_2234_: i32;
    var phi_1438_: bool;
    var phi_2247_: i32;
    var phi_2239_: vec4<u32>;
    var phi_2246_: i32;
    var phi_2238_: vec4<u32>;
    var phi_2245_: i32;
    var phi_2243_: vec4<u32>;
    var phi_2242_: u32;
    var phi_2249_: vec2<i32>;
    var phi_2250_: vec4<u32>;
    var phi_2254_: f32;
    var phi_2325_: f32;
    var phi_2268_: f32;
    var phi_2324_: f32;
    var phi_2272_: f32;
    var phi_2269_: f32;
    var phi_2266_: f32;
    var phi_2276_: f32;
    var phi_2322_: f32;
    var phi_2275_: f32;
    var phi_2331_: f32;
    var phi_2328_: f32;
    var phi_2360_: f32;
    var phi_2346_: f32;
    var phi_1726_: bool;
    var phi_2351_: f32;
    var phi_2368_: vec2<f32>;
    var phi_2367_: vec2<f32>;
    var phi_2366_: vec2<f32>;
    var phi_2389_: bool;
    var phi_2384_: vec2<f32>;
    var phi_2369_: vec2<f32>;
    var phi_2412_: u32;
    var phi_2413_: f32;
    var phi_2414_: f32;
    var phi_2453_: f32;
    var phi_2451_: vec4<f32>;
    var phi_2452_: vec4<f32>;
    var phi_1141_: bool;
    var phi_2454_: f32;
    var phi_2470_: vec4<f32>;

    let _e84 = gl_InstanceIndex_1;
    let _e85 = WB_1;
    let _e86 = XB_1;
    let _e88 = i32(_e85.x);
    let _e91 = bitcast<i32>(_e85.w);
    let _e93 = (_e91 >> bitcast<u32>(2i));
    let _e94 = (_e91 & 3i);
    let _e96 = min(_e88, (_e93 - 1i));
    let _e98 = ((_e84 * _e93) + _e96);
    let _e103 = textureLoad(TB, vec2<i32>((_e98 & 2047i), (_e98 >> bitcast<u32>(11i))), 0i);
    let _e110 = ZC.k2_[(max((_e103.w & 65535u), 1u) - 1u)];
    let _e112 = bitcast<vec2<f32>>(_e110.xy);
    let _e114 = (_e110.z & 65535u);
    let _e116 = (_e114 * 4u);
    let _e119 = LB.k2_[_e116];
    let _e120 = bitcast<vec4<f32>>(_e119);
    let _e127 = mat2x2<f32>(vec2<f32>(_e120.x, _e120.y), vec2<f32>(_e120.z, _e120.w));
    let _e131 = LB.k2_[(_e116 + 1u)];
    let _e135 = bitcast<f32>(_e131.z);
    let _e137 = bitcast<f32>(_e131.w);
    let _e138 = (_e103.w & 8388608u);
    phi_2262_ = _e85.y;
    phi_2234_ = _e88;
    if (_e138 != 0u) {
        phi_2262_ = _e86.y;
        phi_2234_ = i32(_e86.x);
    }
    let _e144 = phi_2262_;
    let _e146 = phi_2234_;
    phi_2245_ = _e98;
    phi_2243_ = _e103;
    phi_2242_ = _e103.w;
    if (_e146 != _e96) {
        let _e149 = ((_e98 + _e146) - _e96);
        let _e154 = textureLoad(TB, vec2<i32>((_e149 & 2047i), (_e149 >> bitcast<u32>(11i))), 0i);
        if ((_e154.w & 8454143u) != (_e103.w & 8454143u)) {
            let _e159 = (_e135 == 0f);
            phi_1438_ = _e159;
            if !(_e159) {
                phi_1438_ = (_e112.x != 0f);
            }
            let _e164 = phi_1438_;
            phi_2247_ = _e98;
            phi_2239_ = _e103;
            if _e164 {
                let _e165 = bitcast<i32>(_e110.w);
                let _e170 = textureLoad(TB, vec2<i32>((_e165 & 2047i), (_e165 >> bitcast<u32>(11i))), 0i);
                phi_2247_ = _e165;
                phi_2239_ = _e170;
            }
            let _e172 = phi_2247_;
            let _e174 = phi_2239_;
            phi_2246_ = _e172;
            phi_2238_ = _e174;
        } else {
            phi_2246_ = _e149;
            phi_2238_ = _e154;
        }
        let _e176 = phi_2246_;
        let _e178 = phi_2238_;
        phi_2245_ = _e176;
        phi_2243_ = _e178;
        phi_2242_ = ((_e178.w & 4286578687u) | _e138);
    }
    let _e183 = phi_2245_;
    let _e185 = phi_2243_;
    let _e187 = phi_2242_;
    let _e188 = (_e187 & 469762048u);
    if ((_e188 == 67108864u) && (_e94 == 0i)) {
        let _e194 = f32((_e185.z & 65535u));
        let _e197 = f32((_e185.z >> bitcast<u32>(16i)));
        let _e203 = vec2<i32>(i32((-1f - _e194)), i32(((_e197 - _e194) + 1f)));
        phi_2249_ = _e203;
        if ((_e187 & 8388608u) != 0u) {
            phi_2249_ = -(_e203);
        }
        let _e208 = phi_2249_;
        let _e210 = (_e183 + _e208.x);
        let _e215 = textureLoad(TB, vec2<i32>((_e210 & 2047i), (_e210 >> bitcast<u32>(11i))), 0i);
        let _e217 = (_e183 + _e208.y);
        let _e222 = textureLoad(TB, vec2<i32>((_e217 & 2047i), (_e217 >> bitcast<u32>(11i))), 0i);
        phi_2250_ = _e222;
        if ((_e222.w & 8454143u) != (_e215.w & 8454143u)) {
            let _e228 = bitcast<i32>(_e110.w);
            let _e233 = textureLoad(TB, vec2<i32>((_e228 & 2047i), (_e228 >> bitcast<u32>(11i))), 0i);
            phi_2250_ = _e233;
        }
        let _e235 = phi_2250_;
        let _e238 = (f32(_e215.z) * 0.0000000014629181f);
        let _e241 = (f32(_e235.z) * 0.0000000014629181f);
        let _e242 = (_e241 - _e238);
        phi_2254_ = _e242;
        if (abs(_e242) > 3.1415927f) {
            phi_2254_ = (_e242 - (6.2831855f * sign(_e242)));
        }
        let _e249 = phi_2254_;
        let _e250 = (_e197 + -2f);
        let _e256 = clamp(round(((abs(_e249) * 0.31830987f) * _e250)), 1f, (_e197 + -3f));
        let _e257 = (_e250 - _e256);
        if (_e194 <= _e257) {
            phi_2325_ = _e144;
            if (_e194 == _e257) {
                phi_2325_ = -(_e144);
            }
            let _e266 = phi_2325_;
            phi_2324_ = _e266;
            phi_2272_ = -(((3.1415927f * sign(_e249)) - _e249));
            phi_2269_ = _e257;
            phi_2266_ = _e194;
        } else {
            let _e268 = (_e194 == (_e257 + 1f));
            if _e268 {
                phi_2268_ = 0f;
            } else {
                phi_2268_ = (_e194 - (_e257 + 2f));
            }
            let _e272 = phi_2268_;
            phi_2324_ = select(_e144, 0f, _e268);
            phi_2272_ = _e249;
            phi_2269_ = select(_e256, 0f, _e268);
            phi_2266_ = _e272;
        }
        let _e276 = phi_2324_;
        let _e278 = phi_2272_;
        let _e280 = phi_2269_;
        let _e282 = phi_2266_;
        if (_e282 == _e280) {
            phi_2276_ = _e241;
        } else {
            phi_2276_ = (_e238 + (_e278 * (_e282 / _e280)));
        }
        let _e288 = phi_2276_;
        phi_2322_ = _e276;
        phi_2275_ = _e288;
    } else {
        phi_2322_ = _e144;
        phi_2275_ = (f32(_e185.z) * 0.0000000014629181f);
    }
    let _e293 = phi_2322_;
    let _e295 = phi_2275_;
    let _e299 = vec2<f32>(sin(_e295), -(cos(_e295)));
    let _e301 = bitcast<vec2<f32>>(_e185.xy);
    phi_2331_ = _e137;
    if (_e137 != 0f) {
        phi_2331_ = max(_e137, (1f / length((_e127 * _e299))));
    }
    let _e308 = phi_2331_;
    if (_e135 != 0f) {
        let _e312 = (_e293 * sign(determinant(_e127)));
        let _e314 = ((_e187 & 1048576u) != 0u);
        phi_2328_ = _e312;
        if _e314 {
            phi_2328_ = min(_e312, 0f);
        }
        let _e317 = phi_2328_;
        phi_2360_ = _e317;
        if ((_e187 & 524288u) != 0u) {
            phi_2360_ = max(_e317, 0f);
        }
        let _e322 = phi_2360_;
        let _e324 = select(0f, _e308, (_e308 != 0f));
        let _e328 = select(_e135, _e324, ((_e324 > _e135) && (_e308 == 0f)));
        let _e329 = (_e328 + _e324);
        let _e330 = (_e299 * _e329);
        phi_2366_ = _e330;
        if (_e188 > 134217728u) {
            let _e336 = f32((_e185.z & 65535u));
            let _e337 = (_e336 * 0.000015259022f);
            let _e341 = sqrt(max((1f - (_e337 * _e337)), 0f));
            phi_2346_ = _e341;
            if (((_e187 & 4194304u) != 0u) == _e314) {
                phi_2346_ = -(_e341);
            }
            let _e345 = phi_2346_;
            let _e350 = (mat2x2<f32>(vec2<f32>(_e337, _e345), vec2<f32>(-(_e345), _e337)) * _e299);
            let _e351 = (_e127 * _e350);
            let _e360 = (_e188 == 335544320u);
            phi_1726_ = _e360;
            if !(_e360) {
                phi_1726_ = ((_e188 == 268435456u) && (_e337 >= 0.25f));
            }
            let _e366 = phi_1726_;
            if _e366 {
                phi_2351_ = (_e328 * (1f / max(_e337, select(0.25f, 1f, ((_e187 & 33554432u) != 0u)))));
            } else {
                phi_2351_ = ((_e328 * _e337) + (((abs(_e351.x) + abs(_e351.y)) * (1f / dot(_e351, _e351))) * 0.5f));
            }
            let _e377 = phi_2351_;
            phi_2367_ = _e330;
            if ((_e187 & 2097152u) != 0u) {
                if (_e329 <= ((_e377 * _e337) + (_e324 * 0.125f))) {
                    phi_2368_ = (_e350 * (_e329 * (65535f / _e336)));
                } else {
                    let _e387 = (_e350 * _e377);
                    phi_2368_ = (vec2<f32>(dot(_e330, _e330), dot(_e387, _e387)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e330, _e387)));
                }
                let _e395 = phi_2368_;
                phi_2367_ = _e395;
            }
            let _e397 = phi_2367_;
            phi_2366_ = _e397;
        }
        let _e399 = phi_2366_;
        phi_2389_ = (_e94 != 0i);
        phi_2384_ = (_e127 * (_e399 * _e322));
        phi_2369_ = _e301;
    } else {
        phi_2389_ = (((_e187 & 2147483648u) != 0u) && (_e94 != 1i));
        phi_2384_ = vec2<f32>(0f, 0f);
        phi_2369_ = select(_e301, _e112, vec2((_e94 == 2i)));
    }
    let _e411 = phi_2389_;
    let _e413 = phi_2384_;
    let _e415 = phi_2369_;
    let _e418 = (((_e127 * _e415) + _e413) + bitcast<vec2<f32>>(_e131.xy));
    let _e422 = LB.k2_[(_e116 + 2u)];
    let _e426 = WC.k2_[_e114];
    let _e428 = (_e426.x & 15u);
    if ki {
        let _e429 = (_e428 == 0u);
        if _e429 {
            phi_2412_ = _e426.y;
        } else {
            phi_2412_ = _e426.x;
        }
        let _e432 = phi_2412_;
        let _e434 = (_e432 >> bitcast<u32>(16i));
        let _e436 = j.T4_;
        if (_e434 == 0u) {
            phi_2413_ = 0f;
        } else {
            phi_2413_ = unpack2x16float(((_e434 + 1023u) * _e436)).x;
        }
        let _e443 = phi_2413_;
        phi_2414_ = _e443;
        if _e429 {
            phi_2414_ = -(_e443);
        }
        let _e446 = phi_2414_;
        l1_[0u] = _e446;
    }
    if mi {
        Q0_ = f32(((_e426.x >> bitcast<u32>(4i)) & 15u));
    }
    if li {
        let _e452 = (_e114 * 8u);
        let _e456 = JB.k2_[(_e452 + 2u)];
        let _e467 = JB.k2_[(_e452 + 3u)];
        if any((_e456 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e472 = ((mat2x2<f32>(vec2<f32>(_e456.x, _e456.y), vec2<f32>(_e456.z, _e456.w)) * _e418) + _e467.xy);
            unnamed.gl_ClipDistance[0i] = (_e472.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e472.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e472.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e472.y);
        } else {
            let _e488 = (_e467.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e488;
            unnamed.gl_ClipDistance[2i] = _e488;
            unnamed.gl_ClipDistance[1i] = _e488;
            unnamed.gl_ClipDistance[0i] = _e488;
        }
    }
    if (_e428 == 1u) {
        a1_ = unpack4x8unorm(_e426.y);
    } else {
        if (ki && (_e428 == 0u)) {
            let _e503 = (_e426.x >> bitcast<u32>(16i));
            let _e505 = j.T4_;
            if (_e503 == 0u) {
                phi_2453_ = 0f;
            } else {
                phi_2453_ = unpack2x16float(((_e503 + 1023u) * _e505)).x;
            }
            let _e512 = phi_2453_;
            l1_[1u] = _e512;
        } else {
            let _e514 = (_e114 * 8u);
            let _e517 = JB.k2_[_e514];
            let _e528 = JB.k2_[(_e514 + 1u)];
            let _e537 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e426.y));
            let _e539 = ((mat2x2<f32>(vec2<f32>(_e517.x, _e517.y), vec2<f32>(_e517.z, _e517.w)) * _e418) + _e528.xy);
            if (_e528.z > 0.9f) {
                phi_2451_ = vec4<f32>(_e537.x, _e537.y, 2f, _e537.w);
            } else {
                phi_2451_ = vec4<f32>(_e537.x, _e537.y, _e528.w, _e537.w);
            }
            let _e554 = phi_2451_;
            if (f32(_e428) == 2f) {
                let _e561 = vec4<f32>(_e539.x, _e554.y, _e554.z, _e554.w);
                phi_2452_ = vec4<f32>(_e561.x, 0f, _e561.z, _e561.w);
            } else {
                let _e573 = vec4<f32>(_e554.x, _e554.y, -(_e554.z), _e554.w);
                let _e579 = vec4<f32>(_e539.x, _e573.y, _e573.z, _e573.w);
                phi_2452_ = vec4<f32>(_e579.x, _e539.y, _e579.z, _e579.w);
            }
            let _e587 = phi_2452_;
            a1_ = _e587;
            let _e589 = a1_[3u];
            a1_[3u] = -(_e589);
        }
    }
    phi_1141_ = si;
    if si {
        phi_1141_ = ((_e426.x & 2048u) != 0u);
    }
    let _e594 = phi_1141_;
    if _e594 {
        let _e595 = (_e114 * 8u);
        let _e599 = JB.k2_[(_e595 + 4u)];
        let _e610 = JB.k2_[(_e595 + 5u)];
        let _e613 = ((mat2x2<f32>(vec2<f32>(_e599.x, _e599.y), vec2<f32>(_e599.z, _e599.w)) * _e418) + _e610.xy);
        phi_2454_ = (1f + _e610.z);
        if ((_e426.x & 4096u) != 0u) {
            phi_2454_ = (-1f - f32(((_e426.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e624 = phi_2454_;
        r1_ = vec3<f32>(_e613.x, _e613.y, _e624);
    } else {
        r1_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e411) {
        let _e630 = j.bg;
        let _e632 = j.cg;
        let _e640 = vec4<f32>(((_e418.x * _e630) - 1f), ((_e418.y * _e632) - sign(_e632)), 0f, 1f);
        phi_2470_ = vec4<f32>(_e640.x, _e640.y, ((f32(((_e422.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e640.w);
    } else {
        let _e653 = j.a3_;
        phi_2470_ = vec4(_e653);
    }
    let _e656 = phi_2470_;
    unnamed.gl_Position = _e656;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) WB: vec4<f32>, @location(1) XB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    WB_1 = WB;
    XB_1 = XB;
    main_1();
    let _e17 = unnamed.gl_Position;
    let _e18 = unnamed.gl_ClipDistance;
    let _e19 = l1_;
    let _e20 = Q0_;
    let _e21 = a1_;
    let _e22 = r1_;
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
