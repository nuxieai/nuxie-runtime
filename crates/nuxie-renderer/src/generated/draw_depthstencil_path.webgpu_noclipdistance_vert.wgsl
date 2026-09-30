struct Lg {
    g2_: array<vec4<u32>>,
}

struct Kg {
    g2_: array<vec4<u32>>,
}

struct qf {
    g2_: array<vec2<u32>>,
}

struct SB {
    yc: f32,
    Id: f32,
    Nf: f32,
    Of: f32,
    r6_: u32,
    Sb: u32,
    zf: u32,
    Af: u32,
    X7_: vec4<i32>,
    kh: vec2<f32>,
    Jd: vec2<f32>,
    f2_: u32,
    oh: f32,
    g6_: u32,
    W2_: f32,
    Kd: f32,
    tf: u32,
    F3_: f32,
    G3_: f32,
    Ld: f32,
    hh: u32,
    Rb: u32,
    ec: f32,
    fc: f32,
}

struct rf {
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

@id(0) override Kh: bool = true;
@id(2) override Mh: bool = true;
@id(8) override Sh: bool = true;

@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> HD: Lg;
@group(0) @binding(2)
var<storage> OB: Kg;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> UB_1: vec4<f32>;
var<private> VB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> CD: qf;
@group(0) @binding(0)
var<uniform> j: SB;
var<private> Y1_: vec2<f32>;
var<private> g1_: f32;
@group(0) @binding(4)
var<storage> PB: rf;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(3) @binding(9)
var ha: sampler;

fn main_1() {
    var phi_2153_: f32;
    var phi_2125_: i32;
    var phi_1373_: bool;
    var phi_2138_: i32;
    var phi_2130_: vec4<u32>;
    var phi_2137_: i32;
    var phi_2129_: vec4<u32>;
    var phi_2136_: i32;
    var phi_2134_: vec4<u32>;
    var phi_2133_: u32;
    var phi_2140_: vec2<i32>;
    var phi_2141_: vec4<u32>;
    var phi_2145_: f32;
    var phi_2216_: f32;
    var phi_2159_: f32;
    var phi_2215_: f32;
    var phi_2163_: f32;
    var phi_2160_: f32;
    var phi_2157_: f32;
    var phi_2167_: f32;
    var phi_2213_: f32;
    var phi_2166_: f32;
    var phi_2222_: f32;
    var phi_2219_: f32;
    var phi_2276_: f32;
    var phi_2248_: i32;
    var phi_2258_: f32;
    var phi_1685_: bool;
    var phi_2265_: f32;
    var phi_2286_: vec2<f32>;
    var phi_2285_: vec2<f32>;
    var phi_2284_: vec2<f32>;
    var phi_2309_: bool;
    var phi_2304_: vec2<f32>;
    var phi_2287_: vec2<f32>;
    var phi_2334_: u32;
    var phi_2335_: f32;
    var phi_2336_: f32;
    var phi_2375_: f32;
    var phi_2373_: vec4<f32>;
    var phi_2374_: vec4<f32>;
    var phi_1092_: bool;
    var phi_2388_: vec4<f32>;

    let _e76 = gl_InstanceIndex_1;
    let _e77 = UB_1;
    let _e78 = VB_1;
    let _e80 = i32(_e77.x);
    let _e83 = bitcast<i32>(_e77.w);
    let _e85 = (_e83 >> bitcast<u32>(2i));
    let _e86 = (_e83 & 3i);
    let _e88 = min(_e80, (_e85 - 1i));
    let _e90 = ((_e76 * _e85) + _e88);
    let _e95 = textureLoad(JC, vec2<i32>((_e90 & 2047i), (_e90 >> bitcast<u32>(11i))), 0i);
    let _e102 = HD.g2_[(max((_e95.w & 65535u), 1u) - 1u)];
    let _e104 = bitcast<vec2<f32>>(_e102.xy);
    let _e106 = (_e102.z & 65535u);
    let _e108 = (_e106 * 4u);
    let _e111 = OB.g2_[_e108];
    let _e112 = bitcast<vec4<f32>>(_e111);
    let _e119 = mat2x2<f32>(vec2<f32>(_e112.x, _e112.y), vec2<f32>(_e112.z, _e112.w));
    let _e123 = OB.g2_[(_e108 + 1u)];
    let _e127 = bitcast<f32>(_e123.z);
    let _e129 = bitcast<f32>(_e123.w);
    let _e130 = (_e95.w & 8388608u);
    phi_2153_ = _e77.y;
    phi_2125_ = _e80;
    if (_e130 != 0u) {
        phi_2153_ = _e78.y;
        phi_2125_ = i32(_e78.x);
    }
    let _e136 = phi_2153_;
    let _e138 = phi_2125_;
    phi_2136_ = _e90;
    phi_2134_ = _e95;
    phi_2133_ = _e95.w;
    if (_e138 != _e88) {
        let _e141 = ((_e90 + _e138) - _e88);
        let _e146 = textureLoad(JC, vec2<i32>((_e141 & 2047i), (_e141 >> bitcast<u32>(11i))), 0i);
        if ((_e146.w & 8454143u) != (_e95.w & 8454143u)) {
            let _e151 = (_e127 == 0f);
            phi_1373_ = _e151;
            if !(_e151) {
                phi_1373_ = (_e104.x != 0f);
            }
            let _e156 = phi_1373_;
            phi_2138_ = _e90;
            phi_2130_ = _e95;
            if _e156 {
                let _e157 = bitcast<i32>(_e102.w);
                let _e162 = textureLoad(JC, vec2<i32>((_e157 & 2047i), (_e157 >> bitcast<u32>(11i))), 0i);
                phi_2138_ = _e157;
                phi_2130_ = _e162;
            }
            let _e164 = phi_2138_;
            let _e166 = phi_2130_;
            phi_2137_ = _e164;
            phi_2129_ = _e166;
        } else {
            phi_2137_ = _e141;
            phi_2129_ = _e146;
        }
        let _e168 = phi_2137_;
        let _e170 = phi_2129_;
        phi_2136_ = _e168;
        phi_2134_ = _e170;
        phi_2133_ = ((_e170.w & 4286578687u) | _e130);
    }
    let _e175 = phi_2136_;
    let _e177 = phi_2134_;
    let _e179 = phi_2133_;
    let _e180 = (_e179 & 469762048u);
    if ((_e180 == 67108864u) && (_e86 == 0i)) {
        let _e186 = f32((_e177.z & 65535u));
        let _e189 = f32((_e177.z >> bitcast<u32>(16i)));
        let _e195 = vec2<i32>(i32((-1f - _e186)), i32(((_e189 - _e186) + 1f)));
        phi_2140_ = _e195;
        if ((_e179 & 8388608u) != 0u) {
            phi_2140_ = -(_e195);
        }
        let _e200 = phi_2140_;
        let _e202 = (_e175 + _e200.x);
        let _e207 = textureLoad(JC, vec2<i32>((_e202 & 2047i), (_e202 >> bitcast<u32>(11i))), 0i);
        let _e209 = (_e175 + _e200.y);
        let _e214 = textureLoad(JC, vec2<i32>((_e209 & 2047i), (_e209 >> bitcast<u32>(11i))), 0i);
        phi_2141_ = _e214;
        if ((_e214.w & 8454143u) != (_e207.w & 8454143u)) {
            let _e220 = bitcast<i32>(_e102.w);
            let _e225 = textureLoad(JC, vec2<i32>((_e220 & 2047i), (_e220 >> bitcast<u32>(11i))), 0i);
            phi_2141_ = _e225;
        }
        let _e227 = phi_2141_;
        let _e229 = bitcast<f32>(_e207.z);
        let _e231 = bitcast<f32>(_e227.z);
        let _e232 = (_e231 - _e229);
        phi_2145_ = _e232;
        if (abs(_e232) > 3.1415927f) {
            phi_2145_ = (_e232 - (6.2831855f * sign(_e232)));
        }
        let _e239 = phi_2145_;
        let _e240 = (_e189 + -2f);
        let _e246 = clamp(round(((abs(_e239) * 0.31830987f) * _e240)), 1f, (_e189 + -3f));
        let _e247 = (_e240 - _e246);
        if (_e186 <= _e247) {
            phi_2216_ = _e136;
            if (_e186 == _e247) {
                phi_2216_ = -(_e136);
            }
            let _e256 = phi_2216_;
            phi_2215_ = _e256;
            phi_2163_ = -(((3.1415927f * sign(_e239)) - _e239));
            phi_2160_ = _e247;
            phi_2157_ = _e186;
        } else {
            let _e258 = (_e186 == (_e247 + 1f));
            if _e258 {
                phi_2159_ = 0f;
            } else {
                phi_2159_ = (_e186 - (_e247 + 2f));
            }
            let _e262 = phi_2159_;
            phi_2215_ = select(_e136, 0f, _e258);
            phi_2163_ = _e239;
            phi_2160_ = select(_e246, 0f, _e258);
            phi_2157_ = _e262;
        }
        let _e266 = phi_2215_;
        let _e268 = phi_2163_;
        let _e270 = phi_2160_;
        let _e272 = phi_2157_;
        if (_e272 == _e270) {
            phi_2167_ = _e231;
        } else {
            phi_2167_ = (_e229 + (_e268 * (_e272 / _e270)));
        }
        let _e278 = phi_2167_;
        phi_2213_ = _e266;
        phi_2166_ = _e278;
    } else {
        phi_2213_ = _e136;
        phi_2166_ = bitcast<f32>(_e177.z);
    }
    let _e282 = phi_2213_;
    let _e284 = phi_2166_;
    let _e288 = vec2<f32>(sin(_e284), -(cos(_e284)));
    let _e290 = bitcast<vec2<f32>>(_e177.xy);
    phi_2222_ = _e129;
    if (_e129 != 0f) {
        phi_2222_ = max(_e129, (1f / length((_e119 * _e288))));
    }
    let _e297 = phi_2222_;
    if (_e127 != 0f) {
        let _e301 = (_e282 * sign(determinant(_e119)));
        let _e303 = ((_e179 & 1048576u) != 0u);
        phi_2219_ = _e301;
        if _e303 {
            phi_2219_ = min(_e301, 0f);
        }
        let _e306 = phi_2219_;
        phi_2276_ = _e306;
        if ((_e179 & 524288u) != 0u) {
            phi_2276_ = max(_e306, 0f);
        }
        let _e311 = phi_2276_;
        let _e313 = select(0f, _e297, (_e297 != 0f));
        let _e317 = select(_e127, _e313, ((_e313 > _e127) && (_e297 == 0f)));
        let _e318 = (_e317 + _e313);
        let _e319 = (_e288 * _e318);
        phi_2284_ = _e319;
        if (_e180 > 134217728u) {
            let _e321 = (_e179 & 4194304u);
            let _e323 = select(2i, -2i, (_e321 == 0u));
            phi_2248_ = _e323;
            if ((_e179 & 8388608u) != 0u) {
                phi_2248_ = -(_e323);
            }
            let _e328 = phi_2248_;
            let _e329 = (_e175 + _e328);
            let _e334 = textureLoad(JC, vec2<i32>((_e329 & 2047i), (_e329 >> bitcast<u32>(11i))), 0i);
            let _e338 = abs((bitcast<f32>(_e334.z) - _e284));
            phi_2258_ = _e338;
            if (_e338 > 3.1415927f) {
                phi_2258_ = (6.2831855f - _e338);
            }
            let _e342 = phi_2258_;
            let _e347 = ((_e342 * select(0.5f, -0.5f, ((_e321 != 0u) == _e303))) + _e284);
            let _e351 = vec2<f32>(sin(_e347), -(cos(_e347)));
            let _e352 = (_e119 * _e351);
            let _e362 = cos((_e342 * 0.5f));
            let _e363 = (_e180 == 335544320u);
            phi_1685_ = _e363;
            if !(_e363) {
                phi_1685_ = ((_e180 == 268435456u) && (_e362 >= 0.25f));
            }
            let _e369 = phi_1685_;
            if _e369 {
                phi_2265_ = (_e317 * (1f / max(_e362, select(0.25f, 1f, ((_e179 & 33554432u) != 0u)))));
            } else {
                phi_2265_ = ((_e317 * _e362) + (((abs(_e352.x) + abs(_e352.y)) * (1f / dot(_e352, _e352))) * 0.5f));
            }
            let _e380 = phi_2265_;
            phi_2285_ = _e319;
            if ((_e179 & 2097152u) != 0u) {
                if (_e318 <= ((_e380 * _e362) + (_e313 * 0.125f))) {
                    phi_2286_ = (_e351 * (_e318 * (1f / _e362)));
                } else {
                    let _e390 = (_e351 * _e380);
                    phi_2286_ = (vec2<f32>(dot(_e319, _e319), dot(_e390, _e390)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e319, _e390)));
                }
                let _e398 = phi_2286_;
                phi_2285_ = _e398;
            }
            let _e400 = phi_2285_;
            phi_2284_ = _e400;
        }
        let _e402 = phi_2284_;
        phi_2309_ = (_e86 != 0i);
        phi_2304_ = (_e119 * (_e402 * _e311));
        phi_2287_ = _e290;
    } else {
        phi_2309_ = (((_e179 & 2147483648u) != 0u) && (_e86 != 1i));
        phi_2304_ = vec2<f32>(0f, 0f);
        phi_2287_ = select(_e290, _e104, vec2((_e86 == 2i)));
    }
    let _e414 = phi_2309_;
    let _e416 = phi_2304_;
    let _e418 = phi_2287_;
    let _e421 = (((_e119 * _e418) + _e416) + bitcast<vec2<f32>>(_e123.xy));
    let _e425 = OB.g2_[(_e108 + 2u)];
    let _e429 = CD.g2_[_e106];
    let _e431 = (_e429.x & 15u);
    if Kh {
        let _e432 = (_e431 == 0u);
        if _e432 {
            phi_2334_ = _e429.y;
        } else {
            phi_2334_ = _e429.x;
        }
        let _e435 = phi_2334_;
        let _e437 = (_e435 >> bitcast<u32>(16i));
        let _e439 = j.g6_;
        if (_e437 == 0u) {
            phi_2335_ = 0f;
        } else {
            phi_2335_ = unpack2x16float(((_e437 + 1023u) * _e439)).x;
        }
        let _e446 = phi_2335_;
        phi_2336_ = _e446;
        if _e432 {
            phi_2336_ = -(_e446);
        }
        let _e449 = phi_2336_;
        Y1_[0u] = _e449;
    }
    if Mh {
        g1_ = f32(((_e429.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e431 == 1u) {
        X1_ = unpack4x8unorm(_e429.y);
    } else {
        if (Kh && (_e431 == 0u)) {
            let _e461 = (_e429.x >> bitcast<u32>(16i));
            let _e463 = j.g6_;
            if (_e461 == 0u) {
                phi_2375_ = 0f;
            } else {
                phi_2375_ = unpack2x16float(((_e461 + 1023u) * _e463)).x;
            }
            let _e470 = phi_2375_;
            Y1_[1u] = _e470;
        } else {
            let _e472 = (_e106 * 8u);
            let _e475 = PB.g2_[_e472];
            let _e486 = PB.g2_[(_e472 + 1u)];
            let _e495 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e429.y));
            let _e497 = ((mat2x2<f32>(vec2<f32>(_e475.x, _e475.y), vec2<f32>(_e475.z, _e475.w)) * _e421) + _e486.xy);
            if (_e486.z > 0.9f) {
                phi_2373_ = vec4<f32>(_e495.x, _e495.y, 2f, _e495.w);
            } else {
                phi_2373_ = vec4<f32>(_e495.x, _e495.y, _e486.w, _e495.w);
            }
            let _e512 = phi_2373_;
            if (f32(_e431) == 2f) {
                let _e519 = vec4<f32>(_e497.x, _e512.y, _e512.z, _e512.w);
                phi_2374_ = vec4<f32>(_e519.x, 0f, _e519.z, _e519.w);
            } else {
                let _e531 = vec4<f32>(_e512.x, _e512.y, -(_e512.z), _e512.w);
                let _e537 = vec4<f32>(_e497.x, _e531.y, _e531.z, _e531.w);
                phi_2374_ = vec4<f32>(_e537.x, _e497.y, _e537.z, _e537.w);
            }
            let _e545 = phi_2374_;
            X1_ = _e545;
            let _e547 = X1_[3u];
            X1_[3u] = -(_e547);
        }
    }
    phi_1092_ = Sh;
    if Sh {
        phi_1092_ = ((_e429.x & 2048u) != 0u);
    }
    let _e552 = phi_1092_;
    if _e552 {
        let _e553 = (_e106 * 8u);
        let _e557 = PB.g2_[(_e553 + 4u)];
        let _e568 = PB.g2_[(_e553 + 5u)];
        let _e571 = ((mat2x2<f32>(vec2<f32>(_e557.x, _e557.y), vec2<f32>(_e557.z, _e557.w)) * _e421) + _e568.xy);
        C2_ = vec3<f32>(_e571.x, _e571.y, (1f + _e568.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e414) {
        let _e579 = j.Nf;
        let _e581 = j.Of;
        let _e589 = vec4<f32>(((_e421.x * _e579) - 1f), ((_e421.y * _e581) - sign(_e581)), 0f, 1f);
        phi_2388_ = vec4<f32>(_e589.x, _e589.y, (1f - (f32(_e425.x) * 0.000061035156f)), _e589.w);
    } else {
        let _e599 = j.W2_;
        phi_2388_ = vec4(_e599);
    }
    let _e602 = phi_2388_;
    unnamed.gl_Position = _e602;
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
