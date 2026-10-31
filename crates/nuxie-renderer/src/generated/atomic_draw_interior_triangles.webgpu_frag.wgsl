struct nd {
    z2_: array<vec4<u32>>,
}

struct n0hf {
    z2_: array<u32>,
}

struct Cg {
    z2_: array<vec4<f32>>,
}

struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct p0hf {
    z2_: array<u32>,
}

struct d5hf {
    z2_: array<u32>,
}

@id(8) override Cj: bool = true;
@id(7) override Bj: bool = true;
@id(9) override Dj: bool = true;
@id(5) override zj: bool = true;
@id(0) override uj: bool = true;
@id(1) override vj: bool = true;
@id(2) override wj: bool = true;

@group(0) @binding(2)
var<storage> JB: nd;
@group(2) @binding(1)
var<storage, read_write> n0_: n0hf;
@group(0) @binding(3)
var<storage> KB: Cg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: RB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var V8_: sampler;
@group(2) @binding(0)
var<storage, read_write> p0_: p0hf;
@group(2) @binding(3)
var<storage, read_write> d5_: d5hf;
var<private> J0_1: u32;
var<private> r1_1: f32;
@group(3) @binding(8)
var ib: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(1) @binding(12)
var x4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_3785_: u32;
    var phi_1620_: bool;
    var phi_3790_: f32;
    var phi_3789_: f32;
    var phi_3791_: f32;
    var phi_3794_: f32;
    var phi_3793_: f32;
    var phi_1657_: bool;
    var phi_3798_: f32;
    var phi_3795_: f32;
    var phi_3796_: f32;
    var phi_3797_: vec2<f32>;
    var phi_3820_: vec4<f32>;
    var phi_4527_: u32;
    var phi_4526_: u32;
    var phi_3818_: vec4<f32>;
    var phi_3827_: f32;
    var phi_4277_: i32;
    var phi_4345_: vec4<f32>;
    var phi_4511_: vec4<f32>;
    var phi_4524_: vec4<f32>;
    var phi_4557_: u32;
    var phi_4551_: vec4<f32>;
    var phi_4552_: vec3<f32>;
    var phi_4554_: vec4<f32>;

    let _e85 = gl_FragCoord_1;
    let _e86 = _e85.xy;
    let _e89 = bitcast<vec2<u32>>(vec2<i32>(floor(_e86)));
    let _e91 = j.V5_;
    let _e120 = bitcast<i32>((((((_e89.y >> bitcast<u32>(5u)) * (((_e91 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e89.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e89.x & 28u) << bitcast<u32>(5u)) + ((_e89.y & 28u) << bitcast<u32>(2i)))) + (((_e89.y & 3u) << bitcast<u32>(2i)) + (_e89.x & 3u))));
    let _e123 = d5_.z2_[_e120];
    let _e125 = (_e123 >> bitcast<u32>(17u));
    let _e126 = J0_1;
    if (_e125 == _e126) {
        phi_3785_ = _e123;
    } else {
        phi_3785_ = ((_e126 << bitcast<u32>(17u)) + 65536u);
    }
    let _e132 = phi_3785_;
    let _e133 = r1_1;
    d5_.z2_[_e120] = (_e132 + bitcast<u32>(i32(round((_e133 * 2048f)))));
    phi_4557_ = 0u;
    phi_4551_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e125 != _e126) {
        let _e143 = ((f32((_e123 & 131071u)) * 0.00048828125f) + -32f);
        let _e148 = JB.z2_[((_e125 * 4u) + 1u)];
        phi_3789_ = _e143;
        if ((_e148.z & 768u) != 0u) {
            let _e152 = abs(_e143);
            phi_1620_ = zj;
            if zj {
                phi_1620_ = ((_e148.z & 512u) != 0u);
            }
            let _e156 = phi_1620_;
            phi_3790_ = _e152;
            if _e156 {
                phi_3790_ = (1f - abs(((fract((_e152 * 0.5f)) * 2f) + -1f)));
            }
            let _e164 = phi_3790_;
            phi_3789_ = _e164;
        }
        let _e166 = phi_3789_;
        let _e167 = clamp(_e166, 0f, 1f);
        phi_3793_ = _e167;
        if uj {
            let _e169 = (_e148.z >> bitcast<u32>(16u));
            phi_3794_ = _e167;
            if (_e169 != 0u) {
                let _e173 = n0_.z2_[_e120];
                if (_e169 == (_e173 >> bitcast<u32>(16i))) {
                    phi_3791_ = min(_e167, unpack2x16float(_e173).x);
                } else {
                    phi_3791_ = 0f;
                }
                let _e181 = phi_3791_;
                phi_3794_ = _e181;
            }
            let _e183 = phi_3794_;
            phi_3793_ = _e183;
        }
        let _e185 = phi_3793_;
        phi_1657_ = vj;
        if vj {
            phi_1657_ = ((_e148.z & 1024u) != 0u);
        }
        let _e189 = phi_1657_;
        phi_3798_ = _e185;
        if _e189 {
            let _e190 = (_e125 * 8u);
            let _e194 = KB.z2_[(_e190 + 2u)];
            let _e205 = KB.z2_[(_e190 + 3u)];
            let _e210 = _e205.zw;
            let _e212 = ((abs(((mat2x2<f32>(vec2<f32>(_e194.x, _e194.y), vec2<f32>(_e194.z, _e194.w)) * _e86) + _e205.xy)) * _e210) - _e210);
            phi_3798_ = min(_e185, clamp((min(_e212.x, _e212.y) + 0.5f), 0f, 1f));
        }
        let _e220 = phi_3798_;
        let _e221 = (_e148.z & 15u);
        let _e224 = ((_e148.z >> bitcast<u32>(4i)) & 15u);
        let _e226 = (wj && (_e224 != 0u));
        if (_e221 <= 1u) {
            let _e303 = (uj && (_e221 == 0u));
            phi_4527_ = 0u;
            if _e303 {
                phi_4527_ = (_e148.w | pack2x16float(vec2<f32>(_e220, 0f)));
            }
            let _e308 = phi_4527_;
            phi_4526_ = _e308;
            phi_3818_ = select(unpack4x8unorm(_e148.w), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e303));
        } else {
            let _e228 = (_e125 * 8u);
            let _e231 = KB.z2_[_e228];
            let _e242 = KB.z2_[(_e228 + 1u)];
            let _e245 = ((mat2x2<f32>(vec2<f32>(_e231.x, _e231.y), vec2<f32>(_e231.z, _e231.w)) * _e86) + _e242.xy);
            let _e249 = f32(_e221);
            let _e251 = j.T8_;
            let _e253 = j.U8_;
            let _e258 = ((_e242.w * _e251) + _e253);
            switch bitcast<i32>(0u) {
                default: {
                    if (Dj && (_e249 == 4f)) {
                        let _e272 = normalize(_e245);
                        let _e275 = acos(clamp(_e272.x, -1f, 1f));
                        if (_e272.y >= 0f) {
                            phi_3796_ = _e275;
                        } else {
                            phi_3796_ = -(_e275);
                        }
                        let _e280 = phi_3796_;
                        phi_3797_ = vec2<f32>((_e280 * 0.15915494f), _e258);
                        break;
                    } else {
                        if (_e249 == 2f) {
                            phi_3795_ = _e245.x;
                        } else {
                            phi_3795_ = length(_e245);
                        }
                        let _e266 = phi_3795_;
                        phi_3797_ = vec2<f32>(((clamp(_e266, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e242.z < 0f))) + ((max(0f, _e242.z) * 0.001953125f) + 0.0009765625f)), _e258);
                        break;
                    }
                }
            }
            let _e284 = phi_3797_;
            let _e285 = textureSampleLevel(YC, V8_, _e284, 0f);
            phi_3820_ = _e285;
            if !(_e226) {
                let _e289 = (_e285.xyz * _e285.w);
                phi_3820_ = vec4<f32>(_e289.x, _e289.y, _e289.z, (_e285.w * abs(bitcast<f32>(_e148.w))));
            }
            let _e299 = phi_3820_;
            phi_4526_ = 0u;
            phi_3818_ = _e299;
        }
        let _e312 = phi_4526_;
        let _e314 = phi_3818_;
        phi_4524_ = _e314;
        if _e226 {
            phi_4511_ = _e314;
            if ((_e314.w * _e220) != 0f) {
                let _e320 = p0_.z2_[_e120];
                let _e321 = unpack4x8unorm(_e320);
                let _e322 = _e314.xyz;
                local_2 = _e322;
                let _e323 = _e321.xyz;
                if (_e321.w != 0f) {
                    phi_3827_ = (1f / _e321.w);
                } else {
                    phi_3827_ = 0f;
                }
                let _e328 = phi_3827_;
                let _e329 = (_e323 * _e328);
                local = _e329;
                switch bitcast<i32>(_e224) {
                    case 11: {
                        let _e613 = local_2;
                        local_1 = (_e613 * _e329);
                        break;
                    }
                    case 1: {
                        let _e609 = local_2;
                        local_1 = ((_e609 + _e329) - (_e609 * _e329));
                        break;
                    }
                    case 2: {
                        let _e601 = local_2;
                        let _e602 = (_e601 * _e329);
                        local_1 = (select(_e602, (((_e601 + _e329) - _e602) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e329 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e599 = local_2;
                        local_1 = min(_e599, _e329);
                        break;
                    }
                    case 4: {
                        let _e597 = local_2;
                        local_1 = max(_e597, _e329);
                        break;
                    }
                    case 5: {
                        let _e568 = clamp(_e323, vec3<f32>(0f, 0f, 0f), _e321.www);
                        let _e574 = vec4<f32>(_e568.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e580 = vec4<f32>(_e574.x, _e568.y, _e574.z, _e574.w);
                        let _e587 = local_2;
                        let _e590 = (clamp((vec3<f32>(1f, 1f, 1f) - _e587), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e321.w);
                        let _e591 = vec4<f32>(_e580.x, _e580.y, _e568.z, _e580.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e591 / _e590)), sign(_e591), (_e590 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e529 = local_2;
                        local_2 = clamp(_e529, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e532 = clamp(_e323, vec3<f32>(0f, 0f, 0f), _e321.www);
                        let _e538 = vec4<f32>(_e532.x, _e321.y, _e321.z, _e321.w);
                        let _e544 = vec4<f32>(_e538.x, _e532.y, _e538.z, _e538.w);
                        phi_4345_ = vec4<f32>(_e544.x, _e544.y, _e532.z, _e544.w);
                        if (_e321.w == 0f) {
                            phi_4345_ = vec4<f32>(_e532.x, _e532.y, _e532.z, 1f);
                        }
                        let _e554 = phi_4345_;
                        let _e558 = (vec3(_e554.w) - _e554.xyz);
                        let _e559 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e558 / (_e559 * _e554.w))), sign(_e558), (_e559 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e521 = local_2;
                        let _e522 = (_e521 * _e329);
                        local_1 = (select(_e522, (((_e521 + _e329) - _e522) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e521 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_4277_ = 0i;
                        loop {
                            let _e490 = phi_4277_;
                            if (_e490 < 3i) {
                                let _e493 = local_2[_e490];
                                if (_e493 <= 0.5f) {
                                    let _e510 = local[_e490];
                                    local_1[_e490] = (1f - _e510);
                                } else {
                                    let _e496 = local[_e490];
                                    if (_e496 <= 0.25f) {
                                        let _e502 = local[_e490];
                                        let _e505 = local[_e490];
                                        local_1[_e490] = ((((16f * _e502) - 12f) * _e505) + 3f);
                                    } else {
                                        let _e498 = local[_e490];
                                        local_1[_e490] = (inverseSqrt(_e498) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_4277_ = (_e490 + 1i);
                            }
                        }
                        let _e514 = local_2;
                        let _e518 = local_1;
                        local_1 = (_e329 + ((_e329 * ((_e514 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e518));
                        break;
                    }
                    case 9: {
                        let _e486 = local_2;
                        local_1 = abs((_e329 - _e486));
                        break;
                    }
                    case 10: {
                        let _e481 = local_2;
                        local_1 = ((_e481 + _e329) - ((_e481 * 2f) * _e329));
                        break;
                    }
                    case 12: {
                        if Bj {
                            let _e432 = local_2;
                            let _e433 = clamp(_e432, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e433;
                            let _e448 = (_e433 - vec3(min(min(_e433.x, _e433.y), _e433.z)));
                            let _e456 = (_e448 * ((max(max(_e329.x, _e329.y), _e329.z) - min(min(_e329.x, _e329.y), _e329.z)) / max(0.000062f, max(max(_e448.x, _e448.y), _e448.z))));
                            let _e457 = dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e460 = (_e456 - vec3(dot(_e456, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e473 = (vec2<f32>(_e457, (1f - _e457)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e460.x, _e460.y), _e460.z)), max(max(_e460.x, _e460.y), _e460.z))));
                            local_1 = ((_e460 * min(1f, min(_e473.x, _e473.y))) + vec3(_e457));
                        }
                        break;
                    }
                    case 13: {
                        if Bj {
                            let _e383 = local_2;
                            let _e384 = clamp(_e383, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e384;
                            let _e399 = (_e329 - vec3(min(min(_e329.x, _e329.y), _e329.z)));
                            let _e407 = (_e399 * ((max(max(_e384.x, _e384.y), _e384.z) - min(min(_e384.x, _e384.y), _e384.z)) / max(0.000062f, max(max(_e399.x, _e399.y), _e399.z))));
                            let _e408 = dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e411 = (_e407 - vec3(dot(_e407, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e424 = (vec2<f32>(_e408, (1f - _e408)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e411.x, _e411.y), _e411.z)), max(max(_e411.x, _e411.y), _e411.z))));
                            local_1 = ((_e411 * min(1f, min(_e424.x, _e424.y))) + vec3(_e408));
                        }
                        break;
                    }
                    case 14: {
                        if Bj {
                            let _e357 = local_2;
                            let _e358 = clamp(_e357, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e358;
                            let _e359 = dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e362 = (_e358 - vec3(dot(_e358, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e375 = (vec2<f32>(_e359, (1f - _e359)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e362.x, _e362.y), _e362.z)), max(max(_e362.x, _e362.y), _e362.z))));
                            local_1 = ((_e362 * min(1f, min(_e375.x, _e375.y))) + vec3(_e359));
                        }
                        break;
                    }
                    case 15: {
                        if Bj {
                            let _e331 = local_2;
                            let _e332 = clamp(_e331, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e332;
                            let _e333 = dot(_e332, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e336 = (_e329 - vec3(dot(_e329, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e349 = (vec2<f32>(_e333, (1f - _e333)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e336.x, _e336.y), _e336.z)), max(max(_e336.x, _e336.y), _e336.z))));
                            local_1 = ((_e336 * min(1f, min(_e349.x, _e349.y))) + vec3(_e333));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e615 = local_1;
                let _e617 = mix(_e322, _e615, vec3(_e321.w));
                let _e623 = vec4<f32>(_e617.x, _e314.y, _e314.z, _e314.w);
                let _e629 = vec4<f32>(_e623.x, _e617.y, _e623.z, _e623.w);
                phi_4511_ = vec4<f32>(_e629.x, _e629.y, _e617.z, _e629.w);
            }
            let _e637 = phi_4511_;
            let _e640 = (_e637.xyz * _e637.w);
            let _e646 = vec4<f32>(_e640.x, _e637.y, _e637.z, _e637.w);
            let _e652 = vec4<f32>(_e646.x, _e640.y, _e646.z, _e646.w);
            phi_4524_ = vec4<f32>(_e652.x, _e652.y, _e640.z, _e652.w);
        }
        let _e660 = phi_4524_;
        phi_4557_ = _e312;
        phi_4551_ = (_e660 * _e220);
    }
    let _e663 = phi_4557_;
    let _e665 = phi_4551_;
    let _e666 = _e665.xyz;
    let _e669 = j.K3_;
    let _e671 = j.L3_;
    if (Cj && (_e665.w != 0f)) {
        phi_4552_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e85.x) + (0.00583715f * _e85.y))))) * _e669) + _e671)) + _e666);
    } else {
        phi_4552_ = _e666;
    }
    let _e687 = phi_4552_;
    let _e693 = vec4<f32>(_e687.x, _e665.y, _e665.z, _e665.w);
    let _e699 = vec4<f32>(_e693.x, _e687.y, _e693.z, _e693.w);
    let _e705 = vec4<f32>(_e699.x, _e699.y, _e687.z, _e699.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e687.x + _e687.y) + _e687.z) + _e665.w) == 0f) {
                break;
            }
            phi_4554_ = _e705;
            if (_e665.w != 1f) {
                let _e714 = p0_.z2_[_e120];
                phi_4554_ = (_e705 + (unpack4x8unorm(_e714) * (1f - _e665.w)));
            }
            let _e720 = phi_4554_;
            p0_.z2_[_e120] = pack4x8unorm(_e720);
            break;
        }
    }
    if (_e663 != 0u) {
        n0_.z2_[_e120] = _e663;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) J0_: u32, @location(0) @interpolate(flat, either) r1_: f32) {
    gl_FragCoord_1 = gl_FragCoord;
    J0_1 = J0_;
    r1_1 = r1_;
    main_1();
}
