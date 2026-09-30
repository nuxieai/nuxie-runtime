struct Ig {
    g2_: array<vec4<u32>>,
}

struct Hg {
    g2_: array<vec4<u32>>,
}

struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
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

struct kf {
    g2_: array<vec2<u32>>,
}

struct lf {
    g2_: array<vec4<f32>>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> HD: Ig;
@group(0) @binding(2)
var<storage> OB: Hg;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
var<private> O: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(3)
var<storage> CD: kf;
@group(0) @binding(4)
var<storage> PB: lf;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_2284_: f32;
    var phi_2222_: f32;
    var phi_2194_: i32;
    var phi_1336_: bool;
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
    var phi_2464_: f32;
    var phi_2421_: i32;
    var phi_2430_: f32;
    var phi_1675_: bool;
    var phi_2437_: f32;
    var phi_2453_: vec2<f32>;
    var phi_2452_: vec2<f32>;
    var phi_2474_: vec4<f32>;
    var phi_2489_: vec2<f32>;
    var phi_2473_: vec4<f32>;
    var phi_2525_: vec4<f32>;
    var phi_2325_: f32;
    var phi_2324_: f32;
    var phi_2326_: f32;
    var phi_2330_: f32;
    var phi_2352_: f32;
    var phi_2350_: f32;
    var phi_2368_: vec4<f32>;
    var phi_2522_: vec2<f32>;
    var phi_2367_: vec4<f32>;
    var phi_2528_: vec4<f32>;
    var phi_2529_: bool;
    var phi_2523_: vec4<f32>;
    var phi_2517_: vec2<f32>;
    var phi_2491_: vec2<f32>;
    var phi_2563_: vec4<f32>;

