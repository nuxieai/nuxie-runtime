enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct mg {
    d2_: array<vec4<u32>>,
}

struct lg {
    d2_: array<vec4<u32>>,
}

struct Re {
    d2_: array<vec2<u32>>,
}

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

struct Se {
    d2_: array<vec4<f32>>,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat, either) member: vec2<f32>,
    @location(6) @interpolate(flat, either) member_1: f32,
    @location(0) member_2: vec4<f32>,
    @location(9) member_3: vec3<f32>,
}

@id(0) override jh: bool = true;
@id(2) override lh: bool = true;
@id(1) override kh: bool = true;
@id(8) override rh: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ED: mg;
@group(0) @binding(2)
var<storage> PB: lg;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> AD: Re;
@group(0) @binding(0)
var<uniform> m: BC;
var<private> V1_: vec2<f32>;
var<private> f2_: f32;
@group(0) @binding(4)
var<storage> QB: Se;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_2180_: f32;
    var phi_2152_: i32;
    var phi_1407_: bool;
    var phi_2165_: i32;
    var phi_2157_: vec4<u32>;
    var phi_2164_: i32;
    var phi_2156_: vec4<u32>;
    var phi_2163_: i32;
    var phi_2161_: vec4<u32>;
    var phi_2160_: u32;
    var phi_2167_: vec2<i32>;
    var phi_2168_: vec4<u32>;
    var phi_2172_: f32;
    var phi_2243_: f32;
    var phi_2186_: f32;
    var phi_2242_: f32;
    var phi_2190_: f32;
    var phi_2187_: f32;
    var phi_2184_: f32;
    var phi_2194_: f32;
    var phi_2240_: f32;
    var phi_2193_: f32;
    var phi_2249_: f32;
    var phi_2246_: f32;
    var phi_2303_: f32;
    var phi_2275_: i32;
    var phi_2285_: f32;
    var phi_1719_: bool;
    var phi_2292_: f32;
    var phi_2313_: vec2<f32>;
    var phi_2312_: vec2<f32>;
    var phi_2311_: vec2<f32>;
    var phi_2329_: vec2<f32>;
    var phi_2314_: vec2<f32>;
    var phi_2362_: u32;
    var phi_2333_: vec2<f32>;
    var phi_2332_: bool;
    var local: u32;
    var phi_2391_: u32;
    var phi_2392_: f32;
    var phi_2393_: f32;
    var local_1: u32;
    var phi_2395_: vec4<f32>;
    var phi_2394_: f32;
    var local_2: u32;
    var phi_1124_: bool;
    var local_3: u32;
    var phi_2411_: vec4<f32>;

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
            let _e106 = ED.d2_[(max((_e99.w & 65535u), 1u) - 1u)];
            let _e108 = bitcast<vec2<f32>>(_e106.xy);
            let _e110 = (_e106.z & 65535u);
            let _e112 = (_e110 * 4u);
            let _e115 = PB.d2_[_e112];
            let _e116 = bitcast<vec4<f32>>(_e115);
            let _e123 = mat2x2<f32>(vec2<f32>(_e116.x, _e116.y), vec2<f32>(_e116.z, _e116.w));
            let _e127 = PB.d2_[(_e112 + 1u)];
            let _e131 = bitcast<f32>(_e127.z);
            let _e133 = bitcast<f32>(_e127.w);
            let _e134 = (_e99.w & 8388608u);
            phi_2180_ = _e80.y;
            phi_2152_ = _e84;
            local = _e110;
            local_1 = _e110;
            local_2 = _e110;
            local_3 = _e110;
            if (_e134 != 0u) {
                phi_2180_ = _e81.y;
                phi_2152_ = i32(_e81.x);
            }
            let _e140 = phi_2180_;
            let _e142 = phi_2152_;
            phi_2163_ = _e94;
            phi_2161_ = _e99;
            phi_2160_ = _e99.w;
            if (_e142 != _e92) {
                let _e145 = ((_e94 + _e142) - _e92);
                let _e150 = textureLoad(KC, vec2<i32>((_e145 & 2047i), (_e145 >> bitcast<u32>(11i))), 0i);
                if ((_e150.w & 8454143u) != (_e99.w & 8454143u)) {
                    let _e155 = (_e131 == 0f);
                    phi_1407_ = _e155;
                    if !(_e155) {
                        phi_1407_ = (_e108.x != 0f);
                    }
                    let _e160 = phi_1407_;
                    phi_2165_ = _e94;
                    phi_2157_ = _e99;
                    if _e160 {
                        let _e161 = bitcast<i32>(_e106.w);
                        let _e166 = textureLoad(KC, vec2<i32>((_e161 & 2047i), (_e161 >> bitcast<u32>(11i))), 0i);
                        phi_2165_ = _e161;
                        phi_2157_ = _e166;
                    }
                    let _e168 = phi_2165_;
                    let _e170 = phi_2157_;
                    phi_2164_ = _e168;
                    phi_2156_ = _e170;
                } else {
                    phi_2164_ = _e145;
                    phi_2156_ = _e150;
                }
                let _e172 = phi_2164_;
                let _e174 = phi_2156_;
                phi_2163_ = _e172;
                phi_2161_ = _e174;
                phi_2160_ = ((_e174.w & 4286578687u) | _e134);
            }
            let _e179 = phi_2163_;
            let _e181 = phi_2161_;
            let _e183 = phi_2160_;
            let _e184 = (_e183 & 469762048u);
            if ((_e184 == 67108864u) && (_e90 == 0i)) {
                let _e190 = f32((_e181.z & 65535u));
                let _e193 = f32((_e181.z >> bitcast<u32>(16i)));
                let _e199 = vec2<i32>(i32((-1f - _e190)), i32(((_e193 - _e190) + 1f)));
                phi_2167_ = _e199;
                if ((_e183 & 8388608u) != 0u) {
                    phi_2167_ = -(_e199);
                }
                let _e204 = phi_2167_;
                let _e206 = (_e179 + _e204.x);
                let _e211 = textureLoad(KC, vec2<i32>((_e206 & 2047i), (_e206 >> bitcast<u32>(11i))), 0i);
                let _e213 = (_e179 + _e204.y);
                let _e218 = textureLoad(KC, vec2<i32>((_e213 & 2047i), (_e213 >> bitcast<u32>(11i))), 0i);
                phi_2168_ = _e218;
                if ((_e218.w & 8454143u) != (_e211.w & 8454143u)) {
                    let _e224 = bitcast<i32>(_e106.w);
                    let _e229 = textureLoad(KC, vec2<i32>((_e224 & 2047i), (_e224 >> bitcast<u32>(11i))), 0i);
                    phi_2168_ = _e229;
                }
                let _e231 = phi_2168_;
                let _e233 = bitcast<f32>(_e211.z);
                let _e235 = bitcast<f32>(_e231.z);
                let _e236 = (_e235 - _e233);
                phi_2172_ = _e236;
                if (abs(_e236) > 3.1415927f) {
                    phi_2172_ = (_e236 - (6.2831855f * sign(_e236)));
                }
                let _e243 = phi_2172_;
                let _e244 = (_e193 + -2f);
                let _e250 = clamp(round(((abs(_e243) * 0.31830987f) * _e244)), 1f, (_e193 + -3f));
                let _e251 = (_e244 - _e250);
                if (_e190 <= _e251) {
                    phi_2243_ = _e140;
                    if (_e190 == _e251) {
                        phi_2243_ = -(_e140);
                    }
                    let _e260 = phi_2243_;
                    phi_2242_ = _e260;
                    phi_2190_ = -(((3.1415927f * sign(_e243)) - _e243));
                    phi_2187_ = _e251;
                    phi_2184_ = _e190;
                } else {
                    let _e262 = (_e190 == (_e251 + 1f));
                    if _e262 {
                        phi_2186_ = 0f;
                    } else {
                        phi_2186_ = (_e190 - (_e251 + 2f));
                    }
                    let _e266 = phi_2186_;
                    phi_2242_ = select(_e140, 0f, _e262);
                    phi_2190_ = _e243;
                    phi_2187_ = select(_e250, 0f, _e262);
                    phi_2184_ = _e266;
                }
                let _e270 = phi_2242_;
                let _e272 = phi_2190_;
                let _e274 = phi_2187_;
                let _e276 = phi_2184_;
                if (_e276 == _e274) {
                    phi_2194_ = _e235;
                } else {
                    phi_2194_ = (_e233 + (_e272 * (_e276 / _e274)));
                }
                let _e282 = phi_2194_;
                phi_2240_ = _e270;
                phi_2193_ = _e282;
            } else {
                phi_2240_ = _e140;
                phi_2193_ = bitcast<f32>(_e181.z);
            }
            let _e286 = phi_2240_;
            let _e288 = phi_2193_;
            let _e292 = vec2<f32>(sin(_e288), -(cos(_e288)));
            let _e294 = bitcast<vec2<f32>>(_e181.xy);
            phi_2249_ = _e133;
            if (_e133 != 0f) {
                phi_2249_ = max(_e133, (1f / length((_e123 * _e292))));
            }
            let _e301 = phi_2249_;
            if (_e131 != 0f) {
                let _e305 = (_e286 * sign(determinant(_e123)));
                let _e307 = ((_e183 & 1048576u) != 0u);
                phi_2246_ = _e305;
                if _e307 {
                    phi_2246_ = min(_e305, 0f);
                }
                let _e310 = phi_2246_;
                phi_2303_ = _e310;
                if ((_e183 & 524288u) != 0u) {
                    phi_2303_ = max(_e310, 0f);
                }
                let _e315 = phi_2303_;
                let _e317 = select(0f, _e301, (_e301 != 0f));
                let _e321 = select(_e131, _e317, ((_e317 > _e131) && (_e301 == 0f)));
                let _e322 = (_e321 + _e317);
                let _e323 = (_e292 * _e322);
                phi_2311_ = _e323;
                if (_e184 > 134217728u) {
                    let _e325 = (_e183 & 4194304u);
                    let _e327 = select(2i, -2i, (_e325 == 0u));
                    phi_2275_ = _e327;
                    if ((_e183 & 8388608u) != 0u) {
                        phi_2275_ = -(_e327);
                    }
                    let _e332 = phi_2275_;
                    let _e333 = (_e179 + _e332);
                    let _e338 = textureLoad(KC, vec2<i32>((_e333 & 2047i), (_e333 >> bitcast<u32>(11i))), 0i);
                    let _e342 = abs((bitcast<f32>(_e338.z) - _e288));
                    phi_2285_ = _e342;
                    if (_e342 > 3.1415927f) {
                        phi_2285_ = (6.2831855f - _e342);
                    }
                    let _e346 = phi_2285_;
                    let _e351 = ((_e346 * select(0.5f, -0.5f, ((_e325 != 0u) == _e307))) + _e288);
                    let _e355 = vec2<f32>(sin(_e351), -(cos(_e351)));
                    let _e356 = (_e123 * _e355);
                    let _e366 = cos((_e346 * 0.5f));
                    let _e367 = (_e184 == 335544320u);
                    phi_1719_ = _e367;
                    if !(_e367) {
                        phi_1719_ = ((_e184 == 268435456u) && (_e366 >= 0.25f));
                    }
                    let _e373 = phi_1719_;
                    if _e373 {
                        phi_2292_ = (_e321 * (1f / max(_e366, select(0.25f, 1f, ((_e183 & 33554432u) != 0u)))));
                    } else {
                        phi_2292_ = ((_e321 * _e366) + (((abs(_e356.x) + abs(_e356.y)) * (1f / dot(_e356, _e356))) * 0.5f));
                    }
                    let _e384 = phi_2292_;
                    phi_2312_ = _e323;
                    if ((_e183 & 2097152u) != 0u) {
                        if (_e322 <= ((_e384 * _e366) + (_e317 * 0.125f))) {
                            phi_2313_ = (_e355 * (_e322 * (1f / _e366)));
                        } else {
                            let _e394 = (_e355 * _e384);
                            phi_2313_ = (vec2<f32>(dot(_e323, _e323), dot(_e394, _e394)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e323, _e394)));
                        }
                        let _e402 = phi_2313_;
                        phi_2312_ = _e402;
                    }
                    let _e404 = phi_2312_;
                    phi_2311_ = _e404;
                }
                let _e406 = phi_2311_;
                if (_e90 != 0i) {
                    phi_2362_ = u32();
                    phi_2333_ = vec2<f32>();
                    phi_2332_ = false;
                    break;
                }
                phi_2329_ = (_e123 * (_e406 * _e315));
                phi_2314_ = _e294;
            } else {
                if (((_e183 & 2147483648u) != 0u) && (_e90 != 1i)) {
                    phi_2362_ = u32();
                    phi_2333_ = vec2<f32>();
                    phi_2332_ = false;
                    break;
                }
                phi_2329_ = vec2<f32>(0f, 0f);
                phi_2314_ = select(_e294, _e108, vec2((_e90 == 2i)));
            }
            let _e418 = phi_2329_;
            let _e420 = phi_2314_;
            let _e427 = PB.d2_[(_e112 + 2u)];
            phi_2362_ = _e427.x;
            phi_2333_ = (((_e123 * _e420) + _e418) + bitcast<vec2<f32>>(_e127.xy));
            phi_2332_ = true;
            break;
        }
    }
    let _e430 = phi_2362_;
    let _e432 = phi_2333_;
    let _e434 = phi_2332_;
    let _e437 = local;
    let _e439 = AD.d2_[_e437];
    let _e441 = (_e439.x & 15u);
    if jh {
        let _e442 = (_e441 == 0u);
        if _e442 {
            phi_2391_ = _e439.y;
        } else {
            phi_2391_ = _e439.x;
        }
        let _e445 = phi_2391_;
        let _e447 = (_e445 >> bitcast<u32>(16i));
        let _e449 = m.d6_;
        if (_e447 == 0u) {
            phi_2392_ = 0f;
        } else {
            phi_2392_ = unpack2x16float(((_e447 + 1023u) * _e449)).x;
        }
        let _e456 = phi_2392_;
        phi_2393_ = _e456;
        if _e442 {
            phi_2393_ = -(_e456);
        }
        let _e459 = phi_2393_;
        V1_[0u] = _e459;
    }
    if lh {
        f2_ = f32(((_e439.x >> bitcast<u32>(4i)) & 15u));
    }
    if kh {
        let _e466 = local_1;
        let _e467 = (_e466 * 8u);
        let _e471 = QB.d2_[(_e467 + 2u)];
        let _e482 = QB.d2_[(_e467 + 3u)];
        if any((_e471 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e487 = ((mat2x2<f32>(vec2<f32>(_e471.x, _e471.y), vec2<f32>(_e471.z, _e471.w)) * _e432) + _e482.xy);
            unnamed.gl_ClipDistance[0i] = (_e487.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e487.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e487.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e487.y);
        } else {
            let _e503 = (_e482.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e503;
            unnamed.gl_ClipDistance[2i] = _e503;
            unnamed.gl_ClipDistance[1i] = _e503;
            unnamed.gl_ClipDistance[0i] = _e503;
        }
    }
    if (_e441 == 1u) {
        let _e514 = unpack4x8unorm(_e439.y);
        if lh {
            phi_2395_ = _e514;
        } else {
            let _e517 = (_e514.xyz * _e514.w);
            let _e523 = vec4<f32>(_e517.x, _e514.y, _e514.z, _e514.w);
            let _e529 = vec4<f32>(_e523.x, _e517.y, _e523.z, _e523.w);
            phi_2395_ = vec4<f32>(_e529.x, _e529.y, _e517.z, _e529.w);
        }
        let _e537 = phi_2395_;
        f1_ = _e537;
    } else {
        if (jh && (_e441 == 0u)) {
            let _e541 = (_e439.x >> bitcast<u32>(16i));
            let _e543 = m.d6_;
            if (_e541 == 0u) {
                phi_2394_ = 0f;
            } else {
                phi_2394_ = unpack2x16float(((_e541 + 1023u) * _e543)).x;
            }
            let _e550 = phi_2394_;
            V1_[1u] = _e550;
        } else {
            let _e553 = local_2;
            let _e554 = (_e553 * 8u);
            let _e557 = QB.d2_[_e554];
            let _e568 = QB.d2_[(_e554 + 1u)];
            let _e571 = ((mat2x2<f32>(vec2<f32>(_e557.x, _e557.y), vec2<f32>(_e557.z, _e557.w)) * _e432) + _e568.xy);
            f1_[3u] = -(bitcast<f32>(_e439.y));
            if (_e568.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e568.w;
            }
            if (_e441 == 2u) {
                f1_[1u] = 0f;
                f1_[0u] = _e571.x;
            } else {
                let _e586 = f1_[2u];
                f1_[2u] = -(_e586);
                f1_[0u] = _e571.x;
                f1_[1u] = _e571.y;
            }
        }
    }
    phi_1124_ = rh;
    if rh {
        phi_1124_ = ((_e439.x & 2048u) != 0u);
    }
    let _e595 = phi_1124_;
    if _e595 {
        let _e597 = local_3;
        let _e598 = (_e597 * 8u);
        let _e602 = QB.d2_[(_e598 + 4u)];
        let _e613 = QB.d2_[(_e598 + 5u)];
        let _e616 = ((mat2x2<f32>(vec2<f32>(_e602.x, _e602.y), vec2<f32>(_e602.z, _e602.w)) * _e432) + _e613.xy);
        A2_ = vec3<f32>(_e616.x, _e616.y, (1f + _e613.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e434 {
        let _e623 = m.of_;
        let _e625 = m.pf;
        let _e633 = vec4<f32>(((_e432.x * _e623) - 1f), ((_e432.y * _e625) - sign(_e625)), 0f, 1f);
        phi_2411_ = vec4<f32>(_e633.x, _e633.y, (1f - (f32(_e430) * 0.000061035156f)), _e633.w);
    } else {
        let _e643 = m.R2_;
        phi_2411_ = vec4(_e643);
    }
    let _e646 = phi_2411_;
    unnamed.gl_Position = _e646;
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
    let _e19 = V1_;
    let _e20 = f2_;
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
