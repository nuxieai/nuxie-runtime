struct TB {
    uc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Ob: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    ih: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    mh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    fh: u32,
    Nb: u32,
    ac: f32,
    bc: f32,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(5)
var ID: texture_2d<u32>;
@group(0) @binding(2)
var OB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
var<private> O: vec4<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(3)
var DD: texture_2d<u32>;
@group(0) @binding(4)
var PB: texture_2d<f32>;
@group(3) @binding(9)
var da: sampler;

fn main_1() {
    var phi_2325_: f32;
    var phi_2263_: f32;
    var phi_2235_: i32;
    var phi_1377_: bool;
    var phi_2248_: i32;
    var phi_2240_: vec4<u32>;
    var phi_2247_: i32;
    var phi_2239_: vec4<u32>;
    var phi_2246_: i32;
    var phi_2244_: vec4<u32>;
    var phi_2243_: u32;
    var phi_2250_: vec2<i32>;
    var phi_2251_: vec4<u32>;
    var phi_2255_: f32;
    var phi_2335_: f32;
    var phi_2269_: f32;
    var phi_2334_: f32;
    var phi_2277_: f32;
    var phi_2270_: f32;
    var phi_2267_: f32;
    var phi_2281_: f32;
    var phi_2356_: f32;
    var phi_2347_: f32;
    var phi_2332_: f32;
    var phi_2280_: f32;
    var phi_2330_: f32;
    var phi_2413_: f32;
    var phi_2424_: f32;
    var phi_2416_: f32;
    var phi_2505_: f32;
    var phi_2462_: i32;
    var phi_2471_: f32;
    var phi_1716_: bool;
    var phi_2478_: f32;
    var phi_2494_: vec2<f32>;
    var phi_2493_: vec2<f32>;
    var phi_2515_: vec4<f32>;
    var phi_2530_: vec2<f32>;
    var phi_2514_: vec4<f32>;
    var phi_2566_: vec4<f32>;
    var phi_2366_: f32;
    var phi_2365_: f32;
    var phi_2367_: f32;
    var phi_2371_: f32;
    var phi_2393_: f32;
    var phi_2391_: f32;
    var phi_2409_: vec4<f32>;
    var phi_2563_: vec2<f32>;
    var phi_2408_: vec4<f32>;
    var phi_2569_: vec4<f32>;
    var phi_2570_: bool;
    var phi_2564_: vec4<f32>;
    var phi_2558_: vec2<f32>;
    var phi_2532_: vec2<f32>;
    var phi_2604_: vec4<f32>;

    let _e72 = gl_InstanceIndex_1;
    let _e73 = VB_1;
    let _e74 = WB_1;
    let _e76 = i32(_e73.x);
    let _e80 = bitcast<i32>(_e73.w);
    let _e82 = (_e80 >> bitcast<u32>(2i));
    let _e83 = (_e80 & 3i);
    let _e85 = min(_e76, (_e82 - 1i));
    let _e87 = ((_e72 * _e82) + _e85);
    let _e92 = textureLoad(KC, vec2<i32>((_e87 & 2047i), (_e87 >> bitcast<u32>(11i))), 0i);
    let _e96 = (max((_e92.w & 65535u), 1u) - 1u);
    let _e103 = textureLoad(ID, vec2<i32>(bitcast<i32>((_e96 & 255u)), bitcast<i32>((_e96 >> bitcast<u32>(8i)))), 0i);
    let _e105 = bitcast<vec2<f32>>(_e103.xy);
    let _e109 = ((_e103.z & 65535u) * 4u);
    let _e116 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e109 & 255u)), bitcast<i32>((_e109 >> bitcast<u32>(8i)))), 0i);
    let _e117 = bitcast<vec4<f32>>(_e116);
    let _e124 = mat2x2<f32>(vec2<f32>(_e117.x, _e117.y), vec2<f32>(_e117.z, _e117.w));
    let _e125 = (_e109 + 1u);
    let _e132 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e125 & 255u)), bitcast<i32>((_e125 >> bitcast<u32>(8i)))), 0i);
    let _e136 = bitcast<f32>(_e132.z);
    let _e138 = bitcast<f32>(_e132.w);
    let _e139 = (_e92.w & 8388608u);
    phi_2325_ = _e73.z;
    phi_2263_ = _e73.y;
    phi_2235_ = _e76;
    if (_e139 != 0u) {
        phi_2325_ = _e74.z;
        phi_2263_ = _e74.y;
        phi_2235_ = i32(_e74.x);
    }
    let _e146 = phi_2325_;
    let _e148 = phi_2263_;
    let _e150 = phi_2235_;
    phi_2246_ = _e87;
    phi_2244_ = _e92;
    phi_2243_ = _e92.w;
    if (_e150 != _e85) {
        let _e153 = ((_e87 + _e150) - _e85);
        let _e158 = textureLoad(KC, vec2<i32>((_e153 & 2047i), (_e153 >> bitcast<u32>(11i))), 0i);
        if ((_e158.w & 8454143u) != (_e92.w & 8454143u)) {
            let _e163 = (_e136 == 0f);
            phi_1377_ = _e163;
            if !(_e163) {
                phi_1377_ = (_e105.x != 0f);
            }
            let _e168 = phi_1377_;
            phi_2248_ = _e87;
            phi_2240_ = _e92;
            if _e168 {
                let _e169 = bitcast<i32>(_e103.w);
                let _e174 = textureLoad(KC, vec2<i32>((_e169 & 2047i), (_e169 >> bitcast<u32>(11i))), 0i);
                phi_2248_ = _e169;
                phi_2240_ = _e174;
            }
            let _e176 = phi_2248_;
            let _e178 = phi_2240_;
            phi_2247_ = _e176;
            phi_2239_ = _e178;
        } else {
            phi_2247_ = _e153;
            phi_2239_ = _e158;
        }
        let _e180 = phi_2247_;
        let _e182 = phi_2239_;
        phi_2246_ = _e180;
        phi_2244_ = _e182;
        phi_2243_ = ((_e182.w & 4286578687u) | _e139);
    }
    let _e187 = phi_2246_;
    let _e189 = phi_2244_;
    let _e191 = phi_2243_;
    let _e192 = (_e191 & 469762048u);
    let _e195 = ((_e192 == 67108864u) && (_e83 == 0i));
    if _e195 {
        let _e198 = f32((_e189.z & 65535u));
        let _e201 = f32((_e189.z >> bitcast<u32>(16i)));
        let _e207 = vec2<i32>(i32((-1f - _e198)), i32(((_e201 - _e198) + 1f)));
        phi_2250_ = _e207;
        if ((_e191 & 8388608u) != 0u) {
            phi_2250_ = -(_e207);
        }
        let _e212 = phi_2250_;
        let _e214 = (_e187 + _e212.x);
        let _e219 = textureLoad(KC, vec2<i32>((_e214 & 2047i), (_e214 >> bitcast<u32>(11i))), 0i);
        let _e221 = (_e187 + _e212.y);
        let _e226 = textureLoad(KC, vec2<i32>((_e221 & 2047i), (_e221 >> bitcast<u32>(11i))), 0i);
        phi_2251_ = _e226;
        if ((_e226.w & 8454143u) != (_e219.w & 8454143u)) {
            let _e232 = bitcast<i32>(_e103.w);
            let _e237 = textureLoad(KC, vec2<i32>((_e232 & 2047i), (_e232 >> bitcast<u32>(11i))), 0i);
            phi_2251_ = _e237;
        }
        let _e239 = phi_2251_;
        let _e241 = bitcast<f32>(_e219.z);
        let _e243 = bitcast<f32>(_e239.z);
        let _e244 = (_e243 - _e241);
        phi_2255_ = _e244;
        if (abs(_e244) > 3.1415927f) {
            phi_2255_ = (_e244 - (6.2831855f * sign(_e244)));
        }
        let _e251 = phi_2255_;
        let _e252 = (_e201 + -2f);
        let _e258 = clamp(round(((abs(_e251) * 0.31830987f) * _e252)), 1f, (_e201 + -3f));
        let _e259 = (_e252 - _e258);
        if (_e198 <= _e259) {
            phi_2335_ = _e148;
            if (_e198 == _e259) {
                phi_2335_ = -(_e148);
            }
            let _e268 = phi_2335_;
            phi_2334_ = _e268;
            phi_2277_ = -(((3.1415927f * sign(_e251)) - _e251));
            phi_2270_ = _e259;
            phi_2267_ = _e198;
        } else {
            let _e270 = (_e198 == (_e259 + 1f));
            if _e270 {
                phi_2269_ = 0f;
            } else {
                phi_2269_ = (_e198 - (_e259 + 2f));
            }
            let _e274 = phi_2269_;
            phi_2334_ = select(_e148, 0f, _e270);
            phi_2277_ = _e251;
            phi_2270_ = select(_e258, 0f, _e270);
            phi_2267_ = _e274;
        }
        let _e278 = phi_2334_;
        let _e280 = phi_2277_;
        let _e282 = phi_2270_;
        let _e284 = phi_2267_;
        if (_e284 == _e282) {
            phi_2281_ = _e243;
        } else {
            phi_2281_ = (_e241 + (_e280 * (_e284 / _e282)));
        }
        let _e290 = phi_2281_;
        phi_2356_ = _e241;
        phi_2347_ = _e280;
        phi_2332_ = _e278;
        phi_2280_ = _e290;
    } else {
        phi_2356_ = f32();
        phi_2347_ = f32();
        phi_2332_ = _e148;
        phi_2280_ = bitcast<f32>(_e189.z);
    }
    let _e294 = phi_2356_;
    let _e296 = phi_2347_;
    let _e298 = phi_2332_;
    let _e300 = phi_2280_;
    let _e304 = vec2<f32>(sin(_e300), -(cos(_e300)));
    let _e306 = bitcast<vec2<f32>>(_e189.xy);
    phi_2330_ = _e138;
    if (_e138 != 0f) {
        phi_2330_ = max(_e138, (1f / length((_e124 * _e304))));
    }
    let _e313 = phi_2330_;
    if (_e136 != 0f) {
        let _e317 = (_e298 * sign(determinant(_e124)));
        let _e319 = ((_e191 & 1048576u) != 0u);
        phi_2413_ = _e317;
        if _e319 {
            phi_2413_ = min(_e317, 0f);
        }
        let _e322 = phi_2413_;
        phi_2424_ = _e322;
        if ((_e191 & 524288u) != 0u) {
            phi_2424_ = max(_e322, 0f);
        }
        let _e327 = phi_2424_;
        let _e328 = (_e313 != 0f);
        if _e328 {
            phi_2416_ = _e313;
        } else {
            let _e329 = (_e124 * _e304);
            phi_2416_ = (((abs(_e329.x) + abs(_e329.y)) * (1f / dot(_e329, _e329))) * 0.5f);
        }
        let _e340 = phi_2416_;
        let _e343 = ((_e340 > _e136) && (_e313 == 0f));
        phi_2505_ = 1f;
        if _e343 {
            phi_2505_ = (_e136 / _e340);
        }
        let _e346 = phi_2505_;
        let _e347 = select(_e136, _e340, _e343);
        let _e348 = (_e347 + _e340);
        let _e349 = (_e304 * _e348);
        let _e350 = (_e327 * _e348);
        let _e357 = (((vec2<f32>(_e350, -(_e350)) + vec2(_e347)) * (0.5f / _e340)) + vec2<f32>(0.5f, 0.5f));
        let _e360 = vec4<f32>(_e357.x, _e357.y, 0f, 0f);
        phi_2530_ = _e349;
        phi_2514_ = _e360;
        if (_e192 > 134217728u) {
            let _e362 = (_e191 & 4194304u);
            let _e364 = select(2i, -2i, (_e362 == 0u));
            phi_2462_ = _e364;
            if ((_e191 & 8388608u) != 0u) {
                phi_2462_ = -(_e364);
            }
            let _e369 = phi_2462_;
            let _e370 = (_e187 + _e369);
            let _e375 = textureLoad(KC, vec2<i32>((_e370 & 2047i), (_e370 >> bitcast<u32>(11i))), 0i);
            let _e379 = abs((bitcast<f32>(_e375.z) - _e300));
            phi_2471_ = _e379;
            if (_e379 > 3.1415927f) {
                phi_2471_ = (6.2831855f - _e379);
            }
            let _e383 = phi_2471_;
            let _e388 = ((_e383 * select(0.5f, -0.5f, ((_e362 != 0u) == _e319))) + _e300);
            let _e392 = vec2<f32>(sin(_e388), -(cos(_e388)));
            let _e393 = (_e124 * _e392);
            let _e401 = ((abs(_e393.x) + abs(_e393.y)) * (1f / dot(_e393, _e393)));
            let _e403 = cos((_e383 * 0.5f));
            let _e404 = (_e192 == 335544320u);
            phi_1716_ = _e404;
            if !(_e404) {
                phi_1716_ = ((_e192 == 268435456u) && (_e403 >= 0.25f));
            }
            let _e410 = phi_1716_;
            if _e410 {
                phi_2478_ = (_e347 * (1f / max(_e403, select(0.25f, 1f, ((_e191 & 33554432u) != 0u)))));
            } else {
                phi_2478_ = ((_e347 * _e403) + (_e401 * 0.5f));
            }
            let _e421 = phi_2478_;
            let _e423 = (_e421 + (_e401 * 0.5f));
            phi_2493_ = _e349;
            if ((_e191 & 2097152u) != 0u) {
                if (_e348 <= ((_e423 * _e403) + (_e340 * 0.125f))) {
                    phi_2494_ = (_e392 * (_e348 * (1f / _e403)));
                } else {
                    let _e433 = (_e392 * _e423);
                    phi_2494_ = (vec2<f32>(dot(_e349, _e349), dot(_e433, _e433)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e349, _e433)));
                }
                let _e441 = phi_2494_;
                phi_2493_ = _e441;
            }
            let _e443 = phi_2493_;
            let _e448 = ((_e423 - dot((_e443 * abs(_e327)), _e392)) / _e401);
            if _e319 {
                phi_2515_ = vec4<f32>(_e360.x, _e448, _e360.z, _e360.w);
            } else {
                phi_2515_ = vec4<f32>(_e448, _e360.y, _e360.z, _e360.w);
            }
            let _e460 = phi_2515_;
            phi_2530_ = _e443;
            phi_2514_ = _e460;
        }
        let _e462 = phi_2530_;
        let _e464 = phi_2514_;
        let _e466 = (_e464.xy * _e346);
        let _e472 = vec4<f32>(_e466.x, _e464.y, _e464.z, _e464.w);
        let _e479 = vec4<f32>(_e472.x, max(_e466.y, 0.0001f), _e472.z, _e472.w);
        phi_2566_ = _e479;
        if _e328 {
            phi_2566_ = vec4<f32>((-2f - _e466.x), _e479.y, _e479.z, _e479.w);
        }
        let _e487 = phi_2566_;
        phi_2570_ = (_e83 != 0i);
        phi_2564_ = _e487;
        phi_2558_ = (_e124 * (_e462 * _e327));
        phi_2532_ = _e306;
    } else {
        let _e491 = vec4<f32>(_e146, -1f, 0f, 0f);
        if (_e313 != 0f) {
            let _e497 = vec4<f32>(_e491.x, -2f, _e491.z, _e491.w);
            let _e502 = vec4<f32>(_e497.x, _e497.y, 1000000f, _e497.w);
            phi_2409_ = vec4<f32>(_e502.x, _e502.y, _e502.z, _e146);
            if _e195 {
                phi_2366_ = _e296;
                phi_2365_ = _e294;
                if (_e296 < 0f) {
                    phi_2366_ = -(_e296);
                    phi_2365_ = (_e294 + _e296);
                }
                let _e512 = phi_2366_;
                let _e514 = phi_2365_;
                let _e516 = ((_e300 - _e514) + 1.5707964f);
                let _e522 = clamp(((_e516 - (floor((_e516 / 6.2831855f)) * 6.2831855f)) - 1.5707964f), 0f, _e512);
                phi_2367_ = _e522;
                if (_e522 > (_e512 * 0.5f)) {
                    phi_2367_ = (_e512 - _e522);
                }
                let _e527 = phi_2367_;
                let _e534 = ((vec2<f32>(1f, 1f) - (vec2<f32>(sin(_e527), cos(_e527)) * abs(_e298))) * 0.5f);
                if (abs((_e512 - 1.5707964f)) < 0.001f) {
                    phi_2393_ = 0f;
                    phi_2391_ = 0f;
                } else {
                    let _e538 = tan(_e512);
                    let _e543 = (sign((1.5707964f - _e512)) / max(abs(_e538), 0.000001f));
                    if (_e543 >= 0f) {
                        phi_2371_ = (_e534.y - ((1f - _e534.x) * _e538));
                    } else {
                        phi_2371_ = (_e534.y + (_e534.x * _e538));
                    }
                    let _e555 = phi_2371_;
                    phi_2393_ = _e555;
                    phi_2391_ = _e543;
                }
                let _e557 = phi_2393_;
                let _e559 = phi_2391_;
                phi_2409_ = vec4<f32>((max(_e534.x, 0f) + 0.25f), (-2f - _e534.y), _e559, _e557);
            }
            let _e567 = phi_2409_;
            phi_2563_ = (_e124 * (_e304 * (_e298 * _e313)));
            phi_2408_ = _e567;
        } else {
            phi_2563_ = (sign(((_e304 * _e298) * _naga_inverse_2x2_f32(_e124))) * 0.5f);
            phi_2408_ = _e491;
        }
        let _e577 = phi_2563_;
        let _e579 = phi_2408_;
        phi_2569_ = _e579;
        if (((_e191 & 8388608u) != 0u) != ((_e191 & 16777216u) != 0u)) {
            phi_2569_ = (_e579 * vec4<f32>(-1f, 1f, 1f, 1f));
        }
        let _e587 = phi_2569_;
        phi_2570_ = (((_e191 & 2147483648u) != 0u) && (_e83 != 1i));
        phi_2564_ = _e587;
        phi_2558_ = _e577;
        phi_2532_ = select(_e306, _e105, vec2((_e83 == 2i)));
    }
    let _e596 = phi_2570_;
    let _e598 = phi_2564_;
    let _e600 = phi_2558_;
    let _e602 = phi_2532_;
    let _e608 = j.fh;
    let _e611 = select(_e598.xy, vec2<f32>(1f, -1f), vec2((_e608 != 0u)));
    let _e617 = vec4<f32>(_e611.x, _e598.y, _e598.z, _e598.w);
    O = vec4<f32>(_e617.x, _e611.y, _e617.z, _e617.w);
    if !(_e596) {
        let _e625 = (_e109 + 2u);
        let _e632 = textureLoad(OB, vec2<i32>(bitcast<i32>((_e625 & 255u)), bitcast<i32>((_e625 >> bitcast<u32>(8i)))), 0i);
        let _e634 = bitcast<vec3<f32>>(_e632.yzw);
        let _e638 = (((((_e124 * _e602) + _e600) + bitcast<vec2<f32>>(_e132.xy)) * _e634.x) + _e634.yz);
        let _e641 = j.Dd[0u];
        let _e644 = j.Dd[1u];
        phi_2604_ = vec4<f32>(((_e638.x * _e641) - 1f), ((_e638.y * _e644) - sign(_e644)), 0f, 1f);
    } else {
        let _e654 = j.W2_;
        phi_2604_ = vec4(_e654);
    }
    let _e657 = phi_2604_;
    unnamed.gl_Position = _e657;
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
