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

var<private> B6_1: vec4<f32>;
var<private> C6_1: vec4<f32>;
var<private> S4_1: vec4<f32>;
var<private> G7_1: u32;
var<private> T4_1: vec3<f32>;
var<private> oh: vec4<u32>;
@group(0) @binding(0)
var<uniform> j: TB;

fn main_1() {
    var phi_928_: vec2<f32>;
    var phi_931_: vec2<f32>;
    var phi_936_: u32;
    var phi_943_: u32;
    var phi_281_: bool;
    var phi_954_: f32;
    var phi_949_: f32;
    var phi_951_: f32;
    var phi_947_: f32;
    var phi_942_: u32;
    var phi_1001_: f32;
    var phi_994_: mat2x2<f32>;
    var phi_955_: u32;
    var phi_950_: f32;
    var phi_946_: f32;
    var phi_309_: bool;
    var phi_1048_: vec2<f32>;
    var phi_1049_: f32;
    var phi_1054_: vec2<f32>;
    var phi_1003_: f32;
    var phi_1002_: i32;
    var phi_1174_: f32;
    var local: f32;
    var local_1: f32;
    var phi_1006_: f32;
    var phi_1009_: f32;
    var phi_1010_: vec2<f32>;
    var phi_1011_: f32;
    var phi_1112_: f32;
    var phi_1043_: f32;
    var phi_1033_: f32;
    var phi_1044_: f32;
    var phi_1111_: f32;
    var phi_1106_: f32;
    var phi_1053_: vec2<f32>;
    var phi_1105_: f32;
    var phi_1050_: vec2<f32>;
    var phi_1135_: f32;
    var phi_1136_: u32;
    var phi_1173_: vec4<u32>;
    var local_2: f32;

    let _e45 = B6_1;
    let _e46 = _e45.xy;
    let _e47 = _e45.zw;
    let _e48 = C6_1;
    let _e49 = _e48.xy;
    let _e50 = _e48.zw;
    if any((_e46 != _e47)) {
        phi_928_ = _e47;
    } else {
        phi_928_ = select(_e50, _e49, vec2(any((_e47 != _e49))));
    }
    let _e58 = phi_928_;
    if any((_e50 != _e49)) {
        phi_931_ = _e49;
    } else {
        phi_931_ = select(_e46, _e47, vec2(any((_e49 != _e47))));
    }
    let _e67 = phi_931_;
    let _e68 = (_e50 - _e67);
    let _e71 = S4_1[0u];
    let _e73 = max(floor(_e71), 0f);
    let _e75 = S4_1[1u];
    let _e77 = S4_1[2u];
    let _e78 = u32(_e77);
    let _e83 = f32((_e78 >> bitcast<u32>(10i)));
    let _e85 = S4_1[3u];
    let _e86 = G7_1;
    let _e87 = (_e75 - _e83);
    let _e88 = (_e73 <= _e87);
    if _e88 {
        phi_1001_ = _e85;
        phi_994_ = mat2x2<f32>((_e58 - _e46), _e68);
        phi_955_ = (_e86 & 3825205247u);
        phi_950_ = _e87;
        phi_946_ = _e73;
    } else {
        let _e90 = T4_1;
        let _e95 = (_e73 - _e87);
        let _e97 = T4_1[2u];
        let _e98 = (_e86 & 469762048u);
        if (_e98 > 134217728u) {
            phi_936_ = _e86;
            if (_e95 < 2.5f) {
                phi_936_ = (_e86 | 4194304u);
            }
            let _e103 = phi_936_;
            phi_943_ = _e103;
            if ((_e95 > 1.5f) && (_e95 < 3.5f)) {
                phi_943_ = (_e103 | 2097152u);
            }
            let _e109 = phi_943_;
            phi_951_ = _e83;
            phi_947_ = _e95;
            phi_942_ = _e109;
        } else {
            let _e111 = ((_e86 & 33554432u) != 0u);
            phi_281_ = _e111;
            if !(_e111) {
                phi_281_ = (_e98 == 67108864u);
            }
            let _e115 = phi_281_;
            phi_954_ = _e83;
            phi_949_ = _e95;
            if _e115 {
                phi_954_ = (_e83 - 2f);
                phi_949_ = (_e95 - 1f);
            }
            let _e119 = phi_954_;
            let _e121 = phi_949_;
            phi_951_ = _e119;
            phi_947_ = _e121;
            phi_942_ = _e86;
        }
        let _e123 = phi_951_;
        let _e125 = phi_947_;
        let _e127 = phi_942_;
        phi_1001_ = _e97;
        phi_994_ = mat2x2<f32>(_e68, vec2<f32>(_e90.x, _e90.y));
        phi_955_ = (_e127 | select(524288u, 1048576u, (_e97 < 0f)));
        phi_950_ = _e123;
        phi_946_ = _e125;
    }
    let _e132 = phi_1001_;
    let _e134 = phi_994_;
    let _e136 = phi_955_;
    let _e138 = phi_950_;
    let _e140 = phi_946_;
    let _e141 = vec2(_e88);
    let _e142 = select(_e50, _e49, _e141);
    let _e143 = select(_e50, _e46, _e141);
    let _e144 = select(_e50, _e47, _e141);
    let _e145 = select(1f, f32((_e78 & 1023u)), _e88);
    let _e148 = ((_e140 == 0f) || (_e140 == _e138));
    phi_309_ = _e148;
    if !(_e148) {
        phi_309_ = ((_e136 & 469762048u) > 134217728u);
    }
    let _e153 = phi_309_;
    if _e153 {
        let _e155 = (_e140 < (_e138 * 0.5f));
        if _e155 {
            phi_1048_ = _e134[0];
        } else {
            phi_1048_ = _e134[1];
        }
        let _e161 = phi_1048_;
        let _e162 = normalize(_e161);
        let _e165 = acos(clamp(_e162.x, -1f, 1f));
        if (_e162.y >= 0f) {
            phi_1049_ = _e165;
        } else {
            phi_1049_ = -(_e165);
        }
        let _e170 = phi_1049_;
        phi_1105_ = _e170;
        phi_1050_ = select(_e50, _e143, vec2(_e155));
    } else {
        if ((_e136 & 2147483648u) != 0u) {
            phi_1054_ = select(select(_e143, _e144, vec2((_e140 >= 8f))), _e142, vec2((_e140 >= 12f)));
            if (_e140 >= 14f) {
                let _e180 = T4_1;
                phi_1054_ = _e180.xy;
            }
            let _e183 = phi_1054_;
            phi_1106_ = 0f;
            phi_1053_ = _e183;
        } else {
            if (_e145 == _e138) {
                phi_1112_ = 0f;
                phi_1043_ = 0f;
                phi_1033_ = (_e140 / _e145);
            } else {
                let _e186 = (_e144 - _e143);
                let _e188 = (_e142 - _e144);
                let _e189 = (_e188 - _e186);
                let _e191 = ((_e188 * -3f) + (_e50 - _e143));
                let _e199 = normalize(_e134[0]);
                let _e200 = abs(_e132);
                phi_1003_ = 0f;
                phi_1002_ = 9i;
                loop {
                    let _e205 = phi_1003_;
                    let _e207 = phi_1002_;
                    local = _e205;
                    local_1 = _e205;
                    if (_e207 >= 0i) {
                        let _e211 = (_e205 + exp2(f32(_e207)));
                        phi_1174_ = _e205;
                        if (_e211 <= min((_e145 - 1f), _e140)) {
                            phi_1174_ = select(_e205, _e211, (dot(normalize(((((_e191 * _e211) + (_e189 * (_e145 * 2f))) * _e211) + (_e186 * (_e145 * _e145)))), _e199) >= cos(min(((_e211 * -(_e200)) + ((1f + _e140) * _e200)), 3.1415927f))));
                        }
                        let _e226 = phi_1174_;
                        local_2 = _e226;
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        let _e415 = local_2;
                        phi_1003_ = _e415;
                        phi_1002_ = (_e207 - 1i);
                    }
                }
                let _e229 = local;
                let _e232 = local_1;
                let _e233 = (_e140 - _e232);
                let _e236 = acos(clamp(_e199.x, -1f, 1f));
                if (_e199.y >= 0f) {
                    phi_1006_ = _e236;
                } else {
                    phi_1006_ = -(_e236);
                }
                let _e241 = phi_1006_;
                let _e243 = ((_e233 * _e132) + _e241);
                let _e247 = vec2<f32>(sin(_e243), -(cos(_e243)));
                let _e248 = dot(_e247, _e191);
                let _e249 = dot(_e247, _e189);
                let _e250 = dot(_e247, _e186);
                let _e252 = (_e248 * _e250);
                let _e255 = sqrt(max(((_e249 * _e249) - _e252), 0f));
                phi_1009_ = _e255;
                if (_e249 > 0f) {
                    phi_1009_ = -(_e255);
                }
                let _e259 = phi_1009_;
                let _e260 = (_e259 - _e249);
                let _e262 = ((-0.5f * _e260) * _e248);
                if (abs(((_e260 * _e260) + _e262)) < abs((_e252 + _e262))) {
                    phi_1010_ = vec2<f32>(_e260, _e248);
                } else {
                    phi_1010_ = vec2<f32>(_e250, _e260);
                }
                let _e272 = phi_1010_;
                if (_e272.y != 0f) {
                    phi_1011_ = (_e272.x / _e272.y);
                } else {
                    phi_1011_ = 0f;
                }
                let _e278 = phi_1011_;
                let _e281 = select(clamp(_e278, 0f, 1f), 0f, (_e233 == 0f));
                phi_1112_ = _e243;
                phi_1043_ = _e281;
                phi_1033_ = max((_e229 / _e145), _e281);
            }
            let _e284 = phi_1112_;
            let _e286 = phi_1043_;
            let _e288 = phi_1033_;
            let _e291 = (((_e144 - _e143) * _e288) + _e143);
            let _e294 = (((_e142 - _e144) * _e288) + _e144);
            let _e300 = (((_e294 - _e291) * _e288) + _e291);
            let _e304 = (((((((_e50 - _e142) * _e288) + _e142) - _e294) * _e288) + _e294) - _e300);
            phi_1111_ = _e284;
            if (_e288 != _e286) {
                let _e308 = normalize(_e304);
                let _e311 = acos(clamp(_e308.x, -1f, 1f));
                if (_e308.y >= 0f) {
                    phi_1044_ = _e311;
                } else {
                    phi_1044_ = -(_e311);
                }
                let _e316 = phi_1044_;
                phi_1111_ = _e316;
            }
            let _e318 = phi_1111_;
            phi_1106_ = _e318;
            phi_1053_ = ((_e304 * _e288) + _e300);
        }
        let _e320 = phi_1106_;
        let _e322 = phi_1053_;
        phi_1105_ = _e320;
        phi_1050_ = _e322;
    }
    let _e324 = phi_1105_;
    let _e326 = phi_1050_;
    let _e327 = bitcast<vec2<u32>>(_e326);
    let _e333 = vec4<u32>(_e327.x, vec4<u32>().y, vec4<u32>().z, vec4<u32>().w);
    let _e339 = vec4<u32>(_e333.x, _e327.y, _e333.z, _e333.w);
    let _e340 = (_e136 & 469762048u);
    if (_e340 == 67108864u) {
        phi_1173_ = vec4<u32>(_e339.x, _e339.y, ((u32(_e138) << bitcast<u32>(16i)) | u32(_e140)), _e339.w);
    } else {
        phi_1136_ = 0u;
        if (_e340 > 134217728u) {
            let _e363 = (dot(_e134[0], _e134[0]) * dot(_e134[1], _e134[1]));
            if (_e363 == 0f) {
                phi_1135_ = 1f;
            } else {
                phi_1135_ = clamp((dot(_e134[0], _e134[1]) * inverseSqrt(_e363)), -1f, 1f);
            }
            let _e369 = phi_1135_;
            phi_1136_ = u32(round((sqrt(((1f + clamp(_e369, -1f, 1f)) * 0.5f)) * 65535f)));
        }
        let _e378 = phi_1136_;
        phi_1173_ = vec4<u32>(_e339.x, _e339.y, (((bitcast<u32>(i32(round((_e324 * 10430.378f)))) & 65535u) << bitcast<u32>(16i)) | _e378), _e339.w);
    }
    let _e388 = phi_1173_;
    oh = vec4<u32>(_e388.x, _e388.y, _e388.z, _e136);
    return;
}

@fragment
fn main(@location(0) B6_: vec4<f32>, @location(1) C6_: vec4<f32>, @location(2) S4_: vec4<f32>, @location(4) @interpolate(flat, either) G7_: u32, @location(3) T4_: vec3<f32>) -> @location(0) vec4<u32> {
    B6_1 = B6_;
    C6_1 = C6_;
    S4_1 = S4_;
    G7_1 = G7_;
    T4_1 = T4_;
    main_1();
    let _e11 = oh;
    return _e11;
}
