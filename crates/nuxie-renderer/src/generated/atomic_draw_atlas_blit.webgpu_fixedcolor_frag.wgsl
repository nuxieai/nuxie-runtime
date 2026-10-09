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
var<private> G0_1: u32;
@group(0) @binding(10)
var HD: texture_2d<f32>;
@group(3) @binding(10)
var Pa: sampler;
var<private> S2_1: vec2<f32>;
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
    var phi_905_: bool;
    var phi_1326_: f32;
    var phi_1325_: f32;
    var phi_1327_: f32;
    var phi_1330_: f32;
    var phi_1329_: f32;
    var phi_942_: bool;
    var phi_1332_: f32;
    var phi_1358_: u32;
    var phi_1331_: f32;
    var phi_1356_: vec4<f32>;
    var phi_1357_: u32;
    var phi_1354_: vec4<f32>;
    var phi_1369_: vec3<f32>;

    let _e63 = gl_FragCoord_1;
    let _e64 = _e63.xy;
    let _e67 = bitcast<vec2<u32>>(vec2<i32>(floor(_e64)));
    let _e69 = j.P6_;
    let _e98 = bitcast<i32>((((((_e67.y >> bitcast<u32>(5u)) * (((_e69 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e67.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e67.x & 28u) << bitcast<u32>(5u)) + ((_e67.y & 28u) << bitcast<u32>(2i)))) + (((_e67.y & 3u) << bitcast<u32>(2i)) + (_e67.x & 3u))));
    let _e101 = R4_.r2_[_e98];
    let _e103 = (_e101 >> bitcast<u32>(17u));
    let _e104 = G0_1;
    let _e108 = S2_1;
    let _e109 = textureSampleLevel(HD, Pa, _e108, 0f);
    R4_.r2_[_e98] = (((_e104 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e109.x, 0f, 1f) * 2048f)))));
    let _e120 = ((f32((_e101 & 131071u)) * 0.00048828125f) + -32f);
    let _e123 = VC.r2_[_e103];
    phi_1325_ = _e120;
    if ((_e123.x & 768u) != 0u) {
        let _e127 = abs(_e120);
        phi_905_ = aj;
        if aj {
            phi_905_ = ((_e123.x & 512u) != 0u);
        }
        let _e131 = phi_905_;
        phi_1326_ = _e127;
        if _e131 {
            phi_1326_ = (1f - abs(((fract((_e127 * 0.5f)) * 2f) + -1f)));
        }
        let _e139 = phi_1326_;
        phi_1325_ = _e139;
    }
    let _e141 = phi_1325_;
    let _e142 = clamp(_e141, 0f, 1f);
    phi_1329_ = _e142;
    if Wi {
        let _e144 = (_e123.x >> bitcast<u32>(16u));
        phi_1330_ = _e142;
        if (_e144 != 0u) {
            let _e148 = m0_.r2_[_e98];
            if (_e144 == (_e148 >> bitcast<u32>(16i))) {
                phi_1327_ = min(_e142, unpack2x16float(_e148).x);
            } else {
                phi_1327_ = 0f;
            }
            let _e156 = phi_1327_;
            phi_1330_ = _e156;
        }
        let _e158 = phi_1330_;
        phi_1329_ = _e158;
    }
    let _e160 = phi_1329_;
    phi_942_ = Xi;
    if Xi {
        phi_942_ = ((_e123.x & 1024u) != 0u);
    }
    let _e164 = phi_942_;
    phi_1332_ = _e160;
    if _e164 {
        let _e165 = (_e103 * 8u);
        let _e169 = JB.r2_[(_e165 + 2u)];
        let _e180 = JB.r2_[(_e165 + 3u)];
        let _e185 = _e180.zw;
        let _e187 = ((abs(((mat2x2<f32>(vec2<f32>(_e169.x, _e169.y), vec2<f32>(_e169.z, _e169.w)) * _e64) + _e180.xy)) * _e185) - _e185);
        phi_1332_ = min(_e160, clamp((min(_e187.x, _e187.y) + 0.5f), 0f, 1f));
    }
    let _e195 = phi_1332_;
    let _e196 = (_e123.x & 15u);
    if (_e196 <= 1u) {
        let _e206 = (Wi && (_e196 == 0u));
        phi_1358_ = 0u;
        if _e206 {
            phi_1358_ = (_e123.y | pack2x16float(vec2<f32>(_e195, 0f)));
        }
        let _e211 = phi_1358_;
        phi_1357_ = _e211;
        phi_1354_ = select(unpack4x8unorm(_e123.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e206));
    } else {
        let _e214 = (_e103 * 8u);
        let _e217 = JB.r2_[_e214];
        let _e228 = JB.r2_[(_e214 + 1u)];
        let _e231 = ((mat2x2<f32>(vec2<f32>(_e217.x, _e217.y), vec2<f32>(_e217.z, _e217.w)) * _e64) + _e228.xy);
        let _e237 = j.L8_;
        let _e239 = j.M8_;
        if (f32(_e196) == 2f) {
            phi_1331_ = _e231.x;
        } else {
            phi_1331_ = length(_e231);
        }
        let _e249 = phi_1331_;
        let _e255 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e249, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e228.z < 0f))) + ((max(0f, _e228.z) * 0.001953125f) + 0.0009765625f)), ((_e228.w * _e237) + _e239)), 0f);
        phi_1356_ = _e255;
        if !((Yi && (((_e123.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e259 = (_e255.xyz * _e255.w);
            phi_1356_ = vec4<f32>(_e259.x, _e259.y, _e259.z, (_e255.w * abs(bitcast<f32>(_e123.y))));
        }
        let _e269 = phi_1356_;
        phi_1357_ = 0u;
        phi_1354_ = _e269;
    }
    let _e271 = phi_1357_;
    let _e273 = phi_1354_;
    let _e274 = (_e273 * _e195);
    let _e275 = _e274.xyz;
    let _e278 = j.F3_;
    let _e280 = j.G3_;
    if (dj && (_e274.w != 0f)) {
        phi_1369_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e63.x) + (0.00583715f * _e63.y))))) * _e278) + _e280)) + _e275);
    } else {
        phi_1369_ = _e275;
    }
    let _e296 = phi_1369_;
    let _e302 = vec4<f32>(_e296.x, _e274.y, _e274.z, _e274.w);
    let _e308 = vec4<f32>(_e302.x, _e296.y, _e302.z, _e302.w);
    L1_ = vec4<f32>(_e308.x, _e308.y, _e296.z, _e308.w);
    if (_e271 != 0u) {
        m0_.r2_[_e98] = _e271;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32, @location(0) S2_: vec2<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    S2_1 = S2_;
    main_1();
    let _e7 = L1_;
    return _e7;
}
