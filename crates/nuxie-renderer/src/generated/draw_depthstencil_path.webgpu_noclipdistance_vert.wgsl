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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct VertexOutput {
    @location(0) member: vec4<f32>,
    @location(2) member_1: vec3<f32>,
    @location(1) @interpolate(flat, either) member_2: f32,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(15) override A6_: bool = false;
@id(2) override Wi: bool = true;
@id(8) override cj: bool = true;

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
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Ta: sampler;

fn main_1() {
    var phi_1128_: i32;
    var phi_1168_: f32;
    var phi_1156_: f32;
    var phi_1134_: bool;
    var phi_1132_: i32;
    var phi_1131_: i32;
    var phi_1129_: i32;
    var phi_1137_: i32;
    var phi_1136_: i32;
    var phi_1140_: bool;
    var phi_1142_: vec4<u32>;
    var phi_1141_: vec4<u32>;
    var phi_1161_: u32;
    var phi_1152_: vec4<u32>;
    var phi_1163_: f32;
    var phi_1173_: f32;
    var phi_1177_: f32;
    var phi_593_: bool;
    var phi_1180_: f32;
    var phi_1191_: vec2<f32>;
    var phi_1190_: vec2<f32>;
    var phi_1197_: f32;
    var phi_1204_: vec2<f32>;
    var phi_1196_: f32;
    var phi_1187_: vec2<f32>;
    var phi_1213_: vec2<f32>;
    var phi_1153_: vec2<f32>;
    var phi_1212_: vec2<f32>;
    var phi_1251_: bool;
    var phi_1249_: vec4<f32>;
    var phi_1250_: vec4<f32>;
    var phi_816_: bool;
    var phi_1268_: u32;
    var phi_1267_: u32;

    let _e75 = gl_VertexIndex_1;
    let _e80 = ((_e75 & 536870912i) != 0i);
    let _e81 = (_e75 & 268435455i);
    if A6_ {
        let _e82 = select(5i, 6i, _e80);
        let _e88 = (_e81 & ((1i << bitcast<u32>(_e82)) - 1i));
        let _e89 = select(1i, 2i, _e80);
        let _e95 = (_e88 & ((1i << bitcast<u32>(_e89)) - 1i));
        phi_1128_ = _e95;
        if !(_e80) {
            phi_1128_ = (_e95 + 1i);
        }
        let _e99 = phi_1128_;
        phi_1168_ = select(1f, 0f, ((_e99 == 0i) || (_e99 == 3i)));
        phi_1156_ = select(1f, -1f, (_e99 < 2i));
        phi_1134_ = false;
        phi_1132_ = (_e81 >> bitcast<u32>(_e82));
        phi_1131_ = 8i;
        phi_1129_ = (_e88 >> bitcast<u32>(_e89));
    } else {
        let _e106 = select(4i, 5i, _e80);
        let _e112 = (_e81 & ((1i << bitcast<u32>(_e106)) - 1i));
        let _e116 = (!(_e80) && (_e112 == 9i));
        phi_1168_ = 1f;
        phi_1156_ = 0f;
        phi_1134_ = _e116;
        phi_1132_ = (_e81 >> bitcast<u32>(_e106));
        phi_1131_ = select(8i, 17i, _e80);
        phi_1129_ = select(_e112, 0i, _e116);
    }
    let _e119 = phi_1168_;
    let _e121 = phi_1156_;
    let _e123 = phi_1134_;
    let _e125 = phi_1132_;
    let _e127 = phi_1131_;
    let _e129 = phi_1129_;
    let _e131 = min(_e129, (_e127 - 1i));
    let _e133 = ((_e125 * _e127) + _e131);
    let _e138 = textureLoad(UB, vec2<i32>((_e133 & 2047i), (_e133 >> bitcast<u32>(11i))), 0i);
    let _e145 = BD.v2_[(max((_e138.w & 65535u), 1u) - 1u)];
    let _e147 = bitcast<vec2<f32>>(_e145.xy);
    let _e149 = (_e145.z & 65535u);
    let _e151 = (_e149 * 4u);
    let _e154 = KB.v2_[_e151];
    let _e155 = bitcast<vec4<f32>>(_e154);
    let _e162 = mat2x2<f32>(vec2<f32>(_e155.x, _e155.y), vec2<f32>(_e155.z, _e155.w));
    let _e166 = KB.v2_[(_e151 + 1u)];
    let _e168 = bitcast<vec2<f32>>(_e166.xy);
    let _e174 = KB.v2_[(_e151 + 2u)];
    let _e176 = (_e138.w & 8388608u);
    if A6_ {
        phi_1136_ = _e129;
    } else {
        phi_1137_ = _e129;
        if (((_e176 != 0u) && !(_e80)) && !(_e123)) {
            phi_1137_ = (_e129 - 1i);
        }
        let _e184 = phi_1137_;
        phi_1136_ = _e184;
    }
    let _e186 = phi_1136_;
    phi_1161_ = _e138.w;
    phi_1152_ = _e138;
    if (_e186 != _e131) {
        let _e189 = ((_e133 + _e186) - _e131);
        let _e194 = textureLoad(UB, vec2<i32>((_e189 & 2047i), (_e189 >> bitcast<u32>(11i))), 0i);
        if ((_e194.w & 8454143u) != (_e138.w & 8454143u)) {
            if A6_ {
                phi_1140_ = (_e147.x != 0f);
            } else {
                phi_1140_ = true;
            }
            let _e202 = phi_1140_;
            phi_1142_ = _e138;
            if _e202 {
                let _e203 = bitcast<i32>(_e145.w);
                let _e208 = textureLoad(UB, vec2<i32>((_e203 & 2047i), (_e203 >> bitcast<u32>(11i))), 0i);
                phi_1142_ = _e208;
            }
            let _e210 = phi_1142_;
            phi_1141_ = _e210;
        } else {
            phi_1141_ = _e194;
        }
        let _e212 = phi_1141_;
        phi_1161_ = ((_e212.w & 4286578687u) | _e176);
        phi_1152_ = _e212;
    }
    let _e217 = phi_1161_;
    let _e219 = phi_1152_;
    if A6_ {
        let _e222 = (f32(_e219.z) * 0.0000000014629181f);
        let _e226 = vec2<f32>(sin(_e222), -(cos(_e222)));
        let _e231 = (_e121 * sign(determinant(_e162)));
        let _e233 = ((_e217 & 1048576u) != 0u);
        phi_1163_ = _e231;
        if _e233 {
            phi_1163_ = min(_e231, 0f);
        }
        let _e236 = phi_1163_;
        phi_1173_ = _e236;
        if ((_e217 & 524288u) != 0u) {
            phi_1173_ = max(_e236, 0f);
        }
        let _e241 = phi_1173_;
        let _e243 = select(0f, _e241, (_e119 == 0f));
        let _e244 = (_e217 & 469762048u);
        phi_1204_ = _e226;
        phi_1196_ = _e243;
        phi_1187_ = _e226;
        if (_e244 > 134217728u) {
            let _e249 = f32((_e219.z & 65535u));
            let _e250 = (_e249 * 0.000015259022f);
            let _e254 = sqrt(max((1f - (_e250 * _e250)), 0f));
            phi_1177_ = _e254;
            if (((_e217 & 4194304u) != 0u) == _e233) {
                phi_1177_ = -(_e254);
            }
            let _e258 = phi_1177_;
            let _e263 = (mat2x2<f32>(vec2<f32>(_e250, _e258), vec2<f32>(-(_e258), _e250)) * _e226);
            let _e264 = (_e244 == 201326592u);
            phi_593_ = _e264;
            if !(_e264) {
                phi_593_ = ((_e244 != 335544320u) && (_e250 < 0.25f));
            }
            let _e270 = phi_593_;
            let _e272 = ((_e217 & 2097152u) != 0u);
            if (_e244 == 335544320u) {
                phi_1190_ = (_e226 + _e263);
            } else {
                phi_1191_ = _e226;
                if (_e272 || !(_e270)) {
                    if _e270 {
                        phi_1180_ = _e250;
                    } else {
                        phi_1180_ = (65535f / _e249);
                    }
                    let _e279 = phi_1180_;
                    phi_1191_ = (_e263 * _e279);
                }
                let _e282 = phi_1191_;
                phi_1190_ = _e282;
            }
            let _e284 = phi_1190_;
            phi_1197_ = _e243;
            if (!(_e80) && _e270) {
                phi_1197_ = (0.5f * _e241);
            }
            let _e292 = phi_1197_;
            phi_1204_ = select(_e226, _e263, vec2((_e270 || _e272)));
            phi_1196_ = _e292;
            phi_1187_ = _e284;
        }
        let _e294 = phi_1204_;
        let _e296 = phi_1196_;
        let _e298 = phi_1187_;
        let _e303 = ((_e162 * (bitcast<vec2<f32>>(_e219.xy) + (_e298 * (_e241 * bitcast<f32>(_e166.z))))) + _e168);
        phi_1213_ = _e303;
        if (_e296 != 0f) {
            phi_1213_ = (_e303 + (sign((_e294 * _naga_inverse_2x2_f32(_e162))) * _e296));
        }
        let _e311 = phi_1213_;
        phi_1212_ = _e311;
    } else {
        if _e123 {
            phi_1153_ = _e147;
        } else {
            phi_1153_ = bitcast<vec2<f32>>(_e219.xy);
        }
        let _e315 = phi_1153_;
        phi_1212_ = ((_e162 * _e315) + _e168);
    }
    let _e319 = phi_1212_;
    if ((_e75 & 268435456i) != 0i) {
        O0_ = vec4<f32>(0f, 0f, 0f, 0f);
        V0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e322 = WC.v2_[_e149];
        let _e324 = (_e322.x & 15u);
        phi_1251_ = false;
        if Wi {
            let _e327 = ((_e322.x >> bitcast<u32>(4i)) & 15u);
            P0_ = f32(_e327);
            phi_1251_ = (_e327 != 0u);
        }
        let _e331 = phi_1251_;
        if (_e324 == 1u) {
            O0_ = unpack4x8unorm(_e322.y);
            if _e331 {
                let _e336 = O0_[3u];
                O0_[3u] = (_e336 * _e119);
            } else {
                let _e338 = O0_;
                O0_ = (_e338 * _e119);
            }
        } else {
            let _e340 = (_e149 * 8u);
            let _e343 = JB.v2_[_e340];
            let _e354 = JB.v2_[(_e340 + 1u)];
            let _e356 = bitcast<f32>(_e322.y);
            let _e359 = ((mat2x2<f32>(vec2<f32>(_e343.x, _e343.y), vec2<f32>(_e343.z, _e343.w)) * _e319) + _e354.xy);
            let _e365 = vec4<f32>(_e359.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
            let _e371 = vec4<f32>(_e365.x, _e359.y, _e365.z, _e365.w);
            let _e376 = vec4<f32>(_e371.x, _e371.y, _e119, _e371.w);
            phi_1249_ = _e376;
            if (_e324 != 2u) {
                phi_1249_ = vec4<f32>(_e376.x, _e376.y, (_e119 + 2f), _e376.w);
            }
            let _e385 = phi_1249_;
            phi_1250_ = _e385;
            if (_e354.z > 0.9f) {
                phi_1250_ = vec4<f32>(_e385.x, _e385.y, -(_e385.z), _e385.w);
            }
            let _e396 = phi_1250_;
            O0_ = vec4<f32>(_e396.x, _e396.y, _e396.z, -(bitcast<f32>(((((_e324 << bitcast<u32>(28i)) | ((u32(_e356) - 1u) << bitcast<u32>(17i))) | (u32((_e354.w * 512f)) << bitcast<u32>(8i))) | u32((fract(_e356) * 256f))))));
        }
        phi_816_ = cj;
        if cj {
            phi_816_ = ((_e322.x & 2048u) != 0u);
        }
        let _e424 = phi_816_;
        if _e424 {
            let _e425 = (_e149 * 8u);
            let _e429 = JB.v2_[(_e425 + 4u)];
            let _e440 = JB.v2_[(_e425 + 5u)];
            let _e443 = ((mat2x2<f32>(vec2<f32>(_e429.x, _e429.y), vec2<f32>(_e429.z, _e429.w)) * _e319) + _e440.xy);
            V0_ = vec3<f32>(_e443.x, _e443.y, (1f + _e440.z));
        } else {
            V0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e450 = j.Dg;
    let _e452 = j.Eg;
    let _e460 = vec4<f32>(((_e319.x * _e450) - 1f), ((_e319.y * _e452) - sign(_e452)), 0f, 1f);
    if A6_ {
        let _e462 = u32((_e119 * 254f));
        phi_1268_ = _e462;
        if ((_e75 & 1073741824i) == 0i) {
            phi_1268_ = (_e462 + bitcast<u32>(1i));
        }
        let _e467 = phi_1268_;
        phi_1267_ = _e467;
    } else {
        phi_1267_ = 255u;
    }
    let _e469 = phi_1267_;
    unnamed.gl_Position = vec4<f32>(_e460.x, _e460.y, ((f32(((_e174.x << bitcast<u32>(8u)) | _e469)) * 0.000000059604645f) + 0.000000029802322f), _e460.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e8 = O0_;
    let _e9 = V0_;
    let _e10 = P0_;
    let _e11 = unnamed.gl_Position;
    return VertexOutput(_e8, _e9, _e10, _e11);
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
