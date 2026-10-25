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
    H8_: vec4<i32>,
    Fi: vec2<f32>,
    Ie: vec2<f32>,
    r2_: u32,
    Ji: f32,
    z6_: u32,
    i3_: f32,
    Je: f32,
    rg: u32,
    G3_: f32,
    H3_: f32,
    Ke: f32,
    Ci: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct T4Ue {
    v2_: array<u32>,
}

@id(7) override lj: bool = true;
@id(4) override ij: bool = true;
@id(0) override ej: bool = true;
@id(1) override fj: bool = true;
@id(2) override gj: bool = true;

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
var Q8_: sampler;
@group(2) @binding(3)
var<storage, read_write> T4_: T4Ue;
var<private> G0_1: u32;
@group(0) @binding(10)
var JD: texture_2d<f32>;
@group(3) @binding(10)
var Sa: sampler;
var<private> U2_1: vec2<f32>;
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
    var phi_958_: bool;
    var phi_1428_: f32;
    var phi_1427_: f32;
    var phi_1429_: f32;
    var phi_1432_: f32;
    var phi_1431_: f32;
    var phi_995_: bool;
    var phi_1436_: f32;
    var phi_1433_: f32;
    var phi_1434_: f32;
    var phi_1435_: vec2<f32>;
    var phi_1462_: vec4<f32>;
    var phi_1464_: u32;
    var phi_1463_: u32;
    var phi_1460_: vec4<f32>;
    var phi_1477_: vec3<f32>;

    let _e65 = gl_FragCoord_1;
    let _e66 = _e65.xy;
    let _e69 = bitcast<vec2<u32>>(vec2<i32>(floor(_e66)));
    let _e71 = j.T6_;
    let _e100 = bitcast<i32>((((((_e69.y >> bitcast<u32>(5u)) * (((_e71 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e69.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e69.x & 28u) << bitcast<u32>(5u)) + ((_e69.y & 28u) << bitcast<u32>(2i)))) + (((_e69.y & 3u) << bitcast<u32>(2i)) + (_e69.x & 3u))));
    let _e103 = T4_.v2_[_e100];
    let _e105 = (_e103 >> bitcast<u32>(17u));
    let _e106 = G0_1;
    let _e110 = U2_1;
    let _e111 = textureSampleLevel(JD, Sa, _e110, 0f);
    T4_.v2_[_e100] = (((_e106 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e111.x, 0f, 1f) * 2048f)))));
    let _e122 = ((f32((_e103 & 131071u)) * 0.00048828125f) + -32f);
    let _e125 = XC.v2_[_e105];
    phi_1427_ = _e122;
    if ((_e125.x & 768u) != 0u) {
        let _e129 = abs(_e122);
        phi_958_ = ij;
        if ij {
            phi_958_ = ((_e125.x & 512u) != 0u);
        }
        let _e133 = phi_958_;
        phi_1428_ = _e129;
        if _e133 {
            phi_1428_ = (1f - abs(((fract((_e129 * 0.5f)) * 2f) + -1f)));
        }
        let _e141 = phi_1428_;
        phi_1427_ = _e141;
    }
    let _e143 = phi_1427_;
    let _e144 = clamp(_e143, 0f, 1f);
    phi_1431_ = _e144;
    if ej {
        let _e146 = (_e125.x >> bitcast<u32>(16u));
        phi_1432_ = _e144;
        if (_e146 != 0u) {
            let _e150 = m0_.v2_[_e100];
            if (_e146 == (_e150 >> bitcast<u32>(16i))) {
                phi_1429_ = min(_e144, unpack2x16float(_e150).x);
            } else {
                phi_1429_ = 0f;
            }
            let _e158 = phi_1429_;
            phi_1432_ = _e158;
        }
        let _e160 = phi_1432_;
        phi_1431_ = _e160;
    }
    let _e162 = phi_1431_;
    phi_995_ = fj;
    if fj {
        phi_995_ = ((_e125.x & 1024u) != 0u);
    }
    let _e166 = phi_995_;
    phi_1436_ = _e162;
    if _e166 {
        let _e167 = (_e105 * 8u);
        let _e171 = JB.v2_[(_e167 + 2u)];
        let _e182 = JB.v2_[(_e167 + 3u)];
        let _e187 = _e182.zw;
        let _e189 = ((abs(((mat2x2<f32>(vec2<f32>(_e171.x, _e171.y), vec2<f32>(_e171.z, _e171.w)) * _e66) + _e182.xy)) * _e187) - _e187);
        phi_1436_ = min(_e162, clamp((min(_e189.x, _e189.y) + 0.5f), 0f, 1f));
    }
    let _e197 = phi_1436_;
    let _e198 = (_e125.x & 15u);
    if (_e198 <= 1u) {
        let _e283 = (ej && (_e198 == 0u));
        phi_1464_ = 0u;
        if _e283 {
            phi_1464_ = (_e125.y | pack2x16float(vec2<f32>(_e197, 0f)));
        }
        let _e288 = phi_1464_;
        phi_1463_ = _e288;
        phi_1460_ = select(unpack4x8unorm(_e125.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e283));
    } else {
        let _e205 = (_e105 * 8u);
        let _e208 = JB.v2_[_e205];
        let _e219 = JB.v2_[(_e205 + 1u)];
        let _e222 = ((mat2x2<f32>(vec2<f32>(_e208.x, _e208.y), vec2<f32>(_e208.z, _e208.w)) * _e66) + _e219.xy);
        let _e226 = f32(_e198);
        let _e228 = j.O8_;
        let _e230 = j.P8_;
        let _e235 = ((_e219.w * _e228) + _e230);
        switch bitcast<i32>(0u) {
            default: {
                if (_e226 == 4f) {
                    let _e252 = normalize(vec2<f32>(_e222.x, -(_e222.y)));
                    let _e255 = acos(clamp(_e252.x, -1f, 1f));
                    if (_e252.y >= 0f) {
                        phi_1434_ = _e255;
                    } else {
                        phi_1434_ = -(_e255);
                    }
                    let _e260 = phi_1434_;
                    phi_1435_ = vec2<f32>((_e260 * 0.15915507f), _e235);
                    break;
                } else {
                    if (_e226 == 2f) {
                        phi_1433_ = _e222.x;
                    } else {
                        phi_1433_ = length(_e222);
                    }
                    let _e242 = phi_1433_;
                    phi_1435_ = vec2<f32>(((clamp(_e242, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e219.z < 0f))) + ((max(0f, _e219.z) * 0.001953125f) + 0.0009765625f)), _e235);
                    break;
                }
            }
        }
        let _e264 = phi_1435_;
        let _e265 = textureSampleLevel(ZC, Q8_, _e264, 0f);
        phi_1462_ = _e265;
        if !((gj && (((_e125.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e269 = (_e265.xyz * _e265.w);
            phi_1462_ = vec4<f32>(_e269.x, _e269.y, _e269.z, (_e265.w * abs(bitcast<f32>(_e125.y))));
        }
        let _e279 = phi_1462_;
        phi_1463_ = 0u;
        phi_1460_ = _e279;
    }
    let _e292 = phi_1463_;
    let _e294 = phi_1460_;
    let _e295 = (_e294 * _e197);
    let _e296 = _e295.xyz;
    let _e299 = j.G3_;
    let _e301 = j.H3_;
    if (lj && (_e295.w != 0f)) {
        phi_1477_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e65.x) + (0.00583715f * _e65.y))))) * _e299) + _e301)) + _e296);
    } else {
        phi_1477_ = _e296;
    }
    let _e317 = phi_1477_;
    let _e323 = vec4<f32>(_e317.x, _e295.y, _e295.z, _e295.w);
    let _e329 = vec4<f32>(_e323.x, _e317.y, _e323.z, _e323.w);
    Q1_ = vec4<f32>(_e329.x, _e329.y, _e317.z, _e329.w);
    if (_e292 != 0u) {
        m0_.v2_[_e100] = _e292;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32, @location(0) U2_: vec2<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    U2_1 = U2_;
    main_1();
    let _e7 = Q1_;
    return _e7;
}
