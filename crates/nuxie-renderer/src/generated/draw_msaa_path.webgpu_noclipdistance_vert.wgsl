struct dg {
    c2_: array<vec4<u32>>,
}

struct cg {
    c2_: array<vec4<u32>>,
}

struct Je {
    c2_: array<vec2<u32>>,
}

struct CC {
    ec: f32,
    od: f32,
    ff: f32,
    gf: f32,
    m6_: u32,
    Gg: u32,
    Re: u32,
    Se: u32,
    R7_: vec4<i32>,
    Cg: vec2<f32>,
    pd: vec2<f32>,
    a2_: u32,
    Hg: f32,
    Z5_: u32,
    P2_: f32,
    qd: f32,
    Me: u32,
    z3_: f32,
    A3_: f32,
    rd: f32,
    zg: u32,
}

struct Ke {
    c2_: array<vec4<f32>>,
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
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Zg: bool = true;
@id(2) override bh: bool = true;

@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ED: dg;
@group(0) @binding(2)
var<storage> PB: cg;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> AD: Je;
@group(0) @binding(0)
var<uniform> n: CC;
var<private> U1_: vec2<f32>;
var<private> e2_: f32;
@group(0) @binding(4)
var<storage> RB: Ke;
var<private> X0_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_2057_: f32;
    var phi_2029_: i32;
    var phi_1342_: bool;
    var phi_2042_: i32;
    var phi_2034_: vec4<u32>;
    var phi_2041_: i32;
    var phi_2033_: vec4<u32>;
    var phi_2040_: i32;
    var phi_2038_: vec4<u32>;
    var phi_2037_: u32;
    var phi_2044_: vec2<i32>;
    var phi_2045_: vec4<u32>;
    var phi_2049_: f32;
    var phi_2120_: f32;
    var phi_2063_: f32;
    var phi_2119_: f32;
    var phi_2067_: f32;
    var phi_2064_: f32;
    var phi_2061_: f32;
    var phi_2071_: f32;
    var phi_2117_: f32;
    var phi_2070_: f32;
    var phi_2126_: f32;
    var phi_2123_: f32;
    var phi_2180_: f32;
    var phi_2152_: i32;
    var phi_2162_: f32;
    var phi_1654_: bool;
    var phi_2169_: f32;
    var phi_2190_: vec2<f32>;
    var phi_2189_: vec2<f32>;
    var phi_2188_: vec2<f32>;
    var phi_2206_: vec2<f32>;
    var phi_2191_: vec2<f32>;
    var phi_2239_: u32;
    var phi_2210_: vec2<f32>;
    var phi_2209_: bool;
    var local: u32;
    var phi_2268_: u32;
    var phi_2269_: f32;
    var phi_2270_: f32;
    var phi_2272_: vec4<f32>;
    var phi_2271_: f32;
    var local_1: u32;
    var local_2: u32;
    var phi_2285_: vec4<f32>;

