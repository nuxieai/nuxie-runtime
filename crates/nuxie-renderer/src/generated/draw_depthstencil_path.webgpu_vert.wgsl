enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
}

struct di {
    v2_: array<vec4<u32>>,
}

struct ci {
    v2_: array<vec4<u32>>,
}

struct hg {
    v2_: array<vec4<f32>>,
}

struct gg {
    v2_: array<vec2<u32>>,
}

struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
}

@id(15) override A6_: bool = false;
@id(1) override Vi: bool = true;
@id(2) override Wi: bool = true;
@id(8) override cj: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> BD: di;
@group(0) @binding(2)
var<storage> KB: ci;
@group(0) @binding(4)
var<storage> JB: hg;
var<private> O0_: vec4<f32>;
var<private> V0_: vec3<f32>;
@group(0) @binding(3)
var<storage> WC: gg;
var<private> P0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Ta: sampler;

fn main_1() {
    var phi_1215_: i32;
    var phi_1255_: f32;
    var phi_1243_: f32;
    var phi_1221_: bool;
    var phi_1219_: i32;
    var phi_1218_: i32;
    var phi_1216_: i32;
    var phi_1224_: i32;
    var phi_1223_: i32;
    var phi_1227_: bool;
    var phi_1229_: vec4<u32>;
    var phi_1228_: vec4<u32>;
    var phi_1248_: u32;
    var phi_1239_: vec4<u32>;
    var phi_1250_: f32;
    var phi_1260_: f32;
    var phi_1264_: f32;
    var phi_642_: bool;
    var phi_1267_: f32;
    var phi_1278_: vec2<f32>;
    var phi_1277_: vec2<f32>;
    var phi_1284_: f32;
    var phi_1291_: vec2<f32>;
    var phi_1283_: f32;
    var phi_1274_: vec2<f32>;
    var phi_1300_: vec2<f32>;
    var phi_1240_: vec2<f32>;
    var phi_1299_: vec2<f32>;
    var phi_1342_: bool;
    var phi_1340_: vec4<f32>;
    var phi_1341_: vec4<f32>;
    var phi_862_: bool;
    var phi_1361_: u32;
    var phi_1360_: u32;

    let _e76 = gl_VertexIndex_1;
    let _e81 = ((_e76 & 536870912i) != 0i);
    let _e82 = (_e76 & 268435455i);
    if A6_ {
        let _e83 = select(5i, 6i, _e81);
        let _e89 = (_e82 & ((1i << bitcast<u32>(_e83)) - 1i));
        let _e90 = select(1i, 2i, _e81);
        let _e96 = (_e89 & ((1i << bitcast<u32>(_e90)) - 1i));
        phi_1215_ = _e96;
        if !(_e81) {
            phi_1215_ = (_e96 + 1i);
        }
        let _e100 = phi_1215_;
        phi_1255_ = select(1f, 0f, ((_e100 == 0i) || (_e100 == 3i)));
        phi_1243_ = select(1f, -1f, (_e100 < 2i));
        phi_1221_ = false;
        phi_1219_ = (_e82 >> bitcast<u32>(_e83));
        phi_1218_ = 8i;
        phi_1216_ = (_e89 >> bitcast<u32>(_e90));
    } else {
        let _e107 = select(4i, 5i, _e81);
        let _e113 = (_e82 & ((1i << bitcast<u32>(_e107)) - 1i));
        let _e117 = (!(_e81) && (_e113 == 9i));
        phi_1255_ = 1f;
        phi_1243_ = 0f;
        phi_1221_ = _e117;
        phi_1219_ = (_e82 >> bitcast<u32>(_e107));
        phi_1218_ = select(8i, 17i, _e81);
        phi_1216_ = select(_e113, 0i, _e117);
    }
    let _e120 = phi_1255_;
    let _e122 = phi_1243_;
    let _e124 = phi_1221_;
    let _e126 = phi_1219_;
    let _e128 = phi_1218_;
    let _e130 = phi_1216_;
    let _e132 = min(_e130, (_e128 - 1i));
    let _e134 = ((_e126 * _e128) + _e132);
    let _e139 = textureLoad(UB, vec2<i32>((_e134 & 2047i), (_e134 >> bitcast<u32>(11i))), 0i);
    let _e146 = BD.v2_[(max((_e139.w & 65535u), 1u) - 1u)];
    let _e148 = bitcast<vec2<f32>>(_e146.xy);
    let _e150 = (_e146.z & 65535u);
    let _e152 = (_e150 * 4u);
    let _e155 = KB.v2_[_e152];
    let _e156 = bitcast<vec4<f32>>(_e155);
    let _e163 = mat2x2<f32>(vec2<f32>(_e156.x, _e156.y), vec2<f32>(_e156.z, _e156.w));
    let _e167 = KB.v2_[(_e152 + 1u)];
    let _e169 = bitcast<vec2<f32>>(_e167.xy);
    let _e175 = KB.v2_[(_e152 + 2u)];
    let _e177 = (_e139.w & 8388608u);
    if A6_ {
        phi_1223_ = _e130;
    } else {
        phi_1224_ = _e130;
        if (((_e177 != 0u) && !(_e81)) && !(_e124)) {
            phi_1224_ = (_e130 - 1i);
        }
        let _e185 = phi_1224_;
        phi_1223_ = _e185;
    }
    let _e187 = phi_1223_;
    phi_1248_ = _e139.w;
    phi_1239_ = _e139;
    if (_e187 != _e132) {
        let _e190 = ((_e134 + _e187) - _e132);
        let _e195 = textureLoad(UB, vec2<i32>((_e190 & 2047i), (_e190 >> bitcast<u32>(11i))), 0i);
        if ((_e195.w & 8454143u) != (_e139.w & 8454143u)) {
            if A6_ {
                phi_1227_ = (_e148.x != 0f);
            } else {
                phi_1227_ = true;
            }
            let _e203 = phi_1227_;
            phi_1229_ = _e139;
            if _e203 {
                let _e204 = bitcast<i32>(_e146.w);
                let _e209 = textureLoad(UB, vec2<i32>((_e204 & 2047i), (_e204 >> bitcast<u32>(11i))), 0i);
                phi_1229_ = _e209;
            }
            let _e211 = phi_1229_;
            phi_1228_ = _e211;
        } else {
            phi_1228_ = _e195;
        }
        let _e213 = phi_1228_;
        phi_1248_ = ((_e213.w & 4286578687u) | _e177);
        phi_1239_ = _e213;
    }
    let _e218 = phi_1248_;
    let _e220 = phi_1239_;
    if A6_ {
        let _e223 = (f32(_e220.z) * 0.0000000014629181f);
        let _e227 = vec2<f32>(sin(_e223), -(cos(_e223)));
        let _e232 = (_e122 * sign(determinant(_e163)));
        let _e234 = ((_e218 & 1048576u) != 0u);
        phi_1250_ = _e232;
        if _e234 {
            phi_1250_ = min(_e232, 0f);
        }
        let _e237 = phi_1250_;
        phi_1260_ = _e237;
        if ((_e218 & 524288u) != 0u) {
            phi_1260_ = max(_e237, 0f);
        }
        let _e242 = phi_1260_;
        let _e244 = select(0f, _e242, (_e120 == 0f));
        let _e245 = (_e218 & 469762048u);
        phi_1291_ = _e227;
        phi_1283_ = _e244;
        phi_1274_ = _e227;
        if (_e245 > 134217728u) {
            let _e250 = f32((_e220.z & 65535u));
            let _e251 = (_e250 * 0.000015259022f);
            let _e255 = sqrt(max((1f - (_e251 * _e251)), 0f));
            phi_1264_ = _e255;
            if (((_e218 & 4194304u) != 0u) == _e234) {
                phi_1264_ = -(_e255);
            }
            let _e259 = phi_1264_;
            let _e264 = (mat2x2<f32>(vec2<f32>(_e251, _e259), vec2<f32>(-(_e259), _e251)) * _e227);
            let _e265 = (_e245 == 201326592u);
            phi_642_ = _e265;
            if !(_e265) {
                phi_642_ = ((_e245 != 335544320u) && (_e251 < 0.25f));
            }
            let _e271 = phi_642_;
            let _e273 = ((_e218 & 2097152u) != 0u);
            if (_e245 == 335544320u) {
                phi_1277_ = (_e227 + _e264);
            } else {
                phi_1278_ = _e227;
                if (_e273 || !(_e271)) {
                    if _e271 {
                        phi_1267_ = _e251;
                    } else {
                        phi_1267_ = (65535f / _e250);
                    }
                    let _e280 = phi_1267_;
                    phi_1278_ = (_e264 * _e280);
                }
                let _e283 = phi_1278_;
                phi_1277_ = _e283;
            }
            let _e285 = phi_1277_;
            phi_1284_ = _e244;
            if (!(_e81) && _e271) {
                phi_1284_ = (0.5f * _e242);
            }
            let _e293 = phi_1284_;
            phi_1291_ = select(_e227, _e264, vec2((_e271 || _e273)));
            phi_1283_ = _e293;
            phi_1274_ = _e285;
        }
        let _e295 = phi_1291_;
        let _e297 = phi_1283_;
        let _e299 = phi_1274_;
        let _e304 = ((_e163 * (bitcast<vec2<f32>>(_e220.xy) + (_e299 * (_e242 * bitcast<f32>(_e167.z))))) + _e169);
        phi_1300_ = _e304;
        if (_e297 != 0f) {
            phi_1300_ = (_e304 + (sign((_e295 * _naga_inverse_2x2_f32(_e163))) * _e297));
        }
        let _e312 = phi_1300_;
        phi_1299_ = _e312;
    } else {
        if _e124 {
            phi_1240_ = _e148;
        } else {
            phi_1240_ = bitcast<vec2<f32>>(_e220.xy);
        }
        let _e316 = phi_1240_;
        phi_1299_ = ((_e163 * _e316) + _e169);
    }
    let _e320 = phi_1299_;
    if Vi {
        let _e321 = (_e150 * 8u);
        let _e325 = JB.v2_[(_e321 + 2u)];
        let _e336 = JB.v2_[(_e321 + 3u)];
        if any((_e325 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e341 = ((mat2x2<f32>(vec2<f32>(_e325.x, _e325.y), vec2<f32>(_e325.z, _e325.w)) * _e320) + _e336.xy);
            unnamed.gl_ClipDistance[0i] = (_e341.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e341.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e341.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e341.y);
        } else {
            let _e357 = (_e336.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e357;
            unnamed.gl_ClipDistance[2i] = _e357;
            unnamed.gl_ClipDistance[1i] = _e357;
            unnamed.gl_ClipDistance[0i] = _e357;
        }
    }
    if ((_e76 & 268435456i) != 0i) {
        O0_ = vec4<f32>(0f, 0f, 0f, 0f);
        V0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e368 = WC.v2_[_e150];
        let _e370 = (_e368.x & 15u);
        phi_1342_ = false;
        if Wi {
            let _e373 = ((_e368.x >> bitcast<u32>(4i)) & 15u);
            P0_ = f32(_e373);
            phi_1342_ = (_e373 != 0u);
        }
        let _e377 = phi_1342_;
        if (_e370 == 1u) {
            O0_ = unpack4x8unorm(_e368.y);
            if _e377 {
                let _e382 = O0_[3u];
                O0_[3u] = (_e382 * _e120);
            } else {
                let _e384 = O0_;
                O0_ = (_e384 * _e120);
            }
        } else {
            let _e386 = (_e150 * 8u);
            let _e389 = JB.v2_[_e386];
            let _e400 = JB.v2_[(_e386 + 1u)];
            let _e402 = bitcast<f32>(_e368.y);
            let _e405 = ((mat2x2<f32>(vec2<f32>(_e389.x, _e389.y), vec2<f32>(_e389.z, _e389.w)) * _e320) + _e400.xy);
            let _e411 = vec4<f32>(_e405.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e417 = vec4<f32>(_e411.x, _e405.y, _e411.z, _e411.w);
            let _e422 = vec4<f32>(_e417.x, _e417.y, _e120, _e417.w);
            phi_1340_ = _e422;
            if (_e370 != 2u) {
                phi_1340_ = vec4<f32>(_e422.x, _e422.y, (_e120 + 2f), _e422.w);
            }
            let _e431 = phi_1340_;
            phi_1341_ = _e431;
            if (_e400.z > 0.9f) {
                phi_1341_ = vec4<f32>(_e431.x, _e431.y, -(_e431.z), _e431.w);
            }
            let _e442 = phi_1341_;
            O0_ = vec4<f32>(_e442.x, _e442.y, _e442.z, -(bitcast<f32>(((((_e370 << bitcast<u32>(28i)) | ((u32(_e402) - 1u) << bitcast<u32>(17i))) | (u32((_e400.w * 512f)) << bitcast<u32>(8i))) | u32((fract(_e402) * 256f))))));
        }
        phi_862_ = cj;
        if cj {
            phi_862_ = ((_e368.x & 2048u) != 0u);
        }
        let _e470 = phi_862_;
        if _e470 {
            let _e471 = (_e150 * 8u);
            let _e475 = JB.v2_[(_e471 + 4u)];
            let _e486 = JB.v2_[(_e471 + 5u)];
            let _e489 = ((mat2x2<f32>(vec2<f32>(_e475.x, _e475.y), vec2<f32>(_e475.z, _e475.w)) * _e320) + _e486.xy);
            V0_ = vec3<f32>(_e489.x, _e489.y, (1f + _e486.z));
        } else {
            V0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e496 = j.Dg;
    let _e498 = j.Eg;
    let _e506 = vec4<f32>(((_e320.x * _e496) - 1f), ((_e320.y * _e498) - sign(_e498)), 0f, 1f);
    if A6_ {
        let _e508 = u32((_e120 * 254f));
        phi_1361_ = _e508;
        if ((_e76 & 1073741824i) == 0i) {
            phi_1361_ = (_e508 + bitcast<u32>(1i));
        }
        let _e513 = phi_1361_;
        phi_1360_ = _e513;
    } else {
        phi_1360_ = 255u;
    }
    let _e515 = phi_1360_;
    unnamed.gl_Position = vec4<f32>(_e506.x, _e506.y, ((f32(((_e175.x << bitcast<u32>(8u)) | _e515)) * 0.000000059604645f) + 0.000000029802322f), _e506.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e9 = unnamed.gl_Position;
    let _e10 = unnamed.gl_ClipDistance;
    let _e11 = O0_;
    let _e12 = V0_;
    let _e13 = P0_;
    return VertexOutput(_e9, _e10, _e11, _e12, _e13);
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
