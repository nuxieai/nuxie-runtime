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
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;
var<private> n2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> T4_: T4Ue;
var<private> U3_1: u32;
var<private> Y1_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var db: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
var<private> N1_1: u32;

fn main_1() {
    var phi_1551_: f32;
    var phi_1011_: bool;
    var phi_1489_: f32;
    var phi_1488_: f32;
    var phi_1490_: f32;
    var phi_1493_: f32;
    var phi_1492_: f32;
    var phi_1048_: bool;
    var phi_1497_: f32;
    var phi_1494_: f32;
    var phi_1495_: f32;
    var phi_1496_: vec2<f32>;
    var phi_1526_: vec4<f32>;
    var phi_1528_: u32;
    var phi_1527_: u32;
    var phi_1524_: vec4<f32>;
    var phi_783_: bool;
    var phi_1542_: u32;
    var phi_1560_: f32;
    var phi_1580_: f32;
    var phi_1585_: vec3<f32>;

    let _e66 = gl_FragCoord_1;
    let _e67 = _e66.xy;
    let _e70 = bitcast<vec2<u32>>(vec2<i32>(floor(_e67)));
    let _e72 = j.T6_;
    let _e101 = bitcast<i32>((((((_e70.y >> bitcast<u32>(5u)) * (((_e72 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e70.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e70.x & 28u) << bitcast<u32>(5u)) + ((_e70.y & 28u) << bitcast<u32>(2i)))) + (((_e70.y & 3u) << bitcast<u32>(2i)) + (_e70.x & 3u))));
    let _e102 = n2_1;
    let _e103 = textureSample(UB, W4_, _e102);
    phi_1551_ = 1f;
    if gj {
        let _e104 = W0_1;
        let _e107 = min(_e104.xy, _e104.zw);
        phi_1551_ = clamp(min(_e107.x, _e107.y), 0f, 1f);
    }
    let _e113 = phi_1551_;
    let _e116 = T4_.v2_[_e101];
    let _e118 = (_e116 >> bitcast<u32>(17u));
    let _e122 = ((f32((_e116 & 131071u)) * 0.00048828125f) + -32f);
    let _e125 = XC.v2_[_e118];
    phi_1488_ = _e122;
    if ((_e125.x & 768u) != 0u) {
        let _e129 = abs(_e122);
        phi_1011_ = kj;
        if kj {
            phi_1011_ = ((_e125.x & 512u) != 0u);
        }
        let _e133 = phi_1011_;
        phi_1489_ = _e129;
        if _e133 {
            phi_1489_ = (1f - abs(((fract((_e129 * 0.5f)) * 2f) + -1f)));
        }
        let _e141 = phi_1489_;
        phi_1488_ = _e141;
    }
    let _e143 = phi_1488_;
    let _e144 = clamp(_e143, 0f, 1f);
    phi_1492_ = _e144;
    if fj {
        let _e146 = (_e125.x >> bitcast<u32>(16u));
        phi_1493_ = _e144;
        if (_e146 != 0u) {
            let _e150 = m0_.v2_[_e101];
            if (_e146 == (_e150 >> bitcast<u32>(16i))) {
                phi_1490_ = min(_e144, unpack2x16float(_e150).x);
            } else {
                phi_1490_ = 0f;
            }
            let _e158 = phi_1490_;
            phi_1493_ = _e158;
        }
        let _e160 = phi_1493_;
        phi_1492_ = _e160;
    }
    let _e162 = phi_1492_;
    phi_1048_ = gj;
    if gj {
        phi_1048_ = ((_e125.x & 1024u) != 0u);
    }
    let _e166 = phi_1048_;
    phi_1497_ = _e162;
    if _e166 {
        let _e167 = (_e118 * 8u);
        let _e171 = JB.v2_[(_e167 + 2u)];
        let _e182 = JB.v2_[(_e167 + 3u)];
        let _e187 = _e182.zw;
        let _e189 = ((abs(((mat2x2<f32>(vec2<f32>(_e171.x, _e171.y), vec2<f32>(_e171.z, _e171.w)) * _e67) + _e182.xy)) * _e187) - _e187);
        phi_1497_ = min(_e162, clamp((min(_e189.x, _e189.y) + 0.5f), 0f, 1f));
    }
    let _e197 = phi_1497_;
    let _e198 = (_e125.x & 15u);
    if (_e198 <= 1u) {
        let _e284 = (fj && (_e198 == 0u));
        phi_1528_ = 0u;
        if _e284 {
            phi_1528_ = (_e125.y | pack2x16float(vec2<f32>(_e197, 0f)));
        }
        let _e289 = phi_1528_;
        phi_1527_ = _e289;
        phi_1524_ = select(unpack4x8unorm(_e125.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e284));
    } else {
        let _e205 = (_e118 * 8u);
        let _e208 = JB.v2_[_e205];
        let _e219 = JB.v2_[(_e205 + 1u)];
        let _e222 = ((mat2x2<f32>(vec2<f32>(_e208.x, _e208.y), vec2<f32>(_e208.z, _e208.w)) * _e67) + _e219.xy);
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
                        phi_1495_ = _e256;
                    } else {
                        phi_1495_ = -(_e256);
                    }
                    let _e261 = phi_1495_;
                    phi_1496_ = vec2<f32>((_e261 * 0.15915494f), _e235);
                    break;
                } else {
                    if (_e226 == 2f) {
                        phi_1494_ = _e222.x;
                    } else {
                        phi_1494_ = length(_e222);
                    }
                    let _e243 = phi_1494_;
                    phi_1496_ = vec2<f32>(((clamp(_e243, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e219.z < 0f))) + ((max(0f, _e219.z) * 0.001953125f) + 0.0009765625f)), _e235);
                    break;
                }
            }
        }
        let _e265 = phi_1496_;
        let _e266 = textureSampleLevel(ZC, R8_, _e265, 0f);
        phi_1526_ = _e266;
        if !((hj && (((_e125.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e270 = (_e266.xyz * _e266.w);
            phi_1526_ = vec4<f32>(_e270.x, _e270.y, _e270.z, (_e266.w * abs(bitcast<f32>(_e125.y))));
        }
        let _e280 = phi_1526_;
        phi_1527_ = 0u;
        phi_1524_ = _e280;
    }
    let _e293 = phi_1527_;
    let _e295 = phi_1524_;
    phi_783_ = fj;
    if fj {
        let _e297 = U3_1;
        phi_783_ = (_e297 != 0u);
    }
    let _e300 = phi_783_;
    phi_1580_ = _e113;
    if _e300 {
        if (_e293 != 0u) {
            phi_1542_ = _e293;
        } else {
            let _e304 = m0_.v2_[_e101];
            phi_1542_ = _e304;
        }
        let _e306 = phi_1542_;
        let _e307 = U3_1;
        if (_e307 == (_e306 >> bitcast<u32>(16i))) {
            phi_1560_ = min(_e113, unpack2x16float(_e306).x);
        } else {
            phi_1560_ = 0f;
        }
        let _e315 = phi_1560_;
        phi_1580_ = _e315;
    }
    let _e317 = phi_1580_;
    let _e318 = Y1_1;
    let _e320 = ((_e103 * _e318) * _e317);
    let _e324 = (((_e295 * _e197) * (1f - _e320.w)) + _e320);
    let _e325 = _e324.xyz;
    let _e328 = j.G3_;
    let _e330 = j.H3_;
    if (nj && (_e324.w != 0f)) {
        phi_1585_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e66.x) + (0.00583715f * _e66.y))))) * _e328) + _e330)) + _e325);
    } else {
        phi_1585_ = _e325;
    }
    let _e346 = phi_1585_;
    let _e352 = vec4<f32>(_e346.x, _e324.y, _e324.z, _e324.w);
    let _e358 = vec4<f32>(_e352.x, _e346.y, _e352.z, _e352.w);
    Q1_ = vec4<f32>(_e358.x, _e358.y, _e346.z, _e358.w);
    if (_e293 != 0u) {
        m0_.v2_[_e101] = _e293;
    }
    T4_.v2_[_e101] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) n2_: vec2<f32>, @location(1) W0_: vec4<f32>, @location(4) @interpolate(flat, either) U3_: u32, @location(3) @interpolate(flat, either) Y1_: vec4<f32>, @location(5) @interpolate(flat, either) N1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    n2_1 = n2_;
    W0_1 = W0_;
    U3_1 = U3_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
    let _e13 = Q1_;
    return _e13;
}
