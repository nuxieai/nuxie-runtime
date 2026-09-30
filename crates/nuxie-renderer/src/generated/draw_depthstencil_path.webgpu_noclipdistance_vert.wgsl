struct Jg {
    g2_: array<vec4<u32>>,
}

struct Ig {
    g2_: array<vec4<u32>>,
}

struct kf {
    g2_: array<vec2<u32>>,
}

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

struct lf {
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

@id(0) override Hh: bool = true;
@id(2) override Jh: bool = true;
@id(8) override Ph: bool = true;

@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> ID: Jg;
@group(0) @binding(2)
var<storage> OB: Ig;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> DD: kf;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> Y1_: vec2<f32>;
var<private> f1_: f32;
@group(0) @binding(4)
var<storage> PB: lf;
var<private> X1_: vec4<f32>;
var<private> D2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_2153_: f32;
    var phi_2125_: i32;
    var phi_1376_: bool;
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
    var phi_2251_: f32;
    var phi_2237_: f32;
    var phi_1664_: bool;
    var phi_2242_: f32;
    var phi_2259_: vec2<f32>;
    var phi_2258_: vec2<f32>;
    var phi_2257_: vec2<f32>;
    var phi_2280_: bool;
    var phi_2275_: vec2<f32>;
    var phi_2260_: vec2<f32>;
    var phi_2303_: u32;
    var phi_2304_: f32;
    var phi_2305_: f32;
    var phi_2342_: f32;
    var phi_2340_: vec4<f32>;
    var phi_2341_: vec4<f32>;
    var phi_1094_: bool;
    var phi_2355_: vec4<f32>;

