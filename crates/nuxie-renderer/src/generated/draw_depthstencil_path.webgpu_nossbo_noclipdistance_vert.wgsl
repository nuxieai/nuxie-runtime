struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
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

@id(0) override Hh: bool = true;
@id(2) override Jh: bool = true;
@id(8) override Ph: bool = true;

@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(5)
var HD: texture_2d<u32>;
@group(0) @binding(2)
var OB: texture_2d<u32>;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
@group(0) @binding(3)
var CD: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: TB;
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
var ca: sampler;

fn main_1() {
    var phi_2270_: f32;
    var phi_2242_: i32;
    var phi_1475_: bool;
    var phi_2255_: i32;
    var phi_2247_: vec4<u32>;
    var phi_2254_: i32;
    var phi_2246_: vec4<u32>;
    var phi_2253_: i32;
    var phi_2251_: vec4<u32>;
    var phi_2250_: u32;
    var phi_2257_: vec2<i32>;
    var phi_2258_: vec4<u32>;
    var phi_2262_: f32;
    var phi_2333_: f32;
    var phi_2276_: f32;
    var phi_2332_: f32;
    var phi_2280_: f32;
    var phi_2277_: f32;
    var phi_2274_: f32;
    var phi_2284_: f32;
    var phi_2330_: f32;
    var phi_2283_: f32;
    var phi_2339_: f32;
    var phi_2336_: f32;
    var phi_2393_: f32;
    var phi_2365_: i32;
    var phi_2375_: f32;
    var phi_1787_: bool;
    var phi_2382_: f32;
    var phi_2403_: vec2<f32>;
    var phi_2402_: vec2<f32>;
    var phi_2401_: vec2<f32>;
    var phi_2426_: bool;
    var phi_2421_: vec2<f32>;
    var phi_2404_: vec2<f32>;
    var phi_2451_: u32;
    var phi_2452_: f32;
    var phi_2453_: f32;
    var phi_2492_: f32;
    var phi_2490_: vec4<f32>;
    var phi_2491_: vec4<f32>;
    var phi_1157_: bool;
    var phi_2505_: vec4<f32>;

    let _e79 = gl_InstanceIndex_1;
    let _e80 = VB_1;
    let _e81 = WB_1;
    let _e83 = i32(_e80.x);
    let _e86 = bitcast<i32>(_e80.w);
    let _e88 = (_e86 >> bitcast<u32>(2i));
    let _e89 = (_e86 & 3i);
    let _e91 = min(_e83, (_e88 - 1i));
    let _e93 = ((_e79 * _e88) + _e91);
    let _e98 = textureLoad(JC, vec2<i32>((_e93 & 2047i), (_e93 >> bitcast<u32>(11i))), 0i);
    let _e102 = (max((_e98.w & 65535u), 1u) - 1u);
    let _e109 = textureLoad(HD, vec2<i32>(bitcast<i32>((_e102 & 255u)), bitcast<i32>((_e102 >> bitcast<u32>(8i)))), 0i);
    let _e111 = bitcast<vec2<f32>>(_e109.xy);
    let _e113 = (_e109.z & 65535u);
    let _e115 = (_e113 * 4u);
    let _e122 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e115 & 255u)), bitcast<i32>((_e115 >> bitcast<u32>(8i)))), 0i);
    let _e123 = bitcast<vec4<f32>>(_e122);
    let _e130 = mat2x2<f32>(vec2<f32>(_e123.x, _e123.y), vec2<f32>(_e123.z, _e123.w));
    let _e131 = (_e115 + 1u);
    let _e138 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e131 & 255u)), bitcast<i32>((_e131 >> bitcast<u32>(8i)))), 0i);
    let _e142 = bitcast<f32>(_e138.z);
    let _e144 = bitcast<f32>(_e138.w);
    let _e145 = (_e98.w & 8388608u);
    phi_2270_ = _e80.y;
    phi_2242_ = _e83;
    if (_e145 != 0u) {
        phi_2270_ = _e81.y;
        phi_2242_ = i32(_e81.x);
    }
    let _e151 = phi_2270_;
    let _e153 = phi_2242_;
    phi_2253_ = _e93;
    phi_2251_ = _e98;
    phi_2250_ = _e98.w;
    if (_e153 != _e91) {
        let _e156 = ((_e93 + _e153) - _e91);
        let _e161 = textureLoad(JC, vec2<i32>((_e156 & 2047i), (_e156 >> bitcast<u32>(11i))), 0i);
        if ((_e161.w & 8454143u) != (_e98.w & 8454143u)) {
            let _e166 = (_e142 == 0f);
            phi_1475_ = _e166;
            if !(_e166) {
                phi_1475_ = (_e111.x != 0f);
            }
            let _e171 = phi_1475_;
            phi_2255_ = _e93;
            phi_2247_ = _e98;
            if _e171 {
                let _e172 = bitcast<i32>(_e109.w);
                let _e177 = textureLoad(JC, vec2<i32>((_e172 & 2047i), (_e172 >> bitcast<u32>(11i))), 0i);
                phi_2255_ = _e172;
                phi_2247_ = _e177;
            }
            let _e179 = phi_2255_;
            let _e181 = phi_2247_;
            phi_2254_ = _e179;
            phi_2246_ = _e181;
        } else {
            phi_2254_ = _e156;
            phi_2246_ = _e161;
        }
        let _e183 = phi_2254_;
        let _e185 = phi_2246_;
        phi_2253_ = _e183;
        phi_2251_ = _e185;
        phi_2250_ = ((_e185.w & 4286578687u) | _e145);
    }
    let _e190 = phi_2253_;
    let _e192 = phi_2251_;
    let _e194 = phi_2250_;
    let _e195 = (_e194 & 469762048u);
    if ((_e195 == 67108864u) && (_e89 == 0i)) {
        let _e201 = f32((_e192.z & 65535u));
        let _e204 = f32((_e192.z >> bitcast<u32>(16i)));
        let _e210 = vec2<i32>(i32((-1f - _e201)), i32(((_e204 - _e201) + 1f)));
        phi_2257_ = _e210;
        if ((_e194 & 8388608u) != 0u) {
            phi_2257_ = -(_e210);
        }
        let _e215 = phi_2257_;
        let _e217 = (_e190 + _e215.x);
        let _e222 = textureLoad(JC, vec2<i32>((_e217 & 2047i), (_e217 >> bitcast<u32>(11i))), 0i);
        let _e224 = (_e190 + _e215.y);
        let _e229 = textureLoad(JC, vec2<i32>((_e224 & 2047i), (_e224 >> bitcast<u32>(11i))), 0i);
        phi_2258_ = _e229;
        if ((_e229.w & 8454143u) != (_e222.w & 8454143u)) {
            let _e235 = bitcast<i32>(_e109.w);
            let _e240 = textureLoad(JC, vec2<i32>((_e235 & 2047i), (_e235 >> bitcast<u32>(11i))), 0i);
            phi_2258_ = _e240;
        }
        let _e242 = phi_2258_;
        let _e244 = bitcast<f32>(_e222.z);
        let _e246 = bitcast<f32>(_e242.z);
        let _e247 = (_e246 - _e244);
        phi_2262_ = _e247;
        if (abs(_e247) > 3.1415927f) {
            phi_2262_ = (_e247 - (6.2831855f * sign(_e247)));
        }
        let _e254 = phi_2262_;
        let _e255 = (_e204 + -2f);
        let _e261 = clamp(round(((abs(_e254) * 0.31830987f) * _e255)), 1f, (_e204 + -3f));
        let _e262 = (_e255 - _e261);
        if (_e201 <= _e262) {
            phi_2333_ = _e151;
            if (_e201 == _e262) {
                phi_2333_ = -(_e151);
            }
            let _e271 = phi_2333_;
            phi_2332_ = _e271;
            phi_2280_ = -(((3.1415927f * sign(_e254)) - _e254));
            phi_2277_ = _e262;
            phi_2274_ = _e201;
        } else {
            let _e273 = (_e201 == (_e262 + 1f));
            if _e273 {
                phi_2276_ = 0f;
            } else {
                phi_2276_ = (_e201 - (_e262 + 2f));
            }
            let _e277 = phi_2276_;
            phi_2332_ = select(_e151, 0f, _e273);
            phi_2280_ = _e254;
            phi_2277_ = select(_e261, 0f, _e273);
            phi_2274_ = _e277;
        }
        let _e281 = phi_2332_;
        let _e283 = phi_2280_;
        let _e285 = phi_2277_;
        let _e287 = phi_2274_;
        if (_e287 == _e285) {
            phi_2284_ = _e246;
        } else {
            phi_2284_ = (_e244 + (_e283 * (_e287 / _e285)));
        }
        let _e293 = phi_2284_;
        phi_2330_ = _e281;
        phi_2283_ = _e293;
    } else {
        phi_2330_ = _e151;
        phi_2283_ = bitcast<f32>(_e192.z);
    }
    let _e297 = phi_2330_;
    let _e299 = phi_2283_;
    let _e303 = vec2<f32>(sin(_e299), -(cos(_e299)));
    let _e305 = bitcast<vec2<f32>>(_e192.xy);
    phi_2339_ = _e144;
    if (_e144 != 0f) {
        phi_2339_ = max(_e144, (1f / length((_e130 * _e303))));
    }
    let _e312 = phi_2339_;
    if (_e142 != 0f) {
        let _e316 = (_e297 * sign(determinant(_e130)));
        let _e318 = ((_e194 & 1048576u) != 0u);
        phi_2336_ = _e316;
        if _e318 {
            phi_2336_ = min(_e316, 0f);
        }
        let _e321 = phi_2336_;
        phi_2393_ = _e321;
        if ((_e194 & 524288u) != 0u) {
            phi_2393_ = max(_e321, 0f);
        }
        let _e326 = phi_2393_;
        let _e328 = select(0f, _e312, (_e312 != 0f));
        let _e332 = select(_e142, _e328, ((_e328 > _e142) && (_e312 == 0f)));
        let _e333 = (_e332 + _e328);
        let _e334 = (_e303 * _e333);
        phi_2401_ = _e334;
        if (_e195 > 134217728u) {
            let _e336 = (_e194 & 4194304u);
            let _e338 = select(2i, -2i, (_e336 == 0u));
            phi_2365_ = _e338;
            if ((_e194 & 8388608u) != 0u) {
                phi_2365_ = -(_e338);
            }
            let _e343 = phi_2365_;
            let _e344 = (_e190 + _e343);
            let _e349 = textureLoad(JC, vec2<i32>((_e344 & 2047i), (_e344 >> bitcast<u32>(11i))), 0i);
            let _e353 = abs((bitcast<f32>(_e349.z) - _e299));
            phi_2375_ = _e353;
            if (_e353 > 3.1415927f) {
                phi_2375_ = (6.2831855f - _e353);
            }
            let _e357 = phi_2375_;
            let _e362 = ((_e357 * select(0.5f, -0.5f, ((_e336 != 0u) == _e318))) + _e299);
            let _e366 = vec2<f32>(sin(_e362), -(cos(_e362)));
            let _e367 = (_e130 * _e366);
            let _e377 = cos((_e357 * 0.5f));
            let _e378 = (_e195 == 335544320u);
            phi_1787_ = _e378;
            if !(_e378) {
                phi_1787_ = ((_e195 == 268435456u) && (_e377 >= 0.25f));
            }
            let _e384 = phi_1787_;
            if _e384 {
                phi_2382_ = (_e332 * (1f / max(_e377, select(0.25f, 1f, ((_e194 & 33554432u) != 0u)))));
            } else {
                phi_2382_ = ((_e332 * _e377) + (((abs(_e367.x) + abs(_e367.y)) * (1f / dot(_e367, _e367))) * 0.5f));
            }
            let _e395 = phi_2382_;
            phi_2402_ = _e334;
            if ((_e194 & 2097152u) != 0u) {
                if (_e333 <= ((_e395 * _e377) + (_e328 * 0.125f))) {
                    phi_2403_ = (_e366 * (_e333 * (1f / _e377)));
                } else {
                    let _e405 = (_e366 * _e395);
                    phi_2403_ = (vec2<f32>(dot(_e334, _e334), dot(_e405, _e405)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e334, _e405)));
                }
                let _e413 = phi_2403_;
                phi_2402_ = _e413;
            }
            let _e415 = phi_2402_;
            phi_2401_ = _e415;
        }
        let _e417 = phi_2401_;
        phi_2426_ = (_e89 != 0i);
        phi_2421_ = (_e130 * (_e417 * _e326));
        phi_2404_ = _e305;
    } else {
        phi_2426_ = (((_e194 & 2147483648u) != 0u) && (_e89 != 1i));
        phi_2421_ = vec2<f32>(0f, 0f);
        phi_2404_ = select(_e305, _e111, vec2((_e89 == 2i)));
    }
    let _e429 = phi_2426_;
    let _e431 = phi_2421_;
    let _e433 = phi_2404_;
    let _e436 = (((_e130 * _e433) + _e431) + bitcast<vec2<f32>>(_e138.xy));
    let _e437 = (_e115 + 2u);
    let _e444 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e437 & 255u)), bitcast<i32>((_e437 >> bitcast<u32>(8i)))), 0i);
    let _e452 = textureLoad(CD, vec2<i32>(bitcast<i32>((_e109.z & 255u)), bitcast<i32>((_e113 >> bitcast<u32>(8i)))), 0i);
    let _e454 = (_e452.x & 15u);
    if Hh {
        let _e455 = (_e454 == 0u);
        if _e455 {
            phi_2451_ = _e452.y;
        } else {
            phi_2451_ = _e452.x;
        }
        let _e458 = phi_2451_;
        let _e460 = (_e458 >> bitcast<u32>(16i));
        let _e462 = j.c6_;
        if (_e460 == 0u) {
            phi_2452_ = 0f;
        } else {
            phi_2452_ = unpack2x16float(((_e460 + 1023u) * _e462)).x;
        }
        let _e469 = phi_2452_;
        phi_2453_ = _e469;
        if _e455 {
            phi_2453_ = -(_e469);
        }
        let _e472 = phi_2453_;
        Y1_[0u] = _e472;
    }
    if Jh {
        g1_ = f32(((_e452.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e454 == 1u) {
        X1_ = unpack4x8unorm(_e452.y);
    } else {
        if (Hh && (_e454 == 0u)) {
            let _e484 = (_e452.x >> bitcast<u32>(16i));
            let _e486 = j.c6_;
            if (_e484 == 0u) {
                phi_2492_ = 0f;
            } else {
                phi_2492_ = unpack2x16float(((_e484 + 1023u) * _e486)).x;
            }
            let _e493 = phi_2492_;
            Y1_[1u] = _e493;
        } else {
            let _e495 = (_e113 * 8u);
            let _e502 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e495 & 255u)), bitcast<i32>((_e495 >> bitcast<u32>(8i)))), 0i);
            let _e510 = (_e495 + 1u);
            let _e517 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e510 & 255u)), bitcast<i32>((_e510 >> bitcast<u32>(8i)))), 0i);
            let _e526 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e452.y));
            let _e528 = ((mat2x2<f32>(vec2<f32>(_e502.x, _e502.y), vec2<f32>(_e502.z, _e502.w)) * _e436) + _e517.xy);
            if (_e517.z > 0.9f) {
                phi_2490_ = vec4<f32>(_e526.x, _e526.y, 2f, _e526.w);
            } else {
                phi_2490_ = vec4<f32>(_e526.x, _e526.y, _e517.w, _e526.w);
            }
            let _e543 = phi_2490_;
            if (f32(_e454) == 2f) {
                let _e550 = vec4<f32>(_e528.x, _e543.y, _e543.z, _e543.w);
                phi_2491_ = vec4<f32>(_e550.x, 0f, _e550.z, _e550.w);
            } else {
                let _e562 = vec4<f32>(_e543.x, _e543.y, -(_e543.z), _e543.w);
                let _e568 = vec4<f32>(_e528.x, _e562.y, _e562.z, _e562.w);
                phi_2491_ = vec4<f32>(_e568.x, _e528.y, _e568.z, _e568.w);
            }
            let _e576 = phi_2491_;
            X1_ = _e576;
            let _e578 = X1_[3u];
            X1_[3u] = -(_e578);
        }
    }
    phi_1157_ = Ph;
    if Ph {
        phi_1157_ = ((_e452.x & 2048u) != 0u);
    }
    let _e583 = phi_1157_;
    if _e583 {
        let _e584 = (_e113 * 8u);
        let _e585 = (_e584 + 4u);
        let _e592 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e585 & 255u)), bitcast<i32>((_e585 >> bitcast<u32>(8i)))), 0i);
        let _e600 = (_e584 + 5u);
        let _e607 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e600 & 255u)), bitcast<i32>((_e600 >> bitcast<u32>(8i)))), 0i);
        let _e610 = ((mat2x2<f32>(vec2<f32>(_e592.x, _e592.y), vec2<f32>(_e592.z, _e592.w)) * _e436) + _e607.xy);
        C2_ = vec3<f32>(_e610.x, _e610.y, (1f + _e607.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e429) {
        let _e618 = j.Hf;
        let _e620 = j.If;
        let _e628 = vec4<f32>(((_e436.x * _e618) - 1f), ((_e436.y * _e620) - sign(_e620)), 0f, 1f);
        phi_2505_ = vec4<f32>(_e628.x, _e628.y, ((f32(((_e444.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e628.w);
    } else {
        let _e641 = j.W2_;
        phi_2505_ = vec4(_e641);
    }
    let _e644 = phi_2505_;
    unnamed.gl_Position = _e644;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) VB: vec4<f32>, @location(1) WB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    VB_1 = VB;
    WB_1 = WB;
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
