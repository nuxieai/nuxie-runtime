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
    wd: f32,
    Ce: f32,
    Hg: f32,
    Ig: f32,
    P6_: u32,
    Ca: u32,
    tg: u32,
    ug: u32,
    E8_: vec4<i32>,
    yi: vec2<f32>,
    De: vec2<f32>,
    q2_: u32,
    Ci: f32,
    w6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    F3_: f32,
    G3_: f32,
    Fe: f32,
    vi: u32,
    Ba: u32,
    L8_: f32,
    M8_: f32,
}

struct R4Pe {
    r2_: array<u32>,
}

@id(7) override dj: bool = true;
@id(4) override aj: bool = true;
@id(0) override Wi: bool = true;
@id(1) override Xi: bool = true;
@id(2) override Yi: bool = true;

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
var N8_: sampler;
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
    var phi_790_: bool;
    var phi_1175_: f32;
    var phi_1174_: f32;
    var phi_1176_: f32;
    var phi_1179_: f32;
    var phi_1178_: f32;
    var phi_827_: bool;
    var phi_1192_: f32;
    var phi_1180_: f32;
    var phi_1197_: vec4<f32>;
    var phi_1195_: vec4<f32>;
    var phi_1198_: vec3<f32>;

    let _e57 = gl_FragCoord_1;
    let _e58 = _e57.xy;
    let _e61 = bitcast<vec2<u32>>(vec2<i32>(floor(_e58)));
    let _e63 = j.P6_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e95 = R4_.r2_[_e92];
    let _e99 = ((f32((_e95 & 131071u)) * 0.00048828125f) + -32f);
    let _e101 = (_e95 >> bitcast<u32>(17u));
    let _e104 = VC.r2_[_e101];
    phi_1174_ = _e99;
    if ((_e104.x & 768u) != 0u) {
        let _e108 = abs(_e99);
        phi_790_ = aj;
        if aj {
            phi_790_ = ((_e104.x & 512u) != 0u);
        }
        let _e112 = phi_790_;
        phi_1175_ = _e108;
        if _e112 {
            phi_1175_ = (1f - abs(((fract((_e108 * 0.5f)) * 2f) + -1f)));
        }
        let _e120 = phi_1175_;
        phi_1174_ = _e120;
    }
    let _e122 = phi_1174_;
    let _e123 = clamp(_e122, 0f, 1f);
    phi_1178_ = _e123;
    if Wi {
        let _e125 = (_e104.x >> bitcast<u32>(16u));
        phi_1179_ = _e123;
        if (_e125 != 0u) {
            let _e129 = m0_.r2_[_e92];
            if (_e125 == (_e129 >> bitcast<u32>(16i))) {
                phi_1176_ = min(_e123, unpack2x16float(_e129).x);
            } else {
                phi_1176_ = 0f;
            }
            let _e137 = phi_1176_;
            phi_1179_ = _e137;
        }
        let _e139 = phi_1179_;
        phi_1178_ = _e139;
    }
    let _e141 = phi_1178_;
    phi_827_ = Xi;
    if Xi {
        phi_827_ = ((_e104.x & 1024u) != 0u);
    }
    let _e145 = phi_827_;
    phi_1192_ = _e141;
    if _e145 {
        let _e146 = (_e101 * 8u);
        let _e150 = JB.r2_[(_e146 + 2u)];
        let _e161 = JB.r2_[(_e146 + 3u)];
        let _e166 = _e161.zw;
        let _e168 = ((abs(((mat2x2<f32>(vec2<f32>(_e150.x, _e150.y), vec2<f32>(_e150.z, _e150.w)) * _e58) + _e161.xy)) * _e166) - _e166);
        phi_1192_ = min(_e141, clamp((min(_e168.x, _e168.y) + 0.5f), 0f, 1f));
    }
    let _e176 = phi_1192_;
    let _e177 = (_e104.x & 15u);
    if (_e177 <= 1u) {
        phi_1195_ = select(unpack4x8unorm(_e104.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((Wi && (_e177 == 0u))));
    } else {
        let _e190 = (_e101 * 8u);
        let _e193 = JB.r2_[_e190];
        let _e204 = JB.r2_[(_e190 + 1u)];
        let _e207 = ((mat2x2<f32>(vec2<f32>(_e193.x, _e193.y), vec2<f32>(_e193.z, _e193.w)) * _e58) + _e204.xy);
        let _e213 = j.L8_;
        let _e215 = j.M8_;
        if (f32(_e177) == 2f) {
            phi_1180_ = _e207.x;
        } else {
            phi_1180_ = length(_e207);
        }
        let _e225 = phi_1180_;
        let _e231 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e225, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e204.z < 0f))) + ((max(0f, _e204.z) * 0.001953125f) + 0.0009765625f)), ((_e204.w * _e213) + _e215)), 0f);
        phi_1197_ = _e231;
        if !((Yi && (((_e104.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e235 = (_e231.xyz * _e231.w);
            phi_1197_ = vec4<f32>(_e235.x, _e235.y, _e235.z, (_e231.w * abs(bitcast<f32>(_e104.y))));
        }
        let _e245 = phi_1197_;
        phi_1195_ = _e245;
    }
    let _e247 = phi_1195_;
    let _e248 = (_e247 * _e176);
    let _e249 = _e248.xyz;
    let _e252 = j.F3_;
    let _e254 = j.G3_;
    if (dj && (_e248.w != 0f)) {
        phi_1198_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e252) + _e254)) + _e249);
    } else {
        phi_1198_ = _e249;
    }
    let _e270 = phi_1198_;
    let _e276 = vec4<f32>(_e270.x, _e248.y, _e248.z, _e248.w);
    let _e282 = vec4<f32>(_e276.x, _e270.y, _e276.z, _e276.w);
    L1_ = vec4<f32>(_e282.x, _e282.y, _e270.z, _e282.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = L1_;
    return _e3;
}
