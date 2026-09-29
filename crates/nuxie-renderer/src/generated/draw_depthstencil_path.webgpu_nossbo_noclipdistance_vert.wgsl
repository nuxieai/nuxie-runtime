struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
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

@id(0) override jh: bool = true;
@id(2) override lh: bool = true;
@id(8) override rh: bool = true;

@group(0) @binding(7)
var KC: texture_2d<u32>;
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
var<uniform> m: BC;
var<private> V1_: vec2<f32>;
var<private> f2_: f32;
@group(0) @binding(4)
var QB: texture_2d<f32>;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_2193_: f32;
    var phi_2165_: i32;
    var phi_1457_: bool;
    var phi_2178_: i32;
    var phi_2170_: vec4<u32>;
    var phi_2177_: i32;
    var phi_2169_: vec4<u32>;
    var phi_2176_: i32;
    var phi_2174_: vec4<u32>;
    var phi_2173_: u32;
    var phi_2180_: vec2<i32>;
    var phi_2181_: vec4<u32>;
    var phi_2185_: f32;
    var phi_2256_: f32;
    var phi_2199_: f32;
    var phi_2255_: f32;
    var phi_2203_: f32;
    var phi_2200_: f32;
    var phi_2197_: f32;
    var phi_2207_: f32;
    var phi_2253_: f32;
    var phi_2206_: f32;
    var phi_2262_: f32;
    var phi_2259_: f32;
    var phi_2316_: f32;
    var phi_2288_: i32;
    var phi_2298_: f32;
    var phi_1769_: bool;
    var phi_2305_: f32;
    var phi_2326_: vec2<f32>;
    var phi_2325_: vec2<f32>;
    var phi_2324_: vec2<f32>;
    var phi_2342_: vec2<f32>;
    var phi_2327_: vec2<f32>;
    var phi_2375_: u32;
    var phi_2346_: vec2<f32>;
    var phi_2345_: bool;
    var local: u32;
    var local_1: u32;
    var phi_2404_: u32;
    var phi_2405_: f32;
    var phi_2406_: f32;
    var phi_2408_: vec4<f32>;
    var phi_2407_: f32;
    var local_2: u32;
    var phi_1134_: bool;
    var local_3: u32;
    var phi_2422_: vec4<f32>;

    let _e79 = gl_InstanceIndex_1;
    let _e80 = UB_1;
    let _e81 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e84 = i32(_e80.x);
            let _e87 = bitcast<i32>(_e80.w);
            let _e89 = (_e87 >> bitcast<u32>(2i));
            let _e90 = (_e87 & 3i);
            let _e92 = min(_e84, (_e89 - 1i));
            let _e94 = ((_e79 * _e89) + _e92);
            let _e99 = textureLoad(KC, vec2<i32>((_e94 & 2047i), (_e94 >> bitcast<u32>(11i))), 0i);
            let _e103 = (max((_e99.w & 65535u), 1u) - 1u);
            let _e110 = textureLoad(ED, vec2<i32>(bitcast<i32>((_e103 & 255u)), bitcast<i32>((_e103 >> bitcast<u32>(8i)))), 0i);
            let _e112 = bitcast<vec2<f32>>(_e110.xy);
            let _e114 = (_e110.z & 65535u);
            let _e116 = (_e114 * 4u);
            let _e123 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e116 & 255u)), bitcast<i32>((_e116 >> bitcast<u32>(8i)))), 0i);
            let _e124 = bitcast<vec4<f32>>(_e123);
            let _e131 = mat2x2<f32>(vec2<f32>(_e124.x, _e124.y), vec2<f32>(_e124.z, _e124.w));
            let _e132 = (_e116 + 1u);
            let _e139 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e132 & 255u)), bitcast<i32>((_e132 >> bitcast<u32>(8i)))), 0i);
            let _e143 = bitcast<f32>(_e139.z);
            let _e145 = bitcast<f32>(_e139.w);
            let _e146 = (_e99.w & 8388608u);
            phi_2193_ = _e80.y;
            phi_2165_ = _e84;
            local = _e110.z;
            local_1 = _e114;
            local_2 = _e114;
            local_3 = _e114;
            if (_e146 != 0u) {
                phi_2193_ = _e81.y;
                phi_2165_ = i32(_e81.x);
            }
            let _e152 = phi_2193_;
            let _e154 = phi_2165_;
            phi_2176_ = _e94;
            phi_2174_ = _e99;
            phi_2173_ = _e99.w;
            if (_e154 != _e92) {
                let _e157 = ((_e94 + _e154) - _e92);
                let _e162 = textureLoad(KC, vec2<i32>((_e157 & 2047i), (_e157 >> bitcast<u32>(11i))), 0i);
                if ((_e162.w & 8454143u) != (_e99.w & 8454143u)) {
                    let _e167 = (_e143 == 0f);
                    phi_1457_ = _e167;
                    if !(_e167) {
                        phi_1457_ = (_e112.x != 0f);
                    }
                    let _e172 = phi_1457_;
                    phi_2178_ = _e94;
                    phi_2170_ = _e99;
                    if _e172 {
                        let _e173 = bitcast<i32>(_e110.w);
                        let _e178 = textureLoad(KC, vec2<i32>((_e173 & 2047i), (_e173 >> bitcast<u32>(11i))), 0i);
                        phi_2178_ = _e173;
                        phi_2170_ = _e178;
                    }
                    let _e180 = phi_2178_;
                    let _e182 = phi_2170_;
                    phi_2177_ = _e180;
                    phi_2169_ = _e182;
                } else {
                    phi_2177_ = _e157;
                    phi_2169_ = _e162;
                }
                let _e184 = phi_2177_;
                let _e186 = phi_2169_;
                phi_2176_ = _e184;
                phi_2174_ = _e186;
                phi_2173_ = ((_e186.w & 4286578687u) | _e146);
            }
            let _e191 = phi_2176_;
            let _e193 = phi_2174_;
            let _e195 = phi_2173_;
            let _e196 = (_e195 & 469762048u);
            if ((_e196 == 67108864u) && (_e90 == 0i)) {
                let _e202 = f32((_e193.z & 65535u));
                let _e205 = f32((_e193.z >> bitcast<u32>(16i)));
                let _e211 = vec2<i32>(i32((-1f - _e202)), i32(((_e205 - _e202) + 1f)));
                phi_2180_ = _e211;
                if ((_e195 & 8388608u) != 0u) {
                    phi_2180_ = -(_e211);
                }
                let _e216 = phi_2180_;
                let _e218 = (_e191 + _e216.x);
                let _e223 = textureLoad(KC, vec2<i32>((_e218 & 2047i), (_e218 >> bitcast<u32>(11i))), 0i);
                let _e225 = (_e191 + _e216.y);
                let _e230 = textureLoad(KC, vec2<i32>((_e225 & 2047i), (_e225 >> bitcast<u32>(11i))), 0i);
                phi_2181_ = _e230;
                if ((_e230.w & 8454143u) != (_e223.w & 8454143u)) {
                    let _e236 = bitcast<i32>(_e110.w);
                    let _e241 = textureLoad(KC, vec2<i32>((_e236 & 2047i), (_e236 >> bitcast<u32>(11i))), 0i);
                    phi_2181_ = _e241;
                }
                let _e243 = phi_2181_;
                let _e245 = bitcast<f32>(_e223.z);
                let _e247 = bitcast<f32>(_e243.z);
                let _e248 = (_e247 - _e245);
                phi_2185_ = _e248;
                if (abs(_e248) > 3.1415927f) {
                    phi_2185_ = (_e248 - (6.2831855f * sign(_e248)));
                }
                let _e255 = phi_2185_;
                let _e256 = (_e205 + -2f);
                let _e262 = clamp(round(((abs(_e255) * 0.31830987f) * _e256)), 1f, (_e205 + -3f));
                let _e263 = (_e256 - _e262);
                if (_e202 <= _e263) {
                    phi_2256_ = _e152;
                    if (_e202 == _e263) {
                        phi_2256_ = -(_e152);
                    }
                    let _e272 = phi_2256_;
                    phi_2255_ = _e272;
                    phi_2203_ = -(((3.1415927f * sign(_e255)) - _e255));
                    phi_2200_ = _e263;
                    phi_2197_ = _e202;
                } else {
                    let _e274 = (_e202 == (_e263 + 1f));
                    if _e274 {
                        phi_2199_ = 0f;
                    } else {
                        phi_2199_ = (_e202 - (_e263 + 2f));
                    }
                    let _e278 = phi_2199_;
                    phi_2255_ = select(_e152, 0f, _e274);
                    phi_2203_ = _e255;
                    phi_2200_ = select(_e262, 0f, _e274);
                    phi_2197_ = _e278;
                }
                let _e282 = phi_2255_;
                let _e284 = phi_2203_;
                let _e286 = phi_2200_;
                let _e288 = phi_2197_;
                if (_e288 == _e286) {
                    phi_2207_ = _e247;
                } else {
                    phi_2207_ = (_e245 + (_e284 * (_e288 / _e286)));
                }
                let _e294 = phi_2207_;
                phi_2253_ = _e282;
                phi_2206_ = _e294;
            } else {
                phi_2253_ = _e152;
                phi_2206_ = bitcast<f32>(_e193.z);
            }
            let _e298 = phi_2253_;
            let _e300 = phi_2206_;
            let _e304 = vec2<f32>(sin(_e300), -(cos(_e300)));
            let _e306 = bitcast<vec2<f32>>(_e193.xy);
            phi_2262_ = _e145;
            if (_e145 != 0f) {
                phi_2262_ = max(_e145, (1f / length((_e131 * _e304))));
            }
            let _e313 = phi_2262_;
            if (_e143 != 0f) {
                let _e317 = (_e298 * sign(determinant(_e131)));
                let _e319 = ((_e195 & 1048576u) != 0u);
                phi_2259_ = _e317;
                if _e319 {
                    phi_2259_ = min(_e317, 0f);
                }
                let _e322 = phi_2259_;
                phi_2316_ = _e322;
                if ((_e195 & 524288u) != 0u) {
                    phi_2316_ = max(_e322, 0f);
                }
                let _e327 = phi_2316_;
                let _e329 = select(0f, _e313, (_e313 != 0f));
                let _e333 = select(_e143, _e329, ((_e329 > _e143) && (_e313 == 0f)));
                let _e334 = (_e333 + _e329);
                let _e335 = (_e304 * _e334);
                phi_2324_ = _e335;
                if (_e196 > 134217728u) {
                    let _e337 = (_e195 & 4194304u);
                    let _e339 = select(2i, -2i, (_e337 == 0u));
                    phi_2288_ = _e339;
                    if ((_e195 & 8388608u) != 0u) {
                        phi_2288_ = -(_e339);
                    }
                    let _e344 = phi_2288_;
                    let _e345 = (_e191 + _e344);
                    let _e350 = textureLoad(KC, vec2<i32>((_e345 & 2047i), (_e345 >> bitcast<u32>(11i))), 0i);
                    let _e354 = abs((bitcast<f32>(_e350.z) - _e300));
                    phi_2298_ = _e354;
                    if (_e354 > 3.1415927f) {
                        phi_2298_ = (6.2831855f - _e354);
                    }
                    let _e358 = phi_2298_;
                    let _e363 = ((_e358 * select(0.5f, -0.5f, ((_e337 != 0u) == _e319))) + _e300);
                    let _e367 = vec2<f32>(sin(_e363), -(cos(_e363)));
                    let _e368 = (_e131 * _e367);
                    let _e378 = cos((_e358 * 0.5f));
                    let _e379 = (_e196 == 335544320u);
                    phi_1769_ = _e379;
                    if !(_e379) {
                        phi_1769_ = ((_e196 == 268435456u) && (_e378 >= 0.25f));
                    }
                    let _e385 = phi_1769_;
                    if _e385 {
                        phi_2305_ = (_e333 * (1f / max(_e378, select(0.25f, 1f, ((_e195 & 33554432u) != 0u)))));
                    } else {
                        phi_2305_ = ((_e333 * _e378) + (((abs(_e368.x) + abs(_e368.y)) * (1f / dot(_e368, _e368))) * 0.5f));
                    }
                    let _e396 = phi_2305_;
                    phi_2325_ = _e335;
                    if ((_e195 & 2097152u) != 0u) {
                        if (_e334 <= ((_e396 * _e378) + (_e329 * 0.125f))) {
                            phi_2326_ = (_e367 * (_e334 * (1f / _e378)));
                        } else {
                            let _e406 = (_e367 * _e396);
                            phi_2326_ = (vec2<f32>(dot(_e335, _e335), dot(_e406, _e406)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e335, _e406)));
                        }
                        let _e414 = phi_2326_;
                        phi_2325_ = _e414;
                    }
                    let _e416 = phi_2325_;
                    phi_2324_ = _e416;
                }
                let _e418 = phi_2324_;
                if (_e90 != 0i) {
                    phi_2375_ = u32();
                    phi_2346_ = vec2<f32>();
                    phi_2345_ = false;
                    break;
                }
                phi_2342_ = (_e131 * (_e418 * _e327));
                phi_2327_ = _e306;
            } else {
                if (((_e195 & 2147483648u) != 0u) && (_e90 != 1i)) {
                    phi_2375_ = u32();
                    phi_2346_ = vec2<f32>();
                    phi_2345_ = false;
                    break;
                }
                phi_2342_ = vec2<f32>(0f, 0f);
                phi_2327_ = select(_e306, _e112, vec2((_e90 == 2i)));
            }
            let _e430 = phi_2342_;
            let _e432 = phi_2327_;
            let _e436 = (_e116 + 2u);
            let _e443 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e436 & 255u)), bitcast<i32>((_e436 >> bitcast<u32>(8i)))), 0i);
            phi_2375_ = _e443.x;
            phi_2346_ = (((_e131 * _e432) + _e430) + bitcast<vec2<f32>>(_e139.xy));
            phi_2345_ = true;
            break;
        }
    }
    let _e446 = phi_2375_;
    let _e448 = phi_2346_;
    let _e450 = phi_2345_;
    let _e452 = local;
    let _e456 = local_1;
    let _e461 = textureLoad(AD, vec2<i32>(bitcast<i32>((_e452 & 255u)), bitcast<i32>((_e456 >> bitcast<u32>(8i)))), 0i);
    let _e463 = (_e461.x & 15u);
    if jh {
        let _e464 = (_e463 == 0u);
        if _e464 {
            phi_2404_ = _e461.y;
        } else {
            phi_2404_ = _e461.x;
        }
        let _e467 = phi_2404_;
        let _e469 = (_e467 >> bitcast<u32>(16i));
        let _e471 = m.d6_;
        if (_e469 == 0u) {
            phi_2405_ = 0f;
        } else {
            phi_2405_ = unpack2x16float(((_e469 + 1023u) * _e471)).x;
        }
        let _e478 = phi_2405_;
        phi_2406_ = _e478;
        if _e464 {
            phi_2406_ = -(_e478);
        }
        let _e481 = phi_2406_;
        V1_[0u] = _e481;
    }
    if lh {
        f2_ = f32(((_e461.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e463 == 1u) {
        let _e489 = unpack4x8unorm(_e461.y);
        if lh {
            phi_2408_ = _e489;
        } else {
            let _e492 = (_e489.xyz * _e489.w);
            let _e498 = vec4<f32>(_e492.x, _e489.y, _e489.z, _e489.w);
            let _e504 = vec4<f32>(_e498.x, _e492.y, _e498.z, _e498.w);
            phi_2408_ = vec4<f32>(_e504.x, _e504.y, _e492.z, _e504.w);
        }
        let _e512 = phi_2408_;
        f1_ = _e512;
    } else {
        if (jh && (_e463 == 0u)) {
            let _e516 = (_e461.x >> bitcast<u32>(16i));
            let _e518 = m.d6_;
            if (_e516 == 0u) {
                phi_2407_ = 0f;
            } else {
                phi_2407_ = unpack2x16float(((_e516 + 1023u) * _e518)).x;
            }
            let _e525 = phi_2407_;
            V1_[1u] = _e525;
        } else {
            let _e528 = local_2;
            let _e529 = (_e528 * 8u);
            let _e536 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e529 & 255u)), bitcast<i32>((_e529 >> bitcast<u32>(8i)))), 0i);
            let _e544 = (_e529 + 1u);
            let _e551 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e544 & 255u)), bitcast<i32>((_e544 >> bitcast<u32>(8i)))), 0i);
            let _e554 = ((mat2x2<f32>(vec2<f32>(_e536.x, _e536.y), vec2<f32>(_e536.z, _e536.w)) * _e448) + _e551.xy);
            f1_[3u] = -(bitcast<f32>(_e461.y));
            if (_e551.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e551.w;
            }
            if (_e463 == 2u) {
                f1_[1u] = 0f;
                f1_[0u] = _e554.x;
            } else {
                let _e569 = f1_[2u];
                f1_[2u] = -(_e569);
                f1_[0u] = _e554.x;
                f1_[1u] = _e554.y;
            }
        }
    }
    phi_1134_ = rh;
    if rh {
        phi_1134_ = ((_e461.x & 2048u) != 0u);
    }
    let _e578 = phi_1134_;
    if _e578 {
        let _e580 = local_3;
        let _e581 = (_e580 * 8u);
        let _e582 = (_e581 + 4u);
        let _e589 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e582 & 255u)), bitcast<i32>((_e582 >> bitcast<u32>(8i)))), 0i);
        let _e597 = (_e581 + 5u);
        let _e604 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e597 & 255u)), bitcast<i32>((_e597 >> bitcast<u32>(8i)))), 0i);
        let _e607 = ((mat2x2<f32>(vec2<f32>(_e589.x, _e589.y), vec2<f32>(_e589.z, _e589.w)) * _e448) + _e604.xy);
        A2_ = vec3<f32>(_e607.x, _e607.y, (1f + _e604.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e450 {
        let _e614 = m.of_;
        let _e616 = m.pf;
        let _e624 = vec4<f32>(((_e448.x * _e614) - 1f), ((_e448.y * _e616) - sign(_e616)), 0f, 1f);
        phi_2422_ = vec4<f32>(_e624.x, _e624.y, (1f - (f32(_e446) * 0.000061035156f)), _e624.w);
    } else {
        let _e634 = m.R2_;
        phi_2422_ = vec4(_e634);
    }
    let _e637 = phi_2422_;
    unnamed.gl_Position = _e637;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e16 = V1_;
    let _e17 = f2_;
    let _e18 = f1_;
    let _e19 = A2_;
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
