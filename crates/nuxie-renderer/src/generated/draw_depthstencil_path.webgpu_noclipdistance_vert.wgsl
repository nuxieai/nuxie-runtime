struct ei {
    r2_: array<vec4<u32>>,
}

struct di {
    r2_: array<vec4<u32>>,
}

struct kg {
    r2_: array<vec4<f32>>,
}

struct jg {
    r2_: array<vec2<u32>>,
}

struct VB {
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
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

@id(15) override K7_: bool = false;
@id(16) override fe: bool = false;
@id(2) override Yi: bool = true;
@id(8) override ej: bool = true;

var<private> gl_VertexIndex_1: i32;
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(5)
var<storage> BD: ei;
@group(0) @binding(2)
var<storage> KB: di;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> O0_: vec4<f32>;
var<private> U0_: vec3<f32>;
@group(0) @binding(3)
var<storage> VC: jg;
var<private> P0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;

fn main_1() {
    var phi_1181_: bool;
    var phi_1182_: i32;
    var phi_1238_: f32;
    var phi_1191_: f32;
    var phi_1188_: bool;
    var phi_1186_: i32;
    var phi_1185_: i32;
    var phi_1183_: i32;
    var phi_1230_: f32;
    var phi_1229_: f32;
    var phi_1194_: bool;
    var phi_1228_: f32;
    var phi_1193_: bool;
    var phi_1199_: i32;
    var phi_1204_: bool;
    var phi_1206_: vec4<u32>;
    var phi_1205_: vec4<u32>;
    var phi_1246_: u32;
    var phi_1223_: vec4<u32>;
    var phi_1212_: bool;
    var phi_1248_: f32;
    var phi_1260_: f32;
    var phi_1252_: f32;
    var phi_612_: bool;
    var phi_1255_: f32;
    var phi_1270_: vec2<f32>;
    var phi_1269_: vec2<f32>;
    var phi_1280_: f32;
    var phi_1288_: vec2<f32>;
    var phi_1277_: f32;
    var phi_1264_: vec2<f32>;
    var phi_1287_: vec2<f32>;
    var phi_1276_: f32;
    var phi_1273_: f32;
    var phi_1263_: vec2<f32>;
    var phi_1299_: vec2<f32>;
    var phi_1224_: vec2<f32>;
    var phi_1298_: vec2<f32>;
    var phi_1345_: bool;
    var phi_1343_: f32;
    var phi_837_: bool;
    var phi_1353_: bool;
    var phi_1362_: u32;
    var phi_1361_: u32;

    let _e74 = gl_VertexIndex_1;
    let _e79 = ((_e74 & 536870912i) != 0i);
    let _e80 = (_e74 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1181_ = true;
                break;
            }
            if fe {
                phi_1181_ = true;
                break;
            }
            phi_1181_ = false;
            break;
        }
    }
    let _e83 = phi_1181_;
    if _e83 {
        let _e84 = select(5i, 6i, _e79);
        let _e90 = (_e80 & ((1i << bitcast<u32>(_e84)) - 1i));
        let _e91 = select(1i, 2i, _e79);
        let _e97 = (_e90 & ((1i << bitcast<u32>(_e91)) - 1i));
        phi_1182_ = _e97;
        if (K7_ && !(_e79)) {
            phi_1182_ = (_e97 + 1i);
        }
        let _e102 = phi_1182_;
        phi_1238_ = select(1f, 0f, ((_e102 == 0i) || (_e102 == 3i)));
        phi_1191_ = select(-1f, 1f, (_e102 < 2i));
        phi_1188_ = false;
        phi_1186_ = (_e80 >> bitcast<u32>(_e84));
        phi_1185_ = 8i;
        phi_1183_ = (_e90 >> bitcast<u32>(_e91));
    } else {
        let _e109 = select(4i, 5i, _e79);
        let _e115 = (_e80 & ((1i << bitcast<u32>(_e109)) - 1i));
        let _e119 = (!(_e79) && (_e115 == 9i));
        phi_1238_ = 1f;
        phi_1191_ = 0f;
        phi_1188_ = _e119;
        phi_1186_ = (_e80 >> bitcast<u32>(_e109));
        phi_1185_ = select(8i, 17i, _e79);
        phi_1183_ = select(_e115, 0i, _e119);
    }
    let _e122 = phi_1238_;
    let _e124 = phi_1191_;
    let _e126 = phi_1188_;
    let _e128 = phi_1186_;
    let _e130 = phi_1185_;
    let _e132 = phi_1183_;
    let _e134 = min(_e132, (_e130 - 1i));
    let _e136 = ((_e128 * _e130) + _e134);
    let _e141 = textureLoad(UB, vec2<i32>((_e136 & 2047i), (_e136 >> bitcast<u32>(11i))), 0i);
    let _e148 = BD.r2_[(max((_e141.w & 65535u), 1u) - 1u)];
    let _e150 = bitcast<vec2<f32>>(_e148.xy);
    let _e152 = (_e148.z & 65535u);
    let _e154 = (_e152 * 4u);
    let _e157 = KB.r2_[_e154];
    let _e158 = bitcast<vec4<f32>>(_e157);
    let _e165 = mat2x2<f32>(vec2<f32>(_e158.x, _e158.y), vec2<f32>(_e158.z, _e158.w));
    let _e169 = KB.r2_[(_e154 + 1u)];
    let _e171 = bitcast<vec2<f32>>(_e169.xy);
    let _e177 = KB.r2_[(_e154 + 2u)];
    let _e179 = (_e141.w & 8388608u);
    if K7_ {
        phi_1228_ = _e124;
        phi_1193_ = false;
    } else {
        if fe {
            let _e180 = (_e179 != 0u);
            phi_1230_ = _e124;
            if _e180 {
                phi_1230_ = -(_e124);
            }
            let _e183 = phi_1230_;
            phi_1229_ = _e183;
            phi_1194_ = _e180;
        } else {
            phi_1229_ = _e124;
            phi_1194_ = (((_e179 != 0u) && !(_e79)) && !(_e126));
        }
        let _e190 = phi_1229_;
        let _e192 = phi_1194_;
        phi_1228_ = _e190;
        phi_1193_ = _e192;
    }
    let _e194 = phi_1228_;
    let _e196 = phi_1193_;
    phi_1199_ = _e132;
    if _e196 {
        phi_1199_ = (_e132 - 1i);
    }
    let _e199 = phi_1199_;
    phi_1246_ = _e141.w;
    phi_1223_ = _e141;
    if (_e199 != _e134) {
        let _e202 = ((_e136 + _e199) - _e134);
        let _e207 = textureLoad(UB, vec2<i32>((_e202 & 2047i), (_e202 >> bitcast<u32>(11i))), 0i);
        if ((_e207.w & 8454143u) != (_e141.w & 8454143u)) {
            if K7_ {
                phi_1204_ = (_e150.x != 0f);
            } else {
                phi_1204_ = true;
            }
            let _e215 = phi_1204_;
            phi_1206_ = _e141;
            if _e215 {
                let _e216 = bitcast<i32>(_e148.w);
                let _e221 = textureLoad(UB, vec2<i32>((_e216 & 2047i), (_e216 >> bitcast<u32>(11i))), 0i);
                phi_1206_ = _e221;
            }
            let _e223 = phi_1206_;
            phi_1205_ = _e223;
        } else {
            phi_1205_ = _e207;
        }
        let _e225 = phi_1205_;
        phi_1246_ = ((_e225.w & 4286578687u) | _e179);
        phi_1223_ = _e225;
    }
    let _e230 = phi_1246_;
    let _e232 = phi_1223_;
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1212_ = true;
                break;
            }
            if fe {
                phi_1212_ = true;
                break;
            }
            phi_1212_ = false;
            break;
        }
    }
    let _e235 = phi_1212_;
    if _e235 {
        let _e238 = (_e194 * sign(determinant(_e165)));
        let _e241 = (f32(_e232.z) * 0.0000000014629181f);
        let _e245 = vec2<f32>(sin(_e241), -(cos(_e241)));
        let _e249 = select(0f, _e238, (_e122 == 0f));
        phi_1287_ = _e245;
        phi_1276_ = _e249;
        phi_1273_ = _e238;
        phi_1263_ = _e245;
        if K7_ {
            let _e251 = ((_e230 & 1048576u) != 0u);
            phi_1248_ = _e238;
            if _e251 {
                phi_1248_ = min(_e238, 0f);
            }
            let _e254 = phi_1248_;
            phi_1260_ = _e254;
            if ((_e230 & 524288u) != 0u) {
                phi_1260_ = max(_e254, 0f);
            }
            let _e259 = phi_1260_;
            let _e260 = (_e230 & 469762048u);
            phi_1288_ = _e245;
            phi_1277_ = _e249;
            phi_1264_ = _e245;
            if (_e260 > 134217728u) {
                let _e265 = f32((_e232.z & 65535u));
                let _e266 = (_e265 * 0.000015259022f);
                let _e270 = sqrt(max((1f - (_e266 * _e266)), 0f));
                phi_1252_ = _e270;
                if (((_e230 & 4194304u) != 0u) == _e251) {
                    phi_1252_ = -(_e270);
                }
                let _e274 = phi_1252_;
                let _e279 = (mat2x2<f32>(vec2<f32>(_e266, _e274), vec2<f32>(-(_e274), _e266)) * _e245);
                let _e280 = (_e260 == 201326592u);
                phi_612_ = _e280;
                if !(_e280) {
                    phi_612_ = ((_e260 != 335544320u) && (_e266 < 0.25f));
                }
                let _e286 = phi_612_;
                let _e288 = ((_e230 & 2097152u) != 0u);
                if (_e260 == 335544320u) {
                    phi_1269_ = (_e245 + _e279);
                } else {
                    phi_1270_ = _e245;
                    if (_e288 || !(_e286)) {
                        if _e286 {
                            phi_1255_ = _e266;
                        } else {
                            phi_1255_ = (65535f / _e265);
                        }
                        let _e295 = phi_1255_;
                        phi_1270_ = (_e279 * _e295);
                    }
                    let _e298 = phi_1270_;
                    phi_1269_ = _e298;
                }
                let _e300 = phi_1269_;
                phi_1280_ = _e249;
                if (!(_e79) && _e286) {
                    phi_1280_ = (0.5f * _e259);
                }
                let _e308 = phi_1280_;
                phi_1288_ = select(_e245, _e279, vec2((_e286 || _e288)));
                phi_1277_ = _e308;
                phi_1264_ = _e300;
            }
            let _e310 = phi_1288_;
            let _e312 = phi_1277_;
            let _e314 = phi_1264_;
            phi_1287_ = _e310;
            phi_1276_ = _e312;
            phi_1273_ = _e259;
            phi_1263_ = _e314;
        }
        let _e316 = phi_1287_;
        let _e318 = phi_1276_;
        let _e320 = phi_1273_;
        let _e322 = phi_1263_;
        let _e327 = ((_e165 * (bitcast<vec2<f32>>(_e232.xy) + (_e322 * (_e320 * bitcast<f32>(_e169.z))))) + _e171);
        phi_1299_ = _e327;
        if (_e318 != 0f) {
            phi_1299_ = (_e327 + (sign((_e316 * _naga_inverse_2x2_f32(_e165))) * _e318));
        }
        let _e335 = phi_1299_;
        phi_1298_ = _e335;
    } else {
        if _e126 {
            phi_1224_ = _e150;
        } else {
            phi_1224_ = bitcast<vec2<f32>>(_e232.xy);
        }
        let _e339 = phi_1224_;
        phi_1298_ = ((_e165 * _e339) + _e171);
    }
    let _e343 = phi_1298_;
    if ((_e74 & 268435456i) != 0i) {
        O0_ = vec4<f32>(0f, 0f, 0f, 0f);
        U0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e346 = VC.r2_[_e152];
        let _e348 = (_e346.x & 15u);
        phi_1345_ = false;
        if Yi {
            let _e351 = ((_e346.x >> bitcast<u32>(4i)) & 15u);
            P0_ = f32(_e351);
            phi_1345_ = (_e351 != 0u);
        }
        let _e355 = phi_1345_;
        if (_e348 == 1u) {
            O0_ = unpack4x8unorm(_e346.y);
            if _e355 {
                let _e360 = O0_[3u];
                O0_[3u] = (_e360 * _e122);
            } else {
                let _e362 = O0_;
                O0_ = (_e362 * _e122);
            }
        } else {
            let _e364 = (_e152 * 8u);
            let _e367 = JB.r2_[_e364];
            let _e378 = JB.r2_[(_e364 + 1u)];
            let _e383 = ((mat2x2<f32>(vec2<f32>(_e367.x, _e367.y), vec2<f32>(_e367.z, _e367.w)) * _e343) + _e378.xy);
            let _e394 = ((_e378.w + (f32(_e348) * 0.125f)) + (max(_e378.z, 0f) * 0.00024414063f));
            if (_e378.z < 0f) {
                phi_1343_ = -(_e394);
            } else {
                phi_1343_ = _e394;
            }
            let _e397 = phi_1343_;
            O0_ = vec4<f32>(_e383.x, _e383.y, _e397, (((_e122 * -0.5f) - 0.25f) - round((bitcast<f32>(_e346.y) * 255f))));
        }
        phi_837_ = ej;
        if ej {
            phi_837_ = ((_e346.x & 2048u) != 0u);
        }
        let _e407 = phi_837_;
        if _e407 {
            let _e408 = (_e152 * 8u);
            let _e412 = JB.r2_[(_e408 + 4u)];
            let _e423 = JB.r2_[(_e408 + 5u)];
            let _e426 = ((mat2x2<f32>(vec2<f32>(_e412.x, _e412.y), vec2<f32>(_e412.z, _e412.w)) * _e343) + _e423.xy);
            U0_ = vec3<f32>(_e426.x, _e426.y, (1f + _e423.z));
        } else {
            U0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e433 = j.Hg;
    let _e435 = j.Ig;
    let _e443 = vec4<f32>(((_e343.x * _e433) - 1f), ((_e343.y * _e435) - sign(_e435)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if K7_ {
                phi_1353_ = true;
                break;
            }
            if fe {
                phi_1353_ = true;
                break;
            }
            phi_1353_ = false;
            break;
        }
    }
    let _e446 = phi_1353_;
    if _e446 {
        let _e448 = u32((_e122 * 254f));
        phi_1362_ = _e448;
        if ((_e74 & 1073741824i) == 0i) {
            phi_1362_ = (_e448 + bitcast<u32>(1i));
        }
        let _e453 = phi_1362_;
        phi_1361_ = _e453;
    } else {
        phi_1361_ = 255u;
    }
    let _e455 = phi_1361_;
    unnamed.gl_Position = vec4<f32>(_e443.x, _e443.y, ((f32(((_e177.x << bitcast<u32>(8u)) | _e455)) * 0.000000059604645f) + 0.000000029802322f), _e443.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e8 = O0_;
    let _e9 = U0_;
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
