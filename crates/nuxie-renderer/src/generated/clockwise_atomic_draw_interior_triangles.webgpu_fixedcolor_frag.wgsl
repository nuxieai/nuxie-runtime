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

struct Sf {
    z2_: array<u32>,
}

struct n0hf {
    z2_: array<u32>,
}

struct Sf_1 {
    z2_: array<atomic<u32>>,
}

@id(8) override Cj: bool = true;
@id(9) override Dj: bool = true;
@id(2) override wj: bool = true;
@id(4) override yj: bool = true;
@id(1) override vj: bool = true;
@id(0) override uj: bool = true;

@group(0) @binding(0)
var<uniform> j: RB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var V8_: sampler;
@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(1) @binding(12)
var x4_: sampler;
@group(0) @binding(5)
var<storage, read_write> c1_: Sf_1;
var<private> gl_FragCoord_1: vec4<f32>;
var<private> O0_1: vec3<f32>;
var<private> S0_1: f32;
var<private> N0_1: vec4<f32>;
var<private> r1_1: f32;
var<private> V4_1: vec2<f32>;
var<private> D3_1: vec2<u32>;
var<private> Y0_1: vec4<f32>;
var<private> l2_1: vec2<f32>;
@group(2) @binding(1)
var<storage, read_write> n0_: n0hf;
var<private> S1_: vec4<f32>;
@group(3) @binding(8)
var ib: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
var<private> J0_1: f32;

