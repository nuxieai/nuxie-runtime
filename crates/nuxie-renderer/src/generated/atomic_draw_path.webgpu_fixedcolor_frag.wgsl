struct jg {
    r2_: array<vec2<u32>>,
}

struct m0Pe {
    r2_: array<u32>,
}

struct kg {
    r2_: array<vec4<f32>>,
}

struct VB {
    xd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    F8_: vec4<i32>,
    zi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Di: f32,
    w6_: u32,
    i3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    wi: u32,
    Ba: u32,
    M8_: f32,
    N8_: f32,
}

struct R4Pe {
    r2_: array<u32>,
}

struct R4Pe_1 {
    r2_: array<atomic<u32>>,
}

@id(7) override ej: bool = true;
@id(4) override bj: bool = true;
@id(0) override Xi: bool = true;
@id(1) override Yi: bool = true;
@id(2) override Zi: bool = true;
@id(3) override aj: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(3)
var<storage> VC: jg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var XC: texture_2d<f32>;
@group(3) @binding(8)
var O8_: sampler;
var<private> S_1: vec4<f32>;
var<private> G0_1: u32;
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe_1;
var<private> L1_: vec4<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;

fn main_1() {
    var phi_940_: bool;
    var phi_953_: bool;
    var phi_1947_: f32;
    var phi_1955_: f32;
    var phi_1963_: f32;
    var phi_1962_: f32;
    var phi_1450_: bool;
    var phi_1966_: f32;
    var phi_1965_: f32;
    var phi_1967_: f32;
    var phi_1970_: f32;
    var phi_1969_: f32;
    var phi_1487_: bool;
    var phi_1974_: f32;
    var phi_1971_: f32;
    var phi_1972_: f32;
    var phi_1973_: vec2<f32>;
    var phi_2012_: vec4<f32>;
    var phi_2014_: u32;
    var phi_2013_: u32;
    var phi_2010_: vec4<f32>;
    var phi_2032_: u32;
    var phi_2027_: vec4<f32>;
    var phi_2029_: vec3<f32>;

    let _e81 = gl_FragCoord_1;
    let _e82 = _e81.xy;
    let _e85 = bitcast<vec2<u32>>(vec2<i32>(floor(_e82)));
    let _e87 = j.P6_;
    let _e116 = bitcast<i32>((((((_e85.y >> bitcast<u32>(5u)) * (((_e87 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e85.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e85.x & 28u) << bitcast<u32>(5u)) + ((_e85.y & 28u) << bitcast<u32>(2i)))) + (((_e85.y & 3u) << bitcast<u32>(2i)) + (_e85.x & 3u))));
    phi_940_ = aj;
    if aj {
        let _e117 = S_1;
        phi_940_ = (_e117.x < -1.5f);
    }
    let _e121 = phi_940_;
    if _e121 {
        let _e194 = S_1;
        let _e198 = textureSampleLevel(YC, ab, vec2<f32>((3f + _e194.x), 0f), 0f);
        let _e204 = textureSampleLevel(YC, ab, vec2<f32>((1f - _e194.y), 0f), 0f);
        phi_1962_ = ((1f - _e198.x) - _e204.x);
    } else {
        phi_953_ = aj;
        if aj {
            let _e122 = S_1;
            phi_953_ = (_e122.y < -1.5f);
        }
        let _e126 = phi_953_;
        if _e126 {
            let _e134 = S_1;
            let _e137 = max(_e134.w, 0f);
            if (_e134.z >= 0f) {
                let _e140 = textureSampleLevel(YC, ab, vec2<f32>(_e137, 0f), 0f);
                phi_1947_ = _e140.x;
            } else {
                phi_1947_ = 0f;
            }
            let _e143 = phi_1947_;
            phi_1955_ = _e143;
            if (abs(_e134.z) < 1000f) {
                let _e150 = (-2f - _e134.y);
                let _e152 = ((_e150 - _e137) * 0.5984134f);
                let _e155 = (vec4(_e137) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e152));
                let _e161 = ((_e155 * -(_e134.z)) + vec4(((_e150 * _e134.z) + (abs(_e134.x) - 0.25f))));
                let _e164 = textureSampleLevel(YC, ab, vec2<f32>(_e161.x, 0f), 0f);
                let _e167 = textureSampleLevel(YC, ab, vec2<f32>(_e161.y, 0f), 0f);
                let _e170 = textureSampleLevel(YC, ab, vec2<f32>(_e161.z, 0f), 0f);
                let _e173 = textureSampleLevel(YC, ab, vec2<f32>(_e161.w, 0f), 0f);
                let _e179 = (_e155 * 5.0959306f);
                phi_1955_ = (_e143 + (dot(vec4<f32>(_e164.x, _e167.x, _e170.x, _e173.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e179) * (_e179 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e152));
            }
            let _e188 = phi_1955_;
            phi_1963_ = (_e188 * sign(_e134.x));
        } else {
            let _e128 = S_1[0u];
            let _e130 = S_1[1u];
            phi_1963_ = min(min(_e128, abs(_e130)), 1f);
        }
        let _e193 = phi_1963_;
        phi_1962_ = _e193;
    }
    let _e208 = phi_1962_;
    let _e212 = u32(round(((_e208 * 2048f) + 65536f)));
    let _e213 = G0_1;
    let _e216 = ((_e213 << bitcast<u32>(17u)) | _e212);
    let _e219 = atomicMax((&R4_.r2_[_e116]), _e216);
    let _e221 = (_e219 >> bitcast<u32>(17u));
    if (_e221 == _e213) {
        let _e400 = S_1;
        if (_e400.y < 0f) {
            let _e407 = atomicAdd((&R4_.r2_[_e116]), ((_e212 + (_e219 - max(_e216, _e219))) - 65536u));
        }
        phi_2032_ = 0u;
        phi_2027_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e226 = ((f32((_e219 & 131071u)) * 0.00048828125f) + -32f);
        let _e229 = VC.r2_[_e221];
        phi_1965_ = _e226;
        if ((_e229.x & 768u) != 0u) {
            let _e233 = abs(_e226);
            phi_1450_ = bj;
            if bj {
                phi_1450_ = ((_e229.x & 512u) != 0u);
            }
            let _e237 = phi_1450_;
            phi_1966_ = _e233;
            if _e237 {
                phi_1966_ = (1f - abs(((fract((_e233 * 0.5f)) * 2f) + -1f)));
            }
            let _e245 = phi_1966_;
            phi_1965_ = _e245;
        }
        let _e247 = phi_1965_;
        let _e248 = clamp(_e247, 0f, 1f);
        phi_1969_ = _e248;
        if Xi {
            let _e250 = (_e229.x >> bitcast<u32>(16u));
            phi_1970_ = _e248;
            if (_e250 != 0u) {
                let _e254 = m0_.r2_[_e116];
                if (_e250 == (_e254 >> bitcast<u32>(16i))) {
                    phi_1967_ = min(_e248, unpack2x16float(_e254).x);
                } else {
                    phi_1967_ = 0f;
                }
                let _e262 = phi_1967_;
                phi_1970_ = _e262;
            }
            let _e264 = phi_1970_;
            phi_1969_ = _e264;
        }
        let _e266 = phi_1969_;
        phi_1487_ = Yi;
        if Yi {
            phi_1487_ = ((_e229.x & 1024u) != 0u);
        }
        let _e270 = phi_1487_;
        phi_1974_ = _e266;
        if _e270 {
            let _e271 = (_e221 * 8u);
            let _e275 = JB.r2_[(_e271 + 2u)];
            let _e286 = JB.r2_[(_e271 + 3u)];
            let _e291 = _e286.zw;
            let _e293 = ((abs(((mat2x2<f32>(vec2<f32>(_e275.x, _e275.y), vec2<f32>(_e275.z, _e275.w)) * _e82) + _e286.xy)) * _e291) - _e291);
            phi_1974_ = min(_e266, clamp((min(_e293.x, _e293.y) + 0.5f), 0f, 1f));
        }
        let _e301 = phi_1974_;
        let _e302 = (_e229.x & 15u);
        if (_e302 <= 1u) {
            let _e387 = (Xi && (_e302 == 0u));
            phi_2014_ = 0u;
            if _e387 {
                phi_2014_ = (_e229.y | pack2x16float(vec2<f32>(_e301, 0f)));
            }
            let _e392 = phi_2014_;
            phi_2013_ = _e392;
            phi_2010_ = select(unpack4x8unorm(_e229.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e387));
        } else {
            let _e309 = (_e221 * 8u);
            let _e312 = JB.r2_[_e309];
            let _e323 = JB.r2_[(_e309 + 1u)];
            let _e326 = ((mat2x2<f32>(vec2<f32>(_e312.x, _e312.y), vec2<f32>(_e312.z, _e312.w)) * _e82) + _e323.xy);
            let _e330 = f32(_e302);
            let _e332 = j.M8_;
            let _e334 = j.N8_;
            let _e339 = ((_e323.w * _e332) + _e334);
            switch bitcast<i32>(0u) {
                default: {
                    if (_e330 == 4f) {
                        let _e356 = normalize(vec2<f32>(_e326.x, -(_e326.y)));
                        let _e359 = acos(clamp(_e356.x, -1f, 1f));
                        if (_e356.y >= 0f) {
                            phi_1972_ = _e359;
                        } else {
                            phi_1972_ = -(_e359);
                        }
                        let _e364 = phi_1972_;
                        phi_1973_ = vec2<f32>((_e364 * 0.15915507f), _e339);
                        break;
                    } else {
                        if (_e330 == 2f) {
                            phi_1971_ = _e326.x;
                        } else {
                            phi_1971_ = length(_e326);
                        }
                        let _e346 = phi_1971_;
                        phi_1973_ = vec2<f32>(((clamp(_e346, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e323.z < 0f))) + ((max(0f, _e323.z) * 0.001953125f) + 0.0009765625f)), _e339);
                        break;
                    }
                }
            }
            let _e368 = phi_1973_;
            let _e369 = textureSampleLevel(XC, O8_, _e368, 0f);
            phi_2012_ = _e369;
            if !((Zi && (((_e229.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e373 = (_e369.xyz * _e369.w);
                phi_2012_ = vec4<f32>(_e373.x, _e373.y, _e373.z, (_e369.w * abs(bitcast<f32>(_e229.y))));
            }
            let _e383 = phi_2012_;
            phi_2013_ = 0u;
            phi_2010_ = _e383;
        }
        let _e396 = phi_2013_;
        let _e398 = phi_2010_;
        phi_2032_ = _e396;
        phi_2027_ = (_e398 * _e301);
    }
    let _e409 = phi_2032_;
    let _e411 = phi_2027_;
    let _e412 = _e411.xyz;
    let _e415 = j.F3_;
    let _e417 = j.G3_;
    if (ej && (_e411.w != 0f)) {
        phi_2029_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e81.x) + (0.00583715f * _e81.y))))) * _e415) + _e417)) + _e412);
    } else {
        phi_2029_ = _e412;
    }
    let _e433 = phi_2029_;
    let _e439 = vec4<f32>(_e433.x, _e411.y, _e411.z, _e411.w);
    let _e445 = vec4<f32>(_e439.x, _e433.y, _e439.z, _e439.w);
    L1_ = vec4<f32>(_e445.x, _e445.y, _e433.z, _e445.w);
    if (_e409 != 0u) {
        m0_.r2_[_e116] = _e409;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) S: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    S_1 = S;
    G0_1 = G0_;
    main_1();
    let _e7 = L1_;
    return _e7;
}
