struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

var<private> L6_1: vec4<f32>;
var<private> M6_1: vec4<f32>;
var<private> a5_1: vec4<f32>;
var<private> R7_1: u32;
var<private> c5_1: vec3<f32>;
var<private> Uh: vec4<u32>;
@group(0) @binding(0)
var<uniform> j: UB;

fn main_1() {
    var phi_957_: vec2<f32>;
    var phi_960_: vec2<f32>;
    var phi_257_: bool;
    var phi_965_: f32;
    var phi_964_: f32;
    var phi_275_: bool;
    var phi_972_: u32;
    var phi_982_: u32;
    var phi_316_: bool;
    var phi_323_: bool;
    var phi_994_: u32;
    var phi_993_: u32;
    var phi_989_: f32;
    var phi_1051_: f32;
    var phi_1041_: mat2x2<f32>;
    var phi_992_: u32;
    var phi_990_: f32;
    var phi_988_: f32;
    var phi_1104_: vec2<f32>;
    var phi_1105_: f32;
    var phi_1110_: vec2<f32>;
    var phi_1059_: f32;
    var phi_1058_: i32;
    var phi_1232_: f32;
    var local: f32;
    var local_1: f32;
    var phi_1062_: f32;
    var phi_1065_: f32;
    var phi_1066_: vec2<f32>;
    var phi_1067_: f32;
    var phi_1170_: f32;
    var phi_1099_: f32;
    var phi_1089_: f32;
    var phi_1100_: f32;
    var phi_1169_: f32;
    var phi_1165_: f32;
    var phi_1109_: vec2<f32>;
    var phi_1164_: f32;
    var phi_1106_: vec2<f32>;
    var phi_1193_: f32;
    var phi_1194_: u32;
    var phi_1231_: vec4<u32>;
    var local_2: f32;

    let _e43 = L6_1;
    let _e44 = _e43.xy;
    let _e45 = _e43.zw;
    let _e46 = M6_1;
    let _e47 = _e46.xy;
    let _e48 = _e46.zw;
    if any((_e44 != _e45)) {
        phi_957_ = _e45;
    } else {
        phi_957_ = select(_e48, _e47, vec2(any((_e45 != _e47))));
    }
    let _e56 = phi_957_;
    if any((_e48 != _e47)) {
        phi_960_ = _e47;
    } else {
        phi_960_ = select(_e44, _e45, vec2(any((_e47 != _e45))));
    }
    let _e65 = phi_960_;
    let _e66 = (_e48 - _e65);
    let _e69 = a5_1[0u];
    let _e71 = max(floor(_e69), 0f);
    let _e73 = a5_1[1u];
    let _e75 = a5_1[2u];
    let _e76 = u32(_e75);
    let _e81 = f32((_e76 >> bitcast<u32>(10i)));
    let _e83 = a5_1[3u];
    let _e84 = R7_1;
    let _e85 = (_e73 - _e81);
    let _e86 = (_e71 <= _e85);
    if _e86 {
        phi_1051_ = _e83;
        phi_1041_ = mat2x2<f32>((_e56 - _e44), _e66);
        phi_992_ = (_e84 & 3825205247u);
        phi_990_ = _e85;
        phi_988_ = _e71;
    } else {
        let _e88 = c5_1;
        let _e93 = (_e71 - _e85);
        let _e95 = c5_1[2u];
        let _e97 = ((_e84 & 33554432u) != 0u);
        phi_257_ = _e97;
        if !(_e97) {
            phi_257_ = ((_e84 & 469762048u) == 67108864u);
        }
        let _e102 = phi_257_;
        phi_965_ = _e81;
        phi_964_ = _e93;
        if _e102 {
            phi_965_ = (_e81 - 2f);
            phi_964_ = (_e93 - 1f);
        }
        let _e106 = phi_965_;
        let _e108 = phi_964_;
        phi_275_ = _e97;
        if _e97 {
            phi_275_ = ((_e108 == 0f) || (_e108 == _e106));
        }
        let _e113 = phi_275_;
        if _e113 {
            phi_972_ = (_e84 & 3825205247u);
        } else {
            phi_972_ = (_e84 | select(524288u, 1048576u, (_e95 < 0f)));
        }
        let _e119 = phi_972_;
        phi_993_ = _e119;
        phi_989_ = _e108;
        if ((_e119 & 469762048u) > 134217728u) {
            let _e122 = (_e106 * 0.5f);
            let _e123 = (_e108 < _e122);
            phi_982_ = _e119;
            if _e123 {
                phi_982_ = (_e119 | 4194304u);
            }
            let _e126 = phi_982_;
            let _e127 = (_e106 > 3f);
            phi_316_ = _e127;
            if _e127 {
                phi_316_ = (_e108 > (_e122 - 1f));
            }
            let _e131 = phi_316_;
            phi_323_ = _e131;
            if _e131 {
                phi_323_ = (_e108 < (_e122 + 1f));
            }
            let _e135 = phi_323_;
            phi_994_ = _e126;
            if _e135 {
                phi_994_ = (_e126 | 2097152u);
            }
            let _e138 = phi_994_;
            phi_993_ = _e138;
            phi_989_ = select(_e106, 0f, _e123);
        }
        let _e141 = phi_993_;
        let _e143 = phi_989_;
        phi_1051_ = _e95;
        phi_1041_ = mat2x2<f32>(_e66, vec2<f32>(_e88.x, _e88.y));
        phi_992_ = _e141;
        phi_990_ = _e106;
        phi_988_ = _e143;
    }
    let _e145 = phi_1051_;
    let _e147 = phi_1041_;
    let _e149 = phi_992_;
    let _e151 = phi_990_;
    let _e153 = phi_988_;
    let _e154 = vec2(_e86);
    let _e155 = select(_e48, _e47, _e154);
    let _e156 = select(_e48, _e44, _e154);
    let _e157 = select(_e48, _e45, _e154);
    let _e158 = select(1f, f32((_e76 & 1023u)), _e86);
    if ((_e153 == 0f) || (_e153 == _e151)) {
        let _e163 = (_e153 < (_e151 * 0.5f));
        if _e163 {
            phi_1104_ = _e147[0];
        } else {
            phi_1104_ = _e147[1];
        }
        let _e169 = phi_1104_;
        let _e170 = normalize(_e169);
        let _e173 = acos(clamp(_e170.x, -1f, 1f));
        if (_e170.y >= 0f) {
            phi_1105_ = _e173;
        } else {
            phi_1105_ = -(_e173);
        }
        let _e178 = phi_1105_;
        phi_1164_ = _e178;
        phi_1106_ = select(_e48, _e156, vec2(_e163));
    } else {
        if ((_e149 & 2147483648u) != 0u) {
            phi_1110_ = select(select(_e156, _e157, vec2((_e153 >= 8f))), _e155, vec2((_e153 >= 12f)));
            if (_e153 >= 14f) {
                let _e188 = c5_1;
                phi_1110_ = _e188.xy;
            }
            let _e191 = phi_1110_;
            phi_1165_ = 0f;
            phi_1109_ = _e191;
        } else {
            if (_e158 == _e151) {
                phi_1170_ = 0f;
                phi_1099_ = 0f;
                phi_1089_ = (_e153 / _e158);
            } else {
                let _e194 = (_e157 - _e156);
                let _e196 = (_e155 - _e157);
                let _e197 = (_e196 - _e194);
                let _e199 = ((_e196 * -3f) + (_e48 - _e156));
                let _e207 = normalize(_e147[0]);
                let _e208 = abs(_e145);
                phi_1059_ = 0f;
                phi_1058_ = 9i;
                loop {
                    let _e213 = phi_1059_;
                    let _e215 = phi_1058_;
                    local = _e213;
                    local_1 = _e213;
                    if (_e215 >= 0i) {
                        let _e219 = (_e213 + exp2(f32(_e215)));
                        phi_1232_ = _e213;
                        if (_e219 <= min((_e158 - 1f), _e153)) {
                            phi_1232_ = select(_e213, _e219, (dot(normalize(((((_e199 * _e219) + (_e197 * (_e158 * 2f))) * _e219) + (_e194 * (_e158 * _e158)))), _e207) >= cos(min(((_e219 * -(_e208)) + ((1f + _e153) * _e208)), 3.1415927f))));
                        }
                        let _e234 = phi_1232_;
                        local_2 = _e234;
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        let _e425 = local_2;
                        phi_1059_ = _e425;
                        phi_1058_ = (_e215 - 1i);
                    }
                }
                let _e237 = local;
                let _e240 = local_1;
                let _e241 = (_e153 - _e240);
                let _e244 = acos(clamp(_e207.x, -1f, 1f));
                if (_e207.y >= 0f) {
                    phi_1062_ = _e244;
                } else {
                    phi_1062_ = -(_e244);
                }
                let _e249 = phi_1062_;
                let _e251 = ((_e241 * _e145) + _e249);
                let _e255 = vec2<f32>(sin(_e251), -(cos(_e251)));
                let _e256 = dot(_e255, _e199);
                let _e257 = dot(_e255, _e197);
                let _e258 = dot(_e255, _e194);
                let _e260 = (_e256 * _e258);
                let _e263 = sqrt(max(((_e257 * _e257) - _e260), 0f));
                phi_1065_ = _e263;
                if (_e257 > 0f) {
                    phi_1065_ = -(_e263);
                }
                let _e267 = phi_1065_;
                let _e268 = (_e267 - _e257);
                let _e270 = ((-0.5f * _e268) * _e256);
                if (abs(((_e268 * _e268) + _e270)) < abs((_e260 + _e270))) {
                    phi_1066_ = vec2<f32>(_e268, _e256);
                } else {
                    phi_1066_ = vec2<f32>(_e258, _e268);
                }
                let _e280 = phi_1066_;
                if (_e280.y != 0f) {
                    phi_1067_ = (_e280.x / _e280.y);
                } else {
                    phi_1067_ = 0f;
                }
                let _e286 = phi_1067_;
                let _e289 = select(clamp(_e286, 0f, 1f), 0f, (_e241 == 0f));
                phi_1170_ = _e251;
                phi_1099_ = _e289;
                phi_1089_ = max((_e237 / _e158), _e289);
            }
            let _e292 = phi_1170_;
            let _e294 = phi_1099_;
            let _e296 = phi_1089_;
            let _e299 = (((_e157 - _e156) * _e296) + _e156);
            let _e302 = (((_e155 - _e157) * _e296) + _e157);
            let _e308 = (((_e302 - _e299) * _e296) + _e299);
            let _e312 = (((((((_e48 - _e155) * _e296) + _e155) - _e302) * _e296) + _e302) - _e308);
            phi_1169_ = _e292;
            if (_e296 != _e294) {
                let _e316 = normalize(_e312);
                let _e319 = acos(clamp(_e316.x, -1f, 1f));
                if (_e316.y >= 0f) {
                    phi_1100_ = _e319;
                } else {
                    phi_1100_ = -(_e319);
                }
                let _e324 = phi_1100_;
                phi_1169_ = _e324;
            }
            let _e326 = phi_1169_;
            phi_1165_ = _e326;
            phi_1109_ = ((_e312 * _e296) + _e308);
        }
        let _e328 = phi_1165_;
        let _e330 = phi_1109_;
        phi_1164_ = _e328;
        phi_1106_ = _e330;
    }
    let _e332 = phi_1164_;
    let _e334 = phi_1106_;
    let _e335 = bitcast<vec2<u32>>(_e334);
    let _e341 = vec4<u32>(_e335.x, vec4<u32>().y, vec4<u32>().z, vec4<u32>().w);
    let _e347 = vec4<u32>(_e341.x, _e335.y, _e341.z, _e341.w);
    let _e348 = (_e149 & 469762048u);
    if (_e348 == 67108864u) {
        phi_1231_ = vec4<u32>(_e347.x, _e347.y, ((u32(_e151) << bitcast<u32>(16i)) | u32(_e153)), _e347.w);
    } else {
        phi_1194_ = 0u;
        if (_e348 > 134217728u) {
            let _e371 = (dot(_e147[0], _e147[0]) * dot(_e147[1], _e147[1]));
            if (_e371 == 0f) {
                phi_1193_ = 1f;
            } else {
                phi_1193_ = clamp((dot(_e147[0], _e147[1]) * inverseSqrt(_e371)), -1f, 1f);
            }
            let _e377 = phi_1193_;
            phi_1194_ = u32(round((sqrt(((1f + clamp(_e377, -1f, 1f)) * 0.5f)) * 65535f)));
        }
        let _e386 = phi_1194_;
        phi_1231_ = vec4<u32>(_e347.x, _e347.y, (((bitcast<u32>(i32(round((_e332 * 10430.378f)))) & 65535u) << bitcast<u32>(16i)) | _e386), _e347.w);
    }
    let _e396 = phi_1231_;
    Uh = vec4<u32>(_e396.x, _e396.y, _e396.z, _e149);
    return;
}

@fragment
fn main(@location(0) L6_: vec4<f32>, @location(1) M6_: vec4<f32>, @location(2) a5_: vec4<f32>, @location(4) @interpolate(flat, either) R7_: u32, @location(3) c5_: vec3<f32>) -> @location(0) vec4<u32> {
    L6_1 = L6_;
    M6_1 = M6_;
    a5_1 = a5_;
    R7_1 = R7_;
    c5_1 = c5_;
    main_1();
    let _e11 = Uh;
    return _e11;
}