    let _e71 = gl_InstanceIndex_1;
    let _e72 = UB_1;
    let _e73 = VB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e76 = i32(_e72.x);
            let _e79 = bitcast<i32>(_e72.w);
            let _e81 = (_e79 >> bitcast<u32>(2i));
            let _e82 = (_e79 & 3i);
            let _e84 = min(_e76, (_e81 - 1i));
            let _e86 = ((_e71 * _e81) + _e84);
            let _e91 = textureLoad(LC, vec2<i32>((_e86 & 2047i), (_e86 >> bitcast<u32>(11i))), 0i);
            let _e98 = ED.c2_[(max((_e91.w & 65535u), 1u) - 1u)];
            let _e100 = bitcast<vec2<f32>>(_e98.xy);
            let _e102 = (_e98.z & 65535u);
            let _e104 = (_e102 * 4u);
            let _e107 = PB.c2_[_e104];
            let _e108 = bitcast<vec4<f32>>(_e107);
            let _e115 = mat2x2<f32>(vec2<f32>(_e108.x, _e108.y), vec2<f32>(_e108.z, _e108.w));
            let _e116 = (_e104 + 1u);
            let _e119 = PB.c2_[_e116];
            let _e123 = bitcast<f32>(_e119.z);
            let _e125 = bitcast<f32>(_e119.w);
            let _e126 = (_e91.w & 8388608u);
            phi_2057_ = _e72.y;
            phi_2029_ = _e76;
            local = _e102;
            local_1 = _e104;
            local_2 = _e116;
            if (_e126 != 0u) {
                phi_2057_ = _e73.y;
                phi_2029_ = i32(_e73.x);
            }
            let _e132 = phi_2057_;
            let _e134 = phi_2029_;
            phi_2040_ = _e86;
            phi_2038_ = _e91;
            phi_2037_ = _e91.w;
            if (_e134 != _e84) {
                let _e137 = ((_e86 + _e134) - _e84);
                let _e142 = textureLoad(LC, vec2<i32>((_e137 & 2047i), (_e137 >> bitcast<u32>(11i))), 0i);
                if ((_e142.w & 8454143u) != (_e91.w & 8454143u)) {
                    let _e147 = (_e123 == 0f);
                    phi_1342_ = _e147;
                    if !(_e147) {
                        phi_1342_ = (_e100.x != 0f);
                    }
                    let _e152 = phi_1342_;
                    phi_2042_ = _e86;
                    phi_2034_ = _e91;
                    if _e152 {
                        let _e153 = bitcast<i32>(_e98.w);
                        let _e158 = textureLoad(LC, vec2<i32>((_e153 & 2047i), (_e153 >> bitcast<u32>(11i))), 0i);
                        phi_2042_ = _e153;
                        phi_2034_ = _e158;
                    }
                    let _e160 = phi_2042_;
                    let _e162 = phi_2034_;
                    phi_2041_ = _e160;
                    phi_2033_ = _e162;
                } else {
                    phi_2041_ = _e137;
                    phi_2033_ = _e142;
                }
                let _e164 = phi_2041_;
                let _e166 = phi_2033_;
                phi_2040_ = _e164;
                phi_2038_ = _e166;
                phi_2037_ = ((_e166.w & 4286578687u) | _e126);
            }
            let _e171 = phi_2040_;
            let _e173 = phi_2038_;
            let _e175 = phi_2037_;
            let _e176 = (_e175 & 469762048u);
            if ((_e176 == 67108864u) && (_e82 == 0i)) {
                let _e182 = f32((_e173.z & 65535u));
                let _e185 = f32((_e173.z >> bitcast<u32>(16i)));
                let _e191 = vec2<i32>(i32((-1f - _e182)), i32(((_e185 - _e182) + 1f)));
                phi_2044_ = _e191;
                if ((_e175 & 8388608u) != 0u) {
                    phi_2044_ = -(_e191);
                }
                let _e196 = phi_2044_;
                let _e198 = (_e171 + _e196.x);
                let _e203 = textureLoad(LC, vec2<i32>((_e198 & 2047i), (_e198 >> bitcast<u32>(11i))), 0i);
                let _e205 = (_e171 + _e196.y);
                let _e210 = textureLoad(LC, vec2<i32>((_e205 & 2047i), (_e205 >> bitcast<u32>(11i))), 0i);
                phi_2045_ = _e210;
                if ((_e210.w & 8454143u) != (_e203.w & 8454143u)) {
                    let _e216 = bitcast<i32>(_e98.w);
                    let _e221 = textureLoad(LC, vec2<i32>((_e216 & 2047i), (_e216 >> bitcast<u32>(11i))), 0i);
                    phi_2045_ = _e221;
                }
                let _e223 = phi_2045_;
                let _e225 = bitcast<f32>(_e203.z);
                let _e227 = bitcast<f32>(_e223.z);
                let _e228 = (_e227 - _e225);
                phi_2049_ = _e228;
                if (abs(_e228) > 3.1415927f) {
                    phi_2049_ = (_e228 - (6.2831855f * sign(_e228)));
                }
                let _e235 = phi_2049_;
                let _e236 = (_e185 + -2f);
                let _e242 = clamp(round(((abs(_e235) * 0.31830987f) * _e236)), 1f, (_e185 + -3f));
                let _e243 = (_e236 - _e242);
                if (_e182 <= _e243) {
                    phi_2120_ = _e132;
                    if (_e182 == _e243) {
                        phi_2120_ = -(_e132);
                    }
                    let _e252 = phi_2120_;
                    phi_2119_ = _e252;
                    phi_2067_ = -(((3.1415927f * sign(_e235)) - _e235));
                    phi_2064_ = _e243;
                    phi_2061_ = _e182;
                } else {
                    let _e254 = (_e182 == (_e243 + 1f));
                    if _e254 {
                        phi_2063_ = 0f;
                    } else {
                        phi_2063_ = (_e182 - (_e243 + 2f));
                    }
                    let _e258 = phi_2063_;
                    phi_2119_ = select(_e132, 0f, _e254);
                    phi_2067_ = _e235;
                    phi_2064_ = select(_e242, 0f, _e254);
                    phi_2061_ = _e258;
                }
                let _e262 = phi_2119_;
                let _e264 = phi_2067_;
                let _e266 = phi_2064_;
                let _e268 = phi_2061_;
                if (_e268 == _e266) {
                    phi_2071_ = _e227;
                } else {
                    phi_2071_ = (_e225 + (_e264 * (_e268 / _e266)));
                }
                let _e274 = phi_2071_;
                phi_2117_ = _e262;
                phi_2070_ = _e274;
            } else {
                phi_2117_ = _e132;
                phi_2070_ = bitcast<f32>(_e173.z);
            }
            let _e278 = phi_2117_;
            let _e280 = phi_2070_;
            let _e284 = vec2<f32>(sin(_e280), -(cos(_e280)));
            let _e286 = bitcast<vec2<f32>>(_e173.xy);
            phi_2126_ = _e125;
            if (_e125 != 0f) {
                phi_2126_ = max(_e125, (1f / length((_e115 * _e284))));
            }
            let _e293 = phi_2126_;
            if (_e123 != 0f) {
                let _e297 = (_e278 * sign(determinant(_e115)));
                let _e299 = ((_e175 & 1048576u) != 0u);
                phi_2123_ = _e297;
                if _e299 {
                    phi_2123_ = min(_e297, 0f);
                }
                let _e302 = phi_2123_;
                phi_2180_ = _e302;
                if ((_e175 & 524288u) != 0u) {
                    phi_2180_ = max(_e302, 0f);
                }
                let _e307 = phi_2180_;
                let _e309 = select(0f, _e293, (_e293 != 0f));
                let _e313 = select(_e123, _e309, ((_e309 > _e123) && (_e293 == 0f)));
                let _e314 = (_e313 + _e309);
                let _e315 = (_e284 * _e314);
                phi_2188_ = _e315;
                if (_e176 > 134217728u) {
                    let _e317 = (_e175 & 4194304u);
                    let _e319 = select(2i, -2i, (_e317 == 0u));
                    phi_2152_ = _e319;
                    if ((_e175 & 8388608u) != 0u) {
                        phi_2152_ = -(_e319);
                    }
                    let _e324 = phi_2152_;
                    let _e325 = (_e171 + _e324);
                    let _e330 = textureLoad(LC, vec2<i32>((_e325 & 2047i), (_e325 >> bitcast<u32>(11i))), 0i);
                    let _e334 = abs((bitcast<f32>(_e330.z) - _e280));
                    phi_2162_ = _e334;
                    if (_e334 > 3.1415927f) {
                        phi_2162_ = (6.2831855f - _e334);
                    }
                    let _e338 = phi_2162_;
                    let _e343 = ((_e338 * select(0.5f, -0.5f, ((_e317 != 0u) == _e299))) + _e280);
                    let _e347 = vec2<f32>(sin(_e343), -(cos(_e343)));
                    let _e348 = (_e115 * _e347);
                    let _e358 = cos((_e338 * 0.5f));
                    let _e359 = (_e176 == 335544320u);
                    phi_1654_ = _e359;
                    if !(_e359) {
                        phi_1654_ = ((_e176 == 268435456u) && (_e358 >= 0.25f));
                    }
                    let _e365 = phi_1654_;
                    if _e365 {
                        phi_2169_ = (_e313 * (1f / max(_e358, select(0.25f, 1f, ((_e175 & 33554432u) != 0u)))));
                    } else {
                        phi_2169_ = ((_e313 * _e358) + (((abs(_e348.x) + abs(_e348.y)) * (1f / dot(_e348, _e348))) * 0.5f));
                    }
                    let _e376 = phi_2169_;
                    phi_2189_ = _e315;
                    if ((_e175 & 2097152u) != 0u) {
                        if (_e314 <= ((_e376 * _e358) + (_e309 * 0.125f))) {
                            phi_2190_ = (_e347 * (_e314 * (1f / _e358)));
                        } else {
                            let _e386 = (_e347 * _e376);
                            phi_2190_ = (vec2<f32>(dot(_e315, _e315), dot(_e386, _e386)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e315, _e386)));
                        }
                        let _e394 = phi_2190_;
                        phi_2189_ = _e394;
                    }
                    let _e396 = phi_2189_;
                    phi_2188_ = _e396;
                }
                let _e398 = phi_2188_;
                if (_e82 != 0i) {
                    phi_2239_ = u32();
                    phi_2210_ = vec2<f32>();
                    phi_2209_ = false;
                    break;
                }
                phi_2206_ = (_e115 * (_e398 * _e307));
                phi_2191_ = _e286;
            } else {
                if (((_e175 & 2147483648u) != 0u) && (_e82 != 1i)) {
                    phi_2239_ = u32();
                    phi_2210_ = vec2<f32>();
                    phi_2209_ = false;
                    break;
                }
                phi_2206_ = vec2<f32>(0f, 0f);
                phi_2191_ = select(_e286, _e100, vec2((_e82 == 2i)));
            }
            let _e410 = phi_2206_;
            let _e412 = phi_2191_;
            let _e419 = PB.c2_[(_e104 + 2u)];
            phi_2239_ = _e419.x;
            phi_2210_ = (((_e115 * _e412) + _e410) + bitcast<vec2<f32>>(_e119.xy));
            phi_2209_ = true;
            break;
        }
    }
    let _e422 = phi_2239_;
    let _e424 = phi_2210_;
    let _e426 = phi_2209_;
    let _e429 = local;
    let _e431 = AD.c2_[_e429];
    let _e433 = (_e431.x & 15u);
    if Zg {
        let _e434 = (_e433 == 0u);
        if _e434 {
            phi_2268_ = _e431.y;
        } else {
            phi_2268_ = _e431.x;
        }
        let _e437 = phi_2268_;
        let _e439 = (_e437 >> bitcast<u32>(16i));
        let _e441 = n.Z5_;
        if (_e439 == 0u) {
            phi_2269_ = 0f;
        } else {
            phi_2269_ = unpack2x16float(((_e439 + 1023u) * _e441)).x;
        }
        let _e448 = phi_2269_;
        phi_2270_ = _e448;
        if _e434 {
            phi_2270_ = -(_e448);
        }
        let _e451 = phi_2270_;
        U1_[0u] = _e451;
    }
    if bh {
        e2_ = f32(((_e431.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e433 == 1u) {
        let _e459 = unpack4x8unorm(_e431.y);
        if bh {
            phi_2272_ = _e459;
        } else {
            let _e462 = (_e459.xyz * _e459.w);
            let _e468 = vec4<f32>(_e462.x, _e459.y, _e459.z, _e459.w);
            let _e474 = vec4<f32>(_e468.x, _e462.y, _e468.z, _e468.w);
            phi_2272_ = vec4<f32>(_e474.x, _e474.y, _e462.z, _e474.w);
        }
        let _e482 = phi_2272_;
        X0_ = _e482;
    } else {
        if (Zg && (_e433 == 0u)) {
            let _e486 = (_e431.x >> bitcast<u32>(16i));
            let _e488 = n.Z5_;
            if (_e486 == 0u) {
                phi_2271_ = 0f;
            } else {
                phi_2271_ = unpack2x16float(((_e486 + 1023u) * _e488)).x;
            }
            let _e495 = phi_2271_;
            U1_[1u] = _e495;
        } else {
            let _e499 = local_1;
            let _e501 = RB.c2_[_e499];
            let _e511 = local_2;
            let _e513 = RB.c2_[_e511];
            let _e516 = ((mat2x2<f32>(vec2<f32>(_e501.x, _e501.y), vec2<f32>(_e501.z, _e501.w)) * _e424) + _e513.xy);
            let _e517 = (_e433 == 2u);
            if (_e517 || (_e433 == 3u)) {
                X0_[3u] = -(bitcast<f32>(_e431.y));
                if (_e513.z > 0.9f) {
                    X0_[2u] = 2f;
                } else {
                    X0_[2u] = _e513.w;
                }
                if _e517 {
                    X0_[1u] = 0f;
                    X0_[0u] = _e516.x;
                } else {
                    let _e533 = X0_[2u];
                    X0_[2u] = -(_e533);
                    X0_[0u] = _e516.x;
                    X0_[1u] = _e516.y;
                }
            } else {
                X0_ = vec4<f32>(_e516.x, _e516.y, bitcast<f32>(_e431.y), (-2f - _e513.z));
            }
        }
    }
    if _e426 {
        let _e547 = n.ff;
        let _e549 = n.gf;
        let _e557 = vec4<f32>(((_e424.x * _e547) - 1f), ((_e424.y * _e549) - sign(_e549)), 0f, 1f);
        phi_2285_ = vec4<f32>(_e557.x, _e557.y, (1f - (f32(_e422) * 0.000061035156f)), _e557.w);
    } else {
        let _e567 = n.P2_;
        phi_2285_ = vec4(_e567);
    }
    let _e570 = phi_2285_;
    unnamed.gl_Position = _e570;
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
    let _e17 = X0_;
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
