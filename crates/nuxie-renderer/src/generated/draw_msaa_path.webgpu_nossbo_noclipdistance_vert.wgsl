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

@id(0) override eh: bool = true;
@id(2) override gh: bool = true;
@id(8) override mh: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_2200_: f32;
    var phi_2172_: i32;
    var phi_1464_: bool;
    var phi_2185_: i32;
    var phi_2177_: vec4<u32>;
    var phi_2184_: i32;
    var phi_2176_: vec4<u32>;
    var phi_2183_: i32;
    var phi_2181_: vec4<u32>;
    var phi_2180_: u32;
    var phi_2187_: vec2<i32>;
    var phi_2188_: vec4<u32>;
    var phi_2192_: f32;
    var phi_2263_: f32;
    var phi_2206_: f32;
    var phi_2262_: f32;
    var phi_2210_: f32;
    var phi_2207_: f32;
    var phi_2204_: f32;
    var phi_2214_: f32;
    var phi_2260_: f32;
    var phi_2213_: f32;
    var phi_2269_: f32;
    var phi_2266_: f32;
    var phi_2323_: f32;
    var phi_2295_: i32;
    var phi_2305_: f32;
    var phi_1776_: bool;
    var phi_2312_: f32;
    var phi_2333_: vec2<f32>;
    var phi_2332_: vec2<f32>;
    var phi_2331_: vec2<f32>;
    var phi_2349_: vec2<f32>;
    var phi_2334_: vec2<f32>;
    var phi_2382_: u32;
    var phi_2353_: vec2<f32>;
    var phi_2352_: bool;
    var local: u32;
    var local_1: u32;
    var phi_2411_: u32;
    var phi_2412_: f32;
    var phi_2413_: f32;
    var phi_2415_: vec4<f32>;
    var phi_2414_: f32;
    var local_2: u32;
    var phi_1141_: bool;
    var local_3: u32;
    var phi_2430_: vec4<f32>;

    let _e79 = gl_InstanceIndex_1;
    let _e80 = VB_1;
    let _e81 = WB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e84 = i32(_e80.x);
            let _e87 = bitcast<i32>(_e80.w);
            let _e89 = (_e87 >> bitcast<u32>(2i));
            let _e90 = (_e87 & 3i);
            let _e92 = min(_e84, (_e89 - 1i));
            let _e94 = ((_e79 * _e89) + _e92);
            let _e99 = textureLoad(MC, vec2<i32>((_e94 & 2047i), (_e94 >> bitcast<u32>(11i))), 0i);
            let _e103 = (max((_e99.w & 65535u), 1u) - 1u);
            let _e110 = textureLoad(FD, vec2<i32>(bitcast<i32>((_e103 & 255u)), bitcast<i32>((_e103 >> bitcast<u32>(8i)))), 0i);
            let _e112 = bitcast<vec2<f32>>(_e110.xy);
            let _e114 = (_e110.z & 65535u);
            let _e116 = (_e114 * 4u);
            let _e123 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e116 & 255u)), bitcast<i32>((_e116 >> bitcast<u32>(8i)))), 0i);
            let _e124 = bitcast<vec4<f32>>(_e123);
            let _e131 = mat2x2<f32>(vec2<f32>(_e124.x, _e124.y), vec2<f32>(_e124.z, _e124.w));
            let _e132 = (_e116 + 1u);
            let _e139 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e132 & 255u)), bitcast<i32>((_e132 >> bitcast<u32>(8i)))), 0i);
            let _e143 = bitcast<f32>(_e139.z);
            let _e145 = bitcast<f32>(_e139.w);
            let _e146 = (_e99.w & 8388608u);
            phi_2200_ = _e80.y;
            phi_2172_ = _e84;
            local = _e110.z;
            local_1 = _e114;
            local_2 = _e114;
            local_3 = _e114;
            if (_e146 != 0u) {
                phi_2200_ = _e81.y;
                phi_2172_ = i32(_e81.x);
            }
            let _e152 = phi_2200_;
            let _e154 = phi_2172_;
            phi_2183_ = _e94;
            phi_2181_ = _e99;
            phi_2180_ = _e99.w;
            if (_e154 != _e92) {
                let _e157 = ((_e94 + _e154) - _e92);
                let _e162 = textureLoad(MC, vec2<i32>((_e157 & 2047i), (_e157 >> bitcast<u32>(11i))), 0i);
                if ((_e162.w & 8454143u) != (_e99.w & 8454143u)) {
                    let _e167 = (_e143 == 0f);
                    phi_1464_ = _e167;
                    if !(_e167) {
                        phi_1464_ = (_e112.x != 0f);
                    }
                    let _e172 = phi_1464_;
                    phi_2185_ = _e94;
                    phi_2177_ = _e99;
                    if _e172 {
                        let _e173 = bitcast<i32>(_e110.w);
                        let _e178 = textureLoad(MC, vec2<i32>((_e173 & 2047i), (_e173 >> bitcast<u32>(11i))), 0i);
                        phi_2185_ = _e173;
                        phi_2177_ = _e178;
                    }
                    let _e180 = phi_2185_;
                    let _e182 = phi_2177_;
                    phi_2184_ = _e180;
                    phi_2176_ = _e182;
                } else {
                    phi_2184_ = _e157;
                    phi_2176_ = _e162;
                }
                let _e184 = phi_2184_;
                let _e186 = phi_2176_;
                phi_2183_ = _e184;
                phi_2181_ = _e186;
                phi_2180_ = ((_e186.w & 4286578687u) | _e146);
            }
            let _e191 = phi_2183_;
            let _e193 = phi_2181_;
            let _e195 = phi_2180_;
            let _e196 = (_e195 & 469762048u);
            if ((_e196 == 67108864u) && (_e90 == 0i)) {
                let _e202 = f32((_e193.z & 65535u));
                let _e205 = f32((_e193.z >> bitcast<u32>(16i)));
                let _e211 = vec2<i32>(i32((-1f - _e202)), i32(((_e205 - _e202) + 1f)));
                phi_2187_ = _e211;
                if ((_e195 & 8388608u) != 0u) {
                    phi_2187_ = -(_e211);
                }
                let _e216 = phi_2187_;
                let _e218 = (_e191 + _e216.x);
                let _e223 = textureLoad(MC, vec2<i32>((_e218 & 2047i), (_e218 >> bitcast<u32>(11i))), 0i);
                let _e225 = (_e191 + _e216.y);
                let _e230 = textureLoad(MC, vec2<i32>((_e225 & 2047i), (_e225 >> bitcast<u32>(11i))), 0i);
                phi_2188_ = _e230;
                if ((_e230.w & 8454143u) != (_e223.w & 8454143u)) {
                    let _e236 = bitcast<i32>(_e110.w);
                    let _e241 = textureLoad(MC, vec2<i32>((_e236 & 2047i), (_e236 >> bitcast<u32>(11i))), 0i);
                    phi_2188_ = _e241;
                }
                let _e243 = phi_2188_;
                let _e245 = bitcast<f32>(_e223.z);
                let _e247 = bitcast<f32>(_e243.z);
                let _e248 = (_e247 - _e245);
                phi_2192_ = _e248;
                if (abs(_e248) > 3.1415927f) {
                    phi_2192_ = (_e248 - (6.2831855f * sign(_e248)));
                }
                let _e255 = phi_2192_;
                let _e256 = (_e205 + -2f);
                let _e262 = clamp(round(((abs(_e255) * 0.31830987f) * _e256)), 1f, (_e205 + -3f));
                let _e263 = (_e256 - _e262);
                if (_e202 <= _e263) {
                    phi_2263_ = _e152;
                    if (_e202 == _e263) {
                        phi_2263_ = -(_e152);
                    }
                    let _e272 = phi_2263_;
                    phi_2262_ = _e272;
                    phi_2210_ = -(((3.1415927f * sign(_e255)) - _e255));
                    phi_2207_ = _e263;
                    phi_2204_ = _e202;
                } else {
                    let _e274 = (_e202 == (_e263 + 1f));
                    if _e274 {
                        phi_2206_ = 0f;
                    } else {
                        phi_2206_ = (_e202 - (_e263 + 2f));
                    }
                    let _e278 = phi_2206_;
                    phi_2262_ = select(_e152, 0f, _e274);
                    phi_2210_ = _e255;
                    phi_2207_ = select(_e262, 0f, _e274);
                    phi_2204_ = _e278;
                }
                let _e282 = phi_2262_;
                let _e284 = phi_2210_;
                let _e286 = phi_2207_;
                let _e288 = phi_2204_;
                if (_e288 == _e286) {
                    phi_2214_ = _e247;
                } else {
                    phi_2214_ = (_e245 + (_e284 * (_e288 / _e286)));
                }
                let _e294 = phi_2214_;
                phi_2260_ = _e282;
                phi_2213_ = _e294;
            } else {
                phi_2260_ = _e152;
                phi_2213_ = bitcast<f32>(_e193.z);
            }
            let _e298 = phi_2260_;
            let _e300 = phi_2213_;
            let _e304 = vec2<f32>(sin(_e300), -(cos(_e300)));
            let _e306 = bitcast<vec2<f32>>(_e193.xy);
            phi_2269_ = _e145;
            if (_e145 != 0f) {
                phi_2269_ = max(_e145, (1f / length((_e131 * _e304))));
            }
            let _e313 = phi_2269_;
            if (_e143 != 0f) {
                let _e317 = (_e298 * sign(determinant(_e131)));
                let _e319 = ((_e195 & 1048576u) != 0u);
                phi_2266_ = _e317;
                if _e319 {
                    phi_2266_ = min(_e317, 0f);
                }
                let _e322 = phi_2266_;
                phi_2323_ = _e322;
                if ((_e195 & 524288u) != 0u) {
                    phi_2323_ = max(_e322, 0f);
                }
                let _e327 = phi_2323_;
                let _e329 = select(0f, _e313, (_e313 != 0f));
                let _e333 = select(_e143, _e329, ((_e329 > _e143) && (_e313 == 0f)));
                let _e334 = (_e333 + _e329);
                let _e335 = (_e304 * _e334);
                phi_2331_ = _e335;
                if (_e196 > 134217728u) {
                    let _e337 = (_e195 & 4194304u);
                    let _e339 = select(2i, -2i, (_e337 == 0u));
                    phi_2295_ = _e339;
                    if ((_e195 & 8388608u) != 0u) {
                        phi_2295_ = -(_e339);
                    }
                    let _e344 = phi_2295_;
                    let _e345 = (_e191 + _e344);
                    let _e350 = textureLoad(MC, vec2<i32>((_e345 & 2047i), (_e345 >> bitcast<u32>(11i))), 0i);
                    let _e354 = abs((bitcast<f32>(_e350.z) - _e300));
                    phi_2305_ = _e354;
                    if (_e354 > 3.1415927f) {
                        phi_2305_ = (6.2831855f - _e354);
                    }
                    let _e358 = phi_2305_;
                    let _e363 = ((_e358 * select(0.5f, -0.5f, ((_e337 != 0u) == _e319))) + _e300);
                    let _e367 = vec2<f32>(sin(_e363), -(cos(_e363)));
                    let _e368 = (_e131 * _e367);
                    let _e378 = cos((_e358 * 0.5f));
                    let _e379 = (_e196 == 335544320u);
                    phi_1776_ = _e379;
                    if !(_e379) {
                        phi_1776_ = ((_e196 == 268435456u) && (_e378 >= 0.25f));
                    }
                    let _e385 = phi_1776_;
                    if _e385 {
                        phi_2312_ = (_e333 * (1f / max(_e378, select(0.25f, 1f, ((_e195 & 33554432u) != 0u)))));
                    } else {
                        phi_2312_ = ((_e333 * _e378) + (((abs(_e368.x) + abs(_e368.y)) * (1f / dot(_e368, _e368))) * 0.5f));
                    }
                    let _e396 = phi_2312_;
                    phi_2332_ = _e335;
                    if ((_e195 & 2097152u) != 0u) {
                        if (_e334 <= ((_e396 * _e378) + (_e329 * 0.125f))) {
                            phi_2333_ = (_e367 * (_e334 * (1f / _e378)));
                        } else {
                            let _e406 = (_e367 * _e396);
                            phi_2333_ = (vec2<f32>(dot(_e335, _e335), dot(_e406, _e406)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e335, _e406)));
                        }
                        let _e414 = phi_2333_;
                        phi_2332_ = _e414;
                    }
                    let _e416 = phi_2332_;
                    phi_2331_ = _e416;
                }
                let _e418 = phi_2331_;
                if (_e90 != 0i) {
                    phi_2382_ = u32();
                    phi_2353_ = vec2<f32>();
                    phi_2352_ = false;
                    break;
                }
                phi_2349_ = (_e131 * (_e418 * _e327));
                phi_2334_ = _e306;
            } else {
                if (((_e195 & 2147483648u) != 0u) && (_e90 != 1i)) {
                    phi_2382_ = u32();
                    phi_2353_ = vec2<f32>();
                    phi_2352_ = false;
                    break;
                }
                phi_2349_ = vec2<f32>(0f, 0f);
                phi_2334_ = select(_e306, _e112, vec2((_e90 == 2i)));
            }
            let _e430 = phi_2349_;
            let _e432 = phi_2334_;
            let _e436 = (_e116 + 2u);
            let _e443 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e436 & 255u)), bitcast<i32>((_e436 >> bitcast<u32>(8i)))), 0i);
            phi_2382_ = _e443.x;
            phi_2353_ = (((_e131 * _e432) + _e430) + bitcast<vec2<f32>>(_e139.xy));
            phi_2352_ = true;
            break;
        }
    }
    let _e446 = phi_2382_;
    let _e448 = phi_2353_;
    let _e450 = phi_2352_;
    let _e452 = local;
    let _e456 = local_1;
    let _e461 = textureLoad(BD, vec2<i32>(bitcast<i32>((_e452 & 255u)), bitcast<i32>((_e456 >> bitcast<u32>(8i)))), 0i);
    let _e463 = (_e461.x & 15u);
    if eh {
        let _e464 = (_e463 == 0u);
        if _e464 {
            phi_2411_ = _e461.y;
        } else {
            phi_2411_ = _e461.x;
        }
        let _e467 = phi_2411_;
        let _e469 = (_e467 >> bitcast<u32>(16i));
        let _e471 = m.c6_;
        if (_e469 == 0u) {
            phi_2412_ = 0f;
        } else {
            phi_2412_ = unpack2x16float(((_e469 + 1023u) * _e471)).x;
        }
        let _e478 = phi_2412_;
        phi_2413_ = _e478;
        if _e464 {
            phi_2413_ = -(_e478);
        }
        let _e481 = phi_2413_;
        U1_[0u] = _e481;
    }
    if gh {
        e2_ = f32(((_e461.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e463 == 1u) {
        let _e489 = unpack4x8unorm(_e461.y);
        if gh {
            phi_2415_ = _e489;
        } else {
            let _e492 = (_e489.xyz * _e489.w);
            let _e498 = vec4<f32>(_e492.x, _e489.y, _e489.z, _e489.w);
            let _e504 = vec4<f32>(_e498.x, _e492.y, _e498.z, _e498.w);
            phi_2415_ = vec4<f32>(_e504.x, _e504.y, _e492.z, _e504.w);
        }
        let _e512 = phi_2415_;
        f1_ = _e512;
    } else {
        if (eh && (_e463 == 0u)) {
            let _e516 = (_e461.x >> bitcast<u32>(16i));
            let _e518 = m.c6_;
            if (_e516 == 0u) {
                phi_2414_ = 0f;
            } else {
                phi_2414_ = unpack2x16float(((_e516 + 1023u) * _e518)).x;
            }
            let _e525 = phi_2414_;
            U1_[1u] = _e525;
        } else {
            let _e528 = local_2;
            let _e529 = (_e528 * 8u);
            let _e536 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e529 & 255u)), bitcast<i32>((_e529 >> bitcast<u32>(8i)))), 0i);
            let _e544 = (_e529 + 1u);
            let _e551 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e544 & 255u)), bitcast<i32>((_e544 >> bitcast<u32>(8i)))), 0i);
            let _e554 = ((mat2x2<f32>(vec2<f32>(_e536.x, _e536.y), vec2<f32>(_e536.z, _e536.w)) * _e448) + _e551.xy);
            let _e555 = (_e463 == 2u);
            if (_e555 || (_e463 == 3u)) {
                f1_[3u] = -(bitcast<f32>(_e461.y));
                if (_e551.z > 0.9f) {
                    f1_[2u] = 2f;
                } else {
                    f1_[2u] = _e551.w;
                }
                if _e555 {
                    f1_[1u] = 0f;
                    f1_[0u] = _e554.x;
                } else {
                    let _e571 = f1_[2u];
                    f1_[2u] = -(_e571);
                    f1_[0u] = _e554.x;
                    f1_[1u] = _e554.y;
                }
            }
        }
    }
    phi_1141_ = mh;
    if mh {
        phi_1141_ = ((_e461.x & 2048u) != 0u);
    }
    let _e580 = phi_1141_;
    if _e580 {
        let _e582 = local_3;
        let _e583 = (_e582 * 8u);
        let _e584 = (_e583 + 4u);
        let _e591 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e584 & 255u)), bitcast<i32>((_e584 >> bitcast<u32>(8i)))), 0i);
        let _e599 = (_e583 + 5u);
        let _e606 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e599 & 255u)), bitcast<i32>((_e599 >> bitcast<u32>(8i)))), 0i);
        let _e609 = ((mat2x2<f32>(vec2<f32>(_e591.x, _e591.y), vec2<f32>(_e591.z, _e591.w)) * _e448) + _e606.xy);
        A2_ = vec3<f32>(_e609.x, _e609.y, (1f + _e606.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e450 {
        let _e616 = m.jf;
        let _e618 = m.kf;
        let _e626 = vec4<f32>(((_e448.x * _e616) - 1f), ((_e448.y * _e618) - sign(_e618)), 0f, 1f);
        phi_2430_ = vec4<f32>(_e626.x, _e626.y, (1f - (f32(_e446) * 0.000061035156f)), _e626.w);
    } else {
        let _e636 = m.R2_;
        phi_2430_ = vec4(_e636);
    }
    let _e639 = phi_2430_;
    unnamed.gl_Position = _e639;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) VB: vec4<f32>, @location(1) WB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    VB_1 = VB;
    WB_1 = WB;
    main_1();
    let _e16 = U1_;
    let _e17 = e2_;
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
