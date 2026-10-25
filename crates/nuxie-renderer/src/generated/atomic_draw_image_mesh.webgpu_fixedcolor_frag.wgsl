struct ug {
    v2_: array<vec2<u32>>,
}

struct m0cf {
    v2_: array<u32>,
}

struct vg {
    v2_: array<vec4<f32>>,
}

struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct W4cf {
    v2_: array<u32>,
}

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;

@group(0) @binding(3)
var<storage> XC: ug;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(0) @binding(4)
var<storage> JB: vg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;
var<private> n2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> V3_1: u32;
var<private> Y1_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> N1_1: u32;

fn main_1() {
    var phi_1539_: f32;
    var phi_1006_: bool;
    var phi_1477_: f32;
    var phi_1476_: f32;
    var phi_1478_: f32;
    var phi_1481_: f32;
    var phi_1480_: f32;
    var phi_1043_: bool;
    var phi_1485_: f32;
    var phi_1482_: f32;
    var phi_1483_: f32;
    var phi_1484_: vec2<f32>;
    var phi_1514_: vec4<f32>;
    var phi_1516_: u32;
    var phi_1515_: u32;
    var phi_1512_: vec4<f32>;
    var phi_778_: bool;
    var phi_1530_: u32;
    var phi_1548_: f32;
    var phi_1568_: f32;
    var phi_1573_: vec3<f32>;

    let _e66 = gl_FragCoord_1;
    let _e67 = _e66.xy;
    let _e70 = bitcast<vec2<u32>>(vec2<i32>(floor(_e67)));
    let _e72 = j.T6_;
    let _e101 = bitcast<i32>((((((_e70.y >> bitcast<u32>(5u)) * (((_e72 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e70.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e70.x & 28u) << bitcast<u32>(5u)) + ((_e70.y & 28u) << bitcast<u32>(2i)))) + (((_e70.y & 3u) << bitcast<u32>(2i)) + (_e70.x & 3u))));
    let _e102 = n2_1;
    let _e103 = textureSample(RB, v4_, _e102);
    phi_1539_ = 1f;
    if kj {
        let _e104 = W0_1;
        let _e107 = min(_e104.xy, _e104.zw);
        phi_1539_ = clamp(min(_e107.x, _e107.y), 0f, 1f);
    }
    let _e113 = phi_1539_;
    let _e116 = W4_.v2_[_e101];
    let _e118 = (_e116 >> bitcast<u32>(17u));
    let _e122 = ((f32((_e116 & 131071u)) * 0.00048828125f) + -32f);
    let _e125 = XC.v2_[_e118];
    phi_1476_ = _e122;
    if ((_e125.x & 768u) != 0u) {
        let _e129 = abs(_e122);
        phi_1006_ = oj;
        if oj {
            phi_1006_ = ((_e125.x & 512u) != 0u);
        }
        let _e133 = phi_1006_;
        phi_1477_ = _e129;
        if _e133 {
            phi_1477_ = (1f - abs(((fract((_e129 * 0.5f)) * 2f) + -1f)));
        }
        let _e141 = phi_1477_;
        phi_1476_ = _e141;
    }
    let _e143 = phi_1476_;
    let _e144 = clamp(_e143, 0f, 1f);
    phi_1480_ = _e144;
    if jj {
        let _e146 = (_e125.x >> bitcast<u32>(16u));
        phi_1481_ = _e144;
        if (_e146 != 0u) {
            let _e150 = m0_.v2_[_e101];
            if (_e146 == (_e150 >> bitcast<u32>(16i))) {
                phi_1478_ = min(_e144, unpack2x16float(_e150).x);
            } else {
                phi_1478_ = 0f;
            }
            let _e158 = phi_1478_;
            phi_1481_ = _e158;
        }
        let _e160 = phi_1481_;
        phi_1480_ = _e160;
    }
    let _e162 = phi_1480_;
    phi_1043_ = kj;
    if kj {
        phi_1043_ = ((_e125.x & 1024u) != 0u);
    }
    let _e166 = phi_1043_;
    phi_1485_ = _e162;
    if _e166 {
        let _e167 = (_e118 * 8u);
        let _e171 = JB.v2_[(_e167 + 2u)];
        let _e182 = JB.v2_[(_e167 + 3u)];
        let _e187 = _e182.zw;
        let _e189 = ((abs(((mat2x2<f32>(vec2<f32>(_e171.x, _e171.y), vec2<f32>(_e171.z, _e171.w)) * _e67) + _e182.xy)) * _e187) - _e187);
        phi_1485_ = min(_e162, clamp((min(_e189.x, _e189.y) + 0.5f), 0f, 1f));
    }
    let _e197 = phi_1485_;
    let _e198 = (_e125.x & 15u);
    if (_e198 <= 1u) {
        let _e280 = (jj && (_e198 == 0u));
        phi_1516_ = 0u;
        if _e280 {
            phi_1516_ = (_e125.y | pack2x16float(vec2<f32>(_e197, 0f)));
        }
        let _e285 = phi_1516_;
        phi_1515_ = _e285;
        phi_1512_ = select(unpack4x8unorm(_e125.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e280));
    } else {
        let _e205 = (_e118 * 8u);
        let _e208 = JB.v2_[_e205];
        let _e219 = JB.v2_[(_e205 + 1u)];
        let _e222 = ((mat2x2<f32>(vec2<f32>(_e208.x, _e208.y), vec2<f32>(_e208.z, _e208.w)) * _e67) + _e219.xy);
        let _e226 = f32(_e198);
        let _e228 = j.Q8_;
        let _e230 = j.R8_;
        let _e235 = ((_e219.w * _e228) + _e230);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e226 == 4f)) {
                    let _e249 = normalize(_e222);
                    let _e252 = acos(clamp(_e249.x, -1f, 1f));
                    if (_e249.y >= 0f) {
                        phi_1483_ = _e252;
                    } else {
                        phi_1483_ = -(_e252);
                    }
                    let _e257 = phi_1483_;
                    phi_1484_ = vec2<f32>((_e257 * 0.15915494f), _e235);
                    break;
                } else {
                    if (_e226 == 2f) {
                        phi_1482_ = _e222.x;
                    } else {
                        phi_1482_ = length(_e222);
                    }
                    let _e243 = phi_1482_;
                    phi_1484_ = vec2<f32>(((clamp(_e243, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e219.z < 0f))) + ((max(0f, _e219.z) * 0.001953125f) + 0.0009765625f)), _e235);
                    break;
                }
            }
        }
        let _e261 = phi_1484_;
        let _e262 = textureSampleLevel(ZC, S8_, _e261, 0f);
        phi_1514_ = _e262;
        if !((lj && (((_e125.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e266 = (_e262.xyz * _e262.w);
            phi_1514_ = vec4<f32>(_e266.x, _e266.y, _e266.z, (_e262.w * abs(bitcast<f32>(_e125.y))));
        }
        let _e276 = phi_1514_;
        phi_1515_ = 0u;
        phi_1512_ = _e276;
    }
    let _e289 = phi_1515_;
    let _e291 = phi_1512_;
    phi_778_ = jj;
    if jj {
        let _e293 = V3_1;
        phi_778_ = (_e293 != 0u);
    }
    let _e296 = phi_778_;
    phi_1568_ = _e113;
    if _e296 {
        if (_e289 != 0u) {
            phi_1530_ = _e289;
        } else {
            let _e300 = m0_.v2_[_e101];
            phi_1530_ = _e300;
        }
        let _e302 = phi_1530_;
        let _e303 = V3_1;
        if (_e303 == (_e302 >> bitcast<u32>(16i))) {
            phi_1548_ = min(_e113, unpack2x16float(_e302).x);
        } else {
            phi_1548_ = 0f;
        }
        let _e311 = phi_1548_;
        phi_1568_ = _e311;
    }
    let _e313 = phi_1568_;
    let _e314 = Y1_1;
    let _e316 = ((_e103 * _e314) * _e313);
    let _e320 = (((_e291 * _e197) * (1f - _e316.w)) + _e316);
    let _e321 = _e320.xyz;
    let _e324 = j.H3_;
    let _e326 = j.I3_;
    if (rj && (_e320.w != 0f)) {
        phi_1573_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e66.x) + (0.00583715f * _e66.y))))) * _e324) + _e326)) + _e321);
    } else {
        phi_1573_ = _e321;
    }
    let _e342 = phi_1573_;
    let _e348 = vec4<f32>(_e342.x, _e320.y, _e320.z, _e320.w);
    let _e354 = vec4<f32>(_e348.x, _e342.y, _e348.z, _e348.w);
    Q1_ = vec4<f32>(_e354.x, _e354.y, _e342.z, _e354.w);
    if (_e289 != 0u) {
        m0_.v2_[_e101] = _e289;
    }
    W4_.v2_[_e101] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) n2_: vec2<f32>, @location(1) W0_: vec4<f32>, @location(4) @interpolate(flat, either) V3_: u32, @location(3) @interpolate(flat, either) Y1_: vec4<f32>, @location(5) @interpolate(flat, either) N1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    n2_1 = n2_;
    W0_1 = W0_;
    V3_1 = V3_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
    let _e13 = Q1_;
    return _e13;
}
