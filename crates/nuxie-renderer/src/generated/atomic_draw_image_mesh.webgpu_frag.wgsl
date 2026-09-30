struct hf {
    e2_: array<vec2<u32>>,
}

struct h0Qd {
    e2_: array<u32>,
}

struct jf {
    e2_: array<vec4<f32>>,
}

struct k0Qd {
    e2_: array<u32>,
}

struct BC {
    qc: f32,
    Ad: f32,
    Ef: f32,
    Ff: f32,
    q6_: u32,
    Nb: u32,
    qf: u32,
    rf: u32,
    V7_: vec4<i32>,
    bh: vec2<f32>,
    Bd: vec2<f32>,
    d2_: u32,
    fh: f32,
    f6_: u32,
    U2_: f32,
    Cd: f32,
    lf: u32,
    C3_: f32,
    D3_: f32,
    Dd: f32,
    Yg: u32,
}

struct x4Qd {
    e2_: array<u32>,
}

@id(7) override Ih: bool = true;
@id(6) override Hh: bool = true;
@id(4) override Fh: bool = true;
@id(0) override Bh: bool = true;
@id(1) override Ch: bool = true;
@id(2) override Dh: bool = true;

@group(0) @binding(3)
var<storage> DD: hf;
@group(2) @binding(1)
var<storage, read_write> h0_: h0Qd;
@group(0) @binding(4)
var<storage> QB: jf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var N9_: sampler;
@group(2) @binding(0)
var<storage, read_write> k0_: k0Qd;
@group(0) @binding(0)
var<uniform> n: BC;
@group(1) @binding(11)
var HC: texture_2d<f32>;
@group(1) @binding(13)
var X5_: sampler;
var<private> Z1_1: vec2<f32>;
var<private> M0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> x4_: x4Qd;
var<private> y3_1: u32;
var<private> H1_1: vec4<f32>;
var<private> A1_1: u32;
@group(3) @binding(9)
var ea: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_5920_: f32;
    var phi_1531_: bool;
    var phi_5190_: f32;
    var phi_5189_: f32;
    var phi_5191_: f32;
    var phi_5194_: f32;
    var phi_5193_: f32;
    var phi_1568_: bool;
    var phi_5196_: f32;
    var phi_5887_: u32;
    var phi_5195_: f32;
    var phi_5886_: u32;
    var phi_5220_: vec4<f32>;
    var phi_1687_: bool;
    var phi_5224_: u32;
    var phi_1696_: bool;
    var phi_5239_: f32;
    var phi_5725_: vec4<f32>;
    var phi_5661_: i32;
    var phi_5882_: vec4<f32>;
    var phi_1270_: bool;
    var phi_5908_: u32;
    var phi_5936_: f32;
    var phi_7315_: f32;
    var phi_1302_: bool;
    var phi_5972_: f32;
    var phi_5973_: f32;
    var phi_7002_: vec4<f32>;
    var phi_6870_: i32;
    var phi_7327_: vec4<f32>;
    var phi_7340_: vec3<f32>;
    var phi_7342_: vec4<f32>;

    let _e82 = gl_FragCoord_1;
    let _e83 = _e82.xy;
    let _e86 = bitcast<vec2<u32>>(vec2<i32>(floor(_e83)));
    let _e88 = n.q6_;
    let _e117 = bitcast<i32>((((((_e86.y >> bitcast<u32>(5u)) * (((_e88 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e86.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e86.x & 28u) << bitcast<u32>(5u)) + ((_e86.y & 28u) << bitcast<u32>(2i)))) + (((_e86.y & 3u) << bitcast<u32>(2i)) + (_e86.x & 3u))));
    let _e118 = Z1_1;
    let _e119 = textureSample(HC, X5_, _e118);
    phi_5920_ = 1f;
    if Ch {
        let _e120 = M0_1;
        let _e123 = min(_e120.xy, _e120.zw);
        phi_5920_ = clamp(min(_e123.x, _e123.y), 0f, 1f);
    }
    let _e129 = phi_5920_;
    let _e132 = x4_.e2_[_e117];
    let _e134 = (_e132 >> bitcast<u32>(17u));
    let _e138 = ((f32((_e132 & 131071u)) * 0.00048828125f) + -32f);
    let _e141 = DD.e2_[_e134];
    phi_5189_ = _e138;
    if ((_e141.x & 768u) != 0u) {
        let _e145 = abs(_e138);
        phi_1531_ = Fh;
        if Fh {
            phi_1531_ = ((_e141.x & 512u) != 0u);
        }
        let _e149 = phi_1531_;
        phi_5190_ = _e145;
        if _e149 {
            phi_5190_ = (1f - abs(((fract((_e145 * 0.5f)) * 2f) + -1f)));
        }
        let _e157 = phi_5190_;
        phi_5189_ = _e157;
    }
    let _e159 = phi_5189_;
    let _e160 = clamp(_e159, 0f, 1f);
    phi_5193_ = _e160;
    if Bh {
        let _e162 = (_e141.x >> bitcast<u32>(16u));
        phi_5194_ = _e160;
        if (_e162 != 0u) {
            let _e166 = h0_.e2_[_e117];
            if (_e162 == (_e166 >> bitcast<u32>(16i))) {
                phi_5191_ = min(_e160, unpack2x16float(_e166).x);
            } else {
                phi_5191_ = 0f;
            }
            let _e174 = phi_5191_;
            phi_5194_ = _e174;
        }
        let _e176 = phi_5194_;
        phi_5193_ = _e176;
    }
    let _e178 = phi_5193_;
    phi_1568_ = Ch;
    if Ch {
        phi_1568_ = ((_e141.x & 1024u) != 0u);
    }
    let _e182 = phi_1568_;
    phi_5196_ = _e178;
    if _e182 {
        let _e183 = (_e134 * 8u);
        let _e187 = QB.e2_[(_e183 + 2u)];
        let _e198 = QB.e2_[(_e183 + 3u)];
        let _e203 = _e198.zw;
        let _e205 = ((abs(((mat2x2<f32>(vec2<f32>(_e187.x, _e187.y), vec2<f32>(_e187.z, _e187.w)) * _e83) + _e198.xy)) * _e203) - _e203);
        phi_5196_ = min(_e178, clamp((min(_e205.x, _e205.y) + 0.5f), 0f, 1f));
    }
    let _e213 = phi_5196_;
    let _e214 = (_e141.x & 15u);
    if (_e214 <= 1u) {
        let _e219 = (Bh && (_e214 == 0u));
        phi_5887_ = 0u;
        if _e219 {
            phi_5887_ = (_e141.y | pack2x16float(vec2<f32>(_e213, 0f)));
        }
        let _e224 = phi_5887_;
        phi_5886_ = _e224;
        phi_5220_ = select(unpack4x8unorm(_e141.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e219));
    } else {
        let _e227 = (_e134 * 8u);
        let _e230 = QB.e2_[_e227];
        let _e241 = QB.e2_[(_e227 + 1u)];
        let _e244 = ((mat2x2<f32>(vec2<f32>(_e230.x, _e230.y), vec2<f32>(_e230.z, _e230.w)) * _e83) + _e241.xy);
        if (_e214 == 2u) {
            phi_5195_ = _e244.x;
        } else {
            phi_5195_ = length(_e244);
        }
        let _e249 = phi_5195_;
        let _e258 = textureSampleLevel(ED, N9_, vec2<f32>(((clamp(_e249, 0f, 1f) * _e241.z) + _e241.w), bitcast<f32>(_e141.y)), 0f);
        phi_5886_ = 0u;
        phi_5220_ = _e258;
    }
    let _e260 = phi_5886_;
    let _e262 = phi_5220_;
    let _e264 = (_e262.w * _e213);
    let _e269 = vec4<f32>(_e262.x, _e262.y, _e262.z, _e264);
    phi_1687_ = Dh;
    if Dh {
        phi_1687_ = (_e264 != 0f);
    }
    let _e272 = phi_1687_;
    phi_5224_ = u32();
    phi_1696_ = _e272;
    if _e272 {
        let _e275 = ((_e141.x >> bitcast<u32>(4i)) & 15u);
        phi_5224_ = _e275;
        phi_1696_ = (_e275 != 0u);
    }
    let _e278 = phi_5224_;
    let _e280 = phi_1696_;
    phi_5882_ = _e269;
    if _e280 {
        let _e283 = k0_.e2_[_e117];
        let _e284 = unpack4x8unorm(_e283);
        let _e285 = _e269.xyz;
        local_5 = _e285;
        let _e286 = _e284.xyz;
        if (_e284.w != 0f) {
            phi_5239_ = (1f / _e284.w);
        } else {
            phi_5239_ = 0f;
        }
        let _e291 = phi_5239_;
        let _e292 = (_e286 * _e291);
        local_3 = _e292;
        switch bitcast<i32>(_e278) {
            case 11: {
                let _e294 = local_5;
                local_4 = (_e294 * _e292);
                break;
            }
            case 1: {
                let _e296 = local_5;
                local_4 = ((_e296 + _e292) - (_e296 * _e292));
                break;
            }
            case 2: {
                let _e300 = local_5;
                let _e301 = (_e300 * _e292);
                local_4 = (select(_e301, (((_e300 + _e292) - _e301) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e292 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e308 = local_5;
                local_4 = min(_e308, _e292);
                break;
            }
            case 4: {
                let _e310 = local_5;
                local_4 = max(_e310, _e292);
                break;
            }
            case 5: {
                let _e313 = clamp(_e286, vec3<f32>(0f, 0f, 0f), _e284.www);
                let _e319 = vec4<f32>(_e313.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e325 = vec4<f32>(_e319.x, _e313.y, _e319.z, _e319.w);
                let _e332 = local_5;
                let _e335 = (clamp((vec3<f32>(1f, 1f, 1f) - _e332), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e284.w);
                let _e336 = vec4<f32>(_e325.x, _e325.y, _e313.z, _e325.w).xyz;
                local_4 = select(min(vec3<f32>(1f, 1f, 1f), (_e336 / _e335)), sign(_e336), (_e335 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e342 = local_5;
                local_5 = clamp(_e342, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e345 = clamp(_e286, vec3<f32>(0f, 0f, 0f), _e284.www);
                let _e351 = vec4<f32>(_e345.x, _e284.y, _e284.z, _e284.w);
                let _e357 = vec4<f32>(_e351.x, _e345.y, _e351.z, _e351.w);
                phi_5725_ = vec4<f32>(_e357.x, _e357.y, _e345.z, _e357.w);
                if (_e284.w == 0f) {
                    phi_5725_ = vec4<f32>(_e345.x, _e345.y, _e345.z, 1f);
                }
                let _e367 = phi_5725_;
                let _e371 = (vec3(_e367.w) - _e367.xyz);
                let _e372 = local_5;
                local_4 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e371 / (_e372 * _e367.w))), sign(_e371), (_e372 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e380 = local_5;
                let _e381 = (_e380 * _e292);
                local_4 = (select(_e381, (((_e380 + _e292) - _e381) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e380 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_5661_ = 0i;
                loop {
                    let _e389 = phi_5661_;
                    if (_e389 < 3i) {
                        let _e392 = local_5[_e389];
                        if (_e392 <= 0.5f) {
                            let _e395 = local_3[_e389];
                            local_4[_e389] = (1f - _e395);
                        } else {
                            let _e399 = local_3[_e389];
                            if (_e399 <= 0.25f) {
                                let _e401 = local_3[_e389];
                                let _e404 = local_3[_e389];
                                local_4[_e389] = ((((16f * _e401) - 12f) * _e404) + 3f);
                            } else {
                                let _e408 = local_3[_e389];
                                local_4[_e389] = (inverseSqrt(_e408) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_5661_ = (_e389 + 1i);
                    }
                }
                let _e413 = local_5;
                let _e417 = local_4;
                local_4 = (_e292 + ((_e292 * ((_e413 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e417));
                break;
            }
            case 9: {
                let _e420 = local_5;
                local_4 = abs((_e292 - _e420));
                break;
            }
            case 10: {
                let _e423 = local_5;
                local_4 = ((_e423 + _e292) - ((_e423 * 2f) * _e292));
                break;
            }
            case 12: {
                if Hh {
                    let _e428 = local_5;
                    let _e429 = clamp(_e428, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e429;
                    let _e444 = (_e429 - vec3(min(min(_e429.x, _e429.y), _e429.z)));
                    let _e452 = (_e444 * ((max(max(_e292.x, _e292.y), _e292.z) - min(min(_e292.x, _e292.y), _e292.z)) / max(0.000062f, max(max(_e444.x, _e444.y), _e444.z))));
                    let _e453 = dot(_e292, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e456 = (_e452 - vec3(dot(_e452, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e469 = (vec2<f32>(_e453, (1f - _e453)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e456.x, _e456.y), _e456.z)), max(max(_e456.x, _e456.y), _e456.z))));
                    local_4 = ((_e456 * min(1f, min(_e469.x, _e469.y))) + vec3(_e453));
                }
                break;
            }
            case 13: {
                if Hh {
                    let _e477 = local_5;
                    let _e478 = clamp(_e477, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e478;
                    let _e493 = (_e292 - vec3(min(min(_e292.x, _e292.y), _e292.z)));
                    let _e501 = (_e493 * ((max(max(_e478.x, _e478.y), _e478.z) - min(min(_e478.x, _e478.y), _e478.z)) / max(0.000062f, max(max(_e493.x, _e493.y), _e493.z))));
                    let _e502 = dot(_e292, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e505 = (_e501 - vec3(dot(_e501, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e518 = (vec2<f32>(_e502, (1f - _e502)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e505.x, _e505.y), _e505.z)), max(max(_e505.x, _e505.y), _e505.z))));
                    local_4 = ((_e505 * min(1f, min(_e518.x, _e518.y))) + vec3(_e502));
                }
                break;
            }
            case 14: {
                if Hh {
                    let _e526 = local_5;
                    let _e527 = clamp(_e526, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e527;
                    let _e528 = dot(_e292, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e531 = (_e527 - vec3(dot(_e527, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e544 = (vec2<f32>(_e528, (1f - _e528)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e531.x, _e531.y), _e531.z)), max(max(_e531.x, _e531.y), _e531.z))));
                    local_4 = ((_e531 * min(1f, min(_e544.x, _e544.y))) + vec3(_e528));
                }
                break;
            }
            case 15: {
                if Hh {
                    let _e552 = local_5;
                    let _e553 = clamp(_e552, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_5 = _e553;
                    let _e554 = dot(_e553, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e557 = (_e292 - vec3(dot(_e292, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e570 = (vec2<f32>(_e554, (1f - _e554)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e557.x, _e557.y), _e557.z)), max(max(_e557.x, _e557.y), _e557.z))));
                    local_4 = ((_e557 * min(1f, min(_e570.x, _e570.y))) + vec3(_e554));
                }
                break;
            }
            default: {
            }
        }
        let _e578 = local_4;
        let _e580 = mix(_e285, _e578, vec3(_e284.w));
        phi_5882_ = vec4<f32>(_e580.x, _e580.y, _e580.z, _e264);
    }
    let _e586 = phi_5882_;
    let _e589 = (_e586.xyz * _e586.w);
    let _e595 = vec4<f32>(_e589.x, _e586.y, _e586.z, _e586.w);
    let _e601 = vec4<f32>(_e595.x, _e589.y, _e595.z, _e595.w);
    let _e607 = vec4<f32>(_e601.x, _e601.y, _e589.z, _e601.w);
    phi_1270_ = Bh;
    if Bh {
        let _e608 = y3_1;
        phi_1270_ = (_e608 != 0u);
    }
    let _e611 = phi_1270_;
    phi_7315_ = _e129;
    if _e611 {
        if (_e260 != 0u) {
            phi_5908_ = _e260;
        } else {
            let _e615 = h0_.e2_[_e117];
            phi_5908_ = _e615;
        }
        let _e617 = phi_5908_;
        let _e618 = y3_1;
        if (_e618 == (_e617 >> bitcast<u32>(16i))) {
            phi_5936_ = min(_e129, unpack2x16float(_e617).x);
        } else {
            phi_5936_ = 0f;
        }
        let _e626 = phi_5936_;
        phi_7315_ = _e626;
    }
    let _e628 = phi_7315_;
    let _e629 = H1_1;
    let _e630 = (_e119 * _e629);
    phi_1302_ = Dh;
    if Dh {
        let _e631 = A1_1;
        phi_1302_ = (_e631 != 0u);
    }
    let _e634 = phi_1302_;
    phi_7327_ = _e630;
    if _e634 {
        let _e637 = k0_.e2_[_e117];
        let _e641 = ((unpack4x8unorm(_e637) * (1f - _e586.w)) + _e607);
        if (_e630.w != 0f) {
            phi_5972_ = (1f / _e630.w);
        } else {
            phi_5972_ = 0f;
        }
        let _e647 = phi_5972_;
        let _e648 = (_e630.xyz * _e647);
        let _e649 = A1_1;
        local_2 = _e648;
        let _e650 = _e641.xyz;
        if (_e641.w != 0f) {
            phi_5973_ = (1f / _e641.w);
        } else {
            phi_5973_ = 0f;
        }
        let _e655 = phi_5973_;
        let _e656 = (_e650 * _e655);
        local = _e656;
        switch bitcast<i32>(_e649) {
            case 11: {
                let _e658 = local_2;
                local_1 = (_e658 * _e656);
                break;
            }
            case 1: {
                let _e660 = local_2;
                local_1 = ((_e660 + _e656) - (_e660 * _e656));
                break;
            }
            case 2: {
                let _e664 = local_2;
                let _e665 = (_e664 * _e656);
                local_1 = (select(_e665, (((_e664 + _e656) - _e665) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e656 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e672 = local_2;
                local_1 = min(_e672, _e656);
                break;
            }
            case 4: {
                let _e674 = local_2;
                local_1 = max(_e674, _e656);
                break;
            }
            case 5: {
                let _e677 = clamp(_e650, vec3<f32>(0f, 0f, 0f), _e641.www);
                let _e683 = vec4<f32>(_e677.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e689 = vec4<f32>(_e683.x, _e677.y, _e683.z, _e683.w);
                let _e696 = local_2;
                let _e699 = (clamp((vec3<f32>(1f, 1f, 1f) - _e696), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e641.w);
                let _e700 = vec4<f32>(_e689.x, _e689.y, _e677.z, _e689.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e700 / _e699)), sign(_e700), (_e699 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e706 = local_2;
                local_2 = clamp(_e706, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e709 = clamp(_e650, vec3<f32>(0f, 0f, 0f), _e641.www);
                let _e715 = vec4<f32>(_e709.x, _e641.y, _e641.z, _e641.w);
                let _e721 = vec4<f32>(_e715.x, _e709.y, _e715.z, _e715.w);
                phi_7002_ = vec4<f32>(_e721.x, _e721.y, _e709.z, _e721.w);
                if (_e641.w == 0f) {
                    phi_7002_ = vec4<f32>(_e709.x, _e709.y, _e709.z, 1f);
                }
                let _e731 = phi_7002_;
                let _e735 = (vec3(_e731.w) - _e731.xyz);
                let _e736 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e735 / (_e736 * _e731.w))), sign(_e735), (_e736 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e744 = local_2;
                let _e745 = (_e744 * _e656);
                local_1 = (select(_e745, (((_e744 + _e656) - _e745) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e744 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_6870_ = 0i;
                loop {
                    let _e753 = phi_6870_;
                    if (_e753 < 3i) {
                        let _e756 = local_2[_e753];
                        if (_e756 <= 0.5f) {
                            let _e759 = local[_e753];
                            local_1[_e753] = (1f - _e759);
                        } else {
                            let _e763 = local[_e753];
                            if (_e763 <= 0.25f) {
                                let _e765 = local[_e753];
                                let _e768 = local[_e753];
                                local_1[_e753] = ((((16f * _e765) - 12f) * _e768) + 3f);
                            } else {
                                let _e772 = local[_e753];
                                local_1[_e753] = (inverseSqrt(_e772) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_6870_ = (_e753 + 1i);
                    }
                }
                let _e777 = local_2;
                let _e781 = local_1;
                local_1 = (_e656 + ((_e656 * ((_e777 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e781));
                break;
            }
            case 9: {
                let _e784 = local_2;
                local_1 = abs((_e656 - _e784));
                break;
            }
            case 10: {
                let _e787 = local_2;
                local_1 = ((_e787 + _e656) - ((_e787 * 2f) * _e656));
                break;
            }
            case 12: {
                if Hh {
                    let _e792 = local_2;
                    let _e793 = clamp(_e792, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e793;
                    let _e808 = (_e793 - vec3(min(min(_e793.x, _e793.y), _e793.z)));
                    let _e816 = (_e808 * ((max(max(_e656.x, _e656.y), _e656.z) - min(min(_e656.x, _e656.y), _e656.z)) / max(0.000062f, max(max(_e808.x, _e808.y), _e808.z))));
                    let _e817 = dot(_e656, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e820 = (_e816 - vec3(dot(_e816, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e833 = (vec2<f32>(_e817, (1f - _e817)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e820.x, _e820.y), _e820.z)), max(max(_e820.x, _e820.y), _e820.z))));
                    local_1 = ((_e820 * min(1f, min(_e833.x, _e833.y))) + vec3(_e817));
                }
                break;
            }
            case 13: {
                if Hh {
                    let _e841 = local_2;
                    let _e842 = clamp(_e841, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e842;
                    let _e857 = (_e656 - vec3(min(min(_e656.x, _e656.y), _e656.z)));
                    let _e865 = (_e857 * ((max(max(_e842.x, _e842.y), _e842.z) - min(min(_e842.x, _e842.y), _e842.z)) / max(0.000062f, max(max(_e857.x, _e857.y), _e857.z))));
                    let _e866 = dot(_e656, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e869 = (_e865 - vec3(dot(_e865, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e882 = (vec2<f32>(_e866, (1f - _e866)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e869.x, _e869.y), _e869.z)), max(max(_e869.x, _e869.y), _e869.z))));
                    local_1 = ((_e869 * min(1f, min(_e882.x, _e882.y))) + vec3(_e866));
                }
                break;
            }
            case 14: {
                if Hh {
                    let _e890 = local_2;
                    let _e891 = clamp(_e890, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e891;
                    let _e892 = dot(_e656, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e895 = (_e891 - vec3(dot(_e891, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e908 = (vec2<f32>(_e892, (1f - _e892)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e895.x, _e895.y), _e895.z)), max(max(_e895.x, _e895.y), _e895.z))));
                    local_1 = ((_e895 * min(1f, min(_e908.x, _e908.y))) + vec3(_e892));
                }
                break;
            }
            case 15: {
                if Hh {
                    let _e916 = local_2;
                    let _e917 = clamp(_e916, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e917;
                    let _e918 = dot(_e917, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e921 = (_e656 - vec3(dot(_e656, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e934 = (vec2<f32>(_e918, (1f - _e918)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e921.x, _e921.y), _e921.z)), max(max(_e921.x, _e921.y), _e921.z))));
                    local_1 = ((_e921 * min(1f, min(_e934.x, _e934.y))) + vec3(_e918));
                }
                break;
            }
            default: {
            }
        }
        let _e942 = local_1;
        let _e945 = (mix(_e648, _e942, vec3(_e641.w)) * _e630.w);
        let _e951 = vec4<f32>(_e945.x, _e630.y, _e630.z, _e630.w);
        let _e957 = vec4<f32>(_e951.x, _e945.y, _e951.z, _e951.w);
        phi_7327_ = vec4<f32>(_e957.x, _e957.y, _e945.z, _e957.w);
    }
    let _e965 = phi_7327_;
    let _e966 = (_e965 * _e628);
    let _e970 = ((_e607 * (1f - _e966.w)) + _e966);
    let _e971 = _e970.xyz;
    let _e974 = n.C3_;
    let _e976 = n.D3_;
    if (Ih && (_e970.w != 0f)) {
        phi_7340_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e82.x) + (0.00583715f * _e82.y))))) * _e974) + _e976)) + _e971);
    } else {
        phi_7340_ = _e971;
    }
    let _e992 = phi_7340_;
    let _e998 = vec4<f32>(_e992.x, _e970.y, _e970.z, _e970.w);
    let _e1004 = vec4<f32>(_e998.x, _e992.y, _e998.z, _e998.w);
    let _e1010 = vec4<f32>(_e1004.x, _e1004.y, _e992.z, _e1004.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e970.w == 0f) {
                break;
            }
            let _e1013 = (1f - _e970.w);
            phi_7342_ = _e1010;
            if (_e1013 != 0f) {
                let _e1017 = k0_.e2_[_e117];
                phi_7342_ = (_e1010 + (unpack4x8unorm(_e1017) * _e1013));
            }
            let _e1022 = phi_7342_;
            k0_.e2_[_e117] = pack4x8unorm(_e1022);
            break;
        }
    }
    if (_e260 != 0u) {
        h0_.e2_[_e117] = _e260;
    }
    x4_.e2_[_e117] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) Z1_: vec2<f32>, @location(1) M0_: vec4<f32>, @location(4) @interpolate(flat, either) y3_: u32, @location(3) @interpolate(flat, either) H1_: vec4<f32>, @location(5) @interpolate(flat, either) A1_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    Z1_1 = Z1_;
    M0_1 = M0_;
    y3_1 = y3_;
    H1_1 = H1_;
    A1_1 = A1_;
    main_1();
}
