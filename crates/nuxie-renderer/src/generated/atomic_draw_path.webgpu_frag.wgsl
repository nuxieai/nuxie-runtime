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

struct d5hf_1 {
    z2_: array<atomic<u32>>,
}

@id(8) override Cj: bool = true;
@id(7) override Bj: bool = true;
@id(9) override Dj: bool = true;
@id(5) override zj: bool = true;
@id(0) override uj: bool = true;
@id(1) override vj: bool = true;
@id(2) override wj: bool = true;
@id(3) override xj: bool = true;

@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var ib: sampler;
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
var<private> S_1: vec4<f32>;
var<private> J0_1: u32;
@group(2) @binding(3)
var<storage, read_write> d5_: d5hf_1;
@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(1) @binding(12)
var x4_: sampler;

fn main_1() {
    var local: vec3<f32>;
    var local_1: vec3<f32>;
    var local_2: vec3<f32>;
    var phi_1595_: bool;
    var phi_1608_: bool;
    var phi_4311_: f32;
    var phi_4319_: f32;
    var phi_4327_: f32;
    var phi_4326_: f32;
    var phi_2116_: bool;
    var phi_4330_: f32;
    var phi_4329_: f32;
    var phi_4331_: f32;
    var phi_4334_: f32;
    var phi_4333_: f32;
    var phi_2153_: bool;
    var phi_4338_: f32;
    var phi_4335_: f32;
    var phi_4336_: f32;
    var phi_4337_: vec2<f32>;
    var phi_4370_: vec4<f32>;
    var phi_5282_: u32;
    var phi_5281_: u32;
    var phi_4368_: vec4<f32>;
    var phi_4377_: f32;
    var phi_4967_: i32;
    var phi_5055_: vec4<f32>;
    var phi_5266_: vec4<f32>;
    var phi_5279_: vec4<f32>;
    var phi_5313_: u32;
    var phi_5306_: vec4<f32>;
    var phi_5308_: vec3<f32>;
    var phi_5310_: vec4<f32>;

    let _e101 = gl_FragCoord_1;
    let _e102 = _e101.xy;
    let _e105 = bitcast<vec2<u32>>(vec2<i32>(floor(_e102)));
    let _e107 = j.V5_;
    let _e136 = bitcast<i32>((((((_e105.y >> bitcast<u32>(5u)) * (((_e107 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e105.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e105.x & 28u) << bitcast<u32>(5u)) + ((_e105.y & 28u) << bitcast<u32>(2i)))) + (((_e105.y & 3u) << bitcast<u32>(2i)) + (_e105.x & 3u))));
    phi_1595_ = xj;
    if xj {
        let _e137 = S_1;
        phi_1595_ = (_e137.x < -1.5f);
    }
    let _e141 = phi_1595_;
    if _e141 {
        let _e214 = S_1;
        let _e218 = textureSampleLevel(ZC, ib, vec2<f32>((3f + _e214.x), 0f), 0f);
        let _e224 = textureSampleLevel(ZC, ib, vec2<f32>((1f - _e214.y), 0f), 0f);
        phi_4326_ = ((1f - _e218.x) - _e224.x);
    } else {
        phi_1608_ = xj;
        if xj {
            let _e142 = S_1;
            phi_1608_ = (_e142.y < -1.5f);
        }
        let _e146 = phi_1608_;
        if _e146 {
            let _e154 = S_1;
            let _e157 = max(_e154.w, 0f);
            if (_e154.z >= 0f) {
                let _e160 = textureSampleLevel(ZC, ib, vec2<f32>(_e157, 0f), 0f);
                phi_4311_ = _e160.x;
            } else {
                phi_4311_ = 0f;
            }
            let _e163 = phi_4311_;
            phi_4319_ = _e163;
            if (abs(_e154.z) < 1000f) {
                let _e170 = (-2f - _e154.y);
                let _e172 = ((_e170 - _e157) * 0.5984134f);
                let _e175 = (vec4(_e157) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e172));
                let _e181 = ((_e175 * -(_e154.z)) + vec4(((_e170 * _e154.z) + (abs(_e154.x) - 0.25f))));
                let _e184 = textureSampleLevel(ZC, ib, vec2<f32>(_e181.x, 0f), 0f);
                let _e187 = textureSampleLevel(ZC, ib, vec2<f32>(_e181.y, 0f), 0f);
                let _e190 = textureSampleLevel(ZC, ib, vec2<f32>(_e181.z, 0f), 0f);
                let _e193 = textureSampleLevel(ZC, ib, vec2<f32>(_e181.w, 0f), 0f);
                let _e199 = (_e175 * 5.0959306f);
                phi_4319_ = (_e163 + (dot(vec4<f32>(_e184.x, _e187.x, _e190.x, _e193.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e199) * (_e199 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e172));
            }
            let _e208 = phi_4319_;
            phi_4327_ = (_e208 * sign(_e154.x));
        } else {
            let _e148 = S_1[0u];
            let _e150 = S_1[1u];
            phi_4327_ = min(min(_e148, abs(_e150)), 1f);
        }
        let _e213 = phi_4327_;
        phi_4326_ = _e213;
    }
    let _e228 = phi_4326_;
    let _e232 = u32(round(((_e228 * 2048f) + 65536f)));
    let _e233 = J0_1;
    let _e236 = ((_e233 << bitcast<u32>(17u)) | _e232);
    let _e239 = atomicMax((&d5_.z2_[_e136]), _e236);
    let _e241 = (_e239 >> bitcast<u32>(17u));
    if (_e241 == _e233) {
        let _e765 = S_1;
        if (_e765.y < 0f) {
            let _e772 = atomicAdd((&d5_.z2_[_e136]), ((_e232 + (_e239 - max(_e236, _e239))) - 65536u));
        }
        phi_5313_ = 0u;
        phi_5306_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e246 = ((f32((_e239 & 131071u)) * 0.00048828125f) + -32f);
        let _e251 = JB.z2_[((_e241 * 4u) + 1u)];
        phi_4329_ = _e246;
        if ((_e251.z & 768u) != 0u) {
            let _e255 = abs(_e246);
            phi_2116_ = zj;
            if zj {
                phi_2116_ = ((_e251.z & 512u) != 0u);
            }
            let _e259 = phi_2116_;
            phi_4330_ = _e255;
            if _e259 {
                phi_4330_ = (1f - abs(((fract((_e255 * 0.5f)) * 2f) + -1f)));
            }
            let _e267 = phi_4330_;
            phi_4329_ = _e267;
        }
        let _e269 = phi_4329_;
        let _e270 = clamp(_e269, 0f, 1f);
        phi_4333_ = _e270;
        if uj {
            let _e272 = (_e251.z >> bitcast<u32>(16u));
            phi_4334_ = _e270;
            if (_e272 != 0u) {
                let _e276 = n0_.z2_[_e136];
                if (_e272 == (_e276 >> bitcast<u32>(16i))) {
                    phi_4331_ = min(_e270, unpack2x16float(_e276).x);
                } else {
                    phi_4331_ = 0f;
                }
                let _e284 = phi_4331_;
                phi_4334_ = _e284;
            }
            let _e286 = phi_4334_;
            phi_4333_ = _e286;
        }
        let _e288 = phi_4333_;
        phi_2153_ = vj;
        if vj {
            phi_2153_ = ((_e251.z & 1024u) != 0u);
        }
        let _e292 = phi_2153_;
        phi_4338_ = _e288;
        if _e292 {
            let _e293 = (_e241 * 8u);
            let _e297 = KB.z2_[(_e293 + 2u)];
            let _e308 = KB.z2_[(_e293 + 3u)];
            let _e313 = _e308.zw;
            let _e315 = ((abs(((mat2x2<f32>(vec2<f32>(_e297.x, _e297.y), vec2<f32>(_e297.z, _e297.w)) * _e102) + _e308.xy)) * _e313) - _e313);
            phi_4338_ = min(_e288, clamp((min(_e315.x, _e315.y) + 0.5f), 0f, 1f));
        }
        let _e323 = phi_4338_;
        let _e324 = (_e251.z & 15u);
        let _e327 = ((_e251.z >> bitcast<u32>(4i)) & 15u);
        let _e329 = (wj && (_e327 != 0u));
        if (_e324 <= 1u) {
            let _e406 = (uj && (_e324 == 0u));
            phi_5282_ = 0u;
            if _e406 {
                phi_5282_ = (_e251.w | pack2x16float(vec2<f32>(_e323, 0f)));
            }
            let _e411 = phi_5282_;
            phi_5281_ = _e411;
            phi_4368_ = select(unpack4x8unorm(_e251.w), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e406));
        } else {
            let _e331 = (_e241 * 8u);
            let _e334 = KB.z2_[_e331];
            let _e345 = KB.z2_[(_e331 + 1u)];
            let _e348 = ((mat2x2<f32>(vec2<f32>(_e334.x, _e334.y), vec2<f32>(_e334.z, _e334.w)) * _e102) + _e345.xy);
            let _e352 = f32(_e324);
            let _e354 = j.T8_;
            let _e356 = j.U8_;
            let _e361 = ((_e345.w * _e354) + _e356);
            switch bitcast<i32>(0u) {
                default: {
                    if (Dj && (_e352 == 4f)) {
                        let _e375 = normalize(_e348);
                        let _e378 = acos(clamp(_e375.x, -1f, 1f));
                        if (_e375.y >= 0f) {
                            phi_4336_ = _e378;
                        } else {
                            phi_4336_ = -(_e378);
                        }
                        let _e383 = phi_4336_;
                        phi_4337_ = vec2<f32>((_e383 * 0.15915494f), _e361);
                        break;
                    } else {
                        if (_e352 == 2f) {
                            phi_4335_ = _e348.x;
                        } else {
                            phi_4335_ = length(_e348);
                        }
                        let _e369 = phi_4335_;
                        phi_4337_ = vec2<f32>(((clamp(_e369, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e345.z < 0f))) + ((max(0f, _e345.z) * 0.001953125f) + 0.0009765625f)), _e361);
                        break;
                    }
                }
            }
            let _e387 = phi_4337_;
            let _e388 = textureSampleLevel(YC, V8_, _e387, 0f);
            phi_4370_ = _e388;
            if !(_e329) {
                let _e392 = (_e388.xyz * _e388.w);
                phi_4370_ = vec4<f32>(_e392.x, _e392.y, _e392.z, (_e388.w * abs(bitcast<f32>(_e251.w))));
            }
            let _e402 = phi_4370_;
            phi_5281_ = 0u;
            phi_4368_ = _e402;
        }
        let _e415 = phi_5281_;
        let _e417 = phi_4368_;
        phi_5279_ = _e417;
        if _e329 {
            phi_5266_ = _e417;
            if ((_e417.w * _e323) != 0f) {
                let _e423 = p0_.z2_[_e136];
                let _e424 = unpack4x8unorm(_e423);
                let _e425 = _e417.xyz;
                local_2 = _e425;
                let _e426 = _e424.xyz;
                if (_e424.w != 0f) {
                    phi_4377_ = (1f / _e424.w);
                } else {
                    phi_4377_ = 0f;
                }
                let _e431 = phi_4377_;
                let _e432 = (_e426 * _e431);
                local = _e432;
                switch bitcast<i32>(_e327) {
                    case 11: {
                        let _e716 = local_2;
                        local_1 = (_e716 * _e432);
                        break;
                    }
                    case 1: {
                        let _e712 = local_2;
                        local_1 = ((_e712 + _e432) - (_e712 * _e432));
                        break;
                    }
                    case 2: {
                        let _e704 = local_2;
                        let _e705 = (_e704 * _e432);
                        local_1 = (select(_e705, (((_e704 + _e432) - _e705) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e432 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 3: {
                        let _e702 = local_2;
                        local_1 = min(_e702, _e432);
                        break;
                    }
                    case 4: {
                        let _e700 = local_2;
                        local_1 = max(_e700, _e432);
                        break;
                    }
                    case 5: {
                        let _e671 = clamp(_e426, vec3<f32>(0f, 0f, 0f), _e424.www);
                        let _e677 = vec4<f32>(_e671.x, vec4<f32>().y, vec4<f32>().z, vec4<f32>().w);
                        let _e683 = vec4<f32>(_e677.x, _e671.y, _e677.z, _e677.w);
                        let _e690 = local_2;
                        let _e693 = (clamp((vec3<f32>(1f, 1f, 1f) - _e690), vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f)) * _e424.w);
                        let _e694 = vec4<f32>(_e683.x, _e683.y, _e671.z, _e683.w).xyz;
                        local_1 = select(min(vec3<f32>(1f, 1f, 1f), (_e694 / _e693)), sign(_e694), (_e693 == vec3<f32>(0f, 0f, 0f)));
                        break;
                    }
                    case 6: {
                        let _e632 = local_2;
                        local_2 = clamp(_e632, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                        let _e635 = clamp(_e426, vec3<f32>(0f, 0f, 0f), _e424.www);
                        let _e641 = vec4<f32>(_e635.x, _e424.y, _e424.z, _e424.w);
                        let _e647 = vec4<f32>(_e641.x, _e635.y, _e641.z, _e641.w);
                        phi_5055_ = vec4<f32>(_e647.x, _e647.y, _e635.z, _e647.w);
                        if (_e424.w == 0f) {
                            phi_5055_ = vec4<f32>(_e635.x, _e635.y, _e635.z, 1f);
                        }
                        let _e657 = phi_5055_;
                        let _e661 = (vec3(_e657.w) - _e657.xyz);
                        let _e662 = local_2;
                        local_1 = (vec3<f32>(1f, 1f, 1f) - select(min(vec3<f32>(1f, 1f, 1f), (_e661 / (_e662 * _e657.w))), sign(_e661), (_e662 == vec3<f32>(0f, 0f, 0f))));
                        break;
                    }
                    case 7: {
                        let _e624 = local_2;
                        let _e625 = (_e624 * _e432);
                        local_1 = (select(_e625, (((_e624 + _e432) - _e625) - vec3<f32>(0.5f, 0.5f, 0.5f)), (_e624 > vec3<f32>(0.5f, 0.5f, 0.5f))) * 2f);
                        break;
                    }
                    case 8: {
                        phi_4967_ = 0i;
                        loop {
                            let _e593 = phi_4967_;
                            if (_e593 < 3i) {
                                let _e596 = local_2[_e593];
                                if (_e596 <= 0.5f) {
                                    let _e613 = local[_e593];
                                    local_1[_e593] = (1f - _e613);
                                } else {
                                    let _e599 = local[_e593];
                                    if (_e599 <= 0.25f) {
                                        let _e605 = local[_e593];
                                        let _e608 = local[_e593];
                                        local_1[_e593] = ((((16f * _e605) - 12f) * _e608) + 3f);
                                    } else {
                                        let _e601 = local[_e593];
                                        local_1[_e593] = (inverseSqrt(_e601) - 1f);
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                            continuing {
                                phi_4967_ = (_e593 + 1i);
                            }
                        }
                        let _e617 = local_2;
                        let _e621 = local_1;
                        local_1 = (_e432 + ((_e432 * ((_e617 * 2f) - vec3<f32>(1f, 1f, 1f))) * _e621));
                        break;
                    }
                    case 9: {
                        let _e589 = local_2;
                        local_1 = abs((_e432 - _e589));
                        break;
                    }
                    case 10: {
                        let _e584 = local_2;
                        local_1 = ((_e584 + _e432) - ((_e584 * 2f) * _e432));
                        break;
                    }
                    case 12: {
                        if Bj {
                            let _e535 = local_2;
                            let _e536 = clamp(_e535, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e536;
                            let _e551 = (_e536 - vec3(min(min(_e536.x, _e536.y), _e536.z)));
                            let _e559 = (_e551 * ((max(max(_e432.x, _e432.y), _e432.z) - min(min(_e432.x, _e432.y), _e432.z)) / max(0.000062f, max(max(_e551.x, _e551.y), _e551.z))));
                            let _e560 = dot(_e432, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e563 = (_e559 - vec3(dot(_e559, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e576 = (vec2<f32>(_e560, (1f - _e560)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e563.x, _e563.y), _e563.z)), max(max(_e563.x, _e563.y), _e563.z))));
                            local_1 = ((_e563 * min(1f, min(_e576.x, _e576.y))) + vec3(_e560));
                        }
                        break;
                    }
                    case 13: {
                        if Bj {
                            let _e486 = local_2;
                            let _e487 = clamp(_e486, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e487;
                            let _e502 = (_e432 - vec3(min(min(_e432.x, _e432.y), _e432.z)));
                            let _e510 = (_e502 * ((max(max(_e487.x, _e487.y), _e487.z) - min(min(_e487.x, _e487.y), _e487.z)) / max(0.000062f, max(max(_e502.x, _e502.y), _e502.z))));
                            let _e511 = dot(_e432, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e514 = (_e510 - vec3(dot(_e510, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e527 = (vec2<f32>(_e511, (1f - _e511)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e514.x, _e514.y), _e514.z)), max(max(_e514.x, _e514.y), _e514.z))));
                            local_1 = ((_e514 * min(1f, min(_e527.x, _e527.y))) + vec3(_e511));
                        }
                        break;
                    }
                    case 14: {
                        if Bj {
                            let _e460 = local_2;
                            let _e461 = clamp(_e460, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e461;
                            let _e462 = dot(_e432, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e465 = (_e461 - vec3(dot(_e461, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e478 = (vec2<f32>(_e462, (1f - _e462)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e465.x, _e465.y), _e465.z)), max(max(_e465.x, _e465.y), _e465.z))));
                            local_1 = ((_e465 * min(1f, min(_e478.x, _e478.y))) + vec3(_e462));
                        }
                        break;
                    }
                    case 15: {
                        if Bj {
                            let _e434 = local_2;
                            let _e435 = clamp(_e434, vec3<f32>(0f, 0f, 0f), vec3<f32>(1f, 1f, 1f));
                            local_2 = _e435;
                            let _e436 = dot(_e435, vec3<f32>(0.3f, 0.59f, 0.11f));
                            let _e439 = (_e432 - vec3(dot(_e432, vec3<f32>(0.3f, 0.59f, 0.11f))));
                            let _e452 = (vec2<f32>(_e436, (1f - _e436)) / max(vec2<f32>(0.000062f, 0.000062f), vec2<f32>(-(min(min(_e439.x, _e439.y), _e439.z)), max(max(_e439.x, _e439.y), _e439.z))));
                            local_1 = ((_e439 * min(1f, min(_e452.x, _e452.y))) + vec3(_e436));
                        }
                        break;
                    }
                    default: {
                    }
                }
                let _e718 = local_1;
                let _e720 = mix(_e425, _e718, vec3(_e424.w));
                let _e726 = vec4<f32>(_e720.x, _e417.y, _e417.z, _e417.w);
                let _e732 = vec4<f32>(_e726.x, _e720.y, _e726.z, _e726.w);
                phi_5266_ = vec4<f32>(_e732.x, _e732.y, _e720.z, _e732.w);
            }
            let _e740 = phi_5266_;
            let _e743 = (_e740.xyz * _e740.w);
            let _e749 = vec4<f32>(_e743.x, _e740.y, _e740.z, _e740.w);
            let _e755 = vec4<f32>(_e749.x, _e743.y, _e749.z, _e749.w);
            phi_5279_ = vec4<f32>(_e755.x, _e755.y, _e743.z, _e755.w);
        }
        let _e763 = phi_5279_;
        phi_5313_ = _e415;
        phi_5306_ = (_e763 * _e323);
    }
    let _e774 = phi_5313_;
    let _e776 = phi_5306_;
    let _e777 = _e776.xyz;
    let _e780 = j.K3_;
    let _e782 = j.L3_;
    if (Cj && (_e776.w != 0f)) {
        phi_5308_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e101.x) + (0.00583715f * _e101.y))))) * _e780) + _e782)) + _e777);
    } else {
        phi_5308_ = _e777;
    }
    let _e798 = phi_5308_;
    let _e804 = vec4<f32>(_e798.x, _e776.y, _e776.z, _e776.w);
    let _e810 = vec4<f32>(_e804.x, _e798.y, _e804.z, _e804.w);
    let _e816 = vec4<f32>(_e810.x, _e810.y, _e798.z, _e810.w);
    switch bitcast<i32>(0u) {
        default: {
            if ((((_e798.x + _e798.y) + _e798.z) + _e776.w) == 0f) {
                break;
            }
            phi_5310_ = _e816;
            if (_e776.w != 1f) {
                let _e825 = p0_.z2_[_e136];
                phi_5310_ = (_e816 + (unpack4x8unorm(_e825) * (1f - _e776.w)));
            }
            let _e831 = phi_5310_;
            p0_.z2_[_e136] = pack4x8unorm(_e831);
            break;
        }
    }
    if (_e774 != 0u) {
        n0_.z2_[_e136] = _e774;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) S: vec4<f32>, @location(1) @interpolate(flat, either) J0_: u32) {
    gl_FragCoord_1 = gl_FragCoord;
    S_1 = S;
    J0_1 = J0_;
    main_1();
}
