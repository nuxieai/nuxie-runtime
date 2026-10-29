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

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;

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
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(1) @binding(12)
var v4_: sampler;
var<private> o2_1: vec2<f32>;
var<private> V0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> V3_1: u32;
var<private> Y1_1: vec4<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
var<private> N1_1: u32;

fn main_1() {
    var phi_1547_: f32;
    var phi_1014_: bool;
    var phi_1485_: f32;
    var phi_1484_: f32;
    var phi_1486_: f32;
    var phi_1489_: f32;
    var phi_1488_: f32;
    var phi_1051_: bool;
    var phi_1493_: f32;
    var phi_1490_: f32;
    var phi_1491_: f32;
    var phi_1492_: vec2<f32>;
    var phi_1522_: vec4<f32>;
    var phi_1524_: u32;
    var phi_1523_: u32;
    var phi_1520_: vec4<f32>;
    var phi_783_: bool;
    var phi_1538_: u32;
    var phi_1556_: f32;
    var phi_1576_: f32;
    var phi_1581_: vec3<f32>;

    let _e67 = gl_FragCoord_1;
    let _e68 = _e67.xy;
    let _e71 = bitcast<vec2<u32>>(vec2<i32>(floor(_e68)));
    let _e73 = j.R6_;
    let _e102 = bitcast<i32>((((((_e71.y >> bitcast<u32>(5u)) * (((_e73 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e71.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e71.x & 28u) << bitcast<u32>(5u)) + ((_e71.y & 28u) << bitcast<u32>(2i)))) + (((_e71.y & 3u) << bitcast<u32>(2i)) + (_e71.x & 3u))));
    let _e103 = o2_1;
    let _e104 = textureSample(RB, v4_, _e103);
    phi_1547_ = 1f;
    if kj {
        let _e105 = V0_1;
        let _e108 = min(_e105.xy, _e105.zw);
        phi_1547_ = clamp(min(_e108.x, _e108.y), 0f, 1f);
    }
    let _e114 = phi_1547_;
    let _e117 = W4_.x2_[_e102];
    let _e119 = (_e117 >> bitcast<u32>(17u));
    let _e123 = ((f32((_e117 & 131071u)) * 0.00048828125f) + -32f);
    let _e128 = JB.x2_[((_e119 * 4u) + 1u)];
    phi_1484_ = _e123;
    if ((_e128.z & 768u) != 0u) {
        let _e132 = abs(_e123);
        phi_1014_ = oj;
        if oj {
            phi_1014_ = ((_e128.z & 512u) != 0u);
        }
        let _e136 = phi_1014_;
        phi_1485_ = _e132;
        if _e136 {
            phi_1485_ = (1f - abs(((fract((_e132 * 0.5f)) * 2f) + -1f)));
        }
        let _e144 = phi_1485_;
        phi_1484_ = _e144;
    }
    let _e146 = phi_1484_;
    let _e147 = clamp(_e146, 0f, 1f);
    phi_1488_ = _e147;
    if jj {
        let _e149 = (_e128.z >> bitcast<u32>(16u));
        phi_1489_ = _e147;
        if (_e149 != 0u) {
            let _e153 = m0_.x2_[_e102];
            if (_e149 == (_e153 >> bitcast<u32>(16i))) {
                phi_1486_ = min(_e147, unpack2x16float(_e153).x);
            } else {
                phi_1486_ = 0f;
            }
            let _e161 = phi_1486_;
            phi_1489_ = _e161;
        }
        let _e163 = phi_1489_;
        phi_1488_ = _e163;
    }
    let _e165 = phi_1488_;
    phi_1051_ = kj;
    if kj {
        phi_1051_ = ((_e128.z & 1024u) != 0u);
    }
    let _e169 = phi_1051_;
    phi_1493_ = _e165;
    if _e169 {
        let _e170 = (_e119 * 8u);
        let _e174 = KB.x2_[(_e170 + 2u)];
        let _e185 = KB.x2_[(_e170 + 3u)];
        let _e190 = _e185.zw;
        let _e192 = ((abs(((mat2x2<f32>(vec2<f32>(_e174.x, _e174.y), vec2<f32>(_e174.z, _e174.w)) * _e68) + _e185.xy)) * _e190) - _e190);
        phi_1493_ = min(_e165, clamp((min(_e192.x, _e192.y) + 0.5f), 0f, 1f));
    }
    let _e200 = phi_1493_;
    let _e201 = (_e128.z & 15u);
    if (_e201 <= 1u) {
        let _e283 = (jj && (_e201 == 0u));
        phi_1524_ = 0u;
        if _e283 {
            phi_1524_ = (_e128.w | pack2x16float(vec2<f32>(_e200, 0f)));
        }
        let _e288 = phi_1524_;
        phi_1523_ = _e288;
        phi_1520_ = select(unpack4x8unorm(_e128.w), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e283));
    } else {
        let _e208 = (_e119 * 8u);
        let _e211 = KB.x2_[_e208];
        let _e222 = KB.x2_[(_e208 + 1u)];
        let _e225 = ((mat2x2<f32>(vec2<f32>(_e211.x, _e211.y), vec2<f32>(_e211.z, _e211.w)) * _e68) + _e222.xy);
        let _e229 = f32(_e201);
        let _e231 = j.O8_;
        let _e233 = j.P8_;
        let _e238 = ((_e222.w * _e231) + _e233);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e229 == 4f)) {
                    let _e252 = normalize(_e225);
                    let _e255 = acos(clamp(_e252.x, -1f, 1f));
                    if (_e252.y >= 0f) {
                        phi_1491_ = _e255;
                    } else {
                        phi_1491_ = -(_e255);
                    }
                    let _e260 = phi_1491_;
                    phi_1492_ = vec2<f32>((_e260 * 0.15915494f), _e238);
                    break;
                } else {
                    if (_e229 == 2f) {
                        phi_1490_ = _e225.x;
                    } else {
                        phi_1490_ = length(_e225);
                    }
                    let _e246 = phi_1490_;
                    phi_1492_ = vec2<f32>(((clamp(_e246, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e222.z < 0f))) + ((max(0f, _e222.z) * 0.001953125f) + 0.0009765625f)), _e238);
                    break;
                }
            }
        }
        let _e264 = phi_1492_;
        let _e265 = textureSampleLevel(YC, Q8_, _e264, 0f);
        phi_1522_ = _e265;
        if !((lj && (((_e128.z >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e269 = (_e265.xyz * _e265.w);
            phi_1522_ = vec4<f32>(_e269.x, _e269.y, _e269.z, (_e265.w * abs(bitcast<f32>(_e128.w))));
        }
        let _e279 = phi_1522_;
        phi_1523_ = 0u;
        phi_1520_ = _e279;
    }
    let _e292 = phi_1523_;
    let _e294 = phi_1520_;
    phi_783_ = jj;
    if jj {
        let _e296 = V3_1;
        phi_783_ = (_e296 != 0u);
    }
    let _e299 = phi_783_;
    phi_1576_ = _e114;
    if _e299 {
        if (_e292 != 0u) {
            phi_1538_ = _e292;
        } else {
            let _e303 = m0_.x2_[_e102];
            phi_1538_ = _e303;
        }
        let _e305 = phi_1538_;
        let _e306 = V3_1;
        if (_e306 == (_e305 >> bitcast<u32>(16i))) {
            phi_1556_ = min(_e114, unpack2x16float(_e305).x);
        } else {
            phi_1556_ = 0f;
        }
        let _e314 = phi_1556_;
        phi_1576_ = _e314;
    }
    let _e316 = phi_1576_;
    let _e317 = Y1_1;
    let _e319 = ((_e104 * _e317) * _e316);
    let _e323 = (((_e294 * _e200) * (1f - _e319.w)) + _e319);
    let _e324 = _e323.xyz;
    let _e327 = j.I3_;
    let _e329 = j.J3_;
    if (rj && (_e323.w != 0f)) {
        phi_1581_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e67.x) + (0.00583715f * _e67.y))))) * _e327) + _e329)) + _e324);
    } else {
        phi_1581_ = _e324;
    }
    let _e345 = phi_1581_;
    let _e351 = vec4<f32>(_e345.x, _e323.y, _e323.z, _e323.w);
    let _e357 = vec4<f32>(_e351.x, _e345.y, _e351.z, _e351.w);
    Q1_ = vec4<f32>(_e357.x, _e357.y, _e345.z, _e357.w);
    if (_e292 != 0u) {
        m0_.x2_[_e102] = _e292;
    }
    W4_.x2_[_e102] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) o2_: vec2<f32>, @location(1) V0_: vec4<f32>, @location(4) @interpolate(flat, either) V3_: u32, @location(3) @interpolate(flat, either) Y1_: vec4<f32>, @location(5) @interpolate(flat, either) N1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    o2_1 = o2_;
    V0_1 = V0_;
    V3_1 = V3_;
    Y1_1 = Y1_;
    N1_1 = N1_;
    main_1();
    let _e13 = Q1_;
    return _e13;
}
