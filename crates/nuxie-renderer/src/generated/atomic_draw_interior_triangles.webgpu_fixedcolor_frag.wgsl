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
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> I0_1: u32;
var<private> o1_1: f32;
var<private> Q1_: vec4<f32>;
@group(3) @binding(8)
var fb: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(10)
var RB: texture_2d<f32>;
@group(1) @binding(12)
var v4_: sampler;

fn main_1() {
    var phi_1418_: u32;
    var phi_957_: bool;
    var phi_1423_: f32;
    var phi_1422_: f32;
    var phi_1424_: f32;
    var phi_1427_: f32;
    var phi_1426_: f32;
    var phi_994_: bool;
    var phi_1431_: f32;
    var phi_1428_: f32;
    var phi_1429_: f32;
    var phi_1430_: vec2<f32>;
    var phi_1459_: vec4<f32>;
    var phi_1461_: u32;
    var phi_1460_: u32;
    var phi_1457_: vec4<f32>;
    var phi_1478_: u32;
    var phi_1474_: vec4<f32>;
    var phi_1475_: vec3<f32>;

    let _e65 = gl_FragCoord_1;
    let _e66 = _e65.xy;
    let _e69 = bitcast<vec2<u32>>(vec2<i32>(floor(_e66)));
    let _e71 = j.R6_;
    let _e100 = bitcast<i32>((((((_e69.y >> bitcast<u32>(5u)) * (((_e71 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e69.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e69.x & 28u) << bitcast<u32>(5u)) + ((_e69.y & 28u) << bitcast<u32>(2i)))) + (((_e69.y & 3u) << bitcast<u32>(2i)) + (_e69.x & 3u))));
    let _e103 = W4_.x2_[_e100];
    let _e105 = (_e103 >> bitcast<u32>(17u));
    let _e106 = I0_1;
    if (_e105 == _e106) {
        phi_1418_ = _e103;
    } else {
        phi_1418_ = ((_e106 << bitcast<u32>(17u)) + 65536u);
    }
    let _e112 = phi_1418_;
    let _e113 = o1_1;
    W4_.x2_[_e100] = (_e112 + bitcast<u32>(i32(round((_e113 * 2048f)))));
    phi_1478_ = 0u;
    phi_1474_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e105 != _e106) {
        let _e123 = ((f32((_e103 & 131071u)) * 0.00048828125f) + -32f);
        let _e128 = JB.x2_[((_e105 * 4u) + 1u)];
        phi_1422_ = _e123;
        if ((_e128.z & 768u) != 0u) {
            let _e132 = abs(_e123);
            phi_957_ = oj;
            if oj {
                phi_957_ = ((_e128.z & 512u) != 0u);
            }
            let _e136 = phi_957_;
            phi_1423_ = _e132;
            if _e136 {
                phi_1423_ = (1f - abs(((fract((_e132 * 0.5f)) * 2f) + -1f)));
            }
            let _e144 = phi_1423_;
            phi_1422_ = _e144;
        }
        let _e146 = phi_1422_;
        let _e147 = clamp(_e146, 0f, 1f);
        phi_1426_ = _e147;
        if jj {
            let _e149 = (_e128.z >> bitcast<u32>(16u));
            phi_1427_ = _e147;
            if (_e149 != 0u) {
                let _e153 = m0_.x2_[_e100];
                if (_e149 == (_e153 >> bitcast<u32>(16i))) {
                    phi_1424_ = min(_e147, unpack2x16float(_e153).x);
                } else {
                    phi_1424_ = 0f;
                }
                let _e161 = phi_1424_;
                phi_1427_ = _e161;
            }
            let _e163 = phi_1427_;
            phi_1426_ = _e163;
        }
        let _e165 = phi_1426_;
        phi_994_ = kj;
        if kj {
            phi_994_ = ((_e128.z & 1024u) != 0u);
        }
        let _e169 = phi_994_;
        phi_1431_ = _e165;
        if _e169 {
            let _e170 = (_e105 * 8u);
            let _e174 = KB.x2_[(_e170 + 2u)];
            let _e185 = KB.x2_[(_e170 + 3u)];
            let _e190 = _e185.zw;
            let _e192 = ((abs(((mat2x2<f32>(vec2<f32>(_e174.x, _e174.y), vec2<f32>(_e174.z, _e174.w)) * _e66) + _e185.xy)) * _e190) - _e190);
            phi_1431_ = min(_e165, clamp((min(_e192.x, _e192.y) + 0.5f), 0f, 1f));
        }
        let _e200 = phi_1431_;
        let _e201 = (_e128.z & 15u);
        if (_e201 <= 1u) {
            let _e283 = (jj && (_e201 == 0u));
            phi_1461_ = 0u;
            if _e283 {
                phi_1461_ = (_e128.w | pack2x16float(vec2<f32>(_e200, 0f)));
            }
            let _e288 = phi_1461_;
            phi_1460_ = _e288;
            phi_1457_ = select(unpack4x8unorm(_e128.w), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e283));
        } else {
            let _e208 = (_e105 * 8u);
            let _e211 = KB.x2_[_e208];
            let _e222 = KB.x2_[(_e208 + 1u)];
            let _e225 = ((mat2x2<f32>(vec2<f32>(_e211.x, _e211.y), vec2<f32>(_e211.z, _e211.w)) * _e66) + _e222.xy);
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
                            phi_1429_ = _e255;
                        } else {
                            phi_1429_ = -(_e255);
                        }
                        let _e260 = phi_1429_;
                        phi_1430_ = vec2<f32>((_e260 * 0.15915494f), _e238);
                        break;
                    } else {
                        if (_e229 == 2f) {
                            phi_1428_ = _e225.x;
                        } else {
                            phi_1428_ = length(_e225);
                        }
                        let _e246 = phi_1428_;
                        phi_1430_ = vec2<f32>(((clamp(_e246, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e222.z < 0f))) + ((max(0f, _e222.z) * 0.001953125f) + 0.0009765625f)), _e238);
                        break;
                    }
                }
            }
            let _e264 = phi_1430_;
            let _e265 = textureSampleLevel(YC, Q8_, _e264, 0f);
            phi_1459_ = _e265;
            if !((lj && (((_e128.z >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e269 = (_e265.xyz * _e265.w);
                phi_1459_ = vec4<f32>(_e269.x, _e269.y, _e269.z, (_e265.w * abs(bitcast<f32>(_e128.w))));
            }
            let _e279 = phi_1459_;
            phi_1460_ = 0u;
            phi_1457_ = _e279;
        }
        let _e292 = phi_1460_;
        let _e294 = phi_1457_;
        phi_1478_ = _e292;
        phi_1474_ = (_e294 * _e200);
    }
    let _e297 = phi_1478_;
    let _e299 = phi_1474_;
    let _e300 = _e299.xyz;
    let _e303 = j.I3_;
    let _e305 = j.J3_;
    if (rj && (_e299.w != 0f)) {
        phi_1475_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e65.x) + (0.00583715f * _e65.y))))) * _e303) + _e305)) + _e300);
    } else {
        phi_1475_ = _e300;
    }
    let _e321 = phi_1475_;
    let _e327 = vec4<f32>(_e321.x, _e299.y, _e299.z, _e299.w);
    let _e333 = vec4<f32>(_e327.x, _e321.y, _e327.z, _e327.w);
    Q1_ = vec4<f32>(_e333.x, _e333.y, _e321.z, _e333.w);
    if (_e297 != 0u) {
        m0_.x2_[_e100] = _e297;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) I0_: u32, @location(0) @interpolate(flat, either) o1_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    I0_1 = I0_;
    o1_1 = o1_;
    main_1();
    let _e7 = Q1_;
    return _e7;
}