    let _e70 = gl_InstanceIndex_1;
    let _e71 = VB_1;
    let _e72 = WB_1;
    let _e74 = i32(_e71.x);
    let _e78 = bitcast<i32>(_e71.w);
    let _e80 = (_e78 >> bitcast<u32>(2i));
    let _e81 = (_e78 & 3i);
    let _e83 = min(_e74, (_e80 - 1i));
    let _e85 = ((_e70 * _e80) + _e83);
    let _e90 = textureLoad(JC, vec2<i32>((_e85 & 2047i), (_e85 >> bitcast<u32>(11i))), 0i);
    let _e97 = HD.g2_[(max((_e90.w & 65535u), 1u) - 1u)];
    let _e99 = bitcast<vec2<f32>>(_e97.xy);
    let _e103 = ((_e97.z & 65535u) * 4u);
    let _e106 = OB.g2_[_e103];
    let _e107 = bitcast<vec4<f32>>(_e106);
    let _e114 = mat2x2<f32>(vec2<f32>(_e107.x, _e107.y), vec2<f32>(_e107.z, _e107.w));
    let _e118 = OB.g2_[(_e103 + 1u)];
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
        let _e144 = textureLoad(JC, vec2<i32>((_e139 & 2047i), (_e139 >> bitcast<u32>(11i))), 0i);
        if ((_e144.w & 8454143u) != (_e90.w & 8454143u)) {
            let _e149 = (_e122 == 0f);
            phi_1336_ = _e149;
            if !(_e149) {
                phi_1336_ = (_e99.x != 0f);
            }
            let _e154 = phi_1336_;
            phi_2207_ = _e85;
            phi_2199_ = _e90;
            if _e154 {
                let _e155 = bitcast<i32>(_e97.w);
                let _e160 = textureLoad(JC, vec2<i32>((_e155 & 2047i), (_e155 >> bitcast<u32>(11i))), 0i);
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
        let _e205 = textureLoad(JC, vec2<i32>((_e200 & 2047i), (_e200 >> bitcast<u32>(11i))), 0i);
        let _e207 = (_e173 + _e198.y);
        let _e212 = textureLoad(JC, vec2<i32>((_e207 & 2047i), (_e207 >> bitcast<u32>(11i))), 0i);
        phi_2210_ = _e212;
        if ((_e212.w & 8454143u) != (_e205.w & 8454143u)) {
            let _e218 = bitcast<i32>(_e97.w);
            let _e223 = textureLoad(JC, vec2<i32>((_e218 & 2047i), (_e218 >> bitcast<u32>(11i))), 0i);
            phi_2210_ = _e223;
        }
        let _e225 = phi_2210_;
        let _e227 = bitcast<f32>(_e205.z);
        let _e229 = bitcast<f32>(_e225.z);
        let _e230 = (_e229 - _e227);
        phi_2214_ = _e230;
        if (abs(_e230) > 3.1415927f) {
            phi_2214_ = (_e230 - (6.2831855f * sign(_e230)));
        }
        let _e237 = phi_2214_;
        let _e238 = (_e187 + -2f);
        let _e244 = clamp(round(((abs(_e237) * 0.31830987f) * _e238)), 1f, (_e187 + -3f));
        let _e245 = (_e238 - _e244);
        if (_e184 <= _e245) {
            phi_2294_ = _e134;
            if (_e184 == _e245) {
                phi_2294_ = -(_e134);
            }
            let _e254 = phi_2294_;
            phi_2293_ = _e254;
            phi_2236_ = -(((3.1415927f * sign(_e237)) - _e237));
            phi_2229_ = _e245;
            phi_2226_ = _e184;
        } else {
            let _e256 = (_e184 == (_e245 + 1f));
            if _e256 {
                phi_2228_ = 0f;
            } else {
                phi_2228_ = (_e184 - (_e245 + 2f));
            }
            let _e260 = phi_2228_;
            phi_2293_ = select(_e134, 0f, _e256);
            phi_2236_ = _e237;
            phi_2229_ = select(_e244, 0f, _e256);
            phi_2226_ = _e260;
        }
        let _e264 = phi_2293_;
        let _e266 = phi_2236_;
        let _e268 = phi_2229_;
        let _e270 = phi_2226_;
        if (_e270 == _e268) {
            phi_2240_ = _e229;
        } else {
            phi_2240_ = (_e227 + (_e266 * (_e270 / _e268)));
        }
        let _e276 = phi_2240_;
        phi_2315_ = _e227;
        phi_2306_ = _e266;
        phi_2291_ = _e264;
        phi_2239_ = _e276;
    } else {
        phi_2315_ = f32();
        phi_2306_ = f32();
        phi_2291_ = _e134;
        phi_2239_ = bitcast<f32>(_e175.z);
    }
    let _e280 = phi_2315_;
    let _e282 = phi_2306_;
    let _e284 = phi_2291_;
    let _e286 = phi_2239_;
    let _e290 = vec2<f32>(sin(_e286), -(cos(_e286)));
    let _e292 = bitcast<vec2<f32>>(_e175.xy);
    phi_2289_ = _e124;
    if (_e124 != 0f) {
        phi_2289_ = max(_e124, (1f / length((_e114 * _e290))));
    }
    let _e299 = phi_2289_;
    if (_e122 != 0f) {
        let _e303 = (_e284 * sign(determinant(_e114)));
        let _e305 = ((_e177 & 1048576u) != 0u);
        phi_2372_ = _e303;
        if _e305 {
            phi_2372_ = min(_e303, 0f);
        }
        let _e308 = phi_2372_;
        phi_2383_ = _e308;
        if ((_e177 & 524288u) != 0u) {
            phi_2383_ = max(_e308, 0f);
        }
        let _e313 = phi_2383_;
        let _e314 = (_e299 != 0f);
        if _e314 {
            phi_2375_ = _e299;
        } else {
            let _e315 = (_e114 * _e290);
            phi_2375_ = (((abs(_e315.x) + abs(_e315.y)) * (1f / dot(_e315, _e315))) * 0.5f);
        }
        let _e326 = phi_2375_;
        let _e329 = ((_e326 > _e122) && (_e299 == 0f));
        phi_2464_ = 1f;
        if _e329 {
            phi_2464_ = (_e122 / _e326);
        }
        let _e332 = phi_2464_;
        let _e333 = select(_e122, _e326, _e329);
        let _e334 = (_e333 + _e326);
        let _e335 = (_e290 * _e334);
        let _e336 = (_e313 * _e334);
        let _e343 = (((vec2<f32>(_e336, -(_e336)) + vec2(_e333)) * (0.5f / _e326)) + vec2<f32>(0.5f, 0.5f));
        let _e346 = vec4<f32>(_e343.x, _e343.y, 0f, 0f);
        phi_2489_ = _e335;
        phi_2473_ = _e346;
        if (_e178 > 134217728u) {
            let _e348 = (_e177 & 4194304u);
            let _e350 = select(2i, -2i, (_e348 == 0u));
            phi_2421_ = _e350;
            if ((_e177 & 8388608u) != 0u) {
                phi_2421_ = -(_e350);
            }
            let _e355 = phi_2421_;
            let _e356 = (_e173 + _e355);
            let _e361 = textureLoad(JC, vec2<i32>((_e356 & 2047i), (_e356 >> bitcast<u32>(11i))), 0i);
            let _e365 = abs((bitcast<f32>(_e361.z) - _e286));
            phi_2430_ = _e365;
            if (_e365 > 3.1415927f) {
                phi_2430_ = (6.2831855f - _e365);
            }
            let _e369 = phi_2430_;
            let _e374 = ((_e369 * select(0.5f, -0.5f, ((_e348 != 0u) == _e305))) + _e286);
            let _e378 = vec2<f32>(sin(_e374), -(cos(_e374)));
            let _e379 = (_e114 * _e378);
            let _e387 = ((abs(_e379.x) + abs(_e379.y)) * (1f / dot(_e379, _e379)));
            let _e389 = cos((_e369 * 0.5f));
            let _e390 = (_e178 == 335544320u);
            phi_1675_ = _e390;
            if !(_e390) {
                phi_1675_ = ((_e178 == 268435456u) && (_e389 >= 0.25f));
            }
            let _e396 = phi_1675_;
            if _e396 {
                phi_2437_ = (_e333 * (1f / max(_e389, select(0.25f, 1f, ((_e177 & 33554432u) != 0u)))));
            } else {
                phi_2437_ = ((_e333 * _e389) + (_e387 * 0.5f));
            }
            let _e407 = phi_2437_;
            let _e409 = (_e407 + (_e387 * 0.5f));
            phi_2452_ = _e335;
            if ((_e177 & 2097152u) != 0u) {
                if (_e334 <= ((_e409 * _e389) + (_e326 * 0.125f))) {
                    phi_2453_ = (_e378 * (_e334 * (1f / _e389)));
                } else {
                    let _e419 = (_e378 * _e409);
                    phi_2453_ = (vec2<f32>(dot(_e335, _e335), dot(_e419, _e419)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e335, _e419)));
                }
                let _e427 = phi_2453_;
                phi_2452_ = _e427;
            }
            let _e429 = phi_2452_;
            let _e434 = ((_e409 - dot((_e429 * abs(_e313)), _e378)) / _e387);
            if _e305 {
                phi_2474_ = vec4<f32>(_e346.x, _e434, _e346.z, _e346.w);
            } else {
                phi_2474_ = vec4<f32>(_e434, _e346.y, _e346.z, _e346.w);
            }
            let _e446 = phi_2474_;
            phi_2489_ = _e429;
            phi_2473_ = _e446;
        }
        let _e448 = phi_2489_;
        let _e450 = phi_2473_;
        let _e452 = (_e450.xy * _e332);
        let _e458 = vec4<f32>(_e452.x, _e450.y, _e450.z, _e450.w);
        let _e465 = vec4<f32>(_e458.x, max(_e452.y, 0.0001f), _e458.z, _e458.w);
        phi_2525_ = _e465;
        if _e314 {
            phi_2525_ = vec4<f32>((-2f - _e452.x), _e465.y, _e465.z, _e465.w);
        }
        let _e473 = phi_2525_;
        phi_2529_ = (_e81 != 0i);
        phi_2523_ = _e473;
        phi_2517_ = (_e114 * (_e448 * _e313));
        phi_2491_ = _e292;
    } else {
        let _e477 = vec4<f32>(_e132, -1f, 0f, 0f);
        if (_e299 != 0f) {
            let _e483 = vec4<f32>(_e477.x, -2f, _e477.z, _e477.w);
            let _e488 = vec4<f32>(_e483.x, _e483.y, 1000000f, _e483.w);
            phi_2368_ = vec4<f32>(_e488.x, _e488.y, _e488.z, _e132);
            if _e181 {
                phi_2325_ = _e282;
                phi_2324_ = _e280;
                if (_e282 < 0f) {
                    phi_2325_ = -(_e282);
                    phi_2324_ = (_e280 + _e282);
                }
                let _e498 = phi_2325_;
                let _e500 = phi_2324_;
                let _e502 = ((_e286 - _e500) + 1.5707964f);
                let _e508 = clamp(((_e502 - (floor((_e502 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e498);
                phi_2326_ = _e508;
                if (_e508 > (_e498 * 0.5f)) {
                    phi_2326_ = (_e498 - _e508);
                }
                let _e513 = phi_2326_;
                let _e520 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e513), cos(_e513)) * abs(_e284))) * 0.5f);
                if (abs((_e498 - 1.5707964f)) < 0.001f) {
                    phi_2352_ = 0f;
                    phi_2350_ = 0f;
                } else {
                    let _e524 = tan(_e498);
                    let _e529 = (sign((1.5707964f - _e498)) / max(abs(_e524), 0.000001f));
                    if (_e529 >= 0f) {
                        phi_2330_ = (_e520.y - ((1f - _e520.x) * _e524));
                    } else {
                        phi_2330_ = (_e520.y + (_e520.x * _e524));
                    }
                    let _e541 = phi_2330_;
                    phi_2352_ = _e541;
                    phi_2350_ = _e529;
                }
                let _e543 = phi_2352_;
                let _e545 = phi_2350_;
                phi_2368_ = vec4<f32>((max(_e520.x, 0f) + 0.25f), (-2f - _e520.y), _e545, _e543);
            }
            let _e553 = phi_2368_;
            phi_2522_ = (_e114 * (_e290 * (_e284 * _e299)));
            phi_2367_ = _e553;
        } else {
            phi_2522_ = (sign(((_e290 * _e284) * _naga_inverse_2x2_f32(_e114))) * 0.5f);
            phi_2367_ = _e477;
        }
        let _e563 = phi_2522_;
        let _e565 = phi_2367_;
        phi_2528_ = _e565;
        if (((_e177 & 8388608u) != 0u) != ((_e177 & 16777216u) != 0u)) {
            phi_2528_ = (_e565 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e573 = phi_2528_;
        phi_2529_ = (((_e177 & 2147483648u) != 0u) && (_e81 != 1i));
        phi_2523_ = _e573;
        phi_2517_ = _e563;
        phi_2491_ = select(_e292, _e99, vec2((_e81 == 2i)));
    }
    let _e582 = phi_2529_;
    let _e584 = phi_2523_;
    let _e586 = phi_2517_;
    let _e588 = phi_2491_;
    let _e594 = j.eh;
    let _e597 = select(_e584.xy, vec2<f32>(1f, -1f), vec2((_e594 != 0u)));
    let _e603 = vec4<f32>(_e597.x, _e584.y, _e584.z, _e584.w);
    O = vec4<f32>(_e603.x, _e597.y, _e603.z, _e603.w);
    if !(_e582) {
        let _e614 = OB.g2_[(_e103 + 2u)];
        let _e616 = bitcast<vec3<f32>>(_e614.yzw);
        let _e620 = (((((_e114 * _e588) + _e586) + bitcast<vec2<f32>>(_e118.xy)) * _e616.x) + _e616.yz);
        let _e623 = j.Dd[0u];
        let _e626 = j.Dd[1u];
        phi_2563_ = vec4<f32>(((_e620.x * _e623) - 1f), ((_e620.y * _e626) - sign(_e626)), 0f, 1f);
    } else {
        let _e636 = j.W2_;
        phi_2563_ = vec4(_e636);
    }
    let _e639 = phi_2563_;
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
    let _e13 = O;
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
