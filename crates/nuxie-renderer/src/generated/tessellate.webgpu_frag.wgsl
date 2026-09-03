struct DC {
    hc: f32,
    rd: f32,
    kf: f32,
    lf: f32,
    p6_: u32,
    Mg: u32,
    Ve: u32,
    We: u32,
    U7_: vec4<i32>,
    Ig: vec2<f32>,
    sd: vec2<f32>,
    c2_: u32,
    Ng: f32,
    d6_: u32,
    R2_: f32,
    td: f32,
    Qe: u32,
    B3_: f32,
    C3_: f32,
    ud: f32,
    Fg: u32,
}

var<private> A6_1: vec4<f32>;
var<private> B6_1: vec4<f32>;
var<private> N4_1: vec4<f32>;
var<private> I7_1: u32;
var<private> O4_1: vec3<f32>;
var<private> Qg: vec4<u32>;
@group(0) @binding(0)
var<uniform> m: DC;

fn main_1() {
    var phi_837_: vec2<f32>;
    var phi_840_: vec2<f32>;
    var phi_845_: u32;
    var phi_852_: u32;
    var phi_250_: bool;
    var phi_863_: f32;
    var phi_858_: f32;
    var phi_860_: f32;
    var phi_856_: f32;
    var phi_851_: u32;
    var phi_910_: f32;
    var phi_903_: mat2x2<f32>;
    var phi_864_: u32;
    var phi_859_: f32;
    var phi_855_: f32;
    var phi_278_: bool;
    var phi_957_: vec2<f32>;
    var phi_958_: f32;
    var phi_963_: vec2<f32>;
    var phi_912_: f32;
    var phi_911_: i32;
    var phi_1059_: f32;
    var local: f32;
    var local_1: f32;
    var phi_915_: f32;
    var phi_918_: f32;
    var phi_919_: vec2<f32>;
    var phi_920_: f32;
    var phi_1021_: f32;
    var phi_952_: f32;
    var phi_942_: f32;
    var phi_953_: f32;
    var phi_1020_: f32;
    var phi_1015_: f32;
    var phi_962_: vec2<f32>;
    var phi_1014_: f32;
    var phi_959_: vec2<f32>;
    var phi_1058_: vec4<u32>;
    var local_2: f32;

    let _e43 = A6_1;
    let _e44 = _e43.xy;
    let _e45 = _e43.zw;
    let _e46 = B6_1;
    let _e47 = _e46.xy;
    let _e48 = _e46.zw;
    if any((_e44 != _e45)) {
        phi_837_ = _e45;
    } else {
        phi_837_ = select(_e48, _e47, vec2(any((_e45 != _e47))));
    }
    let _e56 = phi_837_;
    if any((_e48 != _e47)) {
        phi_840_ = _e47;
    } else {
        phi_840_ = select(_e44, _e45, vec2(any((_e47 != _e45))));
    }
    let _e65 = phi_840_;
    let _e66 = (_e48 - _e65);
    let _e69 = N4_1[0u];
    let _e71 = max(floor(_e69), 0f);
    let _e73 = N4_1[1u];
    let _e75 = N4_1[2u];
    let _e76 = u32(_e75);
    let _e81 = f32((_e76 >> bitcast<u32>(10i)));
    let _e83 = N4_1[3u];
    let _e84 = I7_1;
    let _e85 = (_e73 - _e81);
    let _e86 = (_e71 <= _e85);
    if _e86 {
        phi_910_ = _e83;
        phi_903_ = mat2x2<f32>((_e56 - _e44), _e66);
        phi_864_ = (_e84 & 3825205247u);
        phi_859_ = _e85;
        phi_855_ = _e71;
    } else {
        let _e88 = O4_1;
        let _e93 = (_e71 - _e85);
        let _e95 = O4_1[2u];
        let _e96 = (_e84 & 469762048u);
        if (_e96 > 134217728u) {
            phi_845_ = _e84;
            if (_e93 < 2.5f) {
                phi_845_ = (_e84 | 4194304u);
            }
            let _e101 = phi_845_;
            phi_852_ = _e101;
            if ((_e93 > 1.5f) && (_e93 < 3.5f)) {
                phi_852_ = (_e101 | 2097152u);
            }
            let _e107 = phi_852_;
            phi_860_ = _e81;
            phi_856_ = _e93;
            phi_851_ = _e107;
        } else {
            let _e109 = ((_e84 & 33554432u) != 0u);
            phi_250_ = _e109;
            if !(_e109) {
                phi_250_ = (_e96 == 67108864u);
            }
            let _e113 = phi_250_;
            phi_863_ = _e81;
            phi_858_ = _e93;
            if _e113 {
                phi_863_ = (_e81 - 2f);
                phi_858_ = (_e93 - 1f);
            }
            let _e117 = phi_863_;
            let _e119 = phi_858_;
            phi_860_ = _e117;
            phi_856_ = _e119;
            phi_851_ = _e84;
        }
        let _e121 = phi_860_;
        let _e123 = phi_856_;
        let _e125 = phi_851_;
        phi_910_ = _e95;
        phi_903_ = mat2x2<f32>(_e66, vec2<f32>(_e88.x, _e88.y));
        phi_864_ = (_e125 | select(524288u, 1048576u, (_e95 < 0f)));
        phi_859_ = _e121;
        phi_855_ = _e123;
    }
    let _e130 = phi_910_;
    let _e132 = phi_903_;
    let _e134 = phi_864_;
    let _e136 = phi_859_;
    let _e138 = phi_855_;
    let _e139 = vec2(_e86);
    let _e140 = select(_e48, _e47, _e139);
    let _e141 = select(_e48, _e44, _e139);
    let _e142 = select(_e48, _e45, _e139);
    let _e143 = select(1f, f32((_e76 & 1023u)), _e86);
    let _e146 = ((_e138 == 0f) || (_e138 == _e136));
    phi_278_ = _e146;
    if !(_e146) {
        phi_278_ = ((_e134 & 469762048u) > 134217728u);
    }
    let _e151 = phi_278_;
    if _e151 {
        let _e153 = (_e138 < (_e136 * 0.5f));
        if _e153 {
            phi_957_ = _e132[0];
        } else {
            phi_957_ = _e132[1];
        }
        let _e159 = phi_957_;
        let _e160 = normalize(_e159);
        let _e163 = acos(clamp(_e160.x, -1f, 1f));
        if (_e160.y >= 0f) {
            phi_958_ = _e163;
        } else {
            phi_958_ = -(_e163);
        }
        let _e168 = phi_958_;
        phi_1014_ = _e168;
        phi_959_ = select(_e48, _e141, vec2(_e153));
    } else {
        if ((_e134 & 2147483648u) != 0u) {
            phi_963_ = select(select(_e141, _e142, vec2((_e138 >= 8f))), _e140, vec2((_e138 >= 12f)));
            if (_e138 >= 14f) {
                let _e178 = O4_1;
                phi_963_ = _e178.xy;
            }
            let _e181 = phi_963_;
            phi_1015_ = 0f;
            phi_962_ = _e181;
        } else {
            if (_e143 == _e136) {
                phi_1021_ = 0f;
                phi_952_ = 0f;
                phi_942_ = (_e138 / _e143);
            } else {
                let _e184 = (_e142 - _e141);
                let _e186 = (_e140 - _e142);
                let _e187 = (_e186 - _e184);
                let _e189 = ((_e186 * -3f) + (_e48 - _e141));
                let _e197 = normalize(_e132[0]);
                let _e198 = abs(_e130);
                phi_912_ = 0f;
                phi_911_ = 9i;
                loop {
                    let _e203 = phi_912_;
                    let _e205 = phi_911_;
                    local = _e203;
                    local_1 = _e203;
                    if (_e205 >= 0i) {
                        let _e209 = (_e203 + exp2(f32(_e205)));
                        phi_1059_ = _e203;
                        if (_e209 <= min((_e143 - 1f), _e138)) {
                            phi_1059_ = select(_e203, _e209, (dot(normalize(((((_e189 * _e209) + (_e187 * (_e143 * 2f))) * _e209) + (_e184 * (_e143 * _e143)))), _e197) >= cos(min(((_e209 * -(_e198)) + ((1f + _e138) * _e198)), 3.1415927f))));
                        }
                        let _e224 = phi_1059_;
                        local_2 = _e224;
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        let _e388 = local_2;
                        phi_912_ = _e388;
                        phi_911_ = (_e205 - 1i);
                    }
                }
                let _e227 = local;
                let _e230 = local_1;
                let _e231 = (_e138 - _e230);
                let _e234 = acos(clamp(_e197.x, -1f, 1f));
                if (_e197.y >= 0f) {
                    phi_915_ = _e234;
                } else {
                    phi_915_ = -(_e234);
                }
                let _e239 = phi_915_;
                let _e241 = ((_e231 * _e130) + _e239);
                let _e245 = vec2<f32>(sin(_e241), -(cos(_e241)));
                let _e246 = dot(_e245, _e189);
                let _e247 = dot(_e245, _e187);
                let _e248 = dot(_e245, _e184);
                let _e250 = (_e246 * _e248);
                let _e253 = sqrt(max(((_e247 * _e247) - _e250), 0f));
                phi_918_ = _e253;
                if (_e247 > 0f) {
                    phi_918_ = -(_e253);
                }
                let _e257 = phi_918_;
                let _e258 = (_e257 - _e247);
                let _e260 = ((-0.5f * _e258) * _e246);
                if (abs(((_e258 * _e258) + _e260)) < abs((_e250 + _e260))) {
                    phi_919_ = vec2<f32>(_e258, _e246);
                } else {
                    phi_919_ = vec2<f32>(_e248, _e258);
                }
                let _e270 = phi_919_;
                if (_e270.y != 0f) {
                    phi_920_ = (_e270.x / _e270.y);
                } else {
                    phi_920_ = 0f;
                }
                let _e276 = phi_920_;
                let _e279 = select(clamp(_e276, 0f, 1f), 0f, (_e231 == 0f));
                phi_1021_ = _e241;
                phi_952_ = _e279;
                phi_942_ = max((_e227 / _e143), _e279);
            }
            let _e282 = phi_1021_;
            let _e284 = phi_952_;
            let _e286 = phi_942_;
            let _e289 = (((_e142 - _e141) * _e286) + _e141);
            let _e292 = (((_e140 - _e142) * _e286) + _e142);
            let _e298 = (((_e292 - _e289) * _e286) + _e289);
            let _e302 = (((((((_e48 - _e140) * _e286) + _e140) - _e292) * _e286) + _e292) - _e298);
            phi_1020_ = _e282;
            if (_e286 != _e284) {
                let _e306 = normalize(_e302);
                let _e309 = acos(clamp(_e306.x, -1f, 1f));
                if (_e306.y >= 0f) {
                    phi_953_ = _e309;
                } else {
                    phi_953_ = -(_e309);
                }
                let _e314 = phi_953_;
                phi_1020_ = _e314;
            }
            let _e316 = phi_1020_;
            phi_1015_ = _e316;
            phi_962_ = ((_e302 * _e286) + _e298);
        }
        let _e318 = phi_1015_;
        let _e320 = phi_962_;
        phi_1014_ = _e318;
        phi_959_ = _e320;
    }
    let _e322 = phi_1014_;
    let _e324 = phi_959_;
    let _e325 = bitcast<vec2<u32>>(_e324);
    let _e331 = vec4<u32>(_e325.x, vec4<u32>().y, vec4<u32>().z, vec4<u32>().w);
    let _e337 = vec4<u32>(_e331.x, _e325.y, _e331.z, _e331.w);
    if ((_e134 & 469762048u) == 67108864u) {
        phi_1058_ = vec4<u32>(_e337.x, _e337.y, ((u32(_e136) << bitcast<u32>(16i)) | u32(_e138)), _e337.w);
    } else {
        phi_1058_ = vec4<u32>(_e337.x, _e337.y, bitcast<u32>((_e322 - (floor((_e322 / 6.2831855f)) * 6.2831855f))), _e337.w);
    }
    let _e361 = phi_1058_;
    Qg = vec4<u32>(_e361.x, _e361.y, _e361.z, _e134);
    return;
}

@fragment
fn main(@location(0) A6_: vec4<f32>, @location(1) B6_: vec4<f32>, @location(2) N4_: vec4<f32>, @location(4) @interpolate(flat, either) I7_: u32, @location(3) O4_: vec3<f32>) -> @location(0) vec4<u32> {
    A6_1 = A6_;
    B6_1 = B6_;
    N4_1 = N4_;
    I7_1 = I7_;
    O4_1 = O4_;
    main_1();
    let _e11 = Qg;
    return _e11;
}