fn main_1() {
    var phi_1603_: f32;
    var phi_1604_: f32;
    var phi_1605_: vec2<f32>;
    var phi_1622_: vec4<f32>;
    var phi_1621_: vec4<f32>;
    var phi_1025_: bool;
    var phi_1040_: bool;
    var phi_1606_: f32;
    var phi_1617_: vec4<f32>;
    var phi_1624_: vec4<f32>;
    var phi_1625_: vec4<f32>;
    var phi_1627_: f32;
    var phi_807_: bool;
    var phi_1628_: f32;
    var phi_1330_: bool;
    var phi_1332_: bool;
    var phi_1649_: f32;
    var phi_1644_: u32;
    var phi_1641_: f32;
    var phi_1648_: f32;
    var phi_1643_: u32;
    var phi_1640_: f32;
    var phi_1645_: f32;
    var phi_1642_: u32;
    var phi_1639_: f32;
    var phi_1650_: f32;
    var phi_1651_: f32;
    var phi_1652_: f32;
    var phi_1662_: f32;
    var phi_1685_: vec3<f32>;

    let _e63 = gl_FragCoord_1;
    let _e67 = bitcast<vec2<u32>>(vec2<i32>(floor(_e63.xy)));
    let _e69 = j.V5_;
    let _e98 = bitcast<i32>((((((_e67.y >> bitcast<u32>(5u)) * (((_e69 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e67.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e67.x & 28u) << bitcast<u32>(5u)) + ((_e67.y & 28u) << bitcast<u32>(2i)))) + (((_e67.y & 3u) << bitcast<u32>(2i)) + (_e67.x & 3u))));
    let _e99 = S0_1;
    let _e101 = O0_1;
    let _e102 = N0_1;
    switch bitcast<i32>(0u) {
        default: {
            let _e105 = (wj && (u32(_e99) != 0u));
            if (_e102.w >= 0f) {
                phi_1621_ = _e102;
            } else {
                let _e109 = j.T8_;
                let _e111 = j.U8_;
                let _e114 = abs(_e102.z);
                let _e115 = floor(_e114);
                let _e118 = (((_e114 - _e115) * 8f) + 0.0009765625f);
                let _e120 = ((_e115 * _e109) + _e111);
                let _e121 = floor(_e118);
                let _e122 = _e102.xy;
                switch bitcast<i32>(0u) {
                    default: {
                        if (Dj && (_e121 == 4f)) {
                            let _e136 = normalize(_e122);
                            let _e139 = acos(clamp(_e136.x, -1f, 1f));
                            if (_e136.y >= 0f) {
                                phi_1604_ = _e139;
                            } else {
                                phi_1604_ = -(_e139);
                            }
                            let _e144 = phi_1604_;
                            phi_1605_ = vec2<f32>((_e144 * 0.15915494f), _e120);
                            break;
                        } else {
                            if (_e121 == 2f) {
                                phi_1603_ = _e102.x;
                            } else {
                                phi_1603_ = length(_e122);
                            }
                            let _e130 = phi_1603_;
                            phi_1605_ = vec2<f32>(((clamp(_e130, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e102.z < 0f))) + _e118), _e120);
                            break;
                        }
                    }
                }
                let _e148 = phi_1605_;
                let _e149 = textureSampleLevel(YC, V8_, _e148, 0f);
                phi_1622_ = _e149;
                if !(_e105) {
                    let _e155 = (_e149.xyz * _e149.w);
                    phi_1622_ = vec4<f32>(_e155.x, _e155.y, _e155.z, (_e149.w * (ceil(_e102.w) * -0.003921569f)));
                }
                let _e162 = phi_1622_;
                phi_1621_ = _e162;
            }
            let _e164 = phi_1621_;
            phi_1025_ = yj;
            if yj {
                phi_1025_ = (_e101.z < 0f);
            }
            let _e168 = phi_1025_;
            if _e168 {
                let _e170 = textureSampleLevel(SB, x4_, _e101.xy, 0f);
                phi_1625_ = _e170;
                break;
            }
            phi_1040_ = yj;
            if yj {
                phi_1040_ = (_e101.z > 0f);
            }
            let _e174 = phi_1040_;
            phi_1624_ = _e164;
            if _e174 {
                let _e178 = textureSampleLevel(SB, x4_, _e101.xy, (_e101.z - 1f));
                phi_1617_ = _e178;
                if _e105 {
                    if (_e178.w != 0f) {
                        phi_1606_ = (1f / _e178.w);
                    } else {
                        phi_1606_ = 0f;
                    }
                    let _e184 = phi_1606_;
                    let _e185 = (_e178.xyz * _e184);
                    phi_1617_ = vec4<f32>(_e185.x, _e185.y, _e185.z, _e178.w);
                }
                let _e191 = phi_1617_;
                phi_1624_ = (_e164 * _e191);
            }
            let _e194 = phi_1624_;
            phi_1625_ = _e194;
            break;
        }
    }
    let _e196 = phi_1625_;
    let _e197 = r1_1;
    let _e198 = V4_1;
    let _e201 = D3_1[1u];
    let _e203 = D3_1[0u];
    let _e204 = vec2<u32>(floor(_e198));
    phi_1627_ = 1f;
    if vj {
        let _e232 = Y0_1;
        let _e235 = min(_e232.xy, _e232.zw);
        phi_1627_ = min(min(_e235.x, _e235.y), 1f);
    }
    let _e241 = phi_1627_;
    phi_807_ = uj;
    if uj {
        let _e243 = l2_1[0u];
        phi_807_ = (_e243 != 0f);
    }
    let _e246 = phi_807_;
    phi_1628_ = _e241;
    if _e246 {
        let _e249 = n0_.z2_[_e98];
        phi_1628_ = min(unpack4x8unorm(_e249).x, _e241);
    }
    let _e254 = phi_1628_;
    let _e256 = clamp(_e197, 0f, max(_e254, 0f));
    switch bitcast<i32>(0u) {
        default: {
            let _e262 = u32(((abs(_e256) * 1024f) + 0.5f));
            let _e265 = atomicLoad((&c1_.z2_[(_e203 + (((((_e204.y >> bitcast<u32>(5u)) * (_e201 << bitcast<u32>(5u))) + ((_e204.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e204.x & 28u) << bitcast<u32>(5u)) + ((_e204.y & 28u) << bitcast<u32>(2i)))) + (((_e204.y & 3u) << bitcast<u32>(2i)) + (_e204.x & 3u))))]));
            let _e267 = (min(_e196.w, _e256) >= 1f);
            phi_1332_ = _e267;
            if _e267 {
                let _e269 = j.y2_;
                let _e270 = (_e265 < _e269);
                phi_1330_ = _e270;
                if !(_e270) {
                    phi_1330_ = (_e265 >= (_e269 | 262144u));
                }
                let _e275 = phi_1330_;
                phi_1332_ = _e275;
            }
            let _e277 = phi_1332_;
            if _e277 {
                phi_1651_ = 1f;
                break;
            }
            let _e279 = j.y2_;
            phi_1645_ = 0f;
            phi_1642_ = _e262;
            phi_1639_ = _e256;
            if (_e265 < _e279) {
                let _e282 = (_e279 | (262144u + _e262));
                let _e283 = atomicMax((&c1_.z2_[(_e203 + (((((_e204.y >> bitcast<u32>(5u)) * (_e201 << bitcast<u32>(5u))) + ((_e204.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e204.x & 28u) << bitcast<u32>(5u)) + ((_e204.y & 28u) << bitcast<u32>(2i)))) + (((_e204.y & 3u) << bitcast<u32>(2i)) + (_e204.x & 3u))))]), _e282);
                if (_e283 <= _e279) {
                    phi_1648_ = min(_e256, 1f);
                    phi_1643_ = _e262;
                    phi_1640_ = 0f;
                } else {
                    phi_1649_ = 0f;
                    phi_1644_ = _e262;
                    phi_1641_ = _e256;
                    if (_e283 < _e282) {
                        let _e287 = ((_e283 & 524287u) - 262144u);
                        let _e289 = (f32(_e287) * 0.0009765625f);
                        phi_1649_ = ((min(_e256, 1f) - _e289) / max((1f - (_e289 * _e196.w)), 0.000062f));
                        phi_1644_ = _e287;
                        phi_1641_ = _e289;
                    }
                    let _e297 = phi_1649_;
                    let _e299 = phi_1644_;
                    let _e301 = phi_1641_;
                    phi_1648_ = _e297;
                    phi_1643_ = _e299;
                    phi_1640_ = _e301;
                }
                let _e304 = phi_1648_;
                let _e306 = phi_1643_;
                let _e308 = phi_1640_;
                phi_1645_ = _e304;
                phi_1642_ = _e306;
                phi_1639_ = _e308;
            }
            let _e310 = phi_1645_;
            let _e312 = phi_1642_;
            let _e314 = phi_1639_;
            phi_1650_ = _e310;
            if (_e314 > 0f) {
                let _e316 = atomicAdd((&c1_.z2_[(_e203 + (((((_e204.y >> bitcast<u32>(5u)) * (_e201 << bitcast<u32>(5u))) + ((_e204.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e204.x & 28u) << bitcast<u32>(5u)) + ((_e204.y & 28u) << bitcast<u32>(2i)))) + (((_e204.y & 3u) << bitcast<u32>(2i)) + (_e204.x & 3u))))]), _e312);
                let _e321 = (f32(bitcast<i32>(((_e316 & 524287u) - 262144u))) * 0.0009765625f);
                let _e323 = clamp(_e321, 0f, 1f);
                phi_1650_ = (_e310 + ((1f - (_e310 * _e196.w)) * ((clamp((_e321 + _e314), 0f, 1f) - _e323) / max((1f - (_e323 * _e196.w)), 0.000062f))));
            }
            let _e335 = phi_1650_;
            phi_1651_ = _e335;
            break;
        }
    }
    let _e337 = phi_1651_;
    phi_1662_ = f32();
    if Cj {
        let _e339 = j.K3_;
        let _e341 = j.L3_;
        if Cj {
            phi_1652_ = ((fract((52.982918f * fract(((0.06711056f * _e63.x) + (0.00583715f * _e63.y))))) * _e339) + _e341);
        } else {
            phi_1652_ = 0f;
        }
        let _e353 = phi_1652_;
        phi_1662_ = _e353;
    }
    let _e355 = phi_1662_;
    let _e356 = (_e196 * _e337);
    let _e357 = _e356.xyz;
    if (Cj && (_e356.w != 0f)) {
        phi_1685_ = (vec3(_e355) + _e357);
    } else {
        phi_1685_ = _e357;
    }
    let _e364 = phi_1685_;
    let _e370 = vec4<f32>(_e364.x, _e356.y, _e356.z, _e356.w);
    let _e376 = vec4<f32>(_e370.x, _e364.y, _e370.z, _e370.w);
    n0_.z2_[_e98] = pack4x8unorm(vec4<f32>(0f, 0f, 0f, 0f));
    S1_ = vec4<f32>(_e376.x, _e376.y, _e364.z, _e376.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(9) O0_: vec3<f32>, @location(6) @interpolate(flat, either) S0_: f32, @location(0) N0_: vec4<f32>, @location(1) @interpolate(flat, either) r1_: f32, @location(8) V4_: vec2<f32>, @location(7) @interpolate(flat, either) D3_: vec2<u32>, @location(5) Y0_: vec4<f32>, @location(4) @interpolate(flat, either) l2_: vec2<f32>, @location(3) @interpolate(flat, either) J0_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    O0_1 = O0_;
    S0_1 = S0_;
    N0_1 = N0_;
    r1_1 = r1_;
    V4_1 = V4_;
    D3_1 = D3_;
    Y0_1 = Y0_;
    l2_1 = l2_;
    J0_1 = J0_;
    main_1();
    let _e21 = S1_;
    return _e21;
}
