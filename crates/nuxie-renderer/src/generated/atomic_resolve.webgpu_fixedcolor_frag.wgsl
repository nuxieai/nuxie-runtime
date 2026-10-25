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
    var phi_845_: bool;
    var phi_1280_: f32;
    var phi_1279_: f32;
    var phi_1281_: f32;
    var phi_1284_: f32;
    var phi_1283_: f32;
    var phi_882_: bool;
    var phi_1299_: f32;
    var phi_1285_: f32;
    var phi_1286_: f32;
    var phi_1287_: vec2<f32>;
    var phi_1306_: vec4<f32>;
    var phi_1304_: vec4<f32>;
    var phi_1307_: vec3<f32>;

    let _e60 = gl_FragCoord_1;
    let _e61 = _e60.xy;
    let _e64 = bitcast<vec2<u32>>(vec2<i32>(floor(_e61)));
    let _e66 = j.T6_;
    let _e95 = bitcast<i32>((((((_e64.y >> bitcast<u32>(5u)) * (((_e66 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e64.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e64.x & 28u) << bitcast<u32>(5u)) + ((_e64.y & 28u) << bitcast<u32>(2i)))) + (((_e64.y & 3u) << bitcast<u32>(2i)) + (_e64.x & 3u))));
    let _e98 = T4_.v2_[_e95];
    let _e102 = ((f32((_e98 & 131071u)) * 0.00048828125f) + -32f);
    let _e104 = (_e98 >> bitcast<u32>(17u));
    let _e107 = XC.v2_[_e104];
    phi_1279_ = _e102;
    if ((_e107.x & 768u) != 0u) {
        let _e111 = abs(_e102);
        phi_845_ = kj;
        if kj {
            phi_845_ = ((_e107.x & 512u) != 0u);
        }
        let _e115 = phi_845_;
        phi_1280_ = _e111;
        if _e115 {
            phi_1280_ = (1f - abs(((fract((_e111 * 0.5f)) * 2f) + -1f)));
        }
        let _e123 = phi_1280_;
        phi_1279_ = _e123;
    }
    let _e125 = phi_1279_;
    let _e126 = clamp(_e125, 0f, 1f);
    phi_1283_ = _e126;
    if fj {
        let _e128 = (_e107.x >> bitcast<u32>(16u));
        phi_1284_ = _e126;
        if (_e128 != 0u) {
            let _e132 = m0_.v2_[_e95];
            if (_e128 == (_e132 >> bitcast<u32>(16i))) {
                phi_1281_ = min(_e126, unpack2x16float(_e132).x);
            } else {
                phi_1281_ = 0f;
            }
            let _e140 = phi_1281_;
            phi_1284_ = _e140;
        }
        let _e142 = phi_1284_;
        phi_1283_ = _e142;
    }
    let _e144 = phi_1283_;
    phi_882_ = gj;
    if gj {
        phi_882_ = ((_e107.x & 1024u) != 0u);
    }
    let _e148 = phi_882_;
    phi_1299_ = _e144;
    if _e148 {
        let _e149 = (_e104 * 8u);
        let _e153 = JB.v2_[(_e149 + 2u)];
        let _e164 = JB.v2_[(_e149 + 3u)];
        let _e169 = _e164.zw;
        let _e171 = ((abs(((mat2x2<f32>(vec2<f32>(_e153.x, _e153.y), vec2<f32>(_e153.z, _e153.w)) * _e61) + _e164.xy)) * _e169) - _e169);
        phi_1299_ = min(_e144, clamp((min(_e171.x, _e171.y) + 0.5f), 0f, 1f));
    }
    let _e179 = phi_1299_;
    let _e180 = (_e107.x & 15u);
    if (_e180 <= 1u) {
        phi_1304_ = select(unpack4x8unorm(_e107.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((fj && (_e180 == 0u))));
    } else {
        let _e187 = (_e104 * 8u);
        let _e190 = JB.v2_[_e187];
        let _e201 = JB.v2_[(_e187 + 1u)];
        let _e204 = ((mat2x2<f32>(vec2<f32>(_e190.x, _e190.y), vec2<f32>(_e190.z, _e190.w)) * _e61) + _e201.xy);
        let _e208 = f32(_e180);
        let _e210 = j.P8_;
        let _e212 = j.Q8_;
        let _e217 = ((_e201.w * _e210) + _e212);
        switch bitcast<i32>(0u) {
            default: {
                if (oj && (_e208 == 4f)) {
                    let _e235 = normalize(vec2<f32>(_e204.x, -(_e204.y)));
                    let _e238 = acos(clamp(_e235.x, -1f, 1f));
                    if (_e235.y >= 0f) {
                        phi_1286_ = _e238;
                    } else {
                        phi_1286_ = -(_e238);
                    }
                    let _e243 = phi_1286_;
                    phi_1287_ = vec2<f32>((_e243 * 0.15915494f), _e217);
                    break;
                } else {
                    if (_e208 == 2f) {
                        phi_1285_ = _e204.x;
                    } else {
                        phi_1285_ = length(_e204);
                    }
                    let _e225 = phi_1285_;
                    phi_1287_ = vec2<f32>(((clamp(_e225, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e201.z < 0f))) + ((max(0f, _e201.z) * 0.001953125f) + 0.0009765625f)), _e217);
                    break;
                }
            }
        }
        let _e247 = phi_1287_;
        let _e248 = textureSampleLevel(ZC, R8_, _e247, 0f);
        phi_1306_ = _e248;
        if !((hj && (((_e107.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e252 = (_e248.xyz * _e248.w);
            phi_1306_ = vec4<f32>(_e252.x, _e252.y, _e252.z, (_e248.w * abs(bitcast<f32>(_e107.y))));
        }
        let _e262 = phi_1306_;
        phi_1304_ = _e262;
    }
    let _e270 = phi_1304_;
    let _e271 = (_e270 * _e179);
    let _e272 = _e271.xyz;
    let _e275 = j.G3_;
    let _e277 = j.H3_;
    if (nj && (_e271.w != 0f)) {
        phi_1307_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e60.x) + (0.00583715f * _e60.y))))) * _e275) + _e277)) + _e272);
    } else {
        phi_1307_ = _e272;
    }
    let _e293 = phi_1307_;
    let _e299 = vec4<f32>(_e293.x, _e271.y, _e271.z, _e271.w);
    let _e305 = vec4<f32>(_e299.x, _e293.y, _e299.z, _e299.w);
    Q1_ = vec4<f32>(_e305.x, _e305.y, _e293.z, _e305.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = Q1_;
    return _e3;
}
