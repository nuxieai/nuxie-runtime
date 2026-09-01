struct ig {
    c2_: array<vec4<u32>>,
}

struct hg {
    c2_: array<vec4<u32>>,
}

struct Me {
    c2_: array<vec2<u32>>,
}

struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

struct Ne {
    c2_: array<vec4<f32>>,
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

@id(0) override eh: bool = true;
@id(2) override gh: bool = true;
@id(8) override mh: bool = true;

@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(5)
var<storage> FD: ig;
@group(0) @binding(2)
var<storage> QB: hg;
var<private> gl_VertexIndex_1: i32;
var<private> gl_InstanceIndex_1: i32;
var<private> VB_1: vec4<f32>;
var<private> WB_1: vec4<f32>;
@group(0) @binding(3)
var<storage> BD: Me;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> U1_: vec2<f32>;
var<private> e2_: f32;
@group(0) @binding(4)
var<storage> RB: Ne;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_2099_: f32;
    var phi_2071_: i32;
    var phi_1371_: bool;
    var phi_2084_: i32;
    var phi_2076_: vec4<u32>;
    var phi_2083_: i32;
    var phi_2075_: vec4<u32>;
    var phi_2082_: i32;
    var phi_2080_: vec4<u32>;
    var phi_2079_: u32;
    var phi_2086_: vec2<i32>;
    var phi_2087_: vec4<u32>;
    var phi_2091_: f32;
    var phi_2162_: f32;
    var phi_2105_: f32;
    var phi_2161_: f32;
    var phi_2109_: f32;
    var phi_2106_: f32;
    var phi_2103_: f32;
    var phi_2113_: f32;
    var phi_2159_: f32;
    var phi_2112_: f32;
    var phi_2168_: f32;
    var phi_2165_: f32;
    var phi_2222_: f32;
    var phi_2194_: i32;
    var phi_2204_: f32;
    var phi_1683_: bool;
    var phi_2211_: f32;
    var phi_2232_: vec2<f32>;
    var phi_2231_: vec2<f32>;
    var phi_2230_: vec2<f32>;
    var phi_2248_: vec2<f32>;
    var phi_2233_: vec2<f32>;
    var phi_2281_: u32;
    var phi_2252_: vec2<f32>;
    var phi_2251_: bool;
    var local: u32;
    var phi_2310_: u32;
    var phi_2311_: f32;
    var phi_2312_: f32;
    var phi_2314_: vec4<f32>;
    var phi_2313_: f32;
    var local_1: u32;
    var phi_1084_: bool;
    var local_2: u32;
    var phi_2329_: vec4<f32>;

