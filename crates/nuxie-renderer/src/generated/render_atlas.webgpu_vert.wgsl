struct pi {
    x2_: array<vec4<u32>>,
}

struct jd {
    x2_: array<vec4<u32>>,
}

struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct ug {
    x2_: array<vec4<f32>>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(4)
var<storage> CD: pi;
@group(0) @binding(2)
var<storage> JB: jd;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> ZB_1: vec4<f32>;
var<private> S: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> KB: ug;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_2269_: f32;
    var phi_2206_: f32;
    var phi_2296_: f32;
    var phi_2232_: f32;
    var phi_2200_: i32;
    var phi_1350_: bool;
    var phi_2217_: i32;
    var phi_2208_: vec4<u32>;
    var phi_2216_: i32;
    var phi_2207_: vec4<u32>;
    var phi_2215_: i32;
    var phi_2213_: vec4<u32>;
    var phi_2212_: u32;
    var phi_2219_: vec2<i32>;
    var phi_2220_: vec4<u32>;
    var phi_2224_: f32;
    var phi_2307_: f32;
    var phi_2239_: f32;
    var phi_2306_: f32;
    var phi_2247_: f32;
    var phi_2240_: f32;
    var phi_2237_: f32;
    var phi_2251_: f32;
    var phi_2329_: f32;
    var phi_2319_: f32;
    var phi_2304_: f32;
    var phi_2250_: f32;
    var phi_2302_: f32;
    var phi_2388_: f32;
    var phi_2399_: f32;
    var phi_2391_: f32;
    var phi_2453_: f32;
    var phi_2427_: f32;
    var phi_1665_: bool;
    var phi_2432_: f32;
    var phi_2444_: vec2<f32>;
    var phi_2443_: vec2<f32>;
    var phi_2461_: vec4<f32>;
    var phi_2474_: vec2<f32>;
    var phi_2460_: vec4<f32>;
    var phi_2508_: vec4<f32>;
    var phi_2340_: f32;
    var phi_2339_: f32;
    var phi_2341_: f32;
    var phi_2345_: f32;
    var phi_2368_: f32;
    var phi_2366_: f32;
    var phi_2384_: vec4<f32>;
    var phi_2505_: vec2<f32>;
    var phi_2383_: vec4<f32>;
    var phi_2511_: vec4<f32>;
    var phi_2512_: bool;
    var phi_2506_: vec4<f32>;
    var phi_2500_: vec2<f32>;
    var phi_2476_: vec2<f32>;
    var phi_2544_: vec4<f32>;

    let _e72 = gl_InstanceIndex_1;
    let _e73 = YB_1;
    let _e74 = ZB_1;
    let _e76 = i32(_e73.x);
    let _e80 = bitcast<i32>(_e73.w);
    let _e82 = (_e80 >> bitcast<u32>(2i));
    let _e83 = (_e80 & 3i);
    let _e85 = min(_e76, (_e82 - 1i));
    let _e87 = ((_e72 * _e82) + _e85);
    let _e92 = textureLoad(VB, vec2<i32>((_e87 & 2047i), (_e87 >> bitcast<u32>(11i))), 0i);
    let _e99 = CD.x2_[(max((_e92.w & 65535u), 1u) - 1u)];
    let _e101 = bitcast<vec2<f32>>(_e99.xy);
    let _e105 = ((_e99.z & 65535u) * 4u);
    let _e108 = JB.x2_[_e105];
    let _e109 = bitcast<vec4<f32>>(_e108);
    let _e116 = mat2x2<f32>(vec2<f32>(_e109.x, _e109.y), vec2<f32>(_e109.z, _e109.w));
    let _e120 = JB.x2_[(_e105 + 1u)];
    phi_2269_ = 0f;
    phi_2206_ = 0f;
    if ((_e120.z & 32768u) != 0u) {
        let _e129 = JB.x2_[(_e105 + 2u)];
        phi_2269_ = bitcast<f32>(_e129.y);
        phi_2206_ = bitcast<f32>(_e129.x);
    }
    let _e135 = phi_2269_;
    let _e137 = phi_2206_;
    let _e138 = (_e92.w & 8388608u);
    phi_2296_ = _e73.z;
    phi_2232_ = _e73.y;
    phi_2200_ = _e76;
    if (_e138 != 0u) {
        phi_2296_ = _e74.z;
        phi_2232_ = _e74.y;
        phi_2200_ = i32(_e74.x);
    }
    let _e145 = phi_2296_;
    let _e147 = phi_2232_;
    let _e149 = phi_2200_;
    phi_2215_ = _e87;
    phi_2213_ = _e92;
    phi_2212_ = _e92.w;
    if (_e149 != _e85) {
        let _e152 = ((_e87 + _e149) - _e85);
        let _e157 = textureLoad(VB, vec2<i32>((_e152 & 2047i), (_e152 >> bitcast<u32>(11i))), 0i);
        if ((_e157.w & 8454143u) != (_e92.w & 8454143u)) {
            let _e162 = (_e137 == 0f);
            phi_1350_ = _e162;
            if !(_e162) {
                phi_1350_ = (_e101.x != 0f);
            }
            let _e167 = phi_1350_;
            phi_2217_ = _e87;
            phi_2208_ = _e92;
            if _e167 {
                let _e168 = bitcast<i32>(_e99.w);
                let _e173 = textureLoad(VB, vec2<i32>((_e168 & 2047i), (_e168 >> bitcast<u32>(11i))), 0i);
                phi_2217_ = _e168;
                phi_2208_ = _e173;
            }
            let _e175 = phi_2217_;
            let _e177 = phi_2208_;
            phi_2216_ = _e175;
            phi_2207_ = _e177;
        } else {
            phi_2216_ = _e152;
            phi_2207_ = _e157;
        }
        let _e179 = phi_2216_;
        let _e181 = phi_2207_;
        phi_2215_ = _e179;
        phi_2213_ = _e181;
        phi_2212_ = ((_e181.w & 4286578687u) | _e138);
    }
    let _e186 = phi_2215_;
    let _e188 = phi_2213_;
    let _e190 = phi_2212_;
    let _e191 = (_e190 & 469762048u);
    let _e194 = ((_e191 == 67108864u) && (_e83 == 0i));
    if _e194 {
        let _e197 = f32((_e188.z & 65535u));
        let _e200 = f32((_e188.z >> bitcast<u32>(16i)));
        let _e206 = vec2<i32>(i32((-1f - _e197)), i32(((_e200 - _e197) + 1f)));
        phi_2219_ = _e206;
        if ((_e190 & 8388608u) != 0u) {
            phi_2219_ = -(_e206);
        }
        let _e211 = phi_2219_;
        let _e213 = (_e186 + _e211.x);
        let _e218 = textureLoad(VB, vec2<i32>((_e213 & 2047i), (_e213 >> bitcast<u32>(11i))), 0i);
        let _e220 = (_e186 + _e211.y);
        let _e225 = textureLoad(VB, vec2<i32>((_e220 & 2047i), (_e220 >> bitcast<u32>(11i))), 0i);
        phi_2220_ = _e225;
        if ((_e225.w & 8454143u) != (_e218.w & 8454143u)) {
            let _e231 = bitcast<i32>(_e99.w);
            let _e236 = textureLoad(VB, vec2<i32>((_e231 & 2047i), (_e231 >> bitcast<u32>(11i))), 0i);
            phi_2220_ = _e236;
        }
        let _e238 = phi_2220_;
        let _e241 = (f32(_e218.z) * 0.0000000014629181f);
        let _e244 = (f32(_e238.z) * 0.0000000014629181f);
        let _e245 = (_e244 - _e241);
        phi_2224_ = _e245;
        if (abs(_e245) > 3.1415927f) {
            phi_2224_ = (_e245 - (6.2831855f * sign(_e245)));
        }
        let _e252 = phi_2224_;
        let _e253 = (_e200 + -2f);
        let _e259 = clamp(round(((abs(_e252) * 0.31830987f) * _e253)), 1f, (_e200 + -3f));
        let _e260 = (_e253 - _e259);
        if (_e197 <= _e260) {
            phi_2307_ = _e147;
            if (_e197 == _e260) {
                phi_2307_ = -(_e147);
            }
            let _e269 = phi_2307_;
            phi_2306_ = _e269;
            phi_2247_ = -(((3.1415927f * sign(_e252)) - _e252));
            phi_2240_ = _e260;
            phi_2237_ = _e197;
        } else {
            let _e271 = (_e197 == (_e260 + 1f));
            if _e271 {
                phi_2239_ = 0f;
            } else {
                phi_2239_ = (_e197 - (_e260 + 2f));
            }
            let _e275 = phi_2239_;
            phi_2306_ = select(_e147, 0f, _e271);
            phi_2247_ = _e252;
            phi_2240_ = select(_e259, 0f, _e271);
            phi_2237_ = _e275;
        }
        let _e279 = phi_2306_;
        let _e281 = phi_2247_;
        let _e283 = phi_2240_;
        let _e285 = phi_2237_;
        if (_e285 == _e283) {
            phi_2251_ = _e244;
        } else {
            phi_2251_ = (_e241 + (_e281 * (_e285 / _e283)));
        }
        let _e291 = phi_2251_;
        phi_2329_ = _e241;
        phi_2319_ = _e281;
        phi_2304_ = _e279;
        phi_2250_ = _e291;
    } else {
        phi_2329_ = f32();
        phi_2319_ = f32();
        phi_2304_ = _e147;
        phi_2250_ = (f32(_e188.z) * 0.0000000014629181f);
    }
    let _e296 = phi_2329_;
    let _e298 = phi_2319_;
    let _e300 = phi_2304_;
    let _e302 = phi_2250_;
    let _e306 = vec2<f32>(sin(_e302), -(cos(_e302)));
    let _e308 = bitcast<vec2<f32>>(_e188.xy);
    phi_2302_ = _e135;
    if (_e135 != 0f) {
        phi_2302_ = max(_e135, (1f / length((_e116 * _e306))));
    }
    let _e315 = phi_2302_;
    if (_e137 != 0f) {
        let _e319 = (_e300 * sign(determinant(_e116)));
        let _e321 = ((_e190 & 1048576u) != 0u);
        phi_2388_ = _e319;
        if _e321 {
            phi_2388_ = min(_e319, 0f);
        }
        let _e324 = phi_2388_;
        phi_2399_ = _e324;
        if ((_e190 & 524288u) != 0u) {
            phi_2399_ = max(_e324, 0f);
        }
        let _e329 = phi_2399_;
        let _e330 = (_e315 != 0f);
        if _e330 {
            phi_2391_ = _e315;
        } else {
            let _e331 = (_e116 * _e306);
            phi_2391_ = (((abs(_e331.x) + abs(_e331.y)) * (1f / dot(_e331, _e331))) * 0.5f);
        }
        let _e342 = phi_2391_;
        let _e345 = ((_e342 > _e137) && (_e315 == 0f));
        phi_2453_ = 1f;
        if _e345 {
            phi_2453_ = (_e137 / _e342);
        }
        let _e348 = phi_2453_;
        let _e349 = select(_e137, _e342, _e345);
        let _e350 = (_e349 + _e342);
        let _e351 = (_e306 * _e350);
        let _e352 = (_e329 * _e350);
        let _e359 = (((vec2<f32>(_e352, -(_e352)) + vec2(_e349)) * (0.5f / _e342)) + vec2<f32>(0.5f, 0.5f));
        let _e362 = vec4<f32>(_e359.x, _e359.y, 0f, 0f);
        phi_2474_ = _e351;
        phi_2460_ = _e362;
        if (_e191 > 201326592u) {
            let _e368 = f32((_e188.z & 65535u));
            let _e369 = (_e368 * 0.000015259022f);
            let _e373 = sqrt(max((1f - (_e369 * _e369)), 0f));
            phi_2427_ = _e373;
            if (((_e190 & 4194304u) != 0u) == _e321) {
                phi_2427_ = -(_e373);
            }
            let _e377 = phi_2427_;
            let _e382 = (mat2x2<f32>(vec2<f32>(_e369, _e377), vec2<f32>(-(_e377), _e369)) * _e306);
            let _e383 = (_e116 * _e382);
            let _e391 = ((abs(_e383.x) + abs(_e383.y)) * (1f / dot(_e383, _e383)));
            let _e392 = (_e191 == 402653184u);
            phi_1665_ = _e392;
            if !(_e392) {
                phi_1665_ = ((_e191 == 335544320u) && (_e369 >= 0.25f));
            }
            let _e398 = phi_1665_;
            if _e398 {
                phi_2432_ = (_e349 * (1f / max(_e369, select(0.25f, 1f, ((_e190 & 33554432u) != 0u)))));
            } else {
                phi_2432_ = ((_e349 * _e369) + (_e391 * 0.5f));
            }
            let _e409 = phi_2432_;
            let _e411 = (_e409 + (_e391 * 0.5f));
            phi_2443_ = _e351;
            if ((_e190 & 2097152u) != 0u) {
                if (_e350 <= ((_e411 * _e369) + (_e342 * 0.125f))) {
                    phi_2444_ = (_e382 * (_e350 * (65535f / _e368)));
                } else {
                    let _e421 = (_e382 * _e411);
                    phi_2444_ = (vec2<f32>(dot(_e351, _e351), dot(_e421, _e421)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e351, _e421)));
                }
                let _e429 = phi_2444_;
                phi_2443_ = _e429;
            }
            let _e431 = phi_2443_;
            let _e436 = ((_e411 - dot((_e431 * abs(_e329)), _e382)) / _e391);
            if _e321 {
                phi_2461_ = vec4<f32>(_e362.x, _e436, _e362.z, _e362.w);
            } else {
                phi_2461_ = vec4<f32>(_e436, _e362.y, _e362.z, _e362.w);
            }
            let _e448 = phi_2461_;
            phi_2474_ = _e431;
            phi_2460_ = _e448;
        }
        let _e450 = phi_2474_;
        let _e452 = phi_2460_;
        let _e454 = (_e452.xy * _e348);
        let _e460 = vec4<f32>(_e454.x, _e452.y, _e452.z, _e452.w);
        let _e467 = vec4<f32>(_e460.x, max(_e454.y, 0.0001f), _e460.z, _e460.w);
        phi_2508_ = _e467;
        if _e330 {
            phi_2508_ = vec4<f32>((-2f - _e454.x), _e467.y, _e467.z, _e467.w);
        }
        let _e475 = phi_2508_;
        phi_2512_ = (_e83 != 0i);
        phi_2506_ = _e475;
        phi_2500_ = (_e116 * (_e450 * _e329));
        phi_2476_ = _e308;
    } else {
        let _e479 = vec4<f32>(_e145, -1f, 0f, 0f);
        if (_e315 != 0f) {
            let _e485 = vec4<f32>(_e479.x, -2f, _e479.z, _e479.w);
            let _e490 = vec4<f32>(_e485.x, _e485.y, 1000000f, _e485.w);
            phi_2384_ = vec4<f32>(_e490.x, _e490.y, _e490.z, _e145);
            if _e194 {
                phi_2340_ = _e298;
                phi_2339_ = _e296;
                if (_e298 < 0f) {
                    phi_2340_ = -(_e298);
                    phi_2339_ = (_e296 + _e298);
                }
                let _e500 = phi_2340_;
                let _e502 = phi_2339_;
                let _e504 = ((_e302 - _e502) + 1.5707964f);
                let _e510 = clamp(((_e504 - (floor((_e504 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e500);
                phi_2341_ = _e510;
                if (_e510 > (_e500 * 0.5f)) {
                    phi_2341_ = (_e500 - _e510);
                }
                let _e515 = phi_2341_;
                let _e522 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e515), cos(_e515)) * abs(_e300))) * 0.5f);
                if (abs((_e500 - 1.5707964f)) < 0.001f) {
                    phi_2368_ = 0f;
                    phi_2366_ = 0f;
                } else {
                    let _e526 = tan(_e500);
                    let _e531 = (sign((1.5707964f - _e500)) / max(abs(_e526), 0.000001f));
                    if (_e531 >= 0f) {
                        phi_2345_ = (_e522.y - ((1f - _e522.x) * _e526));
                    } else {
                        phi_2345_ = (_e522.y + (_e522.x * _e526));
                    }
                    let _e543 = phi_2345_;
                    phi_2368_ = _e543;
                    phi_2366_ = _e531;
                }
                let _e545 = phi_2368_;
                let _e547 = phi_2366_;
                phi_2384_ = vec4<f32>((max(_e522.x, 0f) + 0.25f), (-2f - _e522.y), _e547, _e545);
            }
            let _e555 = phi_2384_;
            phi_2505_ = (_e116 * (_e306 * (_e300 * _e315)));
            phi_2383_ = _e555;
        } else {
            phi_2505_ = (sign(((_e306 * _e300) * _naga_inverse_2x2_f32(_e116))) * 0.5f);
            phi_2383_ = _e479;
        }
        let _e565 = phi_2505_;
        let _e567 = phi_2383_;
        phi_2511_ = _e567;
        if (((_e190 & 8388608u) != 0u) != ((_e190 & 16777216u) != 0u)) {
            phi_2511_ = (_e567 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e575 = phi_2511_;
        phi_2512_ = (((_e190 & 2147483648u) != 0u) && (_e83 != 1i));
        phi_2506_ = _e575;
        phi_2500_ = _e565;
        phi_2476_ = select(_e308, _e101, vec2((_e83 == 2i)));
    }
    let _e584 = phi_2512_;
    let _e586 = phi_2506_;
    let _e588 = phi_2500_;
    let _e590 = phi_2476_;
    let _e596 = j.Gi;
    let _e599 = select(_e586.xy, vec2<f32>(1f, -1f), vec2((_e596 != 0u)));
    let _e605 = vec4<f32>(_e599.x, _e586.y, _e586.z, _e586.w);
    S = vec4<f32>(_e605.x, _e599.y, _e605.z, _e605.w);
    if !(_e584) {
        let _e616 = JB.x2_[(_e105 + 3u)];
        let _e618 = bitcast<vec3<f32>>(_e616.xyz);
        let _e622 = (((((_e116 * _e590) + _e588) + bitcast<vec2<f32>>(_e120.xy)) * _e618.x) + _e618.yz);
        let _e625 = j.Qe[0u];
        let _e628 = j.Qe[1u];
        phi_2544_ = vec4<f32>(((_e622.x * _e625) - 1f), ((_e622.y * _e628) - sign(_e628)), 0f, 1f);
    } else {
        let _e638 = j.j3_;
        phi_2544_ = vec4(_e638);
    }
    let _e641 = phi_2544_;
    unnamed.gl_Position = _e641;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) YB: vec4<f32>, @location(1) ZB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    YB_1 = YB;
    ZB_1 = ZB;
    main_1();
    let _e13 = S;
    let _e14 = unnamed.gl_Position;
    return VertexOutput(_e13, _e14);
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
