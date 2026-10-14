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

@id(7) override ej: bool = true;
@id(4) override bj: bool = true;
@id(0) override Xi: bool = true;
@id(1) override Yi: bool = true;
@id(2) override Zi: bool = true;

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
@group(2) @binding(3)
var<storage, read_write> R4_: R4Pe;
var<private> L1_: vec4<f32>;
@group(3) @binding(9)
var ab: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var U4_: sampler;

fn main_1() {
    var phi_843_: bool;
    var phi_1277_: f32;
    var phi_1276_: f32;
    var phi_1278_: f32;
    var phi_1281_: f32;
    var phi_1280_: f32;
    var phi_880_: bool;
    var phi_1296_: f32;
    var phi_1282_: f32;
    var phi_1283_: f32;
    var phi_1284_: vec2<f32>;
    var phi_1303_: vec4<f32>;
    var phi_1301_: vec4<f32>;
    var phi_1304_: vec3<f32>;

    let _e59 = gl_FragCoord_1;
    let _e60 = _e59.xy;
    let _e63 = bitcast<vec2<u32>>(vec2<i32>(floor(_e60)));
    let _e65 = j.P6_;
    let _e94 = bitcast<i32>((((((_e63.y >> bitcast<u32>(5u)) * (((_e65 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e63.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e63.x & 28u) << bitcast<u32>(5u)) + ((_e63.y & 28u) << bitcast<u32>(2i)))) + (((_e63.y & 3u) << bitcast<u32>(2i)) + (_e63.x & 3u))));
    let _e97 = R4_.r2_[_e94];
    let _e101 = ((f32((_e97 & 131071u)) * 0.00048828125f) + -32f);
    let _e103 = (_e97 >> bitcast<u32>(17u));
    let _e106 = VC.r2_[_e103];
    phi_1276_ = _e101;
    if ((_e106.x & 768u) != 0u) {
        let _e110 = abs(_e101);
        phi_843_ = bj;
        if bj {
            phi_843_ = ((_e106.x & 512u) != 0u);
        }
        let _e114 = phi_843_;
        phi_1277_ = _e110;
        if _e114 {
            phi_1277_ = (1f - abs(((fract((_e110 * 0.5f)) * 2f) + -1f)));
        }
        let _e122 = phi_1277_;
        phi_1276_ = _e122;
    }
    let _e124 = phi_1276_;
    let _e125 = clamp(_e124, 0f, 1f);
    phi_1280_ = _e125;
    if Xi {
        let _e127 = (_e106.x >> bitcast<u32>(16u));
        phi_1281_ = _e125;
        if (_e127 != 0u) {
            let _e131 = m0_.r2_[_e94];
            if (_e127 == (_e131 >> bitcast<u32>(16i))) {
                phi_1278_ = min(_e125, unpack2x16float(_e131).x);
            } else {
                phi_1278_ = 0f;
            }
            let _e139 = phi_1278_;
            phi_1281_ = _e139;
        }
        let _e141 = phi_1281_;
        phi_1280_ = _e141;
    }
    let _e143 = phi_1280_;
    phi_880_ = Yi;
    if Yi {
        phi_880_ = ((_e106.x & 1024u) != 0u);
    }
    let _e147 = phi_880_;
    phi_1296_ = _e143;
    if _e147 {
        let _e148 = (_e103 * 8u);
        let _e152 = JB.r2_[(_e148 + 2u)];
        let _e163 = JB.r2_[(_e148 + 3u)];
        let _e168 = _e163.zw;
        let _e170 = ((abs(((mat2x2<f32>(vec2<f32>(_e152.x, _e152.y), vec2<f32>(_e152.z, _e152.w)) * _e60) + _e163.xy)) * _e168) - _e168);
        phi_1296_ = min(_e143, clamp((min(_e170.x, _e170.y) + 0.5f), 0f, 1f));
    }
    let _e178 = phi_1296_;
    let _e179 = (_e106.x & 15u);
    if (_e179 <= 1u) {
        phi_1301_ = select(unpack4x8unorm(_e106.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((Xi && (_e179 == 0u))));
    } else {
        let _e186 = (_e103 * 8u);
        let _e189 = JB.r2_[_e186];
        let _e200 = JB.r2_[(_e186 + 1u)];
        let _e203 = ((mat2x2<f32>(vec2<f32>(_e189.x, _e189.y), vec2<f32>(_e189.z, _e189.w)) * _e60) + _e200.xy);
        let _e207 = f32(_e179);
        let _e209 = j.M8_;
        let _e211 = j.N8_;
        let _e216 = ((_e200.w * _e209) + _e211);
        switch bitcast<i32>(0u) {
            default: {
                if (_e207 == 4f) {
                    let _e233 = normalize(vec2<f32>(_e203.x, -(_e203.y)));
                    let _e236 = acos(clamp(_e233.x, -1f, 1f));
                    if (_e233.y >= 0f) {
                        phi_1283_ = _e236;
                    } else {
                        phi_1283_ = -(_e236);
                    }
                    let _e241 = phi_1283_;
                    phi_1284_ = vec2<f32>((_e241 * 0.15915507f), _e216);
                    break;
                } else {
                    if (_e207 == 2f) {
                        phi_1282_ = _e203.x;
                    } else {
                        phi_1282_ = length(_e203);
                    }
                    let _e223 = phi_1282_;
                    phi_1284_ = vec2<f32>(((clamp(_e223, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e200.z < 0f))) + ((max(0f, _e200.z) * 0.001953125f) + 0.0009765625f)), _e216);
                    break;
                }
            }
        }
        let _e245 = phi_1284_;
        let _e246 = textureSampleLevel(XC, O8_, _e245, 0f);
        phi_1303_ = _e246;
        if !((Zi && (((_e106.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e250 = (_e246.xyz * _e246.w);
            phi_1303_ = vec4<f32>(_e250.x, _e250.y, _e250.z, (_e246.w * abs(bitcast<f32>(_e106.y))));
        }
        let _e260 = phi_1303_;
        phi_1301_ = _e260;
    }
    let _e268 = phi_1301_;
    let _e269 = (_e268 * _e178);
    let _e270 = _e269.xyz;
    let _e273 = j.F3_;
    let _e275 = j.G3_;
    if (ej && (_e269.w != 0f)) {
        phi_1304_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e59.x) + (0.00583715f * _e59.y))))) * _e273) + _e275)) + _e270);
    } else {
        phi_1304_ = _e270;
    }
    let _e291 = phi_1304_;
    let _e297 = vec4<f32>(_e291.x, _e269.y, _e269.z, _e269.w);
    let _e303 = vec4<f32>(_e297.x, _e291.y, _e297.z, _e297.w);
    L1_ = vec4<f32>(_e303.x, _e303.y, _e291.z, _e303.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = L1_;
    return _e3;
}
