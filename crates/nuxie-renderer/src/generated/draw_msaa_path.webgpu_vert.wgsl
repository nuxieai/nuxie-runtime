enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct Yf {
    c2_: array<vec4<u32>>,
}

struct Xf {
    c2_: array<vec4<u32>>,
}

struct Fe {
    c2_: array<vec2<u32>>,
}

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

struct Ge {
    c2_: array<vec4<f32>>,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(4) @interpolate(flat) member: vec2<f32>,
    @location(6) @interpolate(flat) member_1: f32,
    @location(0) member_2: vec4<f32>,
}

@id(0) override Ug: bool = true;
@id(2) override Wg: bool = true;
@id(1) override Vg: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ED: Yf;
@group(0) @binding(2)
var<storage> PB: Xf;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> AD: Fe;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> U1_: vec2<f32>;
var<private> e2_: f32;
@group(0) @binding(4)
var<storage> RB: Ge;
var<private> f1_: vec4<f32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_2144_: f32;
    var phi_2116_: i32;
    var phi_1384_: bool;
    var phi_2129_: i32;
    var phi_2121_: vec4<u32>;
    var phi_2128_: i32;
    var phi_2120_: vec4<u32>;
    var phi_2127_: i32;
    var phi_2125_: vec4<u32>;
    var phi_2124_: u32;
    var phi_2131_: vec2<i32>;
    var phi_2132_: vec4<u32>;
    var phi_2136_: f32;
    var phi_2207_: f32;
    var phi_2150_: f32;
    var phi_2206_: f32;
    var phi_2154_: f32;
    var phi_2151_: f32;
    var phi_2148_: f32;
    var phi_2158_: f32;
    var phi_2204_: f32;
    var phi_2157_: f32;
    var phi_2213_: f32;
    var phi_2210_: f32;
    var phi_2267_: f32;
    var phi_2239_: i32;
    var phi_2249_: f32;
    var phi_1696_: bool;
    var phi_2256_: f32;
    var phi_2277_: vec2<f32>;
    var phi_2276_: vec2<f32>;
    var phi_2275_: vec2<f32>;
    var phi_2293_: vec2<f32>;
    var phi_2278_: vec2<f32>;
    var phi_2326_: u32;
    var phi_2297_: vec2<f32>;
    var phi_2296_: bool;
    var local: u32;
    var phi_2355_: u32;
    var phi_2356_: f32;
    var phi_2357_: f32;
    var local_1: u32;
    var local_2: u32;
    var phi_2359_: vec4<f32>;
    var phi_2358_: f32;
    var local_3: u32;
    var local_4: u32;
    var phi_2374_: vec4<f32>;

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
            let _e100 = ED.c2_[(max((_e93.w & 65535u), 1u) - 1u)];
            let _e102 = bitcast<vec2<f32>>(_e100.xy);
            let _e104 = (_e100.z & 65535u);
            let _e106 = (_e104 * 4u);
            let _e109 = PB.c2_[_e106];
            let _e110 = bitcast<vec4<f32>>(_e109);
            let _e117 = mat2x2<f32>(vec2<f32>(_e110.x, _e110.y), vec2<f32>(_e110.z, _e110.w));
            let _e118 = (_e106 + 1u);
            let _e121 = PB.c2_[_e118];
            let _e125 = bitcast<f32>(_e121.z);
            let _e127 = bitcast<f32>(_e121.w);
            let _e128 = (_e93.w & 8388608u);
            phi_2144_ = _e74.y;
            phi_2116_ = _e78;
            local = _e104;
            local_1 = _e106;
            local_2 = _e106;
            local_3 = _e106;
            local_4 = _e118;
            if (_e128 != 0u) {
                phi_2144_ = _e75.y;
                phi_2116_ = i32(_e75.x);
            }
            let _e134 = phi_2144_;
            let _e136 = phi_2116_;
            phi_2127_ = _e88;
            phi_2125_ = _e93;
            phi_2124_ = _e93.w;
            if (_e136 != _e86) {
                let _e139 = ((_e88 + _e136) - _e86);
                let _e144 = textureLoad(LC, vec2<i32>((_e139 & 2047i), (_e139 >> bitcast<u32>(11i))), 0i);
                if ((_e144.w & 8454143u) != (_e93.w & 8454143u)) {
                    let _e149 = (_e125 == 0f);
                    phi_1384_ = _e149;
                    if !(_e149) {
                        phi_1384_ = (_e102.x != 0f);
                    }
                    let _e154 = phi_1384_;
                    phi_2129_ = _e88;
                    phi_2121_ = _e93;
                    if _e154 {
                        let _e155 = bitcast<i32>(_e100.w);
                        let _e160 = textureLoad(LC, vec2<i32>((_e155 & 2047i), (_e155 >> bitcast<u32>(11i))), 0i);
                        phi_2129_ = _e155;
                        phi_2121_ = _e160;
                    }
                    let _e162 = phi_2129_;
                    let _e164 = phi_2121_;
                    phi_2128_ = _e162;
                    phi_2120_ = _e164;
                } else {
                    phi_2128_ = _e139;
                    phi_2120_ = _e144;
                }
                let _e166 = phi_2128_;
                let _e168 = phi_2120_;
                phi_2127_ = _e166;
                phi_2125_ = _e168;
                phi_2124_ = ((_e168.w & 4286578687u) | _e128);
            }
            let _e173 = phi_2127_;
            let _e175 = phi_2125_;
            let _e177 = phi_2124_;
            let _e178 = (_e177 & 469762048u);
            if ((_e178 == 67108864u) && (_e84 == 0i)) {
                let _e184 = f32((_e175.z & 65535u));
                let _e187 = f32((_e175.z >> bitcast<u32>(16i)));
                let _e193 = vec2<i32>(i32((-1f - _e184)), i32(((_e187 - _e184) + 1f)));
                phi_2131_ = _e193;
                if ((_e177 & 8388608u) != 0u) {
                    phi_2131_ = -(_e193);
                }
                let _e198 = phi_2131_;
                let _e200 = (_e173 + _e198.x);
                let _e205 = textureLoad(LC, vec2<i32>((_e200 & 2047i), (_e200 >> bitcast<u32>(11i))), 0i);
                let _e207 = (_e173 + _e198.y);
                let _e212 = textureLoad(LC, vec2<i32>((_e207 & 2047i), (_e207 >> bitcast<u32>(11i))), 0i);
                phi_2132_ = _e212;
                if ((_e212.w & 8454143u) != (_e205.w & 8454143u)) {
                    let _e218 = bitcast<i32>(_e100.w);
                    let _e223 = textureLoad(LC, vec2<i32>((_e218 & 2047i), (_e218 >> bitcast<u32>(11i))), 0i);
                    phi_2132_ = _e223;
                }
                let _e225 = phi_2132_;
                let _e227 = bitcast<f32>(_e205.z);
                let _e229 = bitcast<f32>(_e225.z);
                let _e230 = (_e229 - _e227);
                phi_2136_ = _e230;
                if (abs(_e230) > 3.1415927f) {
                    phi_2136_ = (_e230 - (6.2831855f * sign(_e230)));
                }
                let _e237 = phi_2136_;
                let _e238 = (_e187 + -2f);
                let _e244 = clamp(round(((abs(_e237) * 0.31830987f) * _e238)), 1f, (_e187 + -3f));
                let _e245 = (_e238 - _e244);
                if (_e184 <= _e245) {
                    phi_2207_ = _e134;
                    if (_e184 == _e245) {
                        phi_2207_ = -(_e134);
                    }
                    let _e254 = phi_2207_;
                    phi_2206_ = _e254;
                    phi_2154_ = -(((3.1415927f * sign(_e237)) - _e237));
                    phi_2151_ = _e245;
                    phi_2148_ = _e184;
                } else {
                    let _e256 = (_e184 == (_e245 + 1f));
                    if _e256 {
                        phi_2150_ = 0f;
                    } else {
                        phi_2150_ = (_e184 - (_e245 + 2f));
                    }
                    let _e260 = phi_2150_;
                    phi_2206_ = select(_e134, 0f, _e256);
                    phi_2154_ = _e237;
                    phi_2151_ = select(_e244, 0f, _e256);
                    phi_2148_ = _e260;
                }
                let _e264 = phi_2206_;
                let _e266 = phi_2154_;
                let _e268 = phi_2151_;
                let _e270 = phi_2148_;
                if (_e270 == _e268) {
                    phi_2158_ = _e229;
                } else {
                    phi_2158_ = (_e227 + (_e266 * (_e270 / _e268)));
                }
                let _e276 = phi_2158_;
                phi_2204_ = _e264;
                phi_2157_ = _e276;
            } else {
                phi_2204_ = _e134;
                phi_2157_ = bitcast<f32>(_e175.z);
            }
            let _e280 = phi_2204_;
            let _e282 = phi_2157_;
            let _e286 = vec2<f32>(sin(_e282), -(cos(_e282)));
            let _e288 = bitcast<vec2<f32>>(_e175.xy);
            phi_2213_ = _e127;
            if (_e127 != 0f) {
                phi_2213_ = max(_e127, (1f / length((_e117 * _e286))));
            }
            let _e295 = phi_2213_;
            if (_e125 != 0f) {
                let _e299 = (_e280 * sign(determinant(_e117)));
                let _e301 = ((_e177 & 1048576u) != 0u);
                phi_2210_ = _e299;
                if _e301 {
                    phi_2210_ = min(_e299, 0f);
                }
                let _e304 = phi_2210_;
                phi_2267_ = _e304;
                if ((_e177 & 524288u) != 0u) {
                    phi_2267_ = max(_e304, 0f);
                }
                let _e309 = phi_2267_;
                let _e311 = select(0f, _e295, (_e295 != 0f));
                let _e315 = select(_e125, _e311, ((_e311 > _e125) && (_e295 == 0f)));
                let _e316 = (_e315 + _e311);
                let _e317 = (_e286 * _e316);
                phi_2275_ = _e317;
                if (_e178 > 134217728u) {
                    let _e319 = (_e177 & 4194304u);
                    let _e321 = select(2i, -2i, (_e319 == 0u));
                    phi_2239_ = _e321;
                    if ((_e177 & 8388608u) != 0u) {
                        phi_2239_ = -(_e321);
                    }
                    let _e326 = phi_2239_;
                    let _e327 = (_e173 + _e326);
                    let _e332 = textureLoad(LC, vec2<i32>((_e327 & 2047i), (_e327 >> bitcast<u32>(11i))), 0i);
                    let _e336 = abs((bitcast<f32>(_e332.z) - _e282));
                    phi_2249_ = _e336;
                    if (_e336 > 3.1415927f) {
                        phi_2249_ = (6.2831855f - _e336);
                    }
                    let _e340 = phi_2249_;
                    let _e345 = ((_e340 * select(0.5f, -0.5f, ((_e319 != 0u) == _e301))) + _e282);
                    let _e349 = vec2<f32>(sin(_e345), -(cos(_e345)));
                    let _e350 = (_e117 * _e349);
                    let _e360 = cos((_e340 * 0.5f));
                    let _e361 = (_e178 == 335544320u);
                    phi_1696_ = _e361;
                    if !(_e361) {
                        phi_1696_ = ((_e178 == 268435456u) && (_e360 >= 0.25f));
                    }
                    let _e367 = phi_1696_;
                    if _e367 {
                        phi_2256_ = (_e315 * (1f / max(_e360, select(0.25f, 1f, ((_e177 & 33554432u) != 0u)))));
                    } else {
                        phi_2256_ = ((_e315 * _e360) + (((abs(_e350.x) + abs(_e350.y)) * (1f / dot(_e350, _e350))) * 0.5f));
                    }
                    let _e378 = phi_2256_;
                    phi_2276_ = _e317;
                    if ((_e177 & 2097152u) != 0u) {
                        if (_e316 <= ((_e378 * _e360) + (_e311 * 0.125f))) {
                            phi_2277_ = (_e349 * (_e316 * (1f / _e360)));
                        } else {
                            let _e388 = (_e349 * _e378);
                            phi_2277_ = (vec2<f32>(dot(_e317, _e317), dot(_e388, _e388)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e317, _e388)));
                        }
                        let _e396 = phi_2277_;
                        phi_2276_ = _e396;
                    }
                    let _e398 = phi_2276_;
                    phi_2275_ = _e398;
                }
                let _e400 = phi_2275_;
                if (_e84 != 0i) {
                    phi_2326_ = u32();
                    phi_2297_ = vec2<f32>();
                    phi_2296_ = false;
                    break;
                }
                phi_2293_ = (_e117 * (_e400 * _e309));
                phi_2278_ = _e288;
            } else {
                if (((_e177 & 2147483648u) != 0u) && (_e84 != 1i)) {
                    phi_2326_ = u32();
                    phi_2297_ = vec2<f32>();
                    phi_2296_ = false;
                    break;
                }
                phi_2293_ = vec2<f32>(0f, 0f);
                phi_2278_ = select(_e288, _e102, vec2((_e84 == 2i)));
            }
            let _e412 = phi_2293_;
            let _e414 = phi_2278_;
            let _e421 = PB.c2_[(_e106 + 2u)];
            phi_2326_ = _e421.x;
            phi_2297_ = (((_e117 * _e414) + _e412) + bitcast<vec2<f32>>(_e121.xy));
            phi_2296_ = true;
            break;
        }
    }
    let _e424 = phi_2326_;
    let _e426 = phi_2297_;
    let _e428 = phi_2296_;
    let _e431 = local;
    let _e433 = AD.c2_[_e431];
    let _e435 = (_e433.x & 15u);
    if Ug {
        let _e436 = (_e435 == 0u);
        if _e436 {
            phi_2355_ = _e433.y;
        } else {
            phi_2355_ = _e433.x;
        }
        let _e439 = phi_2355_;
        let _e441 = (_e439 >> bitcast<u32>(16i));
        let _e443 = m.Z5_;
        if (_e441 == 0u) {
            phi_2356_ = 0f;
        } else {
            phi_2356_ = unpack2x16float(((_e441 + 1023u) * _e443)).x;
        }
        let _e450 = phi_2356_;
        phi_2357_ = _e450;
        if _e436 {
            phi_2357_ = -(_e450);
        }
        let _e453 = phi_2357_;
        U1_[0u] = _e453;
    }
    if Wg {
        e2_ = f32(((_e433.x >> bitcast<u32>(4i)) & 15u));
    }
    if Vg {
        let _e460 = local_1;
        let _e464 = RB.c2_[(_e460 + 2u)];
        let _e473 = local_2;
        let _e477 = RB.c2_[(_e473 + 3u)];
        if any((_e464 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e482 = ((mat2x2<f32>(vec2<f32>(_e464.x, _e464.y), vec2<f32>(_e464.z, _e464.w)) * _e426) + _e477.xy);
            unnamed.gl_ClipDistance[0i] = (_e482.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e482.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e482.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e482.y);
        } else {
            let _e498 = (_e477.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e498;
            unnamed.gl_ClipDistance[2i] = _e498;
            unnamed.gl_ClipDistance[1i] = _e498;
            unnamed.gl_ClipDistance[0i] = _e498;
        }
    }
    if (_e435 == 1u) {
        let _e509 = unpack4x8unorm(_e433.y);
        if Wg {
            phi_2359_ = _e509;
        } else {
            let _e512 = (_e509.xyz * _e509.w);
            let _e518 = vec4<f32>(_e512.x, _e509.y, _e509.z, _e509.w);
            let _e524 = vec4<f32>(_e518.x, _e512.y, _e518.z, _e518.w);
            phi_2359_ = vec4<f32>(_e524.x, _e524.y, _e512.z, _e524.w);
        }
        let _e532 = phi_2359_;
        f1_ = _e532;
    } else {
        if (Ug && (_e435 == 0u)) {
            let _e536 = (_e433.x >> bitcast<u32>(16i));
            let _e538 = m.Z5_;
            if (_e536 == 0u) {
                phi_2358_ = 0f;
            } else {
                phi_2358_ = unpack2x16float(((_e536 + 1023u) * _e538)).x;
            }
            let _e545 = phi_2358_;
            U1_[1u] = _e545;
        } else {
            let _e549 = local_3;
            let _e551 = RB.c2_[_e549];
            let _e561 = local_4;
            let _e563 = RB.c2_[_e561];
            let _e566 = ((mat2x2<f32>(vec2<f32>(_e551.x, _e551.y), vec2<f32>(_e551.z, _e551.w)) * _e426) + _e563.xy);
            let _e567 = (_e435 == 2u);
            if (_e567 || (_e435 == 3u)) {
                f1_[3u] = -(bitcast<f32>(_e433.y));
                if (_e563.z > 0.9f) {
                    f1_[2u] = 2f;
                } else {
                    f1_[2u] = _e563.w;
                }
                if _e567 {
                    f1_[1u] = 0f;
                    f1_[0u] = _e566.x;
                } else {
                    let _e583 = f1_[2u];
                    f1_[2u] = -(_e583);
                    f1_[0u] = _e566.x;
                    f1_[1u] = _e566.y;
                }
            } else {
                f1_ = vec4<f32>(_e566.x, _e566.y, bitcast<f32>(_e433.y), (-2f - _e563.z));
            }
        }
    }
    if _e428 {
        let _e597 = m.bf;
        let _e599 = m.cf;
        let _e607 = vec4<f32>(((_e426.x * _e597) - 1f), ((_e426.y * _e599) - sign(_e599)), 0f, 1f);
        phi_2374_ = vec4<f32>(_e607.x, _e607.y, (1f - (f32(_e424) * 0.000061035156f)), _e607.w);
    } else {
        let _e617 = m.N2_;
        phi_2374_ = vec4(_e617);
    }
    let _e620 = phi_2374_;
    unnamed.gl_Position = _e620;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) UB: vec4<f32>, @location(1) VB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    UB_1 = UB;
    VB_1 = VB;
    main_1();
    let _e16 = unnamed.gl_Position;
    let _e17 = unnamed.gl_ClipDistance;
    let _e18 = U1_;
    let _e19 = e2_;
    let _e20 = f1_;
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
