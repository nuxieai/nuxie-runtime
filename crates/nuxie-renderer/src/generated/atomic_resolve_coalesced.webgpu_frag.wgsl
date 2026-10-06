struct Gf {
    k2_: array<vec2<u32>>,
}

struct m0he {
    k2_: array<u32>,
}

struct Hf {
    k2_: array<vec4<f32>>,
}

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

struct n0he {
    k2_: array<u32>,
}

struct L4he {
    k2_: array<u32>,
}

@id(6) override si: bool = true;
@id(4) override qi: bool = true;
@id(0) override mi: bool = true;
@id(1) override ni: bool = true;
@id(2) override oi: bool = true;

@group(0) @binding(3)
var<storage> WC: Gf;
@group(2) @binding(1)
var<storage, read_write> m0_: m0he;
@group(0) @binding(4)
var<storage> JB: Hf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
@group(2) @binding(0)
var<storage, read_write> n0_: n0he;
@group(2) @binding(3)
var<storage, read_write> L4_: L4he;
var<private> L1_: vec4<f32>;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(1) @binding(13)
var r5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1279_: bool;
    var phi_3228_: f32;
    var phi_3227_: f32;
    var phi_3229_: f32;
    var phi_3232_: f32;
    var phi_3231_: f32;
    var phi_1316_: bool;
    var phi_3248_: f32;
    var phi_3233_: f32;
    var phi_3245_: vec4<f32>;
    var phi_3243_: vec4<f32>;
    var phi_3251_: f32;
    var phi_3674_: vec4<f32>;
    var phi_3618_: i32;
    var phi_3813_: vec4<f32>;
    var phi_3826_: vec4<f32>;
    var phi_3827_: vec4<f32>;

    let _e70 = gl_FragCoord_1;
    let _e71 = _e70.xy;
    let _e74 = bitcast<vec2<u32>>(vec2<i32>(floor(_e71)));
    let _e76 = j.A6_;
    let _e105 = bitcast<i32>((((((_e74.y >> bitcast<u32>(5u)) * (((_e76 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e74.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e74.x & 28u) << bitcast<u32>(5u)) + ((_e74.y & 28u) << bitcast<u32>(2i)))) + (((_e74.y & 3u) << bitcast<u32>(2i)) + (_e74.x & 3u))));
    let _e108 = L4_.k2_[_e105];
    let _e112 = ((f32((_e108 & 131071u)) * 0.00048828125f) + -32f);
    let _e114 = (_e108 >> bitcast<u32>(17u));
    let _e117 = WC.k2_[_e114];
    phi_3227_ = _e112;
    if ((_e117.x & 768u) != 0u) {
        let _e121 = abs(_e112);
        phi_1279_ = qi;
        if qi {
            phi_1279_ = ((_e117.x & 512u) != 0u);
        }
        let _e125 = phi_1279_;
        phi_3228_ = _e121;
        if _e125 {
            phi_3228_ = (1f - abs(((fract((_e121 * 0.5f)) * 2f) + -1f)));
        }
        let _e133 = phi_3228_;
        phi_3227_ = _e133;
    }
    let _e135 = phi_3227_;
    let _e136 = clamp(_e135, 0f, 1f);
    phi_3231_ = _e136;
    if mi {
        let _e138 = (_e117.x >> bitcast<u32>(16u));
        phi_3232_ = _e136;
        if (_e138 != 0u) {
            let _e142 = m0_.k2_[_e105];
            if (_e138 == (_e142 >> bitcast<u32>(16i))) {
                phi_3229_ = min(_e136, unpack2x16float(_e142).x);
            } else {
                phi_3229_ = 0f;
            }
            let _e150 = phi_3229_;
            phi_3232_ = _e150;
        }
        let _e152 = phi_3232_;
        phi_3231_ = _e152;
    }
    let _e154 = phi_3231_;
    phi_1316_ = ni;
    if ni {
        phi_1316_ = ((_e117.x & 1024u) != 0u);
    }
    let _e158 = phi_1316_;
    phi_3248_ = _e154;
    if _e158 {
        let _e159 = (_e114 * 8u);
        let _e163 = JB.k2_[(_e159 + 2u)];
        let _e174 = JB.k2_[(_e159 + 3u)];
        let _e179 = _e174.zw;
        let _e181 = ((abs(((mat2x2<f32>(vec2<f32>(_e163.x, _e163.y), vec2<f32>(_e163.z, _e163.w)) * _e71) + _e174.xy)) * _e179) - _e179);
        phi_3248_ = min(_e154, clamp((min(_e181.x, _e181.y) + 0.5f), 0f, 1f));
    }
    let _e189 = phi_3248_;
    let _e190 = (_e117.x & 15u);
    let _e193 = ((_e117.x >> bitcast<u32>(4i)) & 15u);
    let _e195 = (oi && (_e193 != 0u));
    if (_e190 <= 1u) {
        phi_3243_ = select(unpack4x8unorm(_e117.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((mi && (_e190 == 0u))));
    } else {
        let _e203 = (_e114 * 8u);
        let _e206 = JB.k2_[_e203];
        let _e217 = JB.k2_[(_e203 + 1u)];
        let _e220 = ((mat2x2<f32>(vec2<f32>(_e206.x, _e206.y), vec2<f32>(_e206.z, _e206.w)) * _e71) + _e217.xy);
        if (_e190 == 2u) {
            phi_3233_ = _e220.x;
        } else {
            phi_3233_ = length(_e220);
        }
        let _e225 = phi_3233_;
        let _e232 = bitcast<f32>(_e117.y);
        let _e235 = j.xc;
        let _e238 = j.yc;
        let _e241 = textureSampleLevel(ED, ia, vec2<f32>(((clamp(_e225, 0f, 1f) * _e217.z) + _e217.w), ((floor(_e232) * _e235) + _e238)), 0f);
        phi_3245_ = _e241;
        if !(_e195) {
            let _e245 = (_e241.xyz * _e241.w);
            phi_3245_ = vec4<f32>(_e245.x, _e245.y, _e245.z, (_e241.w * (fract(_e232) * 1.0039216f)));
        }
        let _e254 = phi_3245_;
        phi_3243_ = _e254;
    }
    let _e256 = phi_3243_;
    phi_3826_ = _e256;
    if _e195 {
        phi_3813_ = _e256;
        if ((_e256.w * _e189) != 0f) {
            let _e262 = n0_.k2_[_e105];
            let _e263 = unpack4x8unorm(_e262);
            let _e264 = _e256.xyz;
            local_2 = _e264;
            let _e265 = _e263.xyz;
            if (_e263.w != 0f) {
                phi_3251_ = (1f / _e263.w);
            } else {
                phi_3251_ = 0f;
            }
            let _e270 = phi_3251_;
            let _e271 = (_e265 * _e270);
            local = _e271;
            switch bitcast<i32>(_e193) {
                case 11: {
                    let _e273 = local_2;
                    local_1 = (_e273 * _e271);
                    break;
                }
                case 1: {
                    let _e275 = local_2;
                    local_1 = ((_e275 + _e271) - (_e275 * _e271));
                    break;
                }
                case 2: {
                    let _e279 = local_2;
                    let _e280 = (_e279 * _e271);
                    local_1 = (select(_e280, (((_e279 + _e271) - _e280) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e271 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 3: {
                    let _e287 = local_2;
                    local_1 = min(_e287, _e271);
                    break;
                }
                case 4: {
                    let _e289 = local_2;
                    local_1 = max(_e289, _e271);
                    break;
                }
                case 5: {
                    let _e292 = clamp(_e265, vec3<f32>(0f, 0f, 0f), _e263.www);
                    let _e298 = vec4<f32>(_e292.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                    let _e304 = vec4<f32>(_e298.x, _e292.y, _e298.z, _e298.w);
                    let _e311 = local_2;
                    let _e314 = (clamp((vec3<f32>(1f, 1f, 1f) - _e311), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e263.w);
                    let _e315 = vec4<f32>(_e304.x, _e304.y, _e292.z, _e304.w).xyz;
                    local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e315 / _e314)), sign(_e315), (_e314 == vec3<f32>(0f, 0f, 0f)));
                    break;
                }
                case 6: {
                    let _e321 = local_2;
                    local_2 = clamp(_e321, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    let _e324 = clamp(_e265, vec3<f32>(0f, 0f, 0f), _e263.www);
                    let _e330 = vec4<f32>(_e324.x, _e263.y, _e263.z, _e263.w);
                    let _e336 = vec4<f32>(_e330.x, _e324.y, _e330.z, _e330.w);
                    phi_3674_ = vec4<f32>(_e336.x, _e336.y, _e324.z, _e336.w);
                    if (_e263.w == 0f) {
                        phi_3674_ = vec4<f32>(_e324.x, _e324.y, _e324.z, 1f);
                    }
                    let _e346 = phi_3674_;
                    let _e350 = (vec3(_e346.w) - _e346.xyz);
                    let _e351 = local_2;
                    local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e350 / (_e351 * _e346.w))), sign(_e350), (_e351 == vec3<f32>(0f, 0f, 0f))));
                    break;
                }
                case 7: {
                    let _e359 = local_2;
                    let _e360 = (_e359 * _e271);
                    local_1 = (select(_e360, (((_e359 + _e271) - _e360) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e359 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                    break;
                }
                case 8: {
                    phi_3618_ = 0i;
                    loop {
                        let _e368 = phi_3618_;
                        if (_e368 < 3i) {
                            let _e371 = local_2[_e368];
                            if (_e371 <= 0.5f) {
                                let _e374 = local[_e368];
                                local_1[_e368] = (1f - _e374);
                            } else {
                                let _e378 = local[_e368];
                                if (_e378 <= 0.25f) {
                                    let _e380 = local[_e368];
                                    let _e383 = local[_e368];
                                    local_1[_e368] = ((((16f * _e380) - 12f) * _e383) + 3f);
                                } else {
                                    let _e387 = local[_e368];
                                    local_1[_e368] = (inverseSqrt(_e387) - 1f);
                                }
                            }
                            continue;
                        } else {
                            break;
                        }
                        continuing {
                            phi_3618_ = (_e368 + 1i);
                        }
                    }
                    let _e392 = local_2;
                    let _e396 = local_1;
                    local_1 = (_e271 + ((_e271 * ((_e392 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e396));
                    break;
                }
                case 9: {
                    let _e399 = local_2;
                    local_1 = abs((_e271 - _e399));
                    break;
                }
                case 10: {
                    let _e402 = local_2;
                    local_1 = ((_e402 + _e271) - ((_e402 * 2f) * _e271));
                    break;
                }
                case 12: {
                    if si {
                        let _e407 = local_2;
                        let _e408 = clamp(_e407, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e408;
                        let _e423 = (_e408 - vec3(min(min(_e408.x, _e408.y), _e408.z)));
                        let _e431 = (_e423 * ((max(max(_e271.x, _e271.y), _e271.z) - min(min(_e271.x, _e271.y), _e271.z)) / max(0.000062f, max(max(_e423.x, _e423.y), _e423.z))));
                        let _e432 = dot(_e271, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e435 = (_e431 - vec3(dot(_e431, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e448 = (vec2<f32>(_e432, (1f - _e432)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e435.x, _e435.y), _e435.z)), max(max(_e435.x, _e435.y), _e435.z))));
                        local_1 = ((_e435 * min(1f, min(_e448.x, _e448.y))) + vec3(_e432));
                    }
                    break;
                }
                case 13: {
                    if si {
                        let _e456 = local_2;
                        let _e457 = clamp(_e456, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e457;
                        let _e472 = (_e271 - vec3(min(min(_e271.x, _e271.y), _e271.z)));
                        let _e480 = (_e472 * ((max(max(_e457.x, _e457.y), _e457.z) - min(min(_e457.x, _e457.y), _e457.z)) / max(0.000062f, max(max(_e472.x, _e472.y), _e472.z))));
                        let _e481 = dot(_e271, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e484 = (_e480 - vec3(dot(_e480, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e497 = (vec2<f32>(_e481, (1f - _e481)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e484.x, _e484.y), _e484.z)), max(max(_e484.x, _e484.y), _e484.z))));
                        local_1 = ((_e484 * min(1f, min(_e497.x, _e497.y))) + vec3(_e481));
                    }
                    break;
                }
                case 14: {
                    if si {
                        let _e505 = local_2;
                        let _e506 = clamp(_e505, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e506;
                        let _e507 = dot(_e271, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e510 = (_e506 - vec3(dot(_e506, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e523 = (vec2<f32>(_e507, (1f - _e507)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e510.x, _e510.y), _e510.z)), max(max(_e510.x, _e510.y), _e510.z))));
                        local_1 = ((_e510 * min(1f, min(_e523.x, _e523.y))) + vec3(_e507));
                    }
                    break;
                }
                case 15: {
                    if si {
                        let _e531 = local_2;
                        let _e532 = clamp(_e531, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        local_2 = _e532;
                        let _e533 = dot(_e532, vec3<f32>(0.3f, 0.59f, 0.11f));
                        let _e536 = (_e271 - vec3(dot(_e271, vec3<f32>(0.3f, 0.59f, 0.11f))));
                        let _e549 = (vec2<f32>(_e533, (1f - _e533)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e536.x, _e536.y), _e536.z)), max(max(_e536.x, _e536.y), _e536.z))));
                        local_1 = ((_e536 * min(1f, min(_e549.x, _e549.y))) + vec3(_e533));
                    }
                    break;
                }
                default: {
                }
            }
            let _e557 = local_1;
            let _e559 = mix(_e264, _e557, vec3(_e263.w));
            let _e565 = vec4<f32>(_e559.x, _e256.y, _e256.z, _e256.w);
            let _e571 = vec4<f32>(_e565.x, _e559.y, _e565.z, _e565.w);
            phi_3813_ = vec4<f32>(_e571.x, _e571.y, _e559.z, _e571.w);
        }
        let _e579 = phi_3813_;
        let _e582 = (_e579.xyz * _e579.w);
        let _e588 = vec4<f32>(_e582.x, _e579.y, _e579.z, _e579.w);
        let _e594 = vec4<f32>(_e588.x, _e582.y, _e588.z, _e588.w);
        phi_3826_ = vec4<f32>(_e594.x, _e594.y, _e582.z, _e594.w);
    }
    let _e602 = phi_3826_;
    let _e603 = (_e602 * _e189);
    let _e605 = (1f - _e603.w);
    phi_3827_ = _e603;
    if (_e605 != 0f) {
        let _e609 = n0_.k2_[_e105];
        phi_3827_ = (_e603 + (unpack4x8unorm(_e609) * _e605));
    }
    let _e614 = phi_3827_;
    L1_ = _e614;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = L1_;
    return _e3;
}
