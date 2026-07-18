struct Fe {
    c2_: array<vec2<u32>>,
}

struct h0xd {
    c2_: array<u32>,
}

struct Ge {
    c2_: array<vec4<f32>>,
}

struct j0xd {
    c2_: array<u32>,
}

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

struct q4xd {
    c2_: array<u32>,
}

@id(7) override bh: bool = true;
@id(6) override ah: bool = true;
@id(4) override Yg: bool = true;
@id(0) override Ug: bool = true;
@id(1) override Vg: bool = true;
@id(2) override Wg: bool = true;

@group(0) @binding(3)
var<storage> AD: Fe;
@group(2) @binding(1)
var<storage, read_write> h0_: h0xd;
@group(0) @binding(4)
var<storage> RB: Ge;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var KD: texture_2d<f32>;
@group(3) @binding(8)
var Jb: sampler;
@group(2) @binding(0)
var<storage, read_write> j0_: j0xd;
@group(0) @binding(0)
var<uniform> m: CC;
@group(2) @binding(3)
var<storage, read_write> q4_: q4xd;
@group(3) @binding(9)
var Z9_: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var IC: texture_2d<f32>;
@group(1) @binding(13)
var S5_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1333_: bool;
    var phi_3315_: f32;
    var phi_3314_: f32;
    var phi_3316_: f32;
    var phi_3319_: f32;
    var phi_3318_: f32;
    var phi_1370_: bool;
    var phi_3332_: f32;
    var phi_3320_: f32;
    var phi_3334_: vec4<f32>;
    var phi_1483_: bool;
    var phi_3338_: u32;
    var phi_1492_: bool;
    var phi_3352_: f32;
    var phi_3807_: vec4<f32>;
    var phi_3747_: i32;
    var phi_3955_: vec4<f32>;
    var phi_3956_: vec3<f32>;
    var phi_3958_: vec4<f32>;

    let _e73 = gl_FragCoord_1;
    let _e74 = _e73.xy;
    let _e77 = bitcast<vec2<u32>>(vec2<i32>(floor(_e74)));
    let _e79 = m.m6_;
    let _e108 = bitcast<i32>((((((_e77.y >> bitcast<u32>(5u)) * (((_e79 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e77.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e77.x & 28u) << bitcast<u32>(5u)) + ((_e77.y & 28u) << bitcast<u32>(2i)))) + (((_e77.y & 3u) << bitcast<u32>(2i)) + (_e77.x & 3u))));
    let _e111 = q4_.c2_[_e108];
    let _e115 = ((f32((_e111 & 131071u)) * 0.00048828125f) + -32f);
    let _e117 = (_e111 >> bitcast<u32>(17u));
    let _e120 = AD.c2_[_e117];
    phi_3314_ = _e115;
    if ((_e120.x & 768u) != 0u) {
        let _e124 = abs(_e115);
        phi_1333_ = Yg;
        if Yg {
            phi_1333_ = ((_e120.x & 512u) != 0u);
        }
        let _e128 = phi_1333_;
        phi_3315_ = _e124;
        if _e128 {
            phi_3315_ = (1f - abs(((fract((_e124 * 0.5f)) * 2f) + -1f)));
        }
        let _e136 = phi_3315_;
        phi_3314_ = _e136;
    }
    let _e138 = phi_3314_;
    let _e139 = clamp(_e138, 0f, 1f);
    phi_3318_ = _e139;
    if Ug {
        let _e141 = (_e120.x >> bitcast<u32>(16u));
        phi_3319_ = _e139;
        if (_e141 != 0u) {
            let _e145 = h0_.c2_[_e108];
            if (_e141 == (_e145 >> bitcast<u32>(16i))) {
                phi_3316_ = min(_e139, unpack2x16float(_e145).x);
            } else {
                phi_3316_ = 0f;
            }
            let _e153 = phi_3316_;
            phi_3319_ = _e153;
        }
        let _e155 = phi_3319_;
        phi_3318_ = _e155;
    }
    let _e157 = phi_3318_;
    phi_1370_ = Vg;
    if Vg {
        phi_1370_ = ((_e120.x & 1024u) != 0u);
    }
    let _e161 = phi_1370_;
    phi_3332_ = _e157;
    if _e161 {
        let _e162 = (_e117 * 4u);
        let _e166 = RB.c2_[(_e162 + 2u)];
        let _e177 = RB.c2_[(_e162 + 3u)];
        let _e182 = _e177.zw;
        let _e184 = ((abs(((mat2x2<f32>(vec2<f32>(_e166.x, _e166.y), vec2<f32>(_e166.z, _e166.w)) * _e74) + _e177.xy)) * _e182) - _e182);
        phi_3332_ = min(_e157, clamp((min(_e184.x, _e184.y) + 0.5f), 0f, 1f));
    }
    let _e192 = phi_3332_;
    let _e193 = (_e120.x & 15u);
    if (_e193 <= 1u) {
        phi_3334_ = select(unpack4x8unorm(_e120.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((Ug && (_e193 == 0u))));
    } else {
        let _e201 = (_e117 * 4u);
        let _e204 = RB.c2_[_e201];
        let _e215 = RB.c2_[(_e201 + 1u)];
        let _e218 = ((mat2x2<f32>(vec2<f32>(_e204.x, _e204.y), vec2<f32>(_e204.z, _e204.w)) * _e74) + _e215.xy);
        if (_e193 == 2u) {
            phi_3320_ = _e218.x;
        } else {
            phi_3320_ = length(_e218);
        }
        let _e223 = phi_3320_;
        let _e232 = textureSampleLevel(KD, Jb, vec2<f32>(((clamp(_e223, 0f, 1f) * _e215.z) + _e215.w), bitcast<f32>(_e120.y)), 0f);
        phi_3334_ = _e232;
    }
    let _e234 = phi_3334_;
    let _e236 = (_e234.w * _e192);
    let _e241 = vec4<f32>(_e234.x, _e234.y, _e234.z, _e236);
    phi_1483_ = Wg;
    if Wg {
        phi_1483_ = (_e236 != 0f);
    }
    let _e244 = phi_1483_;
    phi_3338_ = u32();
    phi_1492_ = _e244;
    if _e244 {
        let _e247 = ((_e120.x >> bitcast<u32>(4i)) & 15u);
        phi_3338_ = _e247;
        phi_1492_ = (_e247 != 0u);
    }
    let _e250 = phi_3338_;
    let _e252 = phi_1492_;
    phi_3955_ = _e241;
    if _e252 {
        let _e255 = j0_.c2_[_e108];
        let _e256 = unpack4x8unorm(_e255);
        let _e257 = _e241.xyz;
        local_2 = _e257;
        let _e258 = _e256.xyz;
        if (_e256.w != 0f) {
            phi_3352_ = (1f / _e256.w);
        } else {
            phi_3352_ = 0f;
        }
        let _e263 = phi_3352_;
        let _e264 = (_e258 * _e263);
        local = _e264;
        switch bitcast<i32>(_e250) {
            case 11: {
                let _e266 = local_2;
                local_1 = (_e266 * _e264);
                break;
            }
            case 1: {
                let _e268 = local_2;
                local_1 = ((_e268 + _e264) - (_e268 * _e264));
                break;
            }
            case 2: {
                let _e272 = local_2;
                let _e273 = (_e272 * _e264);
                local_1 = (select(_e273, (((_e272 + _e264) - _e273) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e264 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 3: {
                let _e280 = local_2;
                local_1 = min(_e280, _e264);
                break;
            }
            case 4: {
                let _e282 = local_2;
                local_1 = max(_e282, _e264);
                break;
            }
            case 5: {
                let _e285 = clamp(_e258, vec3<f32>(0f, 0f, 0f), _e256.www);
                let _e291 = vec4<f32>(_e285.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                let _e297 = vec4<f32>(_e291.x, _e285.y, _e291.z, _e291.w);
                let _e304 = local_2;
                let _e307 = (clamp((vec3<f32>(1f, 1f, 1f) - _e304), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e256.w);
                let _e308 = vec4<f32>(_e297.x, _e297.y, _e285.z, _e297.w).xyz;
                local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e308 / _e307)), sign(_e308), (_e307 == vec3<f32>(0f, 0f, 0f)));
                break;
            }
            case 6: {
                let _e314 = local_2;
                local_2 = clamp(_e314, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                let _e317 = clamp(_e258, vec3<f32>(0f, 0f, 0f), _e256.www);
                let _e323 = vec4<f32>(_e317.x, _e256.y, _e256.z, _e256.w);
                let _e329 = vec4<f32>(_e323.x, _e317.y, _e323.z, _e323.w);
                phi_3807_ = vec4<f32>(_e329.x, _e329.y, _e317.z, _e329.w);
                if (_e256.w == 0f) {
                    phi_3807_ = vec4<f32>(_e317.x, _e317.y, _e317.z, 1f);
                }
                let _e339 = phi_3807_;
                let _e343 = (vec3(_e339.w) - _e339.xyz);
                let _e344 = local_2;
                local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e343 / (_e344 * _e339.w))), sign(_e343), (_e344 == vec3<f32>(0f, 0f, 0f))));
                break;
            }
            case 7: {
                let _e352 = local_2;
                let _e353 = (_e352 * _e264);
                local_1 = (select(_e353, (((_e352 + _e264) - _e353) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e352 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                break;
            }
            case 8: {
                phi_3747_ = 0i;
                loop {
                    let _e361 = phi_3747_;
                    if (_e361 < 3i) {
                        let _e364 = local_2[_e361];
                        if (_e364 <= 0.5f) {
                            let _e367 = local[_e361];
                            local_1[_e361] = (1f - _e367);
                        } else {
                            let _e371 = local[_e361];
                            if (_e371 <= 0.25f) {
                                let _e373 = local[_e361];
                                let _e376 = local[_e361];
                                local_1[_e361] = ((((16f * _e373) - 12f) * _e376) + 3f);
                            } else {
                                let _e380 = local[_e361];
                                local_1[_e361] = (inverseSqrt(_e380) - 1f);
                            }
                        }
                        continue;
                    } else {
                        break;
                    }
                    continuing {
                        phi_3747_ = (_e361 + 1i);
                    }
                }
                let _e385 = local_2;
                let _e389 = local_1;
                local_1 = (_e264 + ((_e264 * ((_e385 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e389));
                break;
            }
            case 9: {
                let _e392 = local_2;
                local_1 = abs((_e264 - _e392));
                break;
            }
            case 10: {
                let _e395 = local_2;
                local_1 = ((_e395 + _e264) - ((_e395 * 2f) * _e264));
                break;
            }
            case 12: {
                if ah {
                    let _e400 = local_2;
                    let _e401 = clamp(_e400, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e401;
                    let _e416 = (_e401 - vec3(min(min(_e401.x, _e401.y), _e401.z)));
                    let _e424 = (_e416 * ((max(max(_e264.x, _e264.y), _e264.z) - min(min(_e264.x, _e264.y), _e264.z)) / max(0.000062f, max(max(_e416.x, _e416.y), _e416.z))));
                    let _e425 = dot(_e264, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e428 = (_e424 - vec3(dot(_e424, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e441 = (vec2<f32>(_e425, (1f - _e425)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e428.x, _e428.y), _e428.z)), max(max(_e428.x, _e428.y), _e428.z))));
                    local_1 = ((_e428 * min(1f, min(_e441.x, _e441.y))) + vec3(_e425));
                }
                break;
            }
            case 13: {
                if ah {
                    let _e449 = local_2;
                    let _e450 = clamp(_e449, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e450;
                    let _e465 = (_e264 - vec3(min(min(_e264.x, _e264.y), _e264.z)));
                    let _e473 = (_e465 * ((max(max(_e450.x, _e450.y), _e450.z) - min(min(_e450.x, _e450.y), _e450.z)) / max(0.000062f, max(max(_e465.x, _e465.y), _e465.z))));
                    let _e474 = dot(_e264, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e477 = (_e473 - vec3(dot(_e473, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e490 = (vec2<f32>(_e474, (1f - _e474)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e477.x, _e477.y), _e477.z)), max(max(_e477.x, _e477.y), _e477.z))));
                    local_1 = ((_e477 * min(1f, min(_e490.x, _e490.y))) + vec3(_e474));
                }
                break;
            }
            case 14: {
                if ah {
                    let _e498 = local_2;
                    let _e499 = clamp(_e498, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e499;
                    let _e500 = dot(_e264, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e503 = (_e499 - vec3(dot(_e499, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e516 = (vec2<f32>(_e500, (1f - _e500)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e503.x, _e503.y), _e503.z)), max(max(_e503.x, _e503.y), _e503.z))));
                    local_1 = ((_e503 * min(1f, min(_e516.x, _e516.y))) + vec3(_e500));
                }
                break;
            }
            case 15: {
                if ah {
                    let _e524 = local_2;
                    let _e525 = clamp(_e524, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                    local_2 = _e525;
                    let _e526 = dot(_e525, vec3<f32>(0.3f, 0.59f, 0.11f));
                    let _e529 = (_e264 - vec3(dot(_e264, vec3<f32>(0.3f, 0.59f, 0.11f))));
                    let _e542 = (vec2<f32>(_e526, (1f - _e526)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e529.x, _e529.y), _e529.z)), max(max(_e529.x, _e529.y), _e529.z))));
                    local_1 = ((_e529 * min(1f, min(_e542.x, _e542.y))) + vec3(_e526));
                }
                break;
            }
            default: {
            }
        }
        let _e550 = local_1;
        let _e552 = mix(_e257, _e550, vec3(_e256.w));
        phi_3955_ = vec4<f32>(_e552.x, _e552.y, _e552.z, _e236);
    }
    let _e558 = phi_3955_;
    let _e561 = (_e558.xyz * _e558.w);
    let _e567 = vec4<f32>(_e561.x, _e558.y, _e558.z, _e558.w);
    let _e573 = vec4<f32>(_e567.x, _e561.y, _e567.z, _e567.w);
    let _e579 = vec4<f32>(_e573.x, _e573.y, _e561.z, _e573.w);
    let _e580 = _e579.xyz;
    let _e582 = m.y3_;
    let _e584 = m.z3_;
    if bh {
        phi_3956_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e73.x) + (0.00583715f * _e73.y))))) * _e582) + _e584)) + _e580);
    } else {
        phi_3956_ = _e580;
    }
    let _e598 = phi_3956_;
    let _e604 = vec4<f32>(_e598.x, _e579.y, _e579.z, _e579.w);
    let _e610 = vec4<f32>(_e604.x, _e598.y, _e604.z, _e604.w);
    let _e616 = vec4<f32>(_e610.x, _e610.y, _e598.z, _e610.w);
    switch bitcast<i32>(0u) {
        default: {
            if (_e558.w == 0f) {
                break;
            }
            let _e619 = (1f - _e558.w);
            phi_3958_ = _e616;
            if (_e619 != 0f) {
                let _e623 = j0_.c2_[_e108];
                phi_3958_ = (_e616 + (unpack4x8unorm(_e623) * _e619));
            }
            let _e628 = phi_3958_;
            j0_.c2_[_e108] = pack4x8unorm(_e628);
            break;
        }
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
}
