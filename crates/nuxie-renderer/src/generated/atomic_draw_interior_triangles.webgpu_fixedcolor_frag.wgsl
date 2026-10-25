struct og {
    v2_: array<vec2<u32>>,
}

struct m0Ue {
    v2_: array<u32>,
}

struct pg {
    v2_: array<vec4<f32>>,
}

struct WB {
    Cd: f32,
    He: f32,
    Mg: f32,
    Ng: f32,
    T6_: u32,
    Ea: u32,
    yg: u32,
    zg: u32,
    I8_: vec4<i32>,
    Gi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ki: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Di: u32,
    Da: u32,
    P8_: f32,
    Q8_: f32,
}

struct T4Ue {
    v2_: array<u32>,
}

@id(8) override nj: bool = true;
@id(9) override oj: bool = true;
@id(5) override kj: bool = true;
@id(0) override fj: bool = true;
@id(1) override gj: bool = true;
@id(2) override hj: bool = true;

@group(0) @binding(3)
var<storage> XC: og;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Ue;
@group(0) @binding(4)
var<storage> JB: pg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var R8_: sampler;
@group(2) @binding(3)
var<storage, read_write> T4_: T4Ue;
var<private> G0_1: u32;
var<private> p1_1: f32;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;

fn main_1() {
    var phi_1422_: u32;
    var phi_954_: bool;
    var phi_1427_: f32;
    var phi_1426_: f32;
    var phi_1428_: f32;
    var phi_1431_: f32;
    var phi_1430_: f32;
    var phi_991_: bool;
    var phi_1435_: f32;
    var phi_1432_: f32;
    var phi_1433_: f32;
    var phi_1434_: vec2<f32>;
    var phi_1463_: vec4<f32>;
    var phi_1465_: u32;
    var phi_1464_: u32;
    var phi_1461_: vec4<f32>;
    var phi_1482_: u32;
    var phi_1478_: vec4<f32>;
    var phi_1479_: vec3<f32>;

    let _e64 = gl_FragCoord_1;
    let _e65 = _e64.xy;
    let _e68 = bitcast<vec2<u32>>(vec2<i32>(floor(_e65)));
    let _e70 = j.T6_;
    let _e99 = bitcast<i32>((((((_e68.y >> bitcast<u32>(5u)) * (((_e70 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e68.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e68.x & 28u) << bitcast<u32>(5u)) + ((_e68.y & 28u) << bitcast<u32>(2i)))) + (((_e68.y & 3u) << bitcast<u32>(2i)) + (_e68.x & 3u))));
    let _e102 = T4_.v2_[_e99];
    let _e104 = (_e102 >> bitcast<u32>(17u));
    let _e105 = G0_1;
    if (_e104 == _e105) {
        phi_1422_ = _e102;
    } else {
        phi_1422_ = ((_e105 << bitcast<u32>(17u)) + 65536u);
    }
    let _e111 = phi_1422_;
    let _e112 = p1_1;
    T4_.v2_[_e99] = (_e111 + bitcast<u32>(i32(round((_e112 * 2048f)))));
    phi_1482_ = 0u;
    phi_1478_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e104 != _e105) {
        let _e122 = ((f32((_e102 & 131071u)) * 0.00048828125f) + -32f);
        let _e125 = XC.v2_[_e104];
        phi_1426_ = _e122;
        if ((_e125.x & 768u) != 0u) {
            let _e129 = abs(_e122);
            phi_954_ = kj;
            if kj {
                phi_954_ = ((_e125.x & 512u) != 0u);
            }
            let _e133 = phi_954_;
            phi_1427_ = _e129;
            if _e133 {
                phi_1427_ = (1f - abs(((fract((_e129 * 0.5f)) * 2f) + -1f)));
            }
            let _e141 = phi_1427_;
            phi_1426_ = _e141;
        }
        let _e143 = phi_1426_;
        let _e144 = clamp(_e143, 0f, 1f);
        phi_1430_ = _e144;
        if fj {
            let _e146 = (_e125.x >> bitcast<u32>(16u));
            phi_1431_ = _e144;
            if (_e146 != 0u) {
                let _e150 = m0_.v2_[_e99];
                if (_e146 == (_e150 >> bitcast<u32>(16i))) {
                    phi_1428_ = min(_e144, unpack2x16float(_e150).x);
                } else {
                    phi_1428_ = 0f;
                }
                let _e158 = phi_1428_;
                phi_1431_ = _e158;
            }
            let _e160 = phi_1431_;
            phi_1430_ = _e160;
        }
        let _e162 = phi_1430_;
        phi_991_ = gj;
        if gj {
            phi_991_ = ((_e125.x & 1024u) != 0u);
        }
        let _e166 = phi_991_;
        phi_1435_ = _e162;
        if _e166 {
            let _e167 = (_e104 * 8u);
            let _e171 = JB.v2_[(_e167 + 2u)];
            let _e182 = JB.v2_[(_e167 + 3u)];
            let _e187 = _e182.zw;
            let _e189 = ((abs(((mat2x2<f32>(vec2<f32>(_e171.x, _e171.y), vec2<f32>(_e171.z, _e171.w)) * _e65) + _e182.xy)) * _e187) - _e187);
            phi_1435_ = min(_e162, clamp((min(_e189.x, _e189.y) + 0.5f), 0f, 1f));
        }
        let _e197 = phi_1435_;
        let _e198 = (_e125.x & 15u);
        if (_e198 <= 1u) {
            let _e284 = (fj && (_e198 == 0u));
            phi_1465_ = 0u;
            if _e284 {
                phi_1465_ = (_e125.y | pack2x16float(vec2<f32>(_e197, 0f)));
            }
            let _e289 = phi_1465_;
            phi_1464_ = _e289;
            phi_1461_ = select(unpack4x8unorm(_e125.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e284));
        } else {
            let _e205 = (_e104 * 8u);
            let _e208 = JB.v2_[_e205];
            let _e219 = JB.v2_[(_e205 + 1u)];
            let _e222 = ((mat2x2<f32>(vec2<f32>(_e208.x, _e208.y), vec2<f32>(_e208.z, _e208.w)) * _e65) + _e219.xy);
            let _e226 = f32(_e198);
            let _e228 = j.P8_;
            let _e230 = j.Q8_;
            let _e235 = ((_e219.w * _e228) + _e230);
            switch bitcast<i32>(0u) {
                default: {
                    if (oj && (_e226 == 4f)) {
                        let _e253 = normalize(vec2<f32>(_e222.x, -(_e222.y)));
                        let _e256 = acos(clamp(_e253.x, -1f, 1f));
                        if (_e253.y >= 0f) {
                            phi_1433_ = _e256;
                        } else {
                            phi_1433_ = -(_e256);
                        }
                        let _e261 = phi_1433_;
                        phi_1434_ = vec2<f32>((_e261 * 0.15915494f), _e235);
                        break;
                    } else {
                        if (_e226 == 2f) {
                            phi_1432_ = _e222.x;
                        } else {
                            phi_1432_ = length(_e222);
                        }
                        let _e243 = phi_1432_;
                        phi_1434_ = vec2<f32>(((clamp(_e243, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e219.z < 0f))) + ((max(0f, _e219.z) * 0.001953125f) + 0.0009765625f)), _e235);
                        break;
                    }
                }
            }
            let _e265 = phi_1434_;
            let _e266 = textureSampleLevel(ZC, R8_, _e265, 0f);
            phi_1463_ = _e266;
            if !((hj && (((_e125.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e270 = (_e266.xyz * _e266.w);
                phi_1463_ = vec4<f32>(_e270.x, _e270.y, _e270.z, (_e266.w * abs(bitcast<f32>(_e125.y))));
            }
            let _e280 = phi_1463_;
            phi_1464_ = 0u;
            phi_1461_ = _e280;
        }
        let _e293 = phi_1464_;
        let _e295 = phi_1461_;
        phi_1482_ = _e293;
        phi_1478_ = (_e295 * _e197);
    }
    let _e298 = phi_1482_;
    let _e300 = phi_1478_;
    let _e301 = _e300.xyz;
    let _e304 = j.G3_;
    let _e306 = j.H3_;
    if (nj && (_e300.w != 0f)) {
        phi_1479_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e64.x) + (0.00583715f * _e64.y))))) * _e304) + _e306)) + _e301);
    } else {
        phi_1479_ = _e301;
    }
    let _e322 = phi_1479_;
    let _e328 = vec4<f32>(_e322.x, _e300.y, _e300.z, _e300.w);
    let _e334 = vec4<f32>(_e328.x, _e322.y, _e328.z, _e328.w);
    Q1_ = vec4<f32>(_e334.x, _e334.y, _e322.z, _e334.w);
    if (_e298 != 0u) {
        m0_.v2_[_e99] = _e298;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32, @location(0) @interpolate(flat, either) p1_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    p1_1 = p1_;
    main_1();
    let _e7 = Q1_;
    return _e7;
}
