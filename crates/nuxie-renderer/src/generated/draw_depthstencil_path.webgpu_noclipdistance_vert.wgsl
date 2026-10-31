struct qi {
    x2_: array<vec4<u32>>,
}

struct kd {
    x2_: array<vec4<u32>>,
}

struct vg {
    x2_: array<vec4<f32>>,
}

struct WB {
    Hd: f32,
    Qe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    I8_: vec4<i32>,
    Ki: vec2<f32>,
    Re: vec2<f32>,
    w2_: u32,
    Oi: f32,
    w6_: u32,
    j3_: f32,
    Se: f32,
    xg: u32,
    I3_: f32,
    J3_: f32,
    Te: f32,
    Hi: u32,
    Ea: u32,
    P8_: f32,
    Q8_: f32,
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

@id(16) override E6_: bool = false;
@id(17) override qe: bool = false;
@id(2) override mj: bool = true;
@id(4) override oj: bool = true;

var<private> gl_VertexIndex_1: i32;
@group(0) @binding(6)
var VB: texture_2d<u32>;
@group(0) @binding(4)
var<storage> CD: qi;
@group(0) @binding(2)
var<storage> JB: kd;
@group(0) @binding(3)
var<storage> KB: vg;
var<private> M0_: vec4<f32>;
var<private> N0_: vec3<f32>;
var<private> R0_: f32;
@group(0) @binding(0)
var<uniform> j: WB;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var fb: sampler;

fn main_1() {
    var phi_1195_: bool;
    var phi_1196_: i32;
    var phi_1258_: f32;
    var phi_1208_: f32;
    var phi_1204_: bool;
    var phi_1200_: i32;
    var phi_1199_: i32;
    var phi_1197_: i32;
    var phi_1301_: f32;
    var phi_1249_: f32;
    var phi_1248_: f32;
    var phi_1211_: bool;
    var phi_1247_: f32;
    var phi_1210_: bool;
    var phi_1217_: i32;
    var phi_1222_: bool;
    var phi_1224_: vec4<u32>;
    var phi_1223_: vec4<u32>;
    var phi_1266_: u32;
    var phi_1242_: vec4<u32>;
    var phi_1231_: bool;
    var phi_1268_: f32;
    var phi_1280_: f32;
    var phi_1272_: f32;
    var phi_614_: bool;
    var phi_1275_: f32;
    var phi_1290_: vec2<f32>;
    var phi_1289_: vec2<f32>;
    var phi_1321_: f32;
    var phi_1329_: vec2<f32>;
    var phi_1318_: f32;
    var phi_1284_: vec2<f32>;
    var phi_1328_: vec2<f32>;
    var phi_1317_: f32;
    var phi_1293_: f32;
    var phi_1283_: vec2<f32>;
    var phi_1340_: vec2<f32>;
    var phi_1243_: vec2<f32>;
    var phi_1339_: vec2<f32>;
    var phi_1387_: bool;
    var phi_1385_: f32;
    var phi_829_: bool;
    var phi_1388_: f32;
    var phi_1397_: bool;
    var phi_1407_: u32;
    var phi_1406_: u32;

    let _e77 = gl_VertexIndex_1;
    let _e82 = ((_e77 & 536870912i) != 0i);
    let _e83 = (_e77 & 268435455i);
    switch bitcast<i32>(0u) {
        default: {
            if E6_ {
                phi_1195_ = true;
                break;
            }
            if qe {
                phi_1195_ = true;
                break;
            }
            phi_1195_ = false;
            break;
        }
    }
    let _e86 = phi_1195_;
    if _e86 {
        let _e87 = select(5i, 6i, _e82);
        let _e93 = (_e83 & ((1i << bitcast<u32>(_e87)) - 1i));
        let _e94 = select(1i, 2i, _e82);
        let _e100 = (_e93 & ((1i << bitcast<u32>(_e94)) - 1i));
        phi_1196_ = _e100;
        if (E6_ && !(_e82)) {
            phi_1196_ = (_e100 + 1i);
        }
        let _e105 = phi_1196_;
        phi_1258_ = select(1f, 0f, ((_e105 == 0i) || (_e105 == 3i)));
        phi_1208_ = select(-1f, 1f, (_e105 < 2i));
        phi_1204_ = false;
        phi_1200_ = (_e83 >> bitcast<u32>(_e87));
        phi_1199_ = 8i;
        phi_1197_ = (_e93 >> bitcast<u32>(_e94));
    } else {
        let _e112 = select(4i, 5i, _e82);
        let _e118 = (_e83 & ((1i << bitcast<u32>(_e112)) - 1i));
        let _e122 = (!(_e82) && (_e118 == 9i));
        phi_1258_ = 1f;
        phi_1208_ = 0f;
        phi_1204_ = _e122;
        phi_1200_ = (_e83 >> bitcast<u32>(_e112));
        phi_1199_ = select(8i, 17i, _e82);
        phi_1197_ = select(_e118, 0i, _e122);
    }
    let _e125 = phi_1258_;
    let _e127 = phi_1208_;
    let _e129 = phi_1204_;
    let _e131 = phi_1200_;
    let _e133 = phi_1199_;
    let _e135 = phi_1197_;
    let _e137 = min(_e135, (_e133 - 1i));
    let _e139 = ((_e131 * _e133) + _e137);
    let _e144 = textureLoad(VB, vec2<i32>((_e139 & 2047i), (_e139 >> bitcast<u32>(11i))), 0i);
    let _e151 = CD.x2_[(max((_e144.w & 65535u), 1u) - 1u)];
    let _e153 = bitcast<vec2<f32>>(_e151.xy);
    let _e155 = (_e151.z & 65535u);
    let _e157 = (_e155 * 4u);
    let _e160 = JB.x2_[_e157];
    let _e161 = bitcast<vec4<f32>>(_e160);
    let _e168 = mat2x2<f32>(vec2<f32>(_e161.x, _e161.y), vec2<f32>(_e161.z, _e161.w));
    let _e172 = JB.x2_[(_e157 + 1u)];
    let _e174 = bitcast<vec2<f32>>(_e172.xy);
    phi_1301_ = 0f;
    if E6_ {
        let _e182 = JB.x2_[(_e157 + 2u)][0u];
        phi_1301_ = bitcast<f32>(_e182);
    }
    let _e185 = phi_1301_;
    let _e186 = (_e144.w & 8388608u);
    if E6_ {
        phi_1247_ = _e127;
        phi_1210_ = false;
    } else {
        if qe {
            let _e187 = (_e186 != 0u);
            phi_1249_ = _e127;
            if _e187 {
                phi_1249_ = -(_e127);
            }
            let _e190 = phi_1249_;
            phi_1248_ = _e190;
            phi_1211_ = _e187;
        } else {
            phi_1248_ = _e127;
            phi_1211_ = (((_e186 != 0u) && !(_e82)) && !(_e129));
        }
        let _e197 = phi_1248_;
        let _e199 = phi_1211_;
        phi_1247_ = _e197;
        phi_1210_ = _e199;
    }
    let _e201 = phi_1247_;
    let _e203 = phi_1210_;
    phi_1217_ = _e135;
    if _e203 {
        phi_1217_ = (_e135 - 1i);
    }
    let _e206 = phi_1217_;
    phi_1266_ = _e144.w;
    phi_1242_ = _e144;
    if (_e206 != _e137) {
        let _e209 = ((_e139 + _e206) - _e137);
        let _e214 = textureLoad(VB, vec2<i32>((_e209 & 2047i), (_e209 >> bitcast<u32>(11i))), 0i);
        if ((_e214.w & 8454143u) != (_e144.w & 8454143u)) {
            if E6_ {
                phi_1222_ = (_e153.x != 0f);
            } else {
                phi_1222_ = true;
            }
            let _e222 = phi_1222_;
            phi_1224_ = _e144;
            if _e222 {
                let _e223 = bitcast<i32>(_e151.w);
                let _e228 = textureLoad(VB, vec2<i32>((_e223 & 2047i), (_e223 >> bitcast<u32>(11i))), 0i);
                phi_1224_ = _e228;
            }
            let _e230 = phi_1224_;
            phi_1223_ = _e230;
        } else {
            phi_1223_ = _e214;
        }
        let _e232 = phi_1223_;
        phi_1266_ = ((_e232.w & 4286578687u) | _e186);
        phi_1242_ = _e232;
    }
    let _e237 = phi_1266_;
    let _e239 = phi_1242_;
    switch bitcast<i32>(0u) {
        default: {
            if E6_ {
                phi_1231_ = true;
                break;
            }
            if qe {
                phi_1231_ = true;
                break;
            }
            phi_1231_ = false;
            break;
        }
    }
    let _e242 = phi_1231_;
    if _e242 {
        let _e245 = (_e201 * sign(determinant(_e168)));
        let _e248 = (f32(_e239.z) * 0.0000000014629181f);
        let _e252 = vec2<f32>(sin(_e248), -(cos(_e248)));
        let _e256 = select(0f, _e245, (_e125 == 0f));
        phi_1328_ = _e252;
        phi_1317_ = _e256;
        phi_1293_ = _e245;
        phi_1283_ = _e252;
        if E6_ {
            let _e258 = ((_e237 & 1048576u) != 0u);
            phi_1268_ = _e245;
            if _e258 {
                phi_1268_ = min(_e245, 0f);
            }
            let _e261 = phi_1268_;
            phi_1280_ = _e261;
            if ((_e237 & 524288u) != 0u) {
                phi_1280_ = max(_e261, 0f);
            }
            let _e266 = phi_1280_;
            let _e267 = (_e237 & 469762048u);
            phi_1329_ = _e252;
            phi_1318_ = _e256;
            phi_1284_ = _e252;
            if (_e267 > 201326592u) {
                let _e272 = f32((_e239.z & 65535u));
                let _e273 = (_e272 * 0.000015259022f);
                let _e277 = sqrt(max((1f - (_e273 * _e273)), 0f));
                phi_1272_ = _e277;
                if (((_e237 & 4194304u) != 0u) == _e258) {
                    phi_1272_ = -(_e277);
                }
                let _e281 = phi_1272_;
                let _e286 = (mat2x2<f32>(vec2<f32>(_e273, _e281), vec2<f32>(-(_e281), _e273)) * _e252);
                let _e287 = (_e267 == 268435456u);
                phi_614_ = _e287;
                if !(_e287) {
                    phi_614_ = ((_e267 != 402653184u) && (_e273 < 0.25f));
                }
                let _e293 = phi_614_;
                let _e295 = ((_e237 & 2097152u) != 0u);
                if (_e267 == 402653184u) {
                    phi_1289_ = (_e252 + _e286);
                } else {
                    phi_1290_ = _e252;
                    if (_e295 || !(_e293)) {
                        if _e293 {
                            phi_1275_ = _e273;
                        } else {
                            phi_1275_ = (65535f / _e272);
                        }
                        let _e302 = phi_1275_;
                        phi_1290_ = (_e286 * _e302);
                    }
                    let _e305 = phi_1290_;
                    phi_1289_ = _e305;
                }
                let _e307 = phi_1289_;
                phi_1321_ = _e256;
                if (!(_e82) && _e293) {
                    phi_1321_ = (0.5f * _e266);
                }
                let _e315 = phi_1321_;
                phi_1329_ = select(_e252, _e286, vec2((_e293 || _e295)));
                phi_1318_ = _e315;
                phi_1284_ = _e307;
            }
            let _e317 = phi_1329_;
            let _e319 = phi_1318_;
            let _e321 = phi_1284_;
            phi_1328_ = _e317;
            phi_1317_ = _e319;
            phi_1293_ = _e266;
            phi_1283_ = _e321;
        }
        let _e323 = phi_1328_;
        let _e325 = phi_1317_;
        let _e327 = phi_1293_;
        let _e329 = phi_1283_;
        let _e334 = ((_e168 * (bitcast<vec2<f32>>(_e239.xy) + (_e329 * (_e327 * _e185)))) + _e174);
        phi_1340_ = _e334;
        if (_e325 != 0f) {
            phi_1340_ = (_e334 + (sign((_e323 * _naga_inverse_2x2_f32(_e168))) * _e325));
        }
        let _e342 = phi_1340_;
        phi_1339_ = _e342;
    } else {
        if _e129 {
            phi_1243_ = _e153;
        } else {
            phi_1243_ = bitcast<vec2<f32>>(_e239.xy);
        }
        let _e346 = phi_1243_;
        phi_1339_ = ((_e168 * _e346) + _e174);
    }
    let _e350 = phi_1339_;
    if ((_e77 & 268435456i) != 0i) {
        M0_ = vec4<f32>(0f, 0f, 0f, 0f);
        N0_ = vec3<f32>(0f, 0f, 0f);
    } else {
        let _e351 = (_e172.z & 15u);
        phi_1387_ = false;
        if mj {
            let _e354 = ((_e172.z >> bitcast<u32>(4i)) & 15u);
            R0_ = f32(_e354);
            phi_1387_ = (_e354 != 0u);
        }
        let _e358 = phi_1387_;
        if (_e351 == 1u) {
            M0_ = unpack4x8unorm(_e172.w);
            if _e358 {
                let _e363 = M0_[3u];
                M0_[3u] = (_e363 * _e125);
            } else {
                let _e365 = M0_;
                M0_ = (_e365 * _e125);
            }
        } else {
            let _e367 = (_e155 * 8u);
            let _e370 = KB.x2_[_e367];
            let _e381 = KB.x2_[(_e367 + 1u)];
            let _e386 = ((mat2x2<f32>(vec2<f32>(_e370.x, _e370.y), vec2<f32>(_e370.z, _e370.w)) * _e350) + _e381.xy);
            let _e397 = ((_e381.w + (f32(_e351) * 0.125f)) + (max(_e381.z, 0f) * 0.00024414063f));
            if (_e381.z < 0f) {
                phi_1385_ = -(_e397);
            } else {
                phi_1385_ = _e397;
            }
            let _e400 = phi_1385_;
            M0_ = vec4<f32>(_e386.x, _e386.y, _e400, (((_e125 * -0.5f) - 0.25f) - round((bitcast<f32>(_e172.w) * 255f))));
        }
        phi_829_ = oj;
        if oj {
            phi_829_ = ((_e172.z & 2048u) != 0u);
        }
        let _e410 = phi_829_;
        if _e410 {
            let _e411 = (_e155 * 8u);
            let _e415 = KB.x2_[(_e411 + 4u)];
            let _e426 = KB.x2_[(_e411 + 5u)];
            let _e429 = ((mat2x2<f32>(vec2<f32>(_e415.x, _e415.y), vec2<f32>(_e415.z, _e415.w)) * _e350) + _e426.xy);
            phi_1388_ = (1f + _e426.z);
            if ((_e172.z & 4096u) != 0u) {
                phi_1388_ = (-1f - f32(((_e172.z & 24576u) >> bitcast<u32>(13u))));
            }
            let _e440 = phi_1388_;
            N0_ = vec3<f32>(_e429.x, _e429.y, _e440);
        } else {
            N0_ = vec3<f32>(0f, 0f, 0f);
        }
    }
    let _e445 = j.Sg;
    let _e447 = j.Tg;
    let _e455 = vec4<f32>(((_e350.x * _e445) - 1f), ((_e350.y * _e447) - sign(_e447)), 0f, 1f);
    switch bitcast<i32>(0u) {
        default: {
            if E6_ {
                phi_1397_ = true;
                break;
            }
            if qe {
                phi_1397_ = true;
                break;
            }
            phi_1397_ = false;
            break;
        }
    }
    let _e458 = phi_1397_;
    if _e458 {
        let _e460 = u32((_e125 * 254f));
        phi_1407_ = _e460;
        if ((_e77 & 1073741824i) == 0i) {
            phi_1407_ = (_e460 + bitcast<u32>(1i));
        }
        let _e465 = phi_1407_;
        phi_1406_ = _e465;
    } else {
        phi_1406_ = 255u;
    }
    let _e467 = phi_1406_;
    unnamed.gl_Position = vec4<f32>(_e455.x, _e455.y, ((f32((((_e172.z >> bitcast<u32>(16u)) << bitcast<u32>(8u)) | _e467)) * 0.000000059604645f) + 0.000000029802322f), _e455.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    main_1();
    let _e8 = M0_;
    let _e9 = N0_;
    let _e10 = R0_;
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