    let _e77 = gl_InstanceIndex_1;
    let _e78 = VB_1;
    let _e79 = WB_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e82 = i32(_e78.x);
            let _e85 = bitcast<i32>(_e78.w);
            let _e87 = (_e85 >> bitcast<u32>(2i));
            let _e88 = (_e85 & 3i);
            let _e90 = min(_e82, (_e87 - 1i));
            let _e92 = ((_e77 * _e87) + _e90);
            let _e97 = textureLoad(MC, vec2<i32>((_e92 & 2047i), (_e92 >> bitcast<u32>(11i))), 0i);
            let _e104 = FD.c2_[(max((_e97.w & 65535u), 1u) - 1u)];
            let _e106 = bitcast<vec2<f32>>(_e104.xy);
            let _e108 = (_e104.z & 65535u);
            let _e110 = (_e108 * 4u);
            let _e113 = QB.c2_[_e110];
            let _e114 = bitcast<vec4<f32>>(_e113);
            let _e121 = mat2x2<f32>(vec2<f32>(_e114.x, _e114.y), vec2<f32>(_e114.z, _e114.w));
            let _e125 = QB.c2_[(_e110 + 1u)];
            let _e129 = bitcast<f32>(_e125.z);
            let _e131 = bitcast<f32>(_e125.w);
            let _e132 = (_e97.w & 8388608u);
            phi_2099_ = _e78.y;
            phi_2071_ = _e82;
            local = _e108;
            local_1 = _e108;
            local_2 = _e108;
            if (_e132 != 0u) {
                phi_2099_ = _e79.y;
                phi_2071_ = i32(_e79.x);
            }
            let _e138 = phi_2099_;
            let _e140 = phi_2071_;
            phi_2082_ = _e92;
            phi_2080_ = _e97;
            phi_2079_ = _e97.w;
            if (_e140 != _e90) {
                let _e143 = ((_e92 + _e140) - _e90);
                let _e148 = textureLoad(MC, vec2<i32>((_e143 & 2047i), (_e143 >> bitcast<u32>(11i))), 0i);
                if ((_e148.w & 8454143u) != (_e97.w & 8454143u)) {
                    let _e153 = (_e129 == 0f);
                    phi_1371_ = _e153;
                    if !(_e153) {
                        phi_1371_ = (_e106.x != 0f);
                    }
                    let _e158 = phi_1371_;
                    phi_2084_ = _e92;
                    phi_2076_ = _e97;
                    if _e158 {
                        let _e159 = bitcast<i32>(_e104.w);
                        let _e164 = textureLoad(MC, vec2<i32>((_e159 & 2047i), (_e159 >> bitcast<u32>(11i))), 0i);
                        phi_2084_ = _e159;
                        phi_2076_ = _e164;
                    }
                    let _e166 = phi_2084_;
                    let _e168 = phi_2076_;
                    phi_2083_ = _e166;
                    phi_2075_ = _e168;
                } else {
                    phi_2083_ = _e143;
                    phi_2075_ = _e148;
                }
                let _e170 = phi_2083_;
                let _e172 = phi_2075_;
                phi_2082_ = _e170;
                phi_2080_ = _e172;
                phi_2079_ = ((_e172.w & 4286578687u) | _e132);
            }
            let _e177 = phi_2082_;
            let _e179 = phi_2080_;
            let _e181 = phi_2079_;
            let _e182 = (_e181 & 469762048u);
            if ((_e182 == 67108864u) && (_e88 == 0i)) {
                let _e188 = f32((_e179.z & 65535u));
                let _e191 = f32((_e179.z >> bitcast<u32>(16i)));
                let _e197 = vec2<i32>(i32((-1f - _e188)), i32(((_e191 - _e188) + 1f)));
                phi_2086_ = _e197;
                if ((_e181 & 8388608u) != 0u) {
                    phi_2086_ = -(_e197);
                }
                let _e202 = phi_2086_;
                let _e204 = (_e177 + _e202.x);
                let _e209 = textureLoad(MC, vec2<i32>((_e204 & 2047i), (_e204 >> bitcast<u32>(11i))), 0i);
                let _e211 = (_e177 + _e202.y);
                let _e216 = textureLoad(MC, vec2<i32>((_e211 & 2047i), (_e211 >> bitcast<u32>(11i))), 0i);
                phi_2087_ = _e216;
                if ((_e216.w & 8454143u) != (_e209.w & 8454143u)) {
                    let _e222 = bitcast<i32>(_e104.w);
                    let _e227 = textureLoad(MC, vec2<i32>((_e222 & 2047i), (_e222 >> bitcast<u32>(11i))), 0i);
                    phi_2087_ = _e227;
                }
                let _e229 = phi_2087_;
                let _e231 = bitcast<f32>(_e209.z);
                let _e233 = bitcast<f32>(_e229.z);
                let _e234 = (_e233 - _e231);
                phi_2091_ = _e234;
                if (abs(_e234) > 3.1415927f) {
                    phi_2091_ = (_e234 - (6.2831855f * sign(_e234)));
                }
                let _e241 = phi_2091_;
                let _e242 = (_e191 + -2f);
                let _e248 = clamp(round(((abs(_e241) * 0.31830987f) * _e242)), 1f, (_e191 + -3f));
                let _e249 = (_e242 - _e248);
                if (_e188 <= _e249) {
                    phi_2162_ = _e138;
                    if (_e188 == _e249) {
                        phi_2162_ = -(_e138);
                    }
                    let _e258 = phi_2162_;
                    phi_2161_ = _e258;
                    phi_2109_ = -(((3.1415927f * sign(_e241)) - _e241));
                    phi_2106_ = _e249;
                    phi_2103_ = _e188;
                } else {
                    let _e260 = (_e188 == (_e249 + 1f));
                    if _e260 {
                        phi_2105_ = 0f;
                    } else {
                        phi_2105_ = (_e188 - (_e249 + 2f));
                    }
                    let _e264 = phi_2105_;
                    phi_2161_ = select(_e138, 0f, _e260);
                    phi_2109_ = _e241;
                    phi_2106_ = select(_e248, 0f, _e260);
                    phi_2103_ = _e264;
                }
                let _e268 = phi_2161_;
                let _e270 = phi_2109_;
                let _e272 = phi_2106_;
                let _e274 = phi_2103_;
                if (_e274 == _e272) {
                    phi_2113_ = _e233;
                } else {
                    phi_2113_ = (_e231 + (_e270 * (_e274 / _e272)));
                }
                let _e280 = phi_2113_;
                phi_2159_ = _e268;
                phi_2112_ = _e280;
            } else {
                phi_2159_ = _e138;
                phi_2112_ = bitcast<f32>(_e179.z);
            }
            let _e284 = phi_2159_;
            let _e286 = phi_2112_;
            let _e290 = vec2<f32>(sin(_e286), -(cos(_e286)));
            let _e292 = bitcast<vec2<f32>>(_e179.xy);
            phi_2168_ = _e131;
            if (_e131 != 0f) {
                phi_2168_ = max(_e131, (1f / length((_e121 * _e290))));
            }
            let _e299 = phi_2168_;
            if (_e129 != 0f) {
                let _e303 = (_e284 * sign(determinant(_e121)));
                let _e305 = ((_e181 & 1048576u) != 0u);
                phi_2165_ = _e303;
                if _e305 {
                    phi_2165_ = min(_e303, 0f);
                }
                let _e308 = phi_2165_;
                phi_2222_ = _e308;
                if ((_e181 & 524288u) != 0u) {
                    phi_2222_ = max(_e308, 0f);
                }
                let _e313 = phi_2222_;
                let _e315 = select(0f, _e299, (_e299 != 0f));
                let _e319 = select(_e129, _e315, ((_e315 > _e129) && (_e299 == 0f)));
                let _e320 = (_e319 + _e315);
                let _e321 = (_e290 * _e320);
                phi_2230_ = _e321;
                if (_e182 > 134217728u) {
                    let _e323 = (_e181 & 4194304u);
                    let _e325 = select(2i, -2i, (_e323 == 0u));
                    phi_2194_ = _e325;
                    if ((_e181 & 8388608u) != 0u) {
                        phi_2194_ = -(_e325);
                    }
                    let _e330 = phi_2194_;
                    let _e331 = (_e177 + _e330);
                    let _e336 = textureLoad(MC, vec2<i32>((_e331 & 2047i), (_e331 >> bitcast<u32>(11i))), 0i);
                    let _e340 = abs((bitcast<f32>(_e336.z) - _e286));
                    phi_2204_ = _e340;
                    if (_e340 > 3.1415927f) {
                        phi_2204_ = (6.2831855f - _e340);
                    }
                    let _e344 = phi_2204_;
                    let _e349 = ((_e344 * select(0.5f, -0.5f, ((_e323 != 0u) == _e305))) + _e286);
                    let _e353 = vec2<f32>(sin(_e349), -(cos(_e349)));
                    let _e354 = (_e121 * _e353);
                    let _e364 = cos((_e344 * 0.5f));
                    let _e365 = (_e182 == 335544320u);
                    phi_1683_ = _e365;
                    if !(_e365) {
                        phi_1683_ = ((_e182 == 268435456u) && (_e364 >= 0.25f));
                    }
                    let _e371 = phi_1683_;
                    if _e371 {
                        phi_2211_ = (_e319 * (1f / max(_e364, select(0.25f, 1f, ((_e181 & 33554432u) != 0u)))));
                    } else {
                        phi_2211_ = ((_e319 * _e364) + (((abs(_e354.x) + abs(_e354.y)) * (1f / dot(_e354, _e354))) * 0.5f));
                    }
                    let _e382 = phi_2211_;
                    phi_2231_ = _e321;
                    if ((_e181 & 2097152u) != 0u) {
                        if (_e320 <= ((_e382 * _e364) + (_e315 * 0.125f))) {
                            phi_2232_ = (_e353 * (_e320 * (1f / _e364)));
                        } else {
                            let _e392 = (_e353 * _e382);
                            phi_2232_ = (vec2<f32>(dot(_e321, _e321), dot(_e392, _e392)) * _naga_inverse_2x2_f32(mat2x2<f32>(_e321, _e392)));
                        }
                        let _e400 = phi_2232_;
                        phi_2231_ = _e400;
                    }
                    let _e402 = phi_2231_;
                    phi_2230_ = _e402;
                }
                let _e404 = phi_2230_;
                if (_e88 != 0i) {
                    phi_2281_ = u32();
                    phi_2252_ = vec2<f32>();
                    phi_2251_ = false;
                    break;
                }
                phi_2248_ = (_e121 * (_e404 * _e313));
                phi_2233_ = _e292;
            } else {
                if (((_e181 & 2147483648u) != 0u) && (_e88 != 1i)) {
                    phi_2281_ = u32();
                    phi_2252_ = vec2<f32>();
                    phi_2251_ = false;
                    break;
                }
                phi_2248_ = vec2<f32>(0f, 0f);
                phi_2233_ = select(_e292, _e106, vec2((_e88 == 2i)));
            }
            let _e416 = phi_2248_;
            let _e418 = phi_2233_;
            let _e425 = QB.c2_[(_e110 + 2u)];
            phi_2281_ = _e425.x;
            phi_2252_ = (((_e121 * _e418) + _e416) + bitcast<vec2<f32>>(_e125.xy));
            phi_2251_ = true;
            break;
        }
    }
    let _e428 = phi_2281_;
    let _e430 = phi_2252_;
    let _e432 = phi_2251_;
    let _e435 = local;
    let _e437 = BD.c2_[_e435];
    let _e439 = (_e437.x & 15u);
    if eh {
        let _e440 = (_e439 == 0u);
        if _e440 {
            phi_2310_ = _e437.y;
        } else {
            phi_2310_ = _e437.x;
        }
        let _e443 = phi_2310_;
        let _e445 = (_e443 >> bitcast<u32>(16i));
        let _e447 = m.c6_;
        if (_e445 == 0u) {
            phi_2311_ = 0f;
        } else {
            phi_2311_ = unpack2x16float(((_e445 + 1023u) * _e447)).x;
        }
        let _e454 = phi_2311_;
        phi_2312_ = _e454;
        if _e440 {
            phi_2312_ = -(_e454);
        }
        let _e457 = phi_2312_;
        U1_[0u] = _e457;
    }
    if gh {
        e2_ = f32(((_e437.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e439 == 1u) {
        let _e465 = unpack4x8unorm(_e437.y);
        if gh {
            phi_2314_ = _e465;
        } else {
            let _e468 = (_e465.xyz * _e465.w);
            let _e474 = vec4<f32>(_e468.x, _e465.y, _e465.z, _e465.w);
            let _e480 = vec4<f32>(_e474.x, _e468.y, _e474.z, _e474.w);
            phi_2314_ = vec4<f32>(_e480.x, _e480.y, _e468.z, _e480.w);
        }
        let _e488 = phi_2314_;
        f1_ = _e488;
    } else {
        if (eh && (_e439 == 0u)) {
            let _e492 = (_e437.x >> bitcast<u32>(16i));
            let _e494 = m.c6_;
            if (_e492 == 0u) {
                phi_2313_ = 0f;
            } else {
                phi_2313_ = unpack2x16float(((_e492 + 1023u) * _e494)).x;
            }
            let _e501 = phi_2313_;
            U1_[1u] = _e501;
        } else {
            let _e504 = local_1;
            let _e505 = (_e504 * 8u);
            let _e508 = RB.c2_[_e505];
            let _e519 = RB.c2_[(_e505 + 1u)];
            let _e522 = ((mat2x2<f32>(vec2<f32>(_e508.x, _e508.y), vec2<f32>(_e508.z, _e508.w)) * _e430) + _e519.xy);
            let _e523 = (_e439 == 2u);
            if (_e523 || (_e439 == 3u)) {
                f1_[3u] = -(bitcast<f32>(_e437.y));
                if (_e519.z > 0.9f) {
                    f1_[2u] = 2f;
                } else {
                    f1_[2u] = _e519.w;
                }
                if _e523 {
                    f1_[1u] = 0f;
                    f1_[0u] = _e522.x;
                } else {
                    let _e539 = f1_[2u];
                    f1_[2u] = -(_e539);
                    f1_[0u] = _e522.x;
                    f1_[1u] = _e522.y;
                }
            }
        }
    }
    phi_1084_ = mh;
    if mh {
        phi_1084_ = ((_e437.x & 2048u) != 0u);
    }
    let _e548 = phi_1084_;
    if _e548 {
        let _e550 = local_2;
        let _e551 = (_e550 * 8u);
        let _e555 = RB.c2_[(_e551 + 4u)];
        let _e566 = RB.c2_[(_e551 + 5u)];
        let _e569 = ((mat2x2<f32>(vec2<f32>(_e555.x, _e555.y), vec2<f32>(_e555.z, _e555.w)) * _e430) + _e566.xy);
        A2_ = vec3<f32>(_e569.x, _e569.y, (1f + _e566.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    if _e432 {
        let _e576 = m.jf;
        let _e578 = m.kf;
        let _e586 = vec4<f32>(((_e430.x * _e576) - 1f), ((_e430.y * _e578) - sign(_e578)), 0f, 1f);
        phi_2329_ = vec4<f32>(_e586.x, _e586.y, (1f - (f32(_e428) * 0.000061035156f)), _e586.w);
    } else {
        let _e596 = m.R2_;
        phi_2329_ = vec4(_e596);
    }
    let _e599 = phi_2329_;
    unnamed.gl_Position = _e599;
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @builtin(instance_index) gl_InstanceIndex: u32, @location(0) VB: vec4<f32>, @location(1) WB: vec4<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    gl_InstanceIndex_1 = i32(gl_InstanceIndex);
    VB_1 = VB;
    WB_1 = WB;
    main_1();
    let _e16 = U1_;
    let _e17 = e2_;
    let _e18 = f1_;
    let _e19 = A2_;
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
