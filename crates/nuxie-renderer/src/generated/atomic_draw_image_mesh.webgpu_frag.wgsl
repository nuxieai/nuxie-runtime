struct He {
    c2_: array<vec2<u32>>,
}

struct h0zd {
    c2_: array<u32>,
}

struct Ie {
    c2_: array<vec4<f32>>,
}

struct j0zd {
    c2_: array<u32>,
}

struct CC {
    cc: f32,
    md: f32,
    df: f32,
    ef: f32,
    m6_: u32,
    Dg: u32,
    Pe: u32,
    Qe: u32,
    R7_: vec4<i32>,
    zg: vec2<f32>,
    nd: vec2<f32>,
    a2_: u32,
    Eg: f32,
    Z5_: u32,
    P2_: f32,
    od: f32,
    Ke: u32,
    z3_: f32,
    A3_: f32,
    pd: f32,
    wg: u32,
}

struct q4zd {
    c2_: array<u32>,
}

@id(7) override dh: bool = true;
@id(6) override ch: bool = true;
@id(4) override ah: bool = true;
@id(0) override Wg: bool = true;
@id(1) override Xg: bool = true;
@id(2) override Yg: bool = true;

@group(0) @binding(3)
var<storage> AD: He;
@group(2) @binding(1)
var<storage, read_write> h0_: h0zd;
@group(0) @binding(4)
var<storage> RB: Ie;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Kb: sampler;
@group(2) @binding(0)
var<storage, read_write> j0_: j0zd;
@group(0) @binding(0)
var<uniform> n: CC;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;
var<private> X1_1: vec2<f32>;
var<private> L0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> q4_: q4zd;
var<private> w3_1: u32;
var<private> A1_1: u32;
var<private> H1_1: f32;
@group(3) @binding(9)
var aa: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var local_3: vec3<f32>;
    var local_4: vec3<f32>;
    var local_5: vec3<f32>;
    var phi_5918_: f32;
    var phi_1529_: bool;
    var phi_5188_: f32;
    var phi_5187_: f32;
    var phi_5189_: f32;
    var phi_5192_: f32;
    var phi_5191_: f32;
    var phi_1566_: bool;
    var phi_5194_: f32;
    var phi_5885_: u32;
    var phi_5193_: f32;
    var phi_5884_: u32;
    var phi_5218_: vec4<f32>;
    var phi_1685_: bool;
    var phi_5222_: u32;
    var phi_1694_: bool;
    var phi_5237_: f32;
    var phi_5723_: vec4<f32>;
    var phi_5659_: i32;
    var phi_5880_: vec4<f32>;
    var phi_1270_: bool;
    var phi_5906_: u32;
    var phi_5934_: f32;
    var phi_7313_: f32;
    var phi_1298_: bool;
    var phi_5970_: f32;
    var phi_5971_: f32;
    var phi_7000_: vec4<f32>;
    var phi_6868_: i32;
    var phi_7325_: vec4<f32>;
    var phi_7338_: vec3<f32>;
    var phi_7340_: vec4<f32>;

    let _e82 = gl_FragCoord_1;
    let _e83 = _e82.xy;
    let _e86 = bitcast<vec2<u32>>(vec2<i32>(floor(_e83)));
    let _e88 = n.m6_;
    let _e117 = bitcast<i32>((((((_e86.y >> bitcast<u32>(5u)) * (((_e88 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e86.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e86.x & 28u) << bitcast<u32>(5u)) + ((_e86.y & 28u) << bitcast<u32>(2i)))) + (((_e86.y & 3u) << bitcast<u32>(2i)) + (_e86.x & 3u))));
    let _e118 = X1_1;
    let _e119 = textureSample(IC, S5_, _e118);
    phi_5918_ = 1f;
    if Xg {
        let _e120 = L0_1;
        let _e123 = min(_e120.xy, _e120.zw);
        phi_5918_ = clamp(min(_e123.x, _e123.y), 0f, 1f);
    }
    let _e129 = phi_5918_;
    let _e132 = q4_.c2_[_e117];
    let _e134 = (_e132 >> bitcast<u32>(17u));
    let _e138 = ((f32((_e132 & 131071u)) * 0.00048828125f) + -32f);
    let _e141 = AD.c2_[_e134];
    phi_5187_ = _e138;
    if ((_e141.x & 768u) != 0u) {
        let _e145 = abs(_e138);
        phi_1529_ = ah;
        if ah {
            phi_1529_ = ((_e141.x & 512u) != 0u);
        }
        let _e149 = phi_1529_;
        phi_5188_ = _e145;
        if _e149 {
            phi_5188_ = (1f - abs(((fract((_e145 * 0.5f)) * 2f) + -1f)));
        }
        let _e157 = phi_5188_;
        phi_5187_ = _e157;
    }
    let _e159 = phi_5187_;
    let _e160 = clamp(_e159, 0f, 1f);
    phi_5191_ = _e160;
    if Wg {
        let _e162 = (_e141.x >> bitcast<u32>(16u));
        phi_5192_ = _e160;
        if (_e162 != 0u) {
            let _e166 = h0_.c2_[_e117];
            if (_e162 == (_e166 >> bitcast<u32>(16i))) {
                phi_5189_ = min(_e160, unpack2x16float(_e166).x);
            } else {
                phi_5189_ = 0f;
            }
            let _e174 = phi_5189_;
            phi_5192_ = _e174;
        }
        let _e176 = phi_5192_;
        phi_5191_ = _e176;
    }
    let _e178 = phi_5191_;
    phi_1566_ = Xg;
    if Xg {
        phi_1566_ = ((_e141.x & 1024u) != 0u);
    }
    let _e182 = phi_1566_;
    phi_5194_ = _e178;
    if _e182 {
        let _e183 = (_e134 * 4u);
        let _e187 = RB.c2_[(_e183 + 2u)];
        let _e198 = RB.c2_[(_e183 + 3u)];
        let _e203 = _e198.zw;
        let _e205 = ((abs(((mat2x2<f32>(vec2<f32>(_e187.x, _e187.y), vec2<f32>(_e187.z, _e187.w)) * _e83) + _e198.xy)) * _e203) - _e203);
        phi_5194_ = min(_e178, clamp((min(_e205.x, _e205.y) + 0.5f), 0f, 1f));
    }
    let _e213 = phi_5194_;
    let _e214 = (_e141.x & 15u);
    if (_e214 <= 1u) {
        let _e219 = (Wg && (_e214 == 0u));
        phi_5885_ = 0u;
        if _e219 {
            phi_5885_ = (_e141.y | pack2x16float(vec2<f32>(_e213, 0f)));
        }
        let _e224 = phi_5885_;
        phi_5884_ = _e224;
        phi_5218_ = select(unpack4x8unorm(_e141.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e219));
    } else {
        let _e227 = (_e134 * 4u);
        let _e230 = RB.c2_[_e227];
        let _e241 = RB.c2_[(_e227 + 1u)];
        let _e244 = ((mat2x2<f32>(vec2<f32>(_e230.x, _e230.y), vec2<f32>(_e230.z, _e230.w)) * _e83) + _e241.xy);
        if (_e214 == 2u) {
            phi_5193_ = _e244.x;
        } else {
            phi_5193_ = length(_e244);
        }
        let _e249 = phi_5193_;
        let _e258 = textureSampleLevel(KD, Kb, vec2<f32>(((clamp(_e249, 0f, 1f) * _e241.z) + _e241.w), bitcast<f32>(_e141.y)), 0f);
        phi_5884_ = 0u;
        phi_5218_ = _e258;
    }
    let _e260 = phi_5884_;
    let _e262 = phi_5218_;
    let _e264 = (_e262.w * _e213);
    let _e269 = vec4<f32>(_e262.x, _e262.y, _e262.z, _e264);
    phi_1685_ = Yg;
    if Yg {
        phi_1685_ = (_e264 != 0f);
    }
    let _e272 = phi_1685_;
    phi_5222_ = u32();
    phi_1694_ = _e272;
    if _e272 {
        let _e275 = ((_e141.x >> bitcast<u32>(4i)) & 15u);
        phi_5222_ = _e275;
        phi_1694_ = (_e275 != 0u);
    }
    let _e278 = phi_5222_;
    let _e280 = phi_1694_;
    phi_5880_ = _e269;
    if _e280 {
        let _e283 = j0_.c2_[_e117];
        let _e284 = unpack4x8unorm(_e283);
        let _e285 = _e269.xyz;
        local_5 = _e285;
        let _e286 = _e284.xyz;
        if (_e284.w != 0f) {
            phi_5237_ = (1f / _e284.w);
        } else {
            phi_5237_ = 0f;
        }
        let _e291 = phi_5237_;
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
                phi_5723_ = vec4<f32>(_e357.x, _e357.y, _e345.z, _e357.w);
                if (_e284.w == 0f) {
                    phi_5723_ = vec4<f32>(_e345.x, _e345.y, _e345.z, 1f);
                }
                let _e367 = phi_5723_;
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
                phi_5659_ = 0i;
                loop {
                    let _e389 = phi_5659_;
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
                        phi_5659_ = (_e389 + 1i);
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
                if ch {
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
                if ch {
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
                if ch {
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
                if ch {
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
        phi_5880_ = vec4<f32>(_e580.x, _e580.y, _e580.z, _e264);
    }
    let _e586 = phi_5880_;
    let _e589 = (_e586.xyz * _e586.w);
    let _e595 = vec4<f32>(_e589.x, _e586.y, _e586.z, _e586.w);
    let _e601 = vec4<f32>(_e595.x, _e589.y, _e595.z, _e595.w);
    let _e607 = vec4<f32>(_e601.x, _e601.y, _e589.z, _e601.w);
    phi_1270_ = Wg;
    if Wg {
        let _e608 = w3_1;
        phi_1270_ = (_e608 != 0u);
    }
    let _e611 = phi_1270_;
    phi_7313_ = _e129;
    if _e611 {
        if (_e260 != 0u) {
            phi_5906_ = _e260;
        } else {
            let _e615 = h0_.c2_[_e117];
            phi_5906_ = _e615;
        }
        let _e617 = phi_5906_;
        let _e618 = w3_1;
        if (_e618 == (_e617 >> bitcast<u32>(16i))) {
            phi_5934_ = min(_e129, unpack2x16float(_e617).x);
        } else {
            phi_5934_ = 0f;
        }
        let _e626 = phi_5934_;
        phi_7313_ = _e626;
    }
    let _e628 = phi_7313_;
    phi_1298_ = Yg;
    if Yg {
        let _e629 = A1_1;
        phi_1298_ = (_e629 != 0u);
    }
    let _e632 = phi_1298_;
    phi_7325_ = _e119;
    if _e632 {
        let _e635 = j0_.c2_[_e117];
        let _e639 = ((unpack4x8unorm(_e635) * (1f - _e586.w)) + _e607);
        if (_e119.w != 0f) {
            phi_5970_ = (1f / _e119.w);
        } else {
            phi_5970_ = 0f;
        }
        let _e645 = phi_5970_;
        let _e646 = (_e119.xyz * _e645);
        let _e647 = A1_1;
        local_2 = _e646;
        let _e648 = _e639.xyz;
        if (_e639.w != 0f) {
            phi_5971_ = (1f / _e639.w);
        } else {
            phi_5971_ = 0f;
        }
        let _e653 = phi_5971_;
        let _e654 = (_e648 * _e653);
        local = _e654;
        switch bitcast<i32>(_e647) {
            case 11: {
                let _e656 = local_2;
                local_1 = (_e656 * _e654);
                break;
            }
            case 1: {
                let _e658 = local_2;
                local_1 = ((_e658 + _e654) - (_e658 * _e654));
                break;
            }
            case 2: {
                let _e662 = local_2;
                let _e663 = (_e662 * _e654);
                local_1 = (select(_e663, (((_e662 + _e654) - _e663) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e654 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e670 = local_2;
                local_1 = min(_e670, _e654);
                break;
            }
            case 4: {
                let _e672 = local_2;
                local_1 = max(_e672, _e654);
                break;
            }
            case 5: {
                let _e675 = clamp(_e648, vec3<f32>(0f, 0f, 0f), _e639.www);
                let _e681 = vec4<f32>(_e675.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e687 = vec4<f32>(_e681.x, _e675.y, _e681.z, _e681.w);
                let _e694 = local_2;
                let _e697 = (clamp((vec3<f32>(1f, 1f, 1f) - _e694), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e639.w);
                let _e698 = vec4<f32>(_e687.x, _e687.y, _e675.z, _e687.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e698 / _e697)), sign(_e698), (_e697 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e704 = local_2;
                local_2 = clamp(_e704, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e707 = clamp(_e648, vec3<f32>(0f, 0f, 0f), _e639.www);
                let _e713 = vec4<f32>(_e707.x, _e639.y, _e639.z, _e639.w);
                let _e719 = vec4<f32>(_e713.x, _e707.y, _e713.z, _e713.w);
                phi_7000_ = vec4<f32>(_e719.x, _e719.y, _e707.z, _e719.w);
                if (_e639.w == 0f) {
                    phi_7000_ = vec4<f32>(_e707.x, _e707.y, _e707.z, 1f);
                }
                let _e729 = phi_7000_;
                let _e733 = (vec3(_e729.w) - _e729.xyz);
                let _e734 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e733 / (_e734 * _e729.w))), sign(_e733), (_e734 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e742 = local_2;
                let _e743 = (_e742 * _e654);
                local_1 = (select(_e743, (((_e742 + _e654) - _e743) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e742 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_6868_ = 0i;
                loop {
                    let _e751 = phi_6868_;
                    if (_e751 < 3i) {
                        let _e754 = local_2[_e751];
                        if (_e754 <= 0.5f) {
                            let _e757 = local[_e751];
                            local_1[_e751] = (1f - _e757);
                        } else {
                            let _e761 = local[_e751];
                            if (_e761 <= 0.25f) {
                                let _e763 = local[_e751];
                                let _e766 = local[_e751];
                                local_1[_e751] = ((((16f * _e763) - 12f) * _e766) + 3f);
                            } else {
                                let _e770 = local[_e751];
                                local_1[_e751] = (inverseSqrt(_e770) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_6868_ = (_e751 + 1i);
                    }
                }
                let _e775 = local_2;
                let _e779 = local_1;
                local_1 = (_e654 + ((_e654 * ((_e775 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e779));
                break;
            }
            case 9: {
                let _e782 = local_2;
                local_1 = abs((_e654 - _e782));
                break;
            }
            case 10: {
                let _e785 = local_2;
                local_1 = ((_e785 + _e654) - ((_e785 * 2f) * _e654));
                break;
            }
            case 12: {
                if ch {
                    let _e790 = local_2;
                    let _e791 = clamp(_e790, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e791;
                    let _e806 = (_e791 - vec3(min(min(_e791.x, _e791.y), _e791.z)));
                    let _e814 = (_e806 * ((max(max(_e654.x, _e654.y), _e654.z) - min(min(_e654.x, _e654.y), _e654.z)) / max(0.000062f, max(max(_e806.x, _e806.y), _e806.z))));
                    let _e815 = dot(_e654, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e818 = (_e814 - vec3(dot(_e814, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e831 = (vec2<f32>(_e815, (1f - _e815)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e818.x, _e818.y), _e818.z)), max(max(_e818.x, _e818.y), _e818.z))));
                    local_1 = ((_e818 * min(1f, min(_e831.x, _e831.y))) + vec3(_e815));
                }
                break;
            }
            case 13: {
                if ch {
                    let _e839 = local_2;
                    let _e840 = clamp(_e839, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e840;
                    let _e855 = (_e654 - vec3(min(min(_e654.x, _e654.y), _e654.z)));
                    let _e863 = (_e855 * ((max(max(_e840.x, _e840.y), _e840.z) - min(min(_e840.x, _e840.y), _e840.z)) / max(0.000062f, max(max(_e855.x, _e855.y), _e855.z))));
                    let _e864 = dot(_e654, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e867 = (_e863 - vec3(dot(_e863, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e880 = (vec2<f32>(_e864, (1f - _e864)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e867.x, _e867.y), _e867.z)), max(max(_e867.x, _e867.y), _e867.z))));
                    local_1 = ((_e867 * min(1f, min(_e880.x, _e880.y))) + vec3(_e864));
                }
                break;
            }
            case 14: {
                if ch {
                    let _e888 = local_2;
                    let _e889 = clamp(_e888, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e889;
                    let _e890 = dot(_e654, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e893 = (_e889 - vec3(dot(_e889, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e906 = (vec2<f32>(_e890, (1f - _e890)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e893.x, _e893.y), _e893.z)), max(max(_e893.x, _e893.y), _e893.z))));
                    local_1 = ((_e893 * min(1f, min(_e906.x, _e906.y))) + vec3(_e890));
                }
                break;
            }
            case 15: {
                if ch {
                    let _e914 = local_2;
                    let _e915 = clamp(_e914, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e915;
                    let _e916 = dot(_e915, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e919 = (_e654 - vec3(dot(_e654, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e932 = (vec2<f32>(_e916, (1f - _e916)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e919.x, _e919.y), _e919.z)), max(max(_e919.x, _e919.y), _e919.z))));
                    local_1 = ((_e919 * min(1f, min(_e932.x, _e932.y))) + vec3(_e916));
                }
                break;
            }
            default: {
            }
        }
        let _e940 = local_1;
        let _e943 = (mix(_e646, _e940, vec3(_e639.w)) * _e119.w);
        let _e949 = vec4<f32>(_e943.x, _e119.y, _e119.z, _e119.w);
        let _e955 = vec4<f32>(_e949.x, _e943.y, _e949.z, _e949.w);
        phi_7325_ = vec4<f32>(_e955.x, _e955.y, _e943.z, _e955.w);
    }
    let _e963 = phi_7325_;
    let _e964 = H1_1;
    let _e966 = (_e963 * (_e628 * _e964));
    let _e970 = ((_e607 * (1f - _e966.w)) + _e966);
    let _e971 = _e970.xyz;
    let _e974 = n.z3_;
    let _e976 = n.A3_;
    if (dh && (_e970.w != 0f)) {
        phi_7338_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e82.x) + (0.00583715f * _e82.y))))) * _e974) + _e976)) + _e971);
    } else {
        phi_7338_ = _e971;
    }
    let _e992 = phi_7338_;
    let _e998 = vec4<f32>(_e992.x, _e970.y, _e970.z, _e970.w);
    let _e1004 = vec4<f32>(_e998.x, _e992.y, _e998.z, _e998.w);
    let _e1010 = vec4<f32>(_e1004.x, _e1004.y, _e992.z, _e1004.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e970.w == 0f) {
                break;
            }
            let _e1013 = (1f - _e970.w);
            phi_7340_ = _e1010;
            if (_e1013 != 0f) {
                let _e1017 = j0_.c2_[_e117];
                phi_7340_ = (_e1010 + (unpack4x8unorm(_e1017) * _e1013));
            }
            let _e1022 = phi_7340_;
            j0_.c2_[_e117] = pack4x8unorm(_e1022);
            break;
        }
    }
    if (_e260 != 0u) {
        h0_.c2_[_e117] = _e260;
    }
    q4_.c2_[_e117] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) X1_: vec2<f32>, @location(1) L0_: vec4<f32>, @location(4) @interpolate(flat, either) w3_: u32, @location(5) @interpolate(flat, either) A1_: u32, @location(3) @interpolate(flat, either) H1_: f32) {
    gl_FragCoord_1 = gl_FragCoord;
    X1_1 = X1_;
    L0_1 = L0_;
    w3_1 = w3_;
    A1_1 = A1_;
    H1_1 = H1_;
    main_1();
}
