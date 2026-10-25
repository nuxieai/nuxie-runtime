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
    var phi_840_: bool;
    var phi_1268_: f32;
    var phi_1267_: f32;
    var phi_1269_: f32;
    var phi_1272_: f32;
    var phi_1271_: f32;
    var phi_877_: bool;
    var phi_1287_: f32;
    var phi_1273_: f32;
    var phi_1274_: f32;
    var phi_1275_: vec2<f32>;
    var phi_1294_: vec4<f32>;
    var phi_1292_: vec4<f32>;
    var phi_1295_: vec3<f32>;

    let _e60 = gl_FragCoord_1;
    let _e61 = _e60.xy;
    let _e64 = bitcast<vec2<u32>>(vec2<i32>(floor(_e61)));
    let _e66 = j.T6_;
    let _e95 = bitcast<i32>((((((_e64.y >> bitcast<u32>(5u)) * (((_e66 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e64.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e64.x & 28u) << bitcast<u32>(5u)) + ((_e64.y & 28u) << bitcast<u32>(2i)))) + (((_e64.y & 3u) << bitcast<u32>(2i)) + (_e64.x & 3u))));
    let _e98 = W4_.v2_[_e95];
    let _e102 = ((f32((_e98 & 131071u)) * 0.00048828125f) + -32f);
    let _e104 = (_e98 >> bitcast<u32>(17u));
    let _e107 = XC.v2_[_e104];
    phi_1267_ = _e102;
    if ((_e107.x & 768u) != 0u) {
        let _e111 = abs(_e102);
        phi_840_ = oj;
        if oj {
            phi_840_ = ((_e107.x & 512u) != 0u);
        }
        let _e115 = phi_840_;
        phi_1268_ = _e111;
        if _e115 {
            phi_1268_ = (1f - abs(((fract((_e111 * 0.5f)) * 2f) + -1f)));
        }
        let _e123 = phi_1268_;
        phi_1267_ = _e123;
    }
    let _e125 = phi_1267_;
    let _e126 = clamp(_e125, 0f, 1f);
    phi_1271_ = _e126;
    if jj {
        let _e128 = (_e107.x >> bitcast<u32>(16u));
        phi_1272_ = _e126;
        if (_e128 != 0u) {
            let _e132 = m0_.v2_[_e95];
            if (_e128 == (_e132 >> bitcast<u32>(16i))) {
                phi_1269_ = min(_e126, unpack2x16float(_e132).x);
            } else {
                phi_1269_ = 0f;
            }
            let _e140 = phi_1269_;
            phi_1272_ = _e140;
        }
        let _e142 = phi_1272_;
        phi_1271_ = _e142;
    }
    let _e144 = phi_1271_;
    phi_877_ = kj;
    if kj {
        phi_877_ = ((_e107.x & 1024u) != 0u);
    }
    let _e148 = phi_877_;
    phi_1287_ = _e144;
    if _e148 {
        let _e149 = (_e104 * 8u);
        let _e153 = JB.v2_[(_e149 + 2u)];
        let _e164 = JB.v2_[(_e149 + 3u)];
        let _e169 = _e164.zw;
        let _e171 = ((abs(((mat2x2<f32>(vec2<f32>(_e153.x, _e153.y), vec2<f32>(_e153.z, _e153.w)) * _e61) + _e164.xy)) * _e169) - _e169);
        phi_1287_ = min(_e144, clamp((min(_e171.x, _e171.y) + 0.5f), 0f, 1f));
    }
    let _e179 = phi_1287_;
    let _e180 = (_e107.x & 15u);
    if (_e180 <= 1u) {
        phi_1292_ = select(unpack4x8unorm(_e107.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((jj && (_e180 == 0u))));
    } else {
        let _e187 = (_e104 * 8u);
        let _e190 = JB.v2_[_e187];
        let _e201 = JB.v2_[(_e187 + 1u)];
        let _e204 = ((mat2x2<f32>(vec2<f32>(_e190.x, _e190.y), vec2<f32>(_e190.z, _e190.w)) * _e61) + _e201.xy);
        let _e208 = f32(_e180);
        let _e210 = j.Q8_;
        let _e212 = j.R8_;
        let _e217 = ((_e201.w * _e210) + _e212);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e208 == 4f)) {
                    let _e231 = normalize(_e204);
                    let _e234 = acos(clamp(_e231.x, -1f, 1f));
                    if (_e231.y >= 0f) {
                        phi_1274_ = _e234;
                    } else {
                        phi_1274_ = -(_e234);
                    }
                    let _e239 = phi_1274_;
                    phi_1275_ = vec2<f32>((_e239 * 0.15915494f), _e217);
                    break;
                } else {
                    if (_e208 == 2f) {
                        phi_1273_ = _e204.x;
                    } else {
                        phi_1273_ = length(_e204);
                    }
                    let _e225 = phi_1273_;
                    phi_1275_ = vec2<f32>(((clamp(_e225, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e201.z < 0f))) + ((max(0f, _e201.z) * 0.001953125f) + 0.0009765625f)), _e217);
                    break;
                }
            }
        }
        let _e243 = phi_1275_;
        let _e244 = textureSampleLevel(ZC, S8_, _e243, 0f);
        phi_1294_ = _e244;
        if !((lj && (((_e107.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e248 = (_e244.xyz * _e244.w);
            phi_1294_ = vec4<f32>(_e248.x, _e248.y, _e248.z, (_e244.w * abs(bitcast<f32>(_e107.y))));
        }
        let _e258 = phi_1294_;
        phi_1292_ = _e258;
    }
    let _e266 = phi_1292_;
    let _e267 = (_e266 * _e179);
    let _e268 = _e267.xyz;
    let _e271 = j.H3_;
    let _e273 = j.I3_;
    if (rj && (_e267.w != 0f)) {
        phi_1295_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e60.x) + (0.00583715f * _e60.y))))) * _e271) + _e273)) + _e268);
    } else {
        phi_1295_ = _e268;
    }
    let _e289 = phi_1295_;
    let _e295 = vec4<f32>(_e289.x, _e267.y, _e267.z, _e267.w);
    let _e301 = vec4<f32>(_e295.x, _e289.y, _e295.z, _e295.w);
    Q1_ = vec4<f32>(_e301.x, _e301.y, _e289.z, _e301.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = Q1_;
    return _e3;
}
