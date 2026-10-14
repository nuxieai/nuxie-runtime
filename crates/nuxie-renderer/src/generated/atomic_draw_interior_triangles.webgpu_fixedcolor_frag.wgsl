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
var<private> G0_1: u32;
var<private> n1_1: f32;
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
    var phi_1419_: u32;
    var phi_952_: bool;
    var phi_1424_: f32;
    var phi_1423_: f32;
    var phi_1425_: f32;
    var phi_1428_: f32;
    var phi_1427_: f32;
    var phi_989_: bool;
    var phi_1432_: f32;
    var phi_1429_: f32;
    var phi_1430_: f32;
    var phi_1431_: vec2<f32>;
    var phi_1460_: vec4<f32>;
    var phi_1462_: u32;
    var phi_1461_: u32;
    var phi_1458_: vec4<f32>;
    var phi_1479_: u32;
    var phi_1475_: vec4<f32>;
    var phi_1476_: vec3<f32>;

    let _e63 = gl_FragCoord_1;
    let _e64 = _e63.xy;
    let _e67 = bitcast<vec2<u32>>(vec2<i32>(floor(_e64)));
    let _e69 = j.P6_;
    let _e98 = bitcast<i32>((((((_e67.y >> bitcast<u32>(5u)) * (((_e69 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e67.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e67.x & 28u) << bitcast<u32>(5u)) + ((_e67.y & 28u) << bitcast<u32>(2i)))) + (((_e67.y & 3u) << bitcast<u32>(2i)) + (_e67.x & 3u))));
    let _e101 = R4_.r2_[_e98];
    let _e103 = (_e101 >> bitcast<u32>(17u));
    let _e104 = G0_1;
    if (_e103 == _e104) {
        phi_1419_ = _e101;
    } else {
        phi_1419_ = ((_e104 << bitcast<u32>(17u)) + 65536u);
    }
    let _e110 = phi_1419_;
    let _e111 = n1_1;
    R4_.r2_[_e98] = (_e110 + bitcast<u32>(i32(round((_e111 * 2048f)))));
    phi_1479_ = 0u;
    phi_1475_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e103 != _e104) {
        let _e121 = ((f32((_e101 & 131071u)) * 0.00048828125f) + -32f);
        let _e124 = VC.r2_[_e103];
        phi_1423_ = _e121;
        if ((_e124.x & 768u) != 0u) {
            let _e128 = abs(_e121);
            phi_952_ = bj;
            if bj {
                phi_952_ = ((_e124.x & 512u) != 0u);
            }
            let _e132 = phi_952_;
            phi_1424_ = _e128;
            if _e132 {
                phi_1424_ = (1f - abs(((fract((_e128 * 0.5f)) * 2f) + -1f)));
            }
            let _e140 = phi_1424_;
            phi_1423_ = _e140;
        }
        let _e142 = phi_1423_;
        let _e143 = clamp(_e142, 0f, 1f);
        phi_1427_ = _e143;
        if Xi {
            let _e145 = (_e124.x >> bitcast<u32>(16u));
            phi_1428_ = _e143;
            if (_e145 != 0u) {
                let _e149 = m0_.r2_[_e98];
                if (_e145 == (_e149 >> bitcast<u32>(16i))) {
                    phi_1425_ = min(_e143, unpack2x16float(_e149).x);
                } else {
                    phi_1425_ = 0f;
                }
                let _e157 = phi_1425_;
                phi_1428_ = _e157;
            }
            let _e159 = phi_1428_;
            phi_1427_ = _e159;
        }
        let _e161 = phi_1427_;
        phi_989_ = Yi;
        if Yi {
            phi_989_ = ((_e124.x & 1024u) != 0u);
        }
        let _e165 = phi_989_;
        phi_1432_ = _e161;
        if _e165 {
            let _e166 = (_e103 * 8u);
            let _e170 = JB.r2_[(_e166 + 2u)];
            let _e181 = JB.r2_[(_e166 + 3u)];
            let _e186 = _e181.zw;
            let _e188 = ((abs(((mat2x2<f32>(vec2<f32>(_e170.x, _e170.y), vec2<f32>(_e170.z, _e170.w)) * _e64) + _e181.xy)) * _e186) - _e186);
            phi_1432_ = min(_e161, clamp((min(_e188.x, _e188.y) + 0.5f), 0f, 1f));
        }
        let _e196 = phi_1432_;
        let _e197 = (_e124.x & 15u);
        if (_e197 <= 1u) {
            let _e282 = (Xi && (_e197 == 0u));
            phi_1462_ = 0u;
            if _e282 {
                phi_1462_ = (_e124.y | pack2x16float(vec2<f32>(_e196, 0f)));
            }
            let _e287 = phi_1462_;
            phi_1461_ = _e287;
            phi_1458_ = select(unpack4x8unorm(_e124.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e282));
        } else {
            let _e204 = (_e103 * 8u);
            let _e207 = JB.r2_[_e204];
            let _e218 = JB.r2_[(_e204 + 1u)];
            let _e221 = ((mat2x2<f32>(vec2<f32>(_e207.x, _e207.y), vec2<f32>(_e207.z, _e207.w)) * _e64) + _e218.xy);
            let _e225 = f32(_e197);
            let _e227 = j.M8_;
            let _e229 = j.N8_;
            let _e234 = ((_e218.w * _e227) + _e229);
            switch bitcast<i32>(0u) {
                default: {
                    if (_e225 == 4f) {
                        let _e251 = normalize(vec2<f32>(_e221.x, -(_e221.y)));
                        let _e254 = acos(clamp(_e251.x, -1f, 1f));
                        if (_e251.y >= 0f) {
                            phi_1430_ = _e254;
                        } else {
                            phi_1430_ = -(_e254);
                        }
                        let _e259 = phi_1430_;
                        phi_1431_ = vec2<f32>((_e259 * 0.15915507f), _e234);
                        break;
                    } else {
                        if (_e225 == 2f) {
                            phi_1429_ = _e221.x;
                        } else {
                            phi_1429_ = length(_e221);
                        }
                        let _e241 = phi_1429_;
                        phi_1431_ = vec2<f32>(((clamp(_e241, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e218.z < 0f))) + ((max(0f, _e218.z) * 0.001953125f) + 0.0009765625f)), _e234);
                        break;
                    }
                }
            }
            let _e263 = phi_1431_;
            let _e264 = textureSampleLevel(XC, O8_, _e263, 0f);
            phi_1460_ = _e264;
            if !((Zi && (((_e124.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e268 = (_e264.xyz * _e264.w);
                phi_1460_ = vec4<f32>(_e268.x, _e268.y, _e268.z, (_e264.w * abs(bitcast<f32>(_e124.y))));
            }
            let _e278 = phi_1460_;
            phi_1461_ = 0u;
            phi_1458_ = _e278;
        }
        let _e291 = phi_1461_;
        let _e293 = phi_1458_;
        phi_1479_ = _e291;
        phi_1475_ = (_e293 * _e196);
    }
    let _e296 = phi_1479_;
    let _e298 = phi_1475_;
    let _e299 = _e298.xyz;
    let _e302 = j.F3_;
    let _e304 = j.G3_;
    if (ej && (_e298.w != 0f)) {
        phi_1476_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e63.x) + (0.00583715f * _e63.y))))) * _e302) + _e304)) + _e299);
    } else {
        phi_1476_ = _e299;
    }
    let _e320 = phi_1476_;
    let _e326 = vec4<f32>(_e320.x, _e298.y, _e298.z, _e298.w);
    let _e332 = vec4<f32>(_e326.x, _e320.y, _e326.z, _e326.w);
    L1_ = vec4<f32>(_e332.x, _e332.y, _e320.z, _e332.w);
    if (_e296 != 0u) {
        m0_.r2_[_e98] = _e296;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32, @location(0) @interpolate(flat, either) n1_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    n1_1 = n1_;
    main_1();
    let _e7 = L1_;
    return _e7;
}
