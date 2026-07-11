struct Sf {
    X1_: array<vec4<u32>>,
}

struct Rf {
    X1_: array<vec4<u32>>,
}

struct Be {
    X1_: array<vec2<u32>>,
}

struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct Ce {
    X1_: array<vec4<f32>>,
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

@id(0) override Mg: bool = true;
@id(2) override Og: bool = true;

@group(0) @binding(8) 
var DC: texture_2d<u32>;
@group(0) @binding(6) 
var<storage> XC: Sf;
@group(0) @binding(3) 
var<storage> MB: Rf;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> SB_1: vec4<f32>;
var<private> TB_1: vec4<f32>;
@group(0) @binding(4) 
var<storage> TC: Be;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> S1_: vec2<f32>;
var<private> Z1_: f32;
@group(0) @binding(5) 
var<storage> PB: Ce;
var<private> i1_: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(10) 
var QC: texture_2d<f32>;
@group(3) @binding(10) 
var T9_: sampler;

fn main_1() {
    var phi_2056_: f32;
    var phi_2028_: i32;
    var phi_1341_: bool;
    var phi_2041_: i32;
    var phi_2033_: vec4<u32>;
    var phi_2040_: i32;
    var phi_2032_: vec4<u32>;
    var phi_2039_: i32;
    var phi_2037_: vec4<u32>;
    var phi_2036_: u32;
    var phi_2043_: vec2<i32>;
    var phi_2044_: vec4<u32>;
    var phi_2048_: f32;
    var phi_2119_: f32;
    var phi_2062_: f32;
    var phi_2118_: f32;
    var phi_2066_: f32;
    var phi_2063_: f32;
    var phi_2060_: f32;
    var phi_2070_: f32;
    var phi_2116_: f32;
    var phi_2069_: f32;
    var phi_2125_: f32;
    var phi_2122_: f32;
    var phi_2179_: f32;
    var phi_2151_: i32;
    var phi_2161_: f32;
    var phi_1653_: bool;
    var phi_2168_: f32;
    var phi_2189_: vec2<f32>;
    var phi_2188_: vec2<f32>;
    var phi_2187_: vec2<f32>;
    var phi_2205_: vec2<f32>;
    var phi_2190_: vec2<f32>;
    var phi_2238_: u32;
    var phi_2209_: vec2<f32>;
    var phi_2208_: bool;
    var local: u32;
    var phi_2267_: u32;
    var phi_2268_: f32;
    var phi_2269_: f32;
    var phi_2271_: vec4<f32>;
    var phi_2270_: f32;
    var local_1: u32;
    var local_2: u32;
    var phi_2284_: vec4<f32>;

    let _e71 = gl_InstanceIndex_1;
    let _e72 = SB_1;
    let _e73 = TB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e76 = i32(_e72.x);
            let _e79 = bitcast<i32>(_e72.w);
            let _e81 = (_e79 >> bitcast<u32>(2i));
            let _e82 = (_e79 & 3i);
            let _e84 = min(_e76, (_e81 - 1i));
            let _e86 = ((_e71 * _e81) + _e84);
            let _e91 = textureLoad(DC, vec2<i32>((_e86 & 2047i), (_e86 >> bitcast<u32>(11i))), 0i);
            let _e98 = XC.X1_[(max((_e91.w & 65535u), 1u) - 1u)];
            let _e100 = bitcast<vec2<f32>>(_e98.xy);
            let _e102 = (_e98.z & 65535u);
            let _e104 = (_e102 * 4u);
            let _e107 = MB.X1_[_e104];
            let _e108 = bitcast<vec4<f32>>(_e107);
            let _e115 = mat2x2<f32>(vec2<f32>(_e108.x, _e108.y), vec2<f32>(_e108.z, _e108.w));
            let _e116 = (_e104 + 1u);
            let _e119 = MB.X1_[_e116];
            let _e123 = bitcast<f32>(_e119.z);
            let _e125 = bitcast<f32>(_e119.w);
            let _e126 = (_e91.w & 8388608u);
            phi_2056_ = _e72.y;
            phi_2028_ = _e76;
            local = _e102;
            local_1 = _e104;
            local_2 = _e116;
            if (_e126 != 0u) {
                phi_2056_ = _e73.y;
                phi_2028_ = i32(_e73.x);
            }
            let _e132 = phi_2056_;
            let _e134 = phi_2028_;
            phi_2039_ = _e86;
            phi_2037_ = _e91;
            phi_2036_ = _e91.w;
            if (_e134 != _e84) {
                let _e137 = ((_e86 + _e134) - _e84);
                let _e142 = textureLoad(DC, vec2<i32>((_e137 & 2047i), (_e137 >> bitcast<u32>(11i))), 0i);
                if ((_e142.w & 8454143u) != (_e91.w & 8454143u)) {
                    let _e147 = (_e123 == 0f);
                    phi_1341_ = _e147;
                    if !(_e147) {
                        phi_1341_ = (_e100.x != 0f);
                    }
                    let _e152 = phi_1341_;
                    phi_2041_ = _e86;
                    phi_2033_ = _e91;
                    if _e152 {
                        let _e153 = bitcast<i32>(_e98.w);
                        let _e158 = textureLoad(DC, vec2<i32>((_e153 & 2047i), (_e153 >> bitcast<u32>(11i))), 0i);
                        phi_2041_ = _e153;
                        phi_2033_ = _e158;
                    }
                    let _e160 = phi_2041_;
                    let _e162 = phi_2033_;
                    phi_2040_ = _e160;
                    phi_2032_ = _e162;
                } else {
                    phi_2040_ = _e137;
                    phi_2032_ = _e142;
                }
                let _e164 = phi_2040_;
                let _e166 = phi_2032_;
                phi_2039_ = _e164;
                phi_2037_ = _e166;
                phi_2036_ = ((_e166.w & 4286578687u) | _e126);
            }
            let _e171 = phi_2039_;
            let _e173 = phi_2037_;
            let _e175 = phi_2036_;
            let _e176 = (_e175 & 469762048u);
            if ((_e176 == 67108864u) && (_e82 == 0i)) {
                let _e182 = f32((_e173.z & 65535u));
                let _e185 = f32((_e173.z >> bitcast<u32>(16i)));
                let _e191 = vec2<i32>(i32((-1f - _e182)), i32(((_e185 - _e182) + 1f)));
                phi_2043_ = _e191;
                if ((_e175 & 8388608u) != 0u) {
                    phi_2043_ = -(_e191);
                }
                let _e196 = phi_2043_;
                let _e198 = (_e171 + _e196.x);
                let _e203 = textureLoad(DC, vec2<i32>((_e198 & 2047i), (_e198 >> bitcast<u32>(11i))), 0i);
                let _e205 = (_e171 + _e196.y);
                let _e210 = textureLoad(DC, vec2<i32>((_e205 & 2047i), (_e205 >> bitcast<u32>(11i))), 0i);
                phi_2044_ = _e210;
                if ((_e210.w & 8454143u) != (_e203.w & 8454143u)) {
                    let _e216 = bitcast<i32>(_e98.w);
                    let _e221 = textureLoad(DC, vec2<i32>((_e216 & 2047i), (_e216 >> bitcast<u32>(11i))), 0i);
                    phi_2044_ = _e221;
                }
                let _e223 = phi_2044_;
                let _e225 = bitcast<f32>(_e203.z);
                let _e227 = bitcast<f32>(_e223.z);
                let _e228 = (_e227 - _e225);
                phi_2048_ = _e228;
                if (abs(_e228) > 3.1415927f) {
                    phi_2048_ = (_e228 - (6.2831855f * sign(_e228)));
                }
                let _e235 = phi_2048_;
                let _e236 = (_e185 + -2f);
                let _e242 = clamp(round(((abs(_e235) * 0.31830987f) * _e236)), 1f, (_e185 + -3f));
                let _e243 = (_e236 - _e242);
                if (_e182 <= _e243) {
                    phi_2119_ = _e132;
                    if (_e182 == _e243) {
                        phi_2119_ = -(_e132);
                    }
                    let _e252 = phi_2119_;
                    phi_2118_ = _e252;
                    phi_2066_ = -(((3.1415927f * sign(_e235)) - _e235));
                    phi_2063_ = _e243;
                    phi_2060_ = _e182;
                } else {
                    let _e254 = (_e182 == (_e243 + 1f));
                    if _e254 {
                        phi_2062_ = 0f;
                    } else {
                        phi_2062_ = (_e182 - (_e243 + 2f));
                    }
                    let _e258 = phi_2062_;
                    phi_2118_ = select(_e132, 0f, _e254);
                    phi_2066_ = _e235;
                    phi_2063_ = select(_e242, 0f, _e254);
                    phi_2060_ = _e258;
                }
                let _e262 = phi_2118_;
                let _e264 = phi_2066_;
                let _e266 = phi_2063_;
                let _e268 = phi_2060_;
                if (_e268 == _e266) {
                    phi_2070_ = _e227;
                } else {
                    phi_2070_ = (_e225 + (_e264 * (_e268 / _e266)));
                }
                let _e274 = phi_2070_;
                phi_2116_ = _e262;
                phi_2069_ = _e274;
            } else {
                phi_2116_ = _e132;
                phi_2069_ = bitcast<f32>(_e173.z);
            }
            let _e278 = phi_2116_;
            let _e280 = phi_2069_;
            let _e284 = vec2<f32>(sin(_e280), -(cos(_e280)));
            let _e286 = bitcast<vec2<f32>>(_e173.xy);
            phi_2125_ = _e125;
            if (_e125 != 0f) {
                phi_2125_ = max(_e125, (1f / length((_e115 * _e284))));
            }
            let _e293 = phi_2125_;
            if (_e123 != 0f) {
                let _e297 = (_e278 * sign(determinant(_e115)));
                let _e299 = ((_e175 & 1048576u) != 0u);
                phi_2122_ = _e297;
                if _e299 {
                    phi_2122_ = min(_e297, 0f);
                }
                let _e302 = phi_2122_;
                phi_2179_ = _e302;
                if ((_e175 & 524288u) != 0u) {
                    phi_2179_ = max(_e302, 0f);
                }
                let _e307 = phi_2179_;
                let _e309 = select(0f, _e293, (_e293 != 0f));
                let _e313 = select(_e123, _e309, ((_e309 > _e123) && (_e293 == 0f)));
                let _e314 = (_e313 + _e309);
                let _e315 = (_e284 * _e314);
                phi_2187_ = _e315;
                if (_e176 > 134217728u) {
                    let _e317 = (_e175 & 4194304u);
                    let _e319 = select(2i, -2i, (_e317 == 0u));
                    phi_2151_ = _e319;
                    if ((_e175 & 8388608u) != 0u) {
                        phi_2151_ = -(_e319);
                    }
                    let _e324 = phi_2151_;
                    let _e325 = (_e171 + _e324);
                    let _e330 = textureLoad(DC, vec2<i32>((_e325 & 2047i), (_e325 >> bitcast<u32>(11i))), 0i);
                    let _e334 = abs((bitcast<f32>(_e330.z) - _e280));
                    phi_2161_ = _e334;
                    if (_e334 > 3.1415927f) {
                        phi_2161_ = (6.2831855f - _e334);
                    }
                    let _e338 = phi_2161_;
                    let _e343 = ((_e338 * select(0.5f, -0.5f, ((_e317 != 0u) == _e299))) + _e280);
                    let _e347 = vec2<f32>(sin(_e343), -(cos(_e343)));
                    let _e348 = (_e115 * _e347);
                    let _e358 = cos((_e338 * 0.5f));
                    let _e359 = (_e176 == 335544320u);
                    phi_1653_ = _e359;
                    if !(_e359) {
                        phi_1653_ = ((_e176 == 268435456u) && (_e358 >= 0.25f));
                    }
                    let _e365 = phi_1653_;
                    if _e365 {
                        phi_2168_ = (_e313 * (1f / max(_e358, select(0.25f, 1f, ((_e175 & 33554432u) != 0u)))));
                    } else {
                        phi_2168_ = ((_e313 * _e358) + (((abs(_e348.x) + abs(_e348.y)) * (1f / dot(_e348, _e348))) * 0.5f));
                    }
                    let _e376 = phi_2168_;
                    phi_2188_ = _e315;
                    if ((_e175 & 2097152u) != 0u) {
                        if (_e314 <= ((_e376 * _e358) + (_e309 * 0.125f))) {
                            phi_2189_ = (_e347 * (_e314 * (1f / _e358)));
                        } else {
                            let _e386 = (_e347 * _e376);
                            phi_2189_ = (vec2<f32>(dot(_e315, _e315), dot(_e386, _e386)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e315, _e386)));
                        }
                        let _e394 = phi_2189_;
                        phi_2188_ = _e394;
                    }
                    let _e396 = phi_2188_;
                    phi_2187_ = _e396;
                }
                let _e398 = phi_2187_;
                if (_e82 != 0i) {
                    phi_2238_ = u32();
                    phi_2209_ = vec2<f32>();
                    phi_2208_ = false;
                    break;
                }
                phi_2205_ = (_e115 * (_e398 * _e307));
                phi_2190_ = _e286;
            } else {
                if (((_e175 & 2147483648u) != 0u) && (_e82 != 1i)) {
                    phi_2238_ = u32();
                    phi_2209_ = vec2<f32>();
                    phi_2208_ = false;
                    break;
                }
                phi_2205_ = vec2<f32>(0f, 0f);
                phi_2190_ = select(_e286, _e100, vec2((_e82 == 2i)));
            }
            let _e410 = phi_2205_;
            let _e412 = phi_2190_;
            let _e419 = MB.X1_[(_e104 + 2u)];
            phi_2238_ = _e419.x;
            phi_2209_ = (((_e115 * _e412) + _e410) + bitcast<vec2<f32>>(_e119.xy));
            phi_2208_ = true;
            break;
        }
    }
    let _e422 = phi_2238_;
    let _e424 = phi_2209_;
    let _e426 = phi_2208_;
    let _e429 = local;
    let _e431 = TC.X1_[_e429];
    let _e433 = (_e431.x & 15u);
    if Mg {
        let _e434 = (_e433 == 0u);
        if _e434 {
            phi_2267_ = _e431.y;
        } else {
            phi_2267_ = _e431.x;
        }
        let _e437 = phi_2267_;
        let _e439 = (_e437 >> bitcast<u32>(16i));
        let _e441 = k.Y5_;
        if (_e439 == 0u) {
            phi_2268_ = 0f;
        } else {
            phi_2268_ = unpack2x16float(((_e439 + 1023u) * _e441)).x;
        }
        let _e448 = phi_2268_;
        phi_2269_ = _e448;
        if _e434 {
            phi_2269_ = -(_e448);
        }
        let _e451 = phi_2269_;
        S1_[0u] = _e451;
    }
    if Og {
        Z1_ = f32(((_e431.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e433 == 1u) {
        let _e459 = unpack4x8unorm(_e431.y);
        if Og {
            phi_2271_ = _e459;
        } else {
            let _e462 = (_e459.xyz * _e459.w);
            let _e468 = vec4<f32>(_e462.x, _e459.y, _e459.z, _e459.w);
            let _e474 = vec4<f32>(_e468.x, _e462.y, _e468.z, _e468.w);
            phi_2271_ = vec4<f32>(_e474.x, _e474.y, _e462.z, _e474.w);
        }
        let _e482 = phi_2271_;
        i1_ = _e482;
    } else {
        if (Mg && (_e433 == 0u)) {
            let _e486 = (_e431.x >> bitcast<u32>(16i));
            let _e488 = k.Y5_;
            if (_e486 == 0u) {
                phi_2270_ = 0f;
            } else {
                phi_2270_ = unpack2x16float(((_e486 + 1023u) * _e488)).x;
            }
            let _e495 = phi_2270_;
            S1_[1u] = _e495;
        } else {
            let _e499 = local_1;
            let _e501 = PB.X1_[_e499];
            let _e511 = local_2;
            let _e513 = PB.X1_[_e511];
            let _e516 = ((mat2x2<f32>(vec2<f32>(_e501.x, _e501.y), vec2<f32>(_e501.z, _e501.w)) * _e424) + _e513.xy);
            let _e517 = (_e433 == 2u);
            if (_e517 || (_e433 == 3u)) {
                i1_[3u] = -(bitcast<f32>(_e431.y));
                if (_e513.z > 0.9f) {
                    i1_[2u] = 2f;
                } else {
                    i1_[2u] = _e513.w;
                }
                if _e517 {
                    i1_[1u] = 0f;
                    i1_[0u] = _e516.x;
                } else {
                    let _e533 = i1_[2u];
                    i1_[2u] = -(_e533);
                    i1_[0u] = _e516.x;
                    i1_[1u] = _e516.y;
                }
            } else {
                i1_ = vec4<f32>(_e516.x, _e516.y, bitcast<f32>(_e431.y), (-2f - _e513.z));
            }
        }
    }
    if _e426 {
        let _e547 = k.Xe;
        let _e549 = k.Ye;
        let _e557 = vec4<f32>(((_e424.x * _e547) - 1f), ((_e424.y * _e549) - sign(_e549)), 0f, 1f);
        phi_2284_ = vec4<f32>(_e557.x, _e557.y, (1f - (f32(_e422) * 0.000061035156f)), _e557.w);
    } else {
        let _e567 = k.P2_;
        phi_2284_ = vec4(_e567);
    }
    let _e570 = phi_2284_;
    unnamed.gl_Position = _e570;
    return;
}

@vertex 
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) SB: vec4<f32>, @location(1) TB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    SB_1 = SB;
    TB_1 = TB;
    main_1();
    let _e15 = S1_;
    let _e16 = Z1_;
    let _e17 = i1_;
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
