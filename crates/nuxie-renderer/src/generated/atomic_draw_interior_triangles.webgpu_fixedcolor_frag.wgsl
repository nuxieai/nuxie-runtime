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
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> I0_1: u32;
var<private> p1_1: f32;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;

fn main_1() {
    var phi_1410_: u32;
    var phi_949_: bool;
    var phi_1415_: f32;
    var phi_1414_: f32;
    var phi_1416_: f32;
    var phi_1419_: f32;
    var phi_1418_: f32;
    var phi_986_: bool;
    var phi_1423_: f32;
    var phi_1420_: f32;
    var phi_1421_: f32;
    var phi_1422_: vec2<f32>;
    var phi_1451_: vec4<f32>;
    var phi_1453_: u32;
    var phi_1452_: u32;
    var phi_1449_: vec4<f32>;
    var phi_1470_: u32;
    var phi_1466_: vec4<f32>;
    var phi_1467_: vec3<f32>;

    let _e64 = gl_FragCoord_1;
    let _e65 = _e64.xy;
    let _e68 = bitcast<vec2<u32>>(vec2<i32>(floor(_e65)));
    let _e70 = j.T6_;
    let _e99 = bitcast<i32>((((((_e68.y >> bitcast<u32>(5u)) * (((_e70 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e68.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e68.x & 28u) << bitcast<u32>(5u)) + ((_e68.y & 28u) << bitcast<u32>(2i)))) + (((_e68.y & 3u) << bitcast<u32>(2i)) + (_e68.x & 3u))));
    let _e102 = W4_.v2_[_e99];
    let _e104 = (_e102 >> bitcast<u32>(17u));
    let _e105 = I0_1;
    if (_e104 == _e105) {
        phi_1410_ = _e102;
    } else {
        phi_1410_ = ((_e105 << bitcast<u32>(17u)) + 65536u);
    }
    let _e111 = phi_1410_;
    let _e112 = p1_1;
    W4_.v2_[_e99] = (_e111 + bitcast<u32>(i32(round((_e112 * 2048f)))));
    phi_1470_ = 0u;
    phi_1466_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e104 != _e105) {
        let _e122 = ((f32((_e102 & 131071u)) * 0.00048828125f) + -32f);
        let _e125 = XC.v2_[_e104];
        phi_1414_ = _e122;
        if ((_e125.x & 768u) != 0u) {
            let _e129 = abs(_e122);
            phi_949_ = oj;
            if oj {
                phi_949_ = ((_e125.x & 512u) != 0u);
            }
            let _e133 = phi_949_;
            phi_1415_ = _e129;
            if _e133 {
                phi_1415_ = (1f - abs(((fract((_e129 * 0.5f)) * 2f) + -1f)));
            }
            let _e141 = phi_1415_;
            phi_1414_ = _e141;
        }
        let _e143 = phi_1414_;
        let _e144 = clamp(_e143, 0f, 1f);
        phi_1418_ = _e144;
        if jj {
            let _e146 = (_e125.x >> bitcast<u32>(16u));
            phi_1419_ = _e144;
            if (_e146 != 0u) {
                let _e150 = m0_.v2_[_e99];
                if (_e146 == (_e150 >> bitcast<u32>(16i))) {
                    phi_1416_ = min(_e144, unpack2x16float(_e150).x);
                } else {
                    phi_1416_ = 0f;
                }
                let _e158 = phi_1416_;
                phi_1419_ = _e158;
            }
            let _e160 = phi_1419_;
            phi_1418_ = _e160;
        }
        let _e162 = phi_1418_;
        phi_986_ = kj;
        if kj {
            phi_986_ = ((_e125.x & 1024u) != 0u);
        }
        let _e166 = phi_986_;
        phi_1423_ = _e162;
        if _e166 {
            let _e167 = (_e104 * 8u);
            let _e171 = JB.v2_[(_e167 + 2u)];
            let _e182 = JB.v2_[(_e167 + 3u)];
            let _e187 = _e182.zw;
            let _e189 = ((abs(((mat2x2<f32>(vec2<f32>(_e171.x, _e171.y), vec2<f32>(_e171.z, _e171.w)) * _e65) + _e182.xy)) * _e187) - _e187);
            phi_1423_ = min(_e162, clamp((min(_e189.x, _e189.y) + 0.5f), 0f, 1f));
        }
        let _e197 = phi_1423_;
        let _e198 = (_e125.x & 15u);
        if (_e198 <= 1u) {
            let _e280 = (jj && (_e198 == 0u));
            phi_1453_ = 0u;
            if _e280 {
                phi_1453_ = (_e125.y | pack2x16float(vec2<f32>(_e197, 0f)));
            }
            let _e285 = phi_1453_;
            phi_1452_ = _e285;
            phi_1449_ = select(unpack4x8unorm(_e125.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e280));
        } else {
            let _e205 = (_e104 * 8u);
            let _e208 = JB.v2_[_e205];
            let _e219 = JB.v2_[(_e205 + 1u)];
            let _e222 = ((mat2x2<f32>(vec2<f32>(_e208.x, _e208.y), vec2<f32>(_e208.z, _e208.w)) * _e65) + _e219.xy);
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
                            phi_1421_ = _e252;
                        } else {
                            phi_1421_ = -(_e252);
                        }
                        let _e257 = phi_1421_;
                        phi_1422_ = vec2<f32>((_e257 * 0.15915494f), _e235);
                        break;
                    } else {
                        if (_e226 == 2f) {
                            phi_1420_ = _e222.x;
                        } else {
                            phi_1420_ = length(_e222);
                        }
                        let _e243 = phi_1420_;
                        phi_1422_ = vec2<f32>(((clamp(_e243, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e219.z < 0f))) + ((max(0f, _e219.z) * 0.001953125f) + 0.0009765625f)), _e235);
                        break;
                    }
                }
            }
            let _e261 = phi_1422_;
            let _e262 = textureSampleLevel(ZC, S8_, _e261, 0f);
            phi_1451_ = _e262;
            if !((lj && (((_e125.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e266 = (_e262.xyz * _e262.w);
                phi_1451_ = vec4<f32>(_e266.x, _e266.y, _e266.z, (_e262.w * abs(bitcast<f32>(_e125.y))));
            }
            let _e276 = phi_1451_;
            phi_1452_ = 0u;
            phi_1449_ = _e276;
        }
        let _e289 = phi_1452_;
        let _e291 = phi_1449_;
        phi_1470_ = _e289;
        phi_1466_ = (_e291 * _e197);
    }
    let _e294 = phi_1470_;
    let _e296 = phi_1466_;
    let _e297 = _e296.xyz;
    let _e300 = j.H3_;
    let _e302 = j.I3_;
    if (rj && (_e296.w != 0f)) {
        phi_1467_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e64.x) + (0.00583715f * _e64.y))))) * _e300) + _e302)) + _e297);
    } else {
        phi_1467_ = _e297;
    }
    let _e318 = phi_1467_;
    let _e324 = vec4<f32>(_e318.x, _e296.y, _e296.z, _e296.w);
    let _e330 = vec4<f32>(_e324.x, _e318.y, _e324.z, _e324.w);
    Q1_ = vec4<f32>(_e330.x, _e330.y, _e318.z, _e330.w);
    if (_e294 != 0u) {
        m0_.v2_[_e99] = _e294;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) I0_: u32, @location(0) @interpolate(flat, either) p1_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    I0_1 = I0_;
    p1_1 = p1_;
    main_1();
    let _e7 = Q1_;
    return _e7;
}
