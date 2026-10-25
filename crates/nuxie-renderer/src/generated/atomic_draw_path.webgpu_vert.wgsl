struct mi {
    v2_: array<vec4<u32>>,
}

struct li {
    v2_: array<vec4<u32>>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct og {
    v2_: array<vec2<u32>>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @location(1) @interpolate(flat, either) member_1: u32,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(7)
var VB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> DD: mi;
@group(0) @binding(2)
var<storage> KB: li;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> YB_1: vec4<f32>;
var<private> ZB_1: vec4<f32>;
var<private> T: vec4<f32>;
var<private> G0_: u32;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(0) @binding(3)
var<storage> XC: og;
@group(0) @binding(4)
var<storage> JB: pg;
@group(3) @binding(9)
var db: sampler;

fn main_1() {
    var phi_2284_: f32;
    var phi_2222_: f32;
    var phi_2194_: i32;
    var phi_1342_: bool;
    var phi_2207_: i32;
    var phi_2199_: vec4<u32>;
    var phi_2206_: i32;
    var phi_2198_: vec4<u32>;
    var phi_2205_: i32;
    var phi_2203_: vec4<u32>;
    var phi_2202_: u32;
    var phi_2209_: vec2<i32>;
    var phi_2210_: vec4<u32>;
    var phi_2214_: f32;
    var phi_2294_: f32;
    var phi_2228_: f32;
    var phi_2293_: f32;
    var phi_2236_: f32;
    var phi_2229_: f32;
    var phi_2226_: f32;
    var phi_2240_: f32;
    var phi_2315_: f32;
    var phi_2306_: f32;
    var phi_2291_: f32;
    var phi_2239_: f32;
    var phi_2289_: f32;
    var phi_2372_: f32;
    var phi_2383_: f32;
    var phi_2375_: f32;
    var phi_2436_: f32;
    var phi_2410_: f32;
    var phi_1657_: bool;
    var phi_2415_: f32;
    var phi_2427_: vec2<f32>;
    var phi_2426_: vec2<f32>;
    var phi_2444_: vec4<f32>;
    var phi_2457_: vec2<f32>;
    var phi_2443_: vec4<f32>;
    var phi_2491_: vec4<f32>;
    var phi_2325_: f32;
    var phi_2324_: f32;
    var phi_2326_: f32;
    var phi_2330_: f32;
    var phi_2352_: f32;
    var phi_2350_: f32;
    var phi_2368_: vec4<f32>;
    var phi_2488_: vec2<f32>;
    var phi_2367_: vec4<f32>;
    var phi_2494_: vec4<f32>;
    var phi_2495_: bool;
    var phi_2489_: vec4<f32>;
    var phi_2483_: vec2<f32>;
    var phi_2459_: vec2<f32>;
    var phi_2527_: vec4<f32>;

    let _e70 = gl_InstanceIndex_1;
    let _e71 = YB_1;
    let _e72 = ZB_1;
    let _e74 = i32(_e71.x);
    let _e78 = bitcast<i32>(_e71.w);
    let _e80 = (_e78 >> bitcast<u32>(2i));
    let _e81 = (_e78 & 3i);
    let _e83 = min(_e74, (_e80 - 1i));
    let _e85 = ((_e70 * _e80) + _e83);
    let _e90 = textureLoad(VB, vec2<i32>((_e85 & 2047i), (_e85 >> bitcast<u32>(11i))), 0i);
    let _e97 = DD.v2_[(max((_e90.w & 65535u), 1u) - 1u)];
    let _e99 = bitcast<vec2<f32>>(_e97.xy);
    let _e101 = (_e97.z & 65535u);
    let _e103 = (_e101 * 4u);
    let _e106 = KB.v2_[_e103];
    let _e107 = bitcast<vec4<f32>>(_e106);
    let _e114 = mat2x2<f32>(vec2<f32>(_e107.x, _e107.y), vec2<f32>(_e107.z, _e107.w));
    let _e118 = KB.v2_[(_e103 + 1u)];
    let _e122 = bitcast<f32>(_e118.z);
    let _e124 = bitcast<f32>(_e118.w);
    let _e125 = (_e90.w & 8388608u);
    phi_2284_ = _e71.z;
    phi_2222_ = _e71.y;
    phi_2194_ = _e74;
    if (_e125 != 0u) {
        phi_2284_ = _e72.z;
        phi_2222_ = _e72.y;
        phi_2194_ = i32(_e72.x);
    }
    let _e132 = phi_2284_;
    let _e134 = phi_2222_;
    let _e136 = phi_2194_;
    phi_2205_ = _e85;
    phi_2203_ = _e90;
    phi_2202_ = _e90.w;
    if (_e136 != _e83) {
        let _e139 = ((_e85 + _e136) - _e83);
        let _e144 = textureLoad(VB, vec2<i32>((_e139 & 2047i), (_e139 >> bitcast<u32>(11i))), 0i);
        if ((_e144.w & 8454143u) != (_e90.w & 8454143u)) {
            let _e149 = (_e122 == 0f);
            phi_1342_ = _e149;
            if !(_e149) {
                phi_1342_ = (_e99.x != 0f);
            }
            let _e154 = phi_1342_;
            phi_2207_ = _e85;
            phi_2199_ = _e90;
            if _e154 {
                let _e155 = bitcast<i32>(_e97.w);
                let _e160 = textureLoad(VB, vec2<i32>((_e155 & 2047i), (_e155 >> bitcast<u32>(11i))), 0i);
                phi_2207_ = _e155;
                phi_2199_ = _e160;
            }
            let _e162 = phi_2207_;
            let _e164 = phi_2199_;
            phi_2206_ = _e162;
            phi_2198_ = _e164;
        } else {
            phi_2206_ = _e139;
            phi_2198_ = _e144;
        }
        let _e166 = phi_2206_;
        let _e168 = phi_2198_;
        phi_2205_ = _e166;
        phi_2203_ = _e168;
        phi_2202_ = ((_e168.w & 4286578687u) | _e125);
    }
    let _e173 = phi_2205_;
    let _e175 = phi_2203_;
    let _e177 = phi_2202_;
    let _e178 = (_e177 & 469762048u);
    let _e181 = ((_e178 == 67108864u) && (_e81 == 0i));
    if _e181 {
        let _e184 = f32((_e175.z & 65535u));
        let _e187 = f32((_e175.z >> bitcast<u32>(16i)));
        let _e193 = vec2<i32>(i32((-1f - _e184)), i32(((_e187 - _e184) + 1f)));
        phi_2209_ = _e193;
        if ((_e177 & 8388608u) != 0u) {
            phi_2209_ = -(_e193);
        }
        let _e198 = phi_2209_;
        let _e200 = (_e173 + _e198.x);
        let _e205 = textureLoad(VB, vec2<i32>((_e200 & 2047i), (_e200 >> bitcast<u32>(11i))), 0i);
        let _e207 = (_e173 + _e198.y);
        let _e212 = textureLoad(VB, vec2<i32>((_e207 & 2047i), (_e207 >> bitcast<u32>(11i))), 0i);
        phi_2210_ = _e212;
        if ((_e212.w & 8454143u) != (_e205.w & 8454143u)) {
            let _e218 = bitcast<i32>(_e97.w);
            let _e223 = textureLoad(VB, vec2<i32>((_e218 & 2047i), (_e218 >> bitcast<u32>(11i))), 0i);
            phi_2210_ = _e223;
        }
        let _e225 = phi_2210_;
        let _e228 = (f32(_e205.z) * 0.0000000014629181f);
        let _e231 = (f32(_e225.z) * 0.0000000014629181f);
        let _e232 = (_e231 - _e228);
        phi_2214_ = _e232;
        if (abs(_e232) > 3.1415927f) {
            phi_2214_ = (_e232 - (6.2831855f * sign(_e232)));
        }
        let _e239 = phi_2214_;
        let _e240 = (_e187 + -2f);
        let _e246 = clamp(round(((abs(_e239) * 0.31830987f) * _e240)), 1f, (_e187 + -3f));
        let _e247 = (_e240 - _e246);
        if (_e184 <= _e247) {
            phi_2294_ = _e134;
            if (_e184 == _e247) {
                phi_2294_ = -(_e134);
            }
            let _e256 = phi_2294_;
            phi_2293_ = _e256;
            phi_2236_ = -(((3.1415927f * sign(_e239)) - _e239));
            phi_2229_ = _e247;
            phi_2226_ = _e184;
        } else {
            let _e258 = (_e184 == (_e247 + 1f));
            if _e258 {
                phi_2228_ = 0f;
            } else {
                phi_2228_ = (_e184 - (_e247 + 2f));
            }
            let _e262 = phi_2228_;
            phi_2293_ = select(_e134, 0f, _e258);
            phi_2236_ = _e239;
            phi_2229_ = select(_e246, 0f, _e258);
            phi_2226_ = _e262;
        }
        let _e266 = phi_2293_;
        let _e268 = phi_2236_;
        let _e270 = phi_2229_;
        let _e272 = phi_2226_;
        if (_e272 == _e270) {
            phi_2240_ = _e231;
        } else {
            phi_2240_ = (_e228 + (_e268 * (_e272 / _e270)));
        }
        let _e278 = phi_2240_;
        phi_2315_ = _e228;
        phi_2306_ = _e268;
        phi_2291_ = _e266;
        phi_2239_ = _e278;
    } else {
        phi_2315_ = f32();
        phi_2306_ = f32();
        phi_2291_ = _e134;
        phi_2239_ = (f32(_e175.z) * 0.0000000014629181f);
    }
    let _e283 = phi_2315_;
    let _e285 = phi_2306_;
    let _e287 = phi_2291_;
    let _e289 = phi_2239_;
    let _e293 = vec2<f32>(sin(_e289), -(cos(_e289)));
    let _e295 = bitcast<vec2<f32>>(_e175.xy);
    phi_2289_ = _e124;
    if (_e124 != 0f) {
        phi_2289_ = max(_e124, (1f / length((_e114 * _e293))));
    }
    let _e302 = phi_2289_;
    if (_e122 != 0f) {
        let _e306 = (_e287 * sign(determinant(_e114)));
        let _e308 = ((_e177 & 1048576u) != 0u);
        phi_2372_ = _e306;
        if _e308 {
            phi_2372_ = min(_e306, 0f);
        }
        let _e311 = phi_2372_;
        phi_2383_ = _e311;
        if ((_e177 & 524288u) != 0u) {
            phi_2383_ = max(_e311, 0f);
        }
        let _e316 = phi_2383_;
        let _e317 = (_e302 != 0f);
        if _e317 {
            phi_2375_ = _e302;
        } else {
            let _e318 = (_e114 * _e293);
            phi_2375_ = (((abs(_e318.x) + abs(_e318.y)) * (1f / dot(_e318, _e318))) * 0.5f);
        }
        let _e329 = phi_2375_;
        let _e332 = ((_e329 > _e122) && (_e302 == 0f));
        phi_2436_ = 1f;
        if _e332 {
            phi_2436_ = (_e122 / _e329);
        }
        let _e335 = phi_2436_;
        let _e336 = select(_e122, _e329, _e332);
        let _e337 = (_e336 + _e329);
        let _e338 = (_e293 * _e337);
        let _e339 = (_e316 * _e337);
        let _e346 = (((vec2<f32>(_e339, -(_e339)) + vec2(_e336)) * (0.5f / _e329)) + vec2<f32>(0.5f, 0.5f));
        let _e349 = vec4<f32>(_e346.x, _e346.y, 0f, 0f);
        phi_2457_ = _e338;
        phi_2443_ = _e349;
        if (_e178 > 201326592u) {
            let _e355 = f32((_e175.z & 65535u));
            let _e356 = (_e355 * 0.000015259022f);
            let _e360 = sqrt(max((1f - (_e356 * _e356)), 0f));
            phi_2410_ = _e360;
            if (((_e177 & 4194304u) != 0u) == _e308) {
                phi_2410_ = -(_e360);
            }
            let _e364 = phi_2410_;
            let _e369 = (mat2x2<f32>(vec2<f32>(_e356, _e364), vec2<f32>(-(_e364), _e356)) * _e293);
            let _e370 = (_e114 * _e369);
            let _e378 = ((abs(_e370.x) + abs(_e370.y)) * (1f / dot(_e370, _e370)));
            let _e379 = (_e178 == 402653184u);
            phi_1657_ = _e379;
            if !(_e379) {
                phi_1657_ = ((_e178 == 335544320u) && (_e356 >= 0.25f));
            }
            let _e385 = phi_1657_;
            if _e385 {
                phi_2415_ = (_e336 * (1f / max(_e356, select(0.25f, 1f, ((_e177 & 33554432u) != 0u)))));
            } else {
                phi_2415_ = ((_e336 * _e356) + (_e378 * 0.5f));
            }
            let _e396 = phi_2415_;
            let _e398 = (_e396 + (_e378 * 0.5f));
            phi_2426_ = _e338;
            if ((_e177 & 2097152u) != 0u) {
                if (_e337 <= ((_e398 * _e356) + (_e329 * 0.125f))) {
                    phi_2427_ = (_e369 * (_e337 * (65535f / _e355)));
                } else {
                    let _e408 = (_e369 * _e398);
                    phi_2427_ = (vec2<f32>(dot(_e338, _e338), dot(_e408, _e408)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e338, _e408)));
                }
                let _e416 = phi_2427_;
                phi_2426_ = _e416;
            }
            let _e418 = phi_2426_;
            let _e423 = ((_e398 - dot((_e418 * abs(_e316)), _e369)) / _e378);
            if _e308 {
                phi_2444_ = vec4<f32>(_e349.x, _e423, _e349.z, _e349.w);
            } else {
                phi_2444_ = vec4<f32>(_e423, _e349.y, _e349.z, _e349.w);
            }
            let _e435 = phi_2444_;
            phi_2457_ = _e418;
            phi_2443_ = _e435;
        }
        let _e437 = phi_2457_;
        let _e439 = phi_2443_;
        let _e441 = (_e439.xy * _e335);
        let _e447 = vec4<f32>(_e441.x, _e439.y, _e439.z, _e439.w);
        let _e454 = vec4<f32>(_e447.x, max(_e441.y, 0.0001f), _e447.z, _e447.w);
        phi_2491_ = _e454;
        if _e317 {
            phi_2491_ = vec4<f32>((-2f - _e441.x), _e454.y, _e454.z, _e454.w);
        }
        let _e462 = phi_2491_;
        phi_2495_ = (_e81 != 0i);
        phi_2489_ = _e462;
        phi_2483_ = (_e114 * (_e437 * _e316));
        phi_2459_ = _e295;
    } else {
        let _e466 = vec4<f32>(_e132, -1f, 0f, 0f);
        if (_e302 != 0f) {
            let _e472 = vec4<f32>(_e466.x, -2f, _e466.z, _e466.w);
            let _e477 = vec4<f32>(_e472.x, _e472.y, 1000000f, _e472.w);
            phi_2368_ = vec4<f32>(_e477.x, _e477.y, _e477.z, _e132);
            if _e181 {
                phi_2325_ = _e285;
                phi_2324_ = _e283;
                if (_e285 < 0f) {
                    phi_2325_ = -(_e285);
                    phi_2324_ = (_e283 + _e285);
                }
                let _e487 = phi_2325_;
                let _e489 = phi_2324_;
                let _e491 = ((_e289 - _e489) + 1.5707964f);
                let _e497 = clamp(((_e491 - (floor((_e491 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e487);
                phi_2326_ = _e497;
                if (_e497 > (_e487 * 0.5f)) {
                    phi_2326_ = (_e487 - _e497);
                }
                let _e502 = phi_2326_;
                let _e509 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e502), cos(_e502)) * abs(_e287))) * 0.5f);
                if (abs((_e487 - 1.5707964f)) < 0.001f) {
                    phi_2352_ = 0f;
                    phi_2350_ = 0f;
                } else {
                    let _e513 = tan(_e487);
                    let _e518 = (sign((1.5707964f - _e487)) / max(abs(_e513), 0.000001f));
                    if (_e518 >= 0f) {
                        phi_2330_ = (_e509.y - ((1f - _e509.x) * _e513));
                    } else {
                        phi_2330_ = (_e509.y + (_e509.x * _e513));
                    }
                    let _e530 = phi_2330_;
                    phi_2352_ = _e530;
                    phi_2350_ = _e518;
                }
                let _e532 = phi_2352_;
                let _e534 = phi_2350_;
                phi_2368_ = vec4<f32>((max(_e509.x, 0f) + 0.25f), (-2f - _e509.y), _e534, _e532);
            }
            let _e542 = phi_2368_;
            phi_2488_ = (_e114 * (_e293 * (_e287 * _e302)));
            phi_2367_ = _e542;
        } else {
            phi_2488_ = (sign(((_e293 * _e287) * _naga_inverse_2x2_f32(_e114))) * 0.5f);
            phi_2367_ = _e466;
        }
        let _e552 = phi_2488_;
        let _e554 = phi_2367_;
        phi_2494_ = _e554;
        if (((_e177 & 8388608u) != 0u) != ((_e177 & 16777216u) != 0u)) {
            phi_2494_ = (_e554 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e562 = phi_2494_;
        phi_2495_ = (((_e177 & 2147483648u) != 0u) && (_e81 != 1i));
        phi_2489_ = _e562;
        phi_2483_ = _e552;
        phi_2459_ = select(_e295, _e99, vec2((_e81 == 2i)));
    }
    let _e571 = phi_2495_;
    let _e573 = phi_2489_;
    let _e575 = phi_2483_;
    let _e577 = phi_2459_;
    let _e580 = (((_e114 * _e577) + _e575) + bitcast<vec2<f32>>(_e118.xy));
    let _e583 = j.Di;
    let _e586 = select(_e573.xy, vec2<f32>(1f, -1f), vec2((_e583 != 0u)));
    let _e592 = vec4<f32>(_e586.x, _e573.y, _e573.z, _e573.w);
    if !(_e571) {
        T = vec4<f32>(_e592.x, _e586.y, _e592.z, _e592.w);
        G0_ = _e101;
        let _e601 = j.Mg;
        let _e603 = j.Ng;
        phi_2527_ = vec4<f32>(((_e580.x * _e601) - 1f), ((_e580.y * _e603) - sign(_e603)), 0f, 1f);
    } else {
        let _e613 = j.i3_;
        phi_2527_ = vec4(_e613);
    }
    let _e616 = phi_2527_;
    unnamed.gl_Position = _e616;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) YB: vec4<f32>, @location(1) ZB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    YB_1 = YB;
    ZB_1 = ZB;
    main_1();
    let _e14 = T;
    let _e15 = G0_;
    let _e16 = unnamed.gl_Position;
    return VertexOutput(_e14, _e15, _e16);
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
