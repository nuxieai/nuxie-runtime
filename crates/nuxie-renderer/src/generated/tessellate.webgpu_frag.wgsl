struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

var<private> x6_1: vec4<f32>;
var<private> y6_1: vec4<f32>;
var<private> L4_1: vec4<f32>;
var<private> E7_1: u32;
var<private> C5_1: vec3<f32>;
var<private> Fg: vec4<u32>;
@group(0) @binding(0)
var<uniform> m: CC;

fn main_1() {
    var phi_818_: vec2<f32>;
    var phi_821_: vec2<f32>;
    var phi_826_: u32;
    var phi_833_: u32;
    var phi_250_: bool;
    var phi_844_: f32;
    var phi_839_: f32;
    var phi_841_: f32;
    var phi_837_: f32;
    var phi_832_: u32;
    var phi_891_: f32;
    var phi_884_: mat2x2<f32>;
    var phi_845_: u32;
    var phi_840_: f32;
    var phi_836_: f32;
    var phi_278_: bool;
    var phi_935_: vec2<f32>;
    var phi_936_: f32;
    var phi_893_: f32;
    var phi_892_: i32;
    var phi_1021_: f32;
    var local: f32;
    var local_1: f32;
    var phi_896_: f32;
    var phi_899_: f32;
    var phi_900_: vec2<f32>;
    var phi_901_: f32;
    var phi_987_: f32;
    var phi_933_: f32;
    var phi_923_: f32;
    var phi_934_: f32;
    var phi_986_: f32;
    var phi_984_: f32;
    var phi_940_: vec2<f32>;
    var phi_983_: f32;
    var phi_937_: vec2<f32>;
    var phi_1020_: vec4<u32>;
    var local_2: f32;

    let _e40 = x6_1;
    let _e41 = _e40.xy;
    let _e42 = _e40.zw;
    let _e43 = y6_1;
    let _e44 = _e43.xy;
    let _e45 = _e43.zw;
    if any((_e41 != _e42)) {
        phi_818_ = _e42;
    } else {
        phi_818_ = select(_e45, _e44, vec2(any((_e42 != _e44))));
    }
    let _e53 = phi_818_;
    if any((_e45 != _e44)) {
        phi_821_ = _e44;
    } else {
        phi_821_ = select(_e41, _e42, vec2(any((_e44 != _e42))));
    }
    let _e62 = phi_821_;
    let _e63 = (_e45 - _e62);
    let _e66 = L4_1[0u];
    let _e68 = max(floor(_e66), 0f);
    let _e70 = L4_1[1u];
    let _e72 = L4_1[2u];
    let _e73 = u32(_e72);
    let _e78 = f32((_e73 >> bitcast<u32>(10i)));
    let _e80 = L4_1[3u];
    let _e81 = E7_1;
    let _e82 = (_e70 - _e78);
    let _e83 = (_e68 <= _e82);
    if _e83 {
        phi_891_ = _e80;
        phi_884_ = mat2x2<f32>((_e53 - _e41), _e63);
        phi_845_ = (_e81 & 3825205247u);
        phi_840_ = _e82;
        phi_836_ = _e68;
    } else {
        let _e85 = C5_1;
        let _e90 = (_e68 - _e82);
        let _e92 = C5_1[2u];
        let _e93 = (_e81 & 469762048u);
        if (_e93 > 134217728u) {
            phi_826_ = _e81;
            if (_e90 < 2.5f) {
                phi_826_ = (_e81 | 4194304u);
            }
            let _e98 = phi_826_;
            phi_833_ = _e98;
            if ((_e90 > 1.5f) && (_e90 < 3.5f)) {
                phi_833_ = (_e98 | 2097152u);
            }
            let _e104 = phi_833_;
            phi_841_ = _e78;
            phi_837_ = _e90;
            phi_832_ = _e104;
        } else {
            let _e106 = ((_e81 & 33554432u) != 0u);
            phi_250_ = _e106;
            if !(_e106) {
                phi_250_ = (_e93 == 67108864u);
            }
            let _e110 = phi_250_;
            phi_844_ = _e78;
            phi_839_ = _e90;
            if _e110 {
                phi_844_ = (_e78 - 2f);
                phi_839_ = (_e90 - 1f);
            }
            let _e114 = phi_844_;
            let _e116 = phi_839_;
            phi_841_ = _e114;
            phi_837_ = _e116;
            phi_832_ = _e81;
        }
        let _e118 = phi_841_;
        let _e120 = phi_837_;
        let _e122 = phi_832_;
        phi_891_ = _e92;
        phi_884_ = mat2x2<f32>(_e63, vec2<f32>(_e85.x, _e85.y));
        phi_845_ = (_e122 | select(524288u, 1048576u, (_e92 < 0f)));
        phi_840_ = _e118;
        phi_836_ = _e120;
    }
    let _e127 = phi_891_;
    let _e129 = phi_884_;
    let _e131 = phi_845_;
    let _e133 = phi_840_;
    let _e135 = phi_836_;
    let _e136 = vec2(_e83);
    let _e137 = select(_e45, _e44, _e136);
    let _e138 = select(_e45, _e41, _e136);
    let _e139 = select(_e45, _e42, _e136);
    let _e140 = select(1f, f32((_e73 & 1023u)), _e83);
    let _e143 = ((_e135 == 0f) || (_e135 == _e133));
    phi_278_ = _e143;
    if !(_e143) {
        phi_278_ = ((_e131 & 469762048u) > 134217728u);
    }
    let _e148 = phi_278_;
    if _e148 {
        let _e150 = (_e135 < (_e133 * 0.5f));
        if _e150 {
            phi_935_ = _e129[0];
        } else {
            phi_935_ = _e129[1];
        }
        let _e156 = phi_935_;
        let _e157 = normalize(_e156);
        let _e160 = acos(clamp(_e157.x, -1f, 1f));
        if (_e157.y >= 0f) {
            phi_936_ = _e160;
        } else {
            phi_936_ = -(_e160);
        }
        let _e165 = phi_936_;
        phi_983_ = _e165;
        phi_937_ = select(_e45, _e138, vec2(_e150));
    } else {
        if ((_e131 & 2147483648u) != 0u) {
            phi_984_ = 0f;
            phi_940_ = _e139;
        } else {
            if (_e140 == _e133) {
                phi_987_ = 0f;
                phi_933_ = 0f;
                phi_923_ = (_e135 / _e140);
            } else {
                let _e170 = (_e139 - _e138);
                let _e172 = (_e137 - _e139);
                let _e173 = (_e172 - _e170);
                let _e175 = ((_e172 * -3f) + (_e45 - _e138));
                let _e183 = normalize(_e129[0]);
                let _e184 = abs(_e127);
                phi_893_ = 0f;
                phi_892_ = 9i;
                loop {
                    let _e189 = phi_893_;
                    let _e191 = phi_892_;
                    local = _e189;
                    local_1 = _e189;
                    if (_e191 >= 0i) {
                        let _e195 = (_e189 + exp2(f32(_e191)));
                        phi_1021_ = _e189;
                        if (_e195 <= min((_e140 - 1f), _e135)) {
                            phi_1021_ = select(_e189, _e195, (dot(normalize(((((_e175 * _e195) + (_e173 * (_e140 * 2f))) * _e195) + (_e170 * (_e140 * _e140)))), _e183) >= cos(min(((_e195 * -(_e184)) + ((1f + _e135) * _e184)), 3.1415927f))));
                        }
                        let _e210 = phi_1021_;
                        local_2 = _e210;
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        let _e373 = local_2;
                        phi_893_ = _e373;
                        phi_892_ = (_e191 - 1i);
                    }
                }
                let _e213 = local;
                let _e216 = local_1;
                let _e217 = (_e135 - _e216);
                let _e220 = acos(clamp(_e183.x, -1f, 1f));
                if (_e183.y >= 0f) {
                    phi_896_ = _e220;
                } else {
                    phi_896_ = -(_e220);
                }
                let _e225 = phi_896_;
                let _e227 = ((_e217 * _e127) + _e225);
                let _e231 = vec2<f32>(sin(_e227), -(cos(_e227)));
                let _e232 = dot(_e231, _e175);
                let _e233 = dot(_e231, _e173);
                let _e234 = dot(_e231, _e170);
                let _e236 = (_e232 * _e234);
                let _e239 = sqrt(max(((_e233 * _e233) - _e236), 0f));
                phi_899_ = _e239;
                if (_e233 > 0f) {
                    phi_899_ = -(_e239);
                }
                let _e243 = phi_899_;
                let _e244 = (_e243 - _e233);
                let _e246 = ((-0.5f * _e244) * _e232);
                if (abs(((_e244 * _e244) + _e246)) < abs((_e236 + _e246))) {
                    phi_900_ = vec2<f32>(_e244, _e232);
                } else {
                    phi_900_ = vec2<f32>(_e234, _e244);
                }
                let _e256 = phi_900_;
                if (_e256.y != 0f) {
                    phi_901_ = (_e256.x / _e256.y);
                } else {
                    phi_901_ = 0f;
                }
                let _e262 = phi_901_;
                let _e265 = select(clamp(_e262, 0f, 1f), 0f, (_e217 == 0f));
                phi_987_ = _e227;
                phi_933_ = _e265;
                phi_923_ = max((_e213 / _e140), _e265);
            }
            let _e268 = phi_987_;
            let _e270 = phi_933_;
            let _e272 = phi_923_;
            let _e275 = (((_e139 - _e138) * _e272) + _e138);
            let _e278 = (((_e137 - _e139) * _e272) + _e139);
            let _e284 = (((_e278 - _e275) * _e272) + _e275);
            let _e288 = (((((((_e45 - _e137) * _e272) + _e137) - _e278) * _e272) + _e278) - _e284);
            phi_986_ = _e268;
            if (_e272 != _e270) {
                let _e292 = normalize(_e288);
                let _e295 = acos(clamp(_e292.x, -1f, 1f));
                if (_e292.y >= 0f) {
                    phi_934_ = _e295;
                } else {
                    phi_934_ = -(_e295);
                }
                let _e300 = phi_934_;
                phi_986_ = _e300;
            }
            let _e302 = phi_986_;
            phi_984_ = _e302;
            phi_940_ = ((_e288 * _e272) + _e284);
        }
        let _e304 = phi_984_;
        let _e306 = phi_940_;
        phi_983_ = _e304;
        phi_937_ = _e306;
    }
    let _e308 = phi_983_;
    let _e310 = phi_937_;
    let _e311 = bitcast<vec2<u32>>(_e310);
    let _e317 = vec4<u32>(_e311.x, vec4<u32>().y, vec4<u32>().z, vec4<u32>().w);
    let _e323 = vec4<u32>(_e317.x, _e311.y, _e317.z, _e317.w);
    if ((_e131 & 469762048u) == 67108864u) {
        phi_1020_ = vec4<u32>(_e323.x, _e323.y, ((u32(_e133) << bitcast<u32>(16i)) | u32(_e135)), _e323.w);
    } else {
        phi_1020_ = vec4<u32>(_e323.x, _e323.y, bitcast<u32>((_e308 - (floor((_e308 / 6.2831855f)) * 6.2831855f))), _e323.w);
    }
    let _e347 = phi_1020_;
    Fg = vec4<u32>(_e347.x, _e347.y, _e347.z, _e131);
    return;
}

@fragment
fn main(@location(0) x6_: vec4<f32>, @location(1) y6_: vec4<f32>, @location(2) L4_: vec4<f32>, @location(4) @interpolate(flat, either) E7_: u32, @location(3) C5_: vec3<f32>) -> @location(0) vec4<u32> {
    x6_1 = x6_;
    y6_1 = y6_;
    L4_1 = L4_;
    E7_1 = E7_;
    C5_1 = C5_;
    main_1();
    let _e11 = Fg;
    return _e11;
}