    let _e79 = gl_InstanceIndex_1;
    let _e80 = VB_1;
    let _e81 = WB_1;
    let _e83 = i32(_e80.x);
    let _e86 = bitcast<i32>(_e80.w);
    let _e88 = (_e86 >> bitcast<u32>(2i));
    let _e89 = (_e86 & 3i);
    let _e91 = min(_e83, (_e88 - 1i));
    let _e93 = ((_e79 * _e88) + _e91);
    let _e98 = textureLoad(MC, vec2<i32>((_e93 & 2047i), (_e93 >> bitcast<u32>(11i))), 0i);
    let _e105 = ID.g2_[(max((_e98.w & 65535u), 1u) - 1u)];
    let _e107 = bitcast<vec2<f32>>(_e105.xy);
    let _e109 = (_e105.z & 65535u);
    let _e111 = (_e109 * 4u);
    let _e114 = OB.g2_[_e111];
    let _e115 = bitcast<vec4<f32>>(_e114);
    let _e122 = mat2x2<f32>(vec2<f32>(_e115.x, _e115.y), vec2<f32>(_e115.z, _e115.w));
    let _e126 = OB.g2_[(_e111 + 1u)];
    let _e130 = bitcast<f32>(_e126.z);
    let _e132 = bitcast<f32>(_e126.w);
    let _e133 = (_e98.w & 8388608u);
    phi_2153_ = _e80.y;
    phi_2125_ = _e83;
    if (_e133 != 0u) {
        phi_2153_ = _e81.y;
        phi_2125_ = i32(_e81.x);
    }
    let _e139 = phi_2153_;
    let _e141 = phi_2125_;
    phi_2136_ = _e93;
    phi_2134_ = _e98;
    phi_2133_ = _e98.w;
    if (_e141 != _e91) {
        let _e144 = ((_e93 + _e141) - _e91);
        let _e149 = textureLoad(MC, vec2<i32>((_e144 & 2047i), (_e144 >> bitcast<u32>(11i))), 0i);
        if ((_e149.w & 8454143u) != (_e98.w & 8454143u)) {
            let _e154 = (_e130 == 0f);
            phi_1376_ = _e154;
            if !(_e154) {
                phi_1376_ = (_e107.x != 0f);
            }
            let _e159 = phi_1376_;
            phi_2138_ = _e93;
            phi_2130_ = _e98;
            if _e159 {
                let _e160 = bitcast<i32>(_e105.w);
                let _e165 = textureLoad(MC, vec2<i32>((_e160 & 2047i), (_e160 >> bitcast<u32>(11i))), 0i);
                phi_2138_ = _e160;
                phi_2130_ = _e165;
            }
            let _e167 = phi_2138_;
            let _e169 = phi_2130_;
            phi_2137_ = _e167;
            phi_2129_ = _e169;
        } else {
            phi_2137_ = _e144;
            phi_2129_ = _e149;
        }
        let _e171 = phi_2137_;
        let _e173 = phi_2129_;
        phi_2136_ = _e171;
        phi_2134_ = _e173;
        phi_2133_ = ((_e173.w & 4286578687u) | _e133);
    }
    let _e178 = phi_2136_;
    let _e180 = phi_2134_;
    let _e182 = phi_2133_;
    let _e183 = (_e182 & 469762048u);
    if ((_e183 == 67108864u) && (_e89 == 0i)) {
        let _e189 = f32((_e180.z & 65535u));
        let _e192 = f32((_e180.z >> bitcast<u32>(16i)));
        let _e198 = vec2<i32>(i32((-1f - _e189)), i32(((_e192 - _e189) + 1f)));
        phi_2140_ = _e198;
        if ((_e182 & 8388608u) != 0u) {
            phi_2140_ = -(_e198);
        }
        let _e203 = phi_2140_;
        let _e205 = (_e178 + _e203.x);
        let _e210 = textureLoad(MC, vec2<i32>((_e205 & 2047i), (_e205 >> bitcast<u32>(11i))), 0i);
        let _e212 = (_e178 + _e203.y);
        let _e217 = textureLoad(MC, vec2<i32>((_e212 & 2047i), (_e212 >> bitcast<u32>(11i))), 0i);
        phi_2141_ = _e217;
        if ((_e217.w & 8454143u) != (_e210.w & 8454143u)) {
            let _e223 = bitcast<i32>(_e105.w);
            let _e228 = textureLoad(MC, vec2<i32>((_e223 & 2047i), (_e223 >> bitcast<u32>(11i))), 0i);
            phi_2141_ = _e228;
        }
        let _e230 = phi_2141_;
        let _e233 = (f32(_e210.z) * 0.0000000014629181f);
        let _e236 = (f32(_e230.z) * 0.0000000014629181f);
        let _e237 = (_e236 - _e233);
        phi_2145_ = _e237;
        if (abs(_e237) > 3.1415927f) {
            phi_2145_ = (_e237 - (6.2831855f * sign(_e237)));
        }
        let _e244 = phi_2145_;
        let _e245 = (_e192 + -2f);
        let _e251 = clamp(round(((abs(_e244) * 0.31830987f) * _e245)), 1f, (_e192 + -3f));
        let _e252 = (_e245 - _e251);
        if (_e189 <= _e252) {
            phi_2216_ = _e139;
            if (_e189 == _e252) {
                phi_2216_ = -(_e139);
            }
            let _e261 = phi_2216_;
            phi_2215_ = _e261;
            phi_2163_ = -(((3.1415927f * sign(_e244)) - _e244));
            phi_2160_ = _e252;
            phi_2157_ = _e189;
        } else {
            let _e263 = (_e189 == (_e252 + 1f));
            if _e263 {
                phi_2159_ = 0f;
            } else {
                phi_2159_ = (_e189 - (_e252 + 2f));
            }
            let _e267 = phi_2159_;
            phi_2215_ = select(_e139, 0f, _e263);
            phi_2163_ = _e244;
            phi_2160_ = select(_e251, 0f, _e263);
            phi_2157_ = _e267;
        }
        let _e271 = phi_2215_;
        let _e273 = phi_2163_;
        let _e275 = phi_2160_;
        let _e277 = phi_2157_;
        if (_e277 == _e275) {
            phi_2167_ = _e236;
        } else {
            phi_2167_ = (_e233 + (_e273 * (_e277 / _e275)));
        }
        let _e283 = phi_2167_;
        phi_2213_ = _e271;
        phi_2166_ = _e283;
    } else {
        phi_2213_ = _e139;
        phi_2166_ = (f32(_e180.z) * 0.0000000014629181f);
    }
    let _e288 = phi_2213_;
    let _e290 = phi_2166_;
    let _e294 = vec2<f32>(sin(_e290), -(cos(_e290)));
    let _e296 = bitcast<vec2<f32>>(_e180.xy);
    phi_2222_ = _e132;
    if (_e132 != 0f) {
        phi_2222_ = max(_e132, (1f / length((_e122 * _e294))));
    }
    let _e303 = phi_2222_;
    if (_e130 != 0f) {
        let _e307 = (_e288 * sign(determinant(_e122)));
        let _e309 = ((_e182 & 1048576u) != 0u);
        phi_2219_ = _e307;
        if _e309 {
            phi_2219_ = min(_e307, 0f);
        }
        let _e312 = phi_2219_;
        phi_2251_ = _e312;
        if ((_e182 & 524288u) != 0u) {
            phi_2251_ = max(_e312, 0f);
        }
        let _e317 = phi_2251_;
        let _e319 = select(0f, _e303, (_e303 != 0f));
        let _e323 = select(_e130, _e319, ((_e319 > _e130) && (_e303 == 0f)));
        let _e324 = (_e323 + _e319);
        let _e325 = (_e294 * _e324);
        phi_2257_ = _e325;
        if (_e183 > 134217728u) {
            let _e331 = f32((_e180.z & 65535u));
            let _e332 = (_e331 * 0.000015259022f);
            let _e336 = sqrt(max((1f - (_e332 * _e332)), 0f));
            phi_2237_ = _e336;
            if (((_e182 & 4194304u) != 0u) == _e309) {
                phi_2237_ = -(_e336);
            }
            let _e340 = phi_2237_;
            let _e345 = (mat2x2<f32>(vec2<f32>(_e332, _e340), vec2<f32>(-(_e340), _e332)) * _e294);
            let _e346 = (_e122 * _e345);
            let _e355 = (_e183 == 335544320u);
            phi_1664_ = _e355;
            if !(_e355) {
                phi_1664_ = ((_e183 == 268435456u) && (_e332 >= 0.25f));
            }
            let _e361 = phi_1664_;
            if _e361 {
                phi_2242_ = (_e323 * (1f / max(_e332, select(0.25f, 1f, ((_e182 & 33554432u) != 0u)))));
            } else {
                phi_2242_ = ((_e323 * _e332) + (((abs(_e346.x) + abs(_e346.y)) * (1f / dot(_e346, _e346))) * 0.5f));
            }
            let _e372 = phi_2242_;
            phi_2258_ = _e325;
            if ((_e182 & 2097152u) != 0u) {
                if (_e324 <= ((_e372 * _e332) + (_e319 * 0.125f))) {
                    phi_2259_ = (_e345 * (_e324 * (65535f / _e331)));
                } else {
                    let _e382 = (_e345 * _e372);
                    phi_2259_ = (vec2<f32>(dot(_e325, _e325), dot(_e382, _e382)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e325, _e382)));
                }
                let _e390 = phi_2259_;
                phi_2258_ = _e390;
            }
            let _e392 = phi_2258_;
            phi_2257_ = _e392;
        }
        let _e394 = phi_2257_;
        phi_2280_ = (_e89 != 0i);
        phi_2275_ = (_e122 * (_e394 * _e317));
        phi_2260_ = _e296;
    } else {
        phi_2280_ = (((_e182 & 2147483648u) != 0u) && (_e89 != 1i));
        phi_2275_ = vec2<f32>(0f, 0f);
        phi_2260_ = select(_e296, _e107, vec2((_e89 == 2i)));
    }
    let _e406 = phi_2280_;
    let _e408 = phi_2275_;
    let _e410 = phi_2260_;
    let _e413 = (((_e122 * _e410) + _e408) + bitcast<vec2<f32>>(_e126.xy));
    let _e417 = OB.g2_[(_e111 + 2u)];
    let _e421 = DD.g2_[_e109];
    let _e423 = (_e421.x & 15u);
    if Hh {
        let _e424 = (_e423 == 0u);
        if _e424 {
            phi_2303_ = _e421.y;
        } else {
            phi_2303_ = _e421.x;
        }
        let _e427 = phi_2303_;
        let _e429 = (_e427 >> bitcast<u32>(16i));
        let _e431 = j.c6_;
        if (_e429 == 0u) {
            phi_2304_ = 0f;
        } else {
            phi_2304_ = unpack2x16float(((_e429 + 1023u) * _e431)).x;
        }
        let _e438 = phi_2304_;
        phi_2305_ = _e438;
        if _e424 {
            phi_2305_ = -(_e438);
        }
        let _e441 = phi_2305_;
        Y1_[0u] = _e441;
    }
    if Jh {
        f1_ = f32(((_e421.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e423 == 1u) {
        X1_ = unpack4x8unorm(_e421.y);
    } else {
        if (Hh && (_e423 == 0u)) {
            let _e453 = (_e421.x >> bitcast<u32>(16i));
            let _e455 = j.c6_;
            if (_e453 == 0u) {
                phi_2342_ = 0f;
            } else {
                phi_2342_ = unpack2x16float(((_e453 + 1023u) * _e455)).x;
            }
            let _e462 = phi_2342_;
            Y1_[1u] = _e462;
        } else {
            let _e464 = (_e109 * 8u);
            let _e467 = PB.g2_[_e464];
            let _e478 = PB.g2_[(_e464 + 1u)];
            let _e487 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e421.y));
            let _e489 = ((mat2x2<f32>(vec2<f32>(_e467.x, _e467.y), vec2<f32>(_e467.z, _e467.w)) * _e413) + _e478.xy);
            if (_e478.z > 0.9f) {
                phi_2340_ = vec4<f32>(_e487.x, _e487.y, 2f, _e487.w);
            } else {
                phi_2340_ = vec4<f32>(_e487.x, _e487.y, _e478.w, _e487.w);
            }
            let _e504 = phi_2340_;
            if (f32(_e423) == 2f) {
                let _e511 = vec4<f32>(_e489.x, _e504.y, _e504.z, _e504.w);
                phi_2341_ = vec4<f32>(_e511.x, 0f, _e511.z, _e511.w);
            } else {
                let _e523 = vec4<f32>(_e504.x, _e504.y, -(_e504.z), _e504.w);
                let _e529 = vec4<f32>(_e489.x, _e523.y, _e523.z, _e523.w);
                phi_2341_ = vec4<f32>(_e529.x, _e489.y, _e529.z, _e529.w);
            }
            let _e537 = phi_2341_;
            X1_ = _e537;
            let _e539 = X1_[3u];
            X1_[3u] = -(_e539);
        }
    }
    phi_1094_ = Ph;
    if Ph {
        phi_1094_ = ((_e421.x & 2048u) != 0u);
    }
    let _e544 = phi_1094_;
    if _e544 {
        let _e545 = (_e109 * 8u);
        let _e549 = PB.g2_[(_e545 + 4u)];
        let _e560 = PB.g2_[(_e545 + 5u)];
        let _e563 = ((mat2x2<f32>(vec2<f32>(_e549.x, _e549.y), vec2<f32>(_e549.z, _e549.w)) * _e413) + _e560.xy);
        D2_ = vec3<f32>(_e563.x, _e563.y, (1f + _e560.z));
    } else {
        D2_ = vec3<f32>(0f, 0f, 0f);
    }
    if !(_e406) {
        let _e571 = j.Hf;
        let _e573 = j.If;
        let _e581 = vec4<f32>(((_e413.x * _e571) - 1f), ((_e413.y * _e573) - sign(_e573)), 0f, 1f);
        phi_2355_ = vec4<f32>(_e581.x, _e581.y, ((f32(((_e417.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e581.w);
    } else {
        let _e594 = j.X2_;
        phi_2355_ = vec4(_e594);
    }
    let _e597 = phi_2355_;
    unnamed.gl_Position = _e597;
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
