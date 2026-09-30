struct TB {
    tc: f32,
    Bd: f32,
    Hf: f32,
    If: f32,
    o6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    T7_: vec4<i32>,
    hh: vec2<f32>,
    Cd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    X2_: f32,
    Dd: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Ed: f32,
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
var MC: texture_2d<u32>;
@group(0) @binding(5)
var ID: texture_2d<u32>;
@group(0) @binding(2)
var OB: texture_2d<u32>;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
@group(0) @binding(3)
var DD: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> Y1_: vec2<f32>;
var<private> f1_: f32;
@group(0) @binding(4)
var PB: texture_2d<f32>;
var<private> X1_: vec4<f32>;
var<private> D2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_2253_: f32;
    var phi_2225_: i32;
    var phi_1468_: bool;
    var phi_2238_: i32;
    var phi_2230_: vec4<u32>;
    var phi_2237_: i32;
    var phi_2229_: vec4<u32>;
    var phi_2236_: i32;
    var phi_2234_: vec4<u32>;
    var phi_2233_: u32;
    var phi_2240_: vec2<i32>;
    var phi_2241_: vec4<u32>;
    var phi_2245_: f32;
    var phi_2316_: f32;
    var phi_2259_: f32;
    var phi_2315_: f32;
    var phi_2263_: f32;
    var phi_2260_: f32;
    var phi_2257_: f32;
    var phi_2267_: f32;
    var phi_2313_: f32;
    var phi_2266_: f32;
    var phi_2322_: f32;
    var phi_2319_: f32;
    var phi_2351_: f32;
    var phi_2337_: f32;
    var phi_1756_: bool;
    var phi_2342_: f32;
    var phi_2359_: vec2<f32>;
    var phi_2358_: vec2<f32>;
    var phi_2357_: vec2<f32>;
    var phi_2380_: bool;
    var phi_2375_: vec2<f32>;
    var phi_2360_: vec2<f32>;
    var phi_2403_: u32;
    var phi_2404_: f32;
    var phi_2405_: f32;
    var phi_2442_: f32;
    var phi_2440_: vec4<f32>;
    var phi_2441_: vec4<f32>;
    var phi_1151_: bool;
    var phi_2455_: vec4<f32>;

    let _e80 = gl_InstanceIndex_1;
    let _e81 = VB_1;
    let _e82 = WB_1;
    let _e84 = i32(_e81.x);
    let _e87 = bitcast<i32>(_e81.w);
    let _e89 = (_e87 >> bitcast<u32>(2i));
    let _e90 = (_e87 & 3i);
    let _e92 = min(_e84, (_e89 - 1i));
    let _e94 = ((_e80 * _e89) + _e92);
    let _e99 = textureLoad(MC, vec2<i32>((_e94 & 2047i), (_e94 >> bitcast<u32>(11i))), 0i);
    let _e103 = (max((_e99.w & 65535u), 1u) - 1u);
    let _e110 = textureLoad(ID, vec2<i32>(bitcast<i32>((_e103 & 255u)), bitcast<i32>((_e103 >> bitcast<u32>(8i)))), 0i);
    let _e112 = bitcast<vec2<f32>>(_e110.xy);
    let _e114 = (_e110.z & 65535u);
    let _e116 = (_e114 * 4u);
    let _e123 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e116 & 255u)), bitcast<i32>((_e116 >> bitcast<u32>(8i)))), 0i);
    let _e124 = bitcast<vec4<f32>>(_e123);
    let _e131 = mat2x2<f32>(vec2<f32>(_e124.x, _e124.y), vec2<f32>(_e124.z, _e124.w));
    let _e132 = (_e116 + 1u);
    let _e139 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e132 & 255u)), bitcast<i32>((_e132 >> bitcast<u32>(8i)))), 0i);
    let _e143 = bitcast<f32>(_e139.z);
    let _e145 = bitcast<f32>(_e139.w);
    let _e146 = (_e99.w & 8388608u);
    phi_2253_ = _e81.y;
    phi_2225_ = _e84;
    if (_e146 != 0u) {
        phi_2253_ = _e82.y;
        phi_2225_ = i32(_e82.x);
    }
    let _e152 = phi_2253_;
    let _e154 = phi_2225_;
    phi_2236_ = _e94;
    phi_2234_ = _e99;
    phi_2233_ = _e99.w;
    if (_e154 != _e92) {
        let _e157 = ((_e94 + _e154) - _e92);
        let _e162 = textureLoad(MC, vec2<i32>((_e157 & 2047i), (_e157 >> bitcast<u32>(11i))), 0i);
        if ((_e162.w & 8454143u) != (_e99.w & 8454143u)) {
            let _e167 = (_e143 == 0f);
            phi_1468_ = _e167;
            if !(_e167) {
                phi_1468_ = (_e112.x != 0f);
            }
            let _e172 = phi_1468_;
            phi_2238_ = _e94;
            phi_2230_ = _e99;
            if _e172 {
                let _e173 = bitcast<i32>(_e110.w);
                let _e178 = textureLoad(MC, vec2<i32>((_e173 & 2047i), (_e173 >> bitcast<u32>(11i))), 0i);
                phi_2238_ = _e173;
                phi_2230_ = _e178;
            }
            let _e180 = phi_2238_;
            let _e182 = phi_2230_;
            phi_2237_ = _e180;
            phi_2229_ = _e182;
        } else {
            phi_2237_ = _e157;
            phi_2229_ = _e162;
        }
        let _e184 = phi_2237_;
        let _e186 = phi_2229_;
        phi_2236_ = _e184;
        phi_2234_ = _e186;
        phi_2233_ = ((_e186.w & 4286578687u) | _e146);
    }
    let _e191 = phi_2236_;
    let _e193 = phi_2234_;
    let _e195 = phi_2233_;
    let _e196 = (_e195 & 469762048u);
    if ((_e196 == 67108864u) && (_e90 == 0i)) {
        let _e202 = f32((_e193.z & 65535u));
        let _e205 = f32((_e193.z >> bitcast<u32>(16i)));
        let _e211 = vec2<i32>(i32((-1f - _e202)), i32(((_e205 - _e202) + 1f)));
        phi_2240_ = _e211;
        if ((_e195 & 8388608u) != 0u) {
            phi_2240_ = -(_e211);
        }
        let _e216 = phi_2240_;
        let _e218 = (_e191 + _e216.x);
        let _e223 = textureLoad(MC, vec2<i32>((_e218 & 2047i), (_e218 >> bitcast<u32>(11i))), 0i);
        let _e225 = (_e191 + _e216.y);
        let _e230 = textureLoad(MC, vec2<i32>((_e225 & 2047i), (_e225 >> bitcast<u32>(11i))), 0i);
        phi_2241_ = _e230;
        if ((_e230.w & 8454143u) != (_e223.w & 8454143u)) {
            let _e236 = bitcast<i32>(_e110.w);
            let _e241 = textureLoad(MC, vec2<i32>((_e236 & 2047i), (_e236 >> bitcast<u32>(11i))), 0i);
            phi_2241_ = _e241;
        }
        let _e243 = phi_2241_;
        let _e246 = (f32(_e223.z) * 0.0000000014629181f);
        let _e249 = (f32(_e243.z) * 0.0000000014629181f);
        let _e250 = (_e249 - _e246);
        phi_2245_ = _e250;
        if (abs(_e250) > 3.1415927f) {
            phi_2245_ = (_e250 - (6.2831855f * sign(_e250)));
        }
        let _e257 = phi_2245_;
        let _e258 = (_e205 + -2f);
        let _e264 = clamp(round(((abs(_e257) * 0.31830987f) * _e258)), 1f, (_e205 + -3f));
        let _e265 = (_e258 - _e264);
        if (_e202 <= _e265) {
            phi_2316_ = _e152;
            if (_e202 == _e265) {
                phi_2316_ = -(_e152);
            }
            let _e274 = phi_2316_;
            phi_2315_ = _e274;
            phi_2263_ = -(((3.1415927f * sign(_e257)) - _e257));
            phi_2260_ = _e265;
            phi_2257_ = _e202;
        } else {
            let _e276 = (_e202 == (_e265 + 1f));
            if _e276 {
                phi_2259_ = 0f;
            } else {
                phi_2259_ = (_e202 - (_e265 + 2f));
            }
            let _e280 = phi_2259_;
            phi_2315_ = select(_e152, 0f, _e276);
            phi_2263_ = _e257;
            phi_2260_ = select(_e264, 0f, _e276);
            phi_2257_ = _e280;
        }
        let _e284 = phi_2315_;
        let _e286 = phi_2263_;
        let _e288 = phi_2260_;
        let _e290 = phi_2257_;
        if (_e290 == _e288) {
            phi_2267_ = _e249;
        } else {
            phi_2267_ = (_e246 + (_e286 * (_e290 / _e288)));
        }
        let _e296 = phi_2267_;
        phi_2313_ = _e284;
        phi_2266_ = _e296;
    } else {
        phi_2313_ = _e152;
        phi_2266_ = (f32(_e193.z) * 0.0000000014629181f);
    }
    let _e301 = phi_2313_;
    let _e303 = phi_2266_;
    let _e307 = vec2<f32>(sin(_e303), -(cos(_e303)));
    let _e309 = bitcast<vec2<f32>>(_e193.xy);
    phi_2322_ = _e145;
    if (_e145 != 0f) {
        phi_2322_ = max(_e145, (1f / length((_e131 * _e307))));
    }
    let _e316 = phi_2322_;
    if (_e143 != 0f) {
        let _e320 = (_e301 * sign(determinant(_e131)));
        let _e322 = ((_e195 & 1048576u) != 0u);
        phi_2319_ = _e320;
        if _e322 {
            phi_2319_ = min(_e320, 0f);
        }
        let _e325 = phi_2319_;
        phi_2351_ = _e325;
        if ((_e195 & 524288u) != 0u) {
            phi_2351_ = max(_e325, 0f);
        }
        let _e330 = phi_2351_;
        let _e332 = select(0f, _e316, (_e316 != 0f));
        let _e336 = select(_e143, _e332, ((_e332 > _e143) && (_e316 == 0f)));
        let _e337 = (_e336 + _e332);
        let _e338 = (_e307 * _e337);
        phi_2357_ = _e338;
        if (_e196 > 134217728u) {
            let _e344 = f32((_e193.z & 65535u));
            let _e345 = (_e344 * 0.000015259022f);
            let _e349 = sqrt(max((1f - (_e345 * _e345)), 0f));
            phi_2337_ = _e349;
            if (((_e195 & 4194304u) != 0u) == _e322) {
                phi_2337_ = -(_e349);
            }
            let _e353 = phi_2337_;
            let _e358 = (mat2x2<f32>(vec2<f32>(_e345, _e353), vec2<f32>(-(_e353), _e345)) * _e307);
            let _e359 = (_e131 * _e358);
            let _e368 = (_e196 == 335544320u);
            phi_1756_ = _e368;
            if !(_e368) {
                phi_1756_ = ((_e196 == 268435456u) && (_e345 >= 0.25f));
            }
            let _e374 = phi_1756_;
            if _e374 {
                phi_2342_ = (_e336 * (1f / max(_e345, select(0.25f, 1f, ((_e195 & 33554432u) != 0u)))));
            } else {
                phi_2342_ = ((_e336 * _e345) + (((abs(_e359.x) + abs(_e359.y)) * (1f / dot(_e359, _e359))) * 0.5f));
            }
            let _e385 = phi_2342_;
            phi_2358_ = _e338;
            if ((_e195 & 2097152u) != 0u) {
                if (_e337 <= ((_e385 * _e345) + (_e332 * 0.125f))) {
                    phi_2359_ = (_e358 * (_e337 * (65535f / _e344)));
                } else {
                    let _e395 = (_e358 * _e385);
                    phi_2359_ = (vec2<f32>(dot(_e338, _e338), dot(_e395, _e395)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e338, _e395)));
                }
                let _e403 = phi_2359_;
                phi_2358_ = _e403;
            }
            let _e405 = phi_2358_;
            phi_2357_ = _e405;
        }
        let _e407 = phi_2357_;
        phi_2380_ = (_e90 != 0i);
        phi_2375_ = (_e131 * (_e407 * _e330));
        phi_2360_ = _e309;
    } else {
        phi_2380_ = (((_e195 & 2147483648u) != 0u) && (_e90 != 1i));
        phi_2375_ = vec2<f32>(0f, 0f);
        phi_2360_ = select(_e309, _e112, vec2((_e90 == 2i)));
    }
    let _e419 = phi_2380_;
    let _e421 = phi_2375_;
    let _e423 = phi_2360_;
    let _e426 = (((_e131 * _e423) + _e421) + bitcast<vec2<f32>>(_e139.xy));
    let _e427 = (_e116 + 2u);
    let _e434 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e427 & 255u)), bitcast<i32>((_e427 >> bitcast<u32>(8i)))), 0i);
    let _e442 = textureLoad(DD, vec2<i32>(bitcast<i32>((_e110.z & 255u)), bitcast<i32>((_e114 >> bitcast<u32>(8i)))), 0i);
    let _e444 = (_e442.x & 15u);
    if Hh {
        let _e445 = (_e444 == 0u);
        if _e445 {
            phi_2403_ = _e442.y;
        } else {
            phi_2403_ = _e442.x;
        }
        let _e448 = phi_2403_;
        let _e450 = (_e448 >> bitcast<u32>(16i));
        let _e452 = j.c6_;
        if (_e450 == 0u) {
            phi_2404_ = 0f;
        } else {
            phi_2404_ = unpack2x16float(((_e450 + 1023u) * _e452)).x;
        }
        let _e459 = phi_2404_;
        phi_2405_ = _e459;
        if _e445 {
            phi_2405_ = -(_e459);
        }
        let _e462 = phi_2405_;
        Y1_[0u] = _e462;
    }
    if Jh {
        f1_ = f32(((_e442.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e444 == 1u) {
        X1_ = unpack4x8unorm(_e442.y);
    } else {
        if (Hh && (_e444 == 0u)) {
            let _e474 = (_e442.x >> bitcast<u32>(16i));
            let _e476 = j.c6_;
            if (_e474 == 0u) {
                phi_2442_ = 0f;
            } else {
                phi_2442_ = unpack2x16float(((_e474 + 1023u) * _e476)).x;
            }
            let _e483 = phi_2442_;
            Y1_[1u] = _e483;
        } else {
            let _e485 = (_e114 * 8u);
            let _e492 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e485 & 255u)), bitcast<i32>((_e485 >> bitcast<u32>(8i)))), 0i);
            let _e500 = (_e485 + 1u);
            let _e507 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e500 & 255u)), bitcast<i32>((_e500 >> bitcast<u32>(8i)))), 0i);
            let _e516 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e442.y));
            let _e518 = ((mat2x2<f32>(vec2<f32>(_e492.x, _e492.y), vec2<f32>(_e492.z, _e492.w)) * _e426) + _e507.xy);
            if (_e507.z > 0.9f) {
                phi_2440_ = vec4<f32>(_e516.x, _e516.y, 2f, _e516.w);
            } else {
                phi_2440_ = vec4<f32>(_e516.x, _e516.y, _e507.w, _e516.w);
            }
            let _e533 = phi_2440_;
            if (f32(_e444) == 2f) {
                let _e540 = vec4<f32>(_e518.x, _e533.y, _e533.z, _e533.w);
                phi_2441_ = vec4<f32>(_e540.x, 0f, _e540.z, _e540.w);
            } else {
                let _e552 = vec4<f32>(_e533.x, _e533.y, -(_e533.z), _e533.w);
                let _e558 = vec4<f32>(_e518.x, _e552.y, _e552.z, _e552.w);
                phi_2441_ = vec4<f32>(_e558.x, _e518.y, _e558.z, _e558.w);
            }
            let _e566 = phi_2441_;
            X1_ = _e566;
            let _e568 = X1_[3u];
            X1_[3u] = -(_e568);
        }
    }
    phi_1151_ = Ph;
    if Ph {
        phi_1151_ = ((_e442.x & 2048u) != 0u);
    }
    let _e573 = phi_1151_;
    if _e573 {
        let _e574 = (_e114 * 8u);
        let _e575 = (_e574 + 4u);
        let _e582 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e575 & 255u)), bitcast<i32>((_e575 >> bitcast<u32>(8i)))), 0i);
        let _e590 = (_e574 + 5u);
        let _e597 = textureLoad(PB, vec2<i32>(bitcast<i32>((_e590 & 255u)), bitcast<i32>((_e590 >> bitcast<u32>(8i)))), 0i);
        let _e600 = ((mat2x2<f32>(vec2<f32>(_e582.x, _e582.y), vec2<f32>(_e582.z, _e582.w)) * _e426) + _e597.xy);
        D2_ = vec3<f32>(_e600.x, _e600.y, (1f + _e597.z));
    } else {
        D2_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e419) {
        let _e608 = j.Hf;
        let _e610 = j.If;
        let _e618 = vec4<f32>(((_e426.x * _e608) - 1f), ((_e426.y * _e610) - sign(_e610)), 0f, 1f);
        phi_2455_ = vec4<f32>(_e618.x, _e618.y, ((f32(((_e434.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e618.w);
    } else {
        let _e631 = j.X2_;
        phi_2455_ = vec4(_e631);
    }
    let _e634 = phi_2455_;
    unnamed.gl_Position = _e634;
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
    let _e17 = f1_;
    let _e18 = X1_;
    let _e19 = D2_;
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
