struct jd {
    x2_: array<vec4<u32>>,
}

struct m0cf {
    x2_: array<u32>,
}

struct ug {
    x2_: array<vec4<f32>>,
}

struct WB {
    Gd: f32,
    Pe: f32,
    Rg: f32,
    Sg: f32,
    R6_: u32,
    Fa: u32,
    Dg: u32,
    Eg: u32,
    H8_: vec4<i32>,
    Ji: vec2<f32>,
    Qe: vec2<f32>,
    w2_: u32,
    Ni: f32,
    w6_: u32,
    j3_: f32,
    Re: f32,
    wg: u32,
    I3_: f32,
    J3_: f32,
    Se: f32,
    Gi: u32,
    Ea: u32,
    O8_: f32,
    P8_: f32,
}

struct W4cf {
    x2_: array<u32>,
}

struct W4cf_1 {
    x2_: array<atomic<u32>>,
}

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;
@id(3) override mj: bool = true;

@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(2)
var<storage> JB: jd;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(0) @binding(3)
var<storage> KB: ug;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var Q8_: sampler;
var<private> S_1: vec4<f32>;
var<private> I0_1: u32;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf_1;
var<private> Q1_: vec4<f32>;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(1) @binding(12)
var v4_: sampler;

fn main_1() {
    var phi_942_: bool;
    var phi_955_: bool;
    var phi_1946_: f32;
    var phi_1954_: f32;
    var phi_1962_: f32;
    var phi_1961_: f32;
    var phi_1455_: bool;
    var phi_1965_: f32;
    var phi_1964_: f32;
    var phi_1966_: f32;
    var phi_1969_: f32;
    var phi_1968_: f32;
    var phi_1492_: bool;
    var phi_1973_: f32;
    var phi_1970_: f32;
    var phi_1971_: f32;
    var phi_1972_: vec2<f32>;
    var phi_2011_: vec4<f32>;
    var phi_2013_: u32;
    var phi_2012_: u32;
    var phi_2009_: vec4<f32>;
    var phi_2031_: u32;
    var phi_2026_: vec4<f32>;
    var phi_2028_: vec3<f32>;

    let _e83 = gl_FragCoord_1;
    let _e84 = _e83.xy;
    let _e87 = bitcast<vec2<u32>>(vec2<i32>(floor(_e84)));
    let _e89 = j.R6_;
    let _e118 = bitcast<i32>((((((_e87.y >> bitcast<u32>(5u)) * (((_e89 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e87.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e87.x & 28u) << bitcast<u32>(5u)) + ((_e87.y & 28u) << bitcast<u32>(2i)))) + (((_e87.y & 3u) << bitcast<u32>(2i)) + (_e87.x & 3u))));
    phi_942_ = mj;
    if mj {
        let _e119 = S_1;
        phi_942_ = (_e119.x < -1.5f);
    }
    let _e123 = phi_942_;
    if _e123 {
        let _e196 = S_1;
        let _e200 = textureSampleLevel(ZC, fb, vec2<f32>((3f + _e196.x), 0f), 0f);
        let _e206 = textureSampleLevel(ZC, fb, vec2<f32>((1f - _e196.y), 0f), 0f);
        phi_1961_ = ((1f - _e200.x) - _e206.x);
    } else {
        phi_955_ = mj;
        if mj {
            let _e124 = S_1;
            phi_955_ = (_e124.y < -1.5f);
        }
        let _e128 = phi_955_;
        if _e128 {
            let _e136 = S_1;
            let _e139 = max(_e136.w, 0f);
            if (_e136.z >= 0f) {
                let _e142 = textureSampleLevel(ZC, fb, vec2<f32>(_e139, 0f), 0f);
                phi_1946_ = _e142.x;
            } else {
                phi_1946_ = 0f;
            }
            let _e145 = phi_1946_;
            phi_1954_ = _e145;
            if (abs(_e136.z) < 1000f) {
                let _e152 = (-2f - _e136.y);
                let _e154 = ((_e152 - _e139) * 0.5984134f);
                let _e157 = (vec4(_e139) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e154));
                let _e163 = ((_e157 * -(_e136.z)) + vec4(((_e152 * _e136.z) + (abs(_e136.x) - 0.25f))));
                let _e166 = textureSampleLevel(ZC, fb, vec2<f32>(_e163.x, 0f), 0f);
                let _e169 = textureSampleLevel(ZC, fb, vec2<f32>(_e163.y, 0f), 0f);
                let _e172 = textureSampleLevel(ZC, fb, vec2<f32>(_e163.z, 0f), 0f);
                let _e175 = textureSampleLevel(ZC, fb, vec2<f32>(_e163.w, 0f), 0f);
                let _e181 = (_e157 * 5.0959306f);
                phi_1954_ = (_e145 + (dot(vec4<f32>(_e166.x, _e169.x, _e172.x, _e175.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e181) * (_e181 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e154));
            }
            let _e190 = phi_1954_;
            phi_1962_ = (_e190 * sign(_e136.x));
        } else {
            let _e130 = S_1[0u];
            let _e132 = S_1[1u];
            phi_1962_ = min(min(_e130, abs(_e132)), 1f);
        }
        let _e195 = phi_1962_;
        phi_1961_ = _e195;
    }
    let _e210 = phi_1961_;
    let _e214 = u32(round(((_e210 * 2048f) + 65536f)));
    let _e215 = I0_1;
    let _e218 = ((_e215 << bitcast<u32>(17u)) | _e214);
    let _e221 = atomicMax((&W4_.x2_[_e118]), _e218);
    let _e223 = (_e221 >> bitcast<u32>(17u));
    if (_e223 == _e215) {
        let _e401 = S_1;
        if (_e401.y < 0f) {
            let _e408 = atomicAdd((&W4_.x2_[_e118]), ((_e214 + (_e221 - max(_e218, _e221))) - 65536u));
        }
        phi_2031_ = 0u;
        phi_2026_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e228 = ((f32((_e221 & 131071u)) * 0.00048828125f) + -32f);
        let _e233 = JB.x2_[((_e223 * 4u) + 1u)];
        phi_1964_ = _e228;
        if ((_e233.z & 768u) != 0u) {
            let _e237 = abs(_e228);
            phi_1455_ = oj;
            if oj {
                phi_1455_ = ((_e233.z & 512u) != 0u);
            }
            let _e241 = phi_1455_;
            phi_1965_ = _e237;
            if _e241 {
                phi_1965_ = (1f - abs(((fract((_e237 * 0.5f)) * 2f) + -1f)));
            }
            let _e249 = phi_1965_;
            phi_1964_ = _e249;
        }
        let _e251 = phi_1964_;
        let _e252 = clamp(_e251, 0f, 1f);
        phi_1968_ = _e252;
        if jj {
            let _e254 = (_e233.z >> bitcast<u32>(16u));
            phi_1969_ = _e252;
            if (_e254 != 0u) {
                let _e258 = m0_.x2_[_e118];
                if (_e254 == (_e258 >> bitcast<u32>(16i))) {
                    phi_1966_ = min(_e252, unpack2x16float(_e258).x);
                } else {
                    phi_1966_ = 0f;
                }
                let _e266 = phi_1966_;
                phi_1969_ = _e266;
            }
            let _e268 = phi_1969_;
            phi_1968_ = _e268;
        }
        let _e270 = phi_1968_;
        phi_1492_ = kj;
        if kj {
            phi_1492_ = ((_e233.z & 1024u) != 0u);
        }
        let _e274 = phi_1492_;
        phi_1973_ = _e270;
        if _e274 {
            let _e275 = (_e223 * 8u);
            let _e279 = KB.x2_[(_e275 + 2u)];
            let _e290 = KB.x2_[(_e275 + 3u)];
            let _e295 = _e290.zw;
            let _e297 = ((abs(((mat2x2<f32>(vec2<f32>(_e279.x, _e279.y), vec2<f32>(_e279.z, _e279.w)) * _e84) + _e290.xy)) * _e295) - _e295);
            phi_1973_ = min(_e270, clamp((min(_e297.x, _e297.y) + 0.5f), 0f, 1f));
        }
        let _e305 = phi_1973_;
        let _e306 = (_e233.z & 15u);
        if (_e306 <= 1u) {
            let _e388 = (jj && (_e306 == 0u));
            phi_2013_ = 0u;
            if _e388 {
                phi_2013_ = (_e233.w | pack2x16float(vec2<f32>(_e305, 0f)));
            }
            let _e393 = phi_2013_;
            phi_2012_ = _e393;
            phi_2009_ = select(unpack4x8unorm(_e233.w), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e388));
        } else {
            let _e313 = (_e223 * 8u);
            let _e316 = KB.x2_[_e313];
            let _e327 = KB.x2_[(_e313 + 1u)];
            let _e330 = ((mat2x2<f32>(vec2<f32>(_e316.x, _e316.y), vec2<f32>(_e316.z, _e316.w)) * _e84) + _e327.xy);
            let _e334 = f32(_e306);
            let _e336 = j.O8_;
            let _e338 = j.P8_;
            let _e343 = ((_e327.w * _e336) + _e338);
            switch bitcast<i32>(0u) {
                default: {
                    if (sj && (_e334 == 4f)) {
                        let _e357 = normalize(_e330);
                        let _e360 = acos(clamp(_e357.x, -1f, 1f));
                        if (_e357.y >= 0f) {
                            phi_1971_ = _e360;
                        } else {
                            phi_1971_ = -(_e360);
                        }
                        let _e365 = phi_1971_;
                        phi_1972_ = vec2<f32>((_e365 * 0.15915494f), _e343);
                        break;
                    } else {
                        if (_e334 == 2f) {
                            phi_1970_ = _e330.x;
                        } else {
                            phi_1970_ = length(_e330);
                        }
                        let _e351 = phi_1970_;
                        phi_1972_ = vec2<f32>(((clamp(_e351, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e327.z < 0f))) + ((max(0f, _e327.z) * 0.001953125f) + 0.0009765625f)), _e343);
                        break;
                    }
                }
            }
            let _e369 = phi_1972_;
            let _e370 = textureSampleLevel(YC, Q8_, _e369, 0f);
            phi_2011_ = _e370;
            if !((lj && (((_e233.z >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e374 = (_e370.xyz * _e370.w);
                phi_2011_ = vec4<f32>(_e374.x, _e374.y, _e374.z, (_e370.w * abs(bitcast<f32>(_e233.w))));
            }
            let _e384 = phi_2011_;
            phi_2012_ = 0u;
            phi_2009_ = _e384;
        }
        let _e397 = phi_2012_;
        let _e399 = phi_2009_;
        phi_2031_ = _e397;
        phi_2026_ = (_e399 * _e305);
    }
    let _e410 = phi_2031_;
    let _e412 = phi_2026_;
    let _e413 = _e412.xyz;
    let _e416 = j.I3_;
    let _e418 = j.J3_;
    if (rj && (_e412.w != 0f)) {
        phi_2028_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e83.x) + (0.00583715f * _e83.y))))) * _e416) + _e418)) + _e413);
    } else {
        phi_2028_ = _e413;
    }
    let _e434 = phi_2028_;
    let _e440 = vec4<f32>(_e434.x, _e412.y, _e412.z, _e412.w);
    let _e446 = vec4<f32>(_e440.x, _e434.y, _e440.z, _e440.w);
    Q1_ = vec4<f32>(_e446.x, _e446.y, _e434.z, _e446.w);
    if (_e410 != 0u) {
        m0_.x2_[_e118] = _e410;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) S: vec4<f32>, @location(1) @interpolate(flat, either) I0_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    S_1 = S;
    I0_1 = I0_;
    main_1();
    let _e7 = Q1_;
    return _e7;
}
