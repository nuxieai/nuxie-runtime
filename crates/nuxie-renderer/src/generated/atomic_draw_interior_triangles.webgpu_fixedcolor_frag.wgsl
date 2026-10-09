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
    var phi_1317_: u32;
    var phi_899_: bool;
    var phi_1322_: f32;
    var phi_1321_: f32;
    var phi_1323_: f32;
    var phi_1326_: f32;
    var phi_1325_: f32;
    var phi_936_: bool;
    var phi_1328_: f32;
    var phi_1356_: u32;
    var phi_1327_: f32;
    var phi_1354_: vec4<f32>;
    var phi_1355_: u32;
    var phi_1352_: vec4<f32>;
    var phi_1371_: u32;
    var phi_1367_: vec4<f32>;
    var phi_1368_: vec3<f32>;

    let _e61 = gl_FragCoord_1;
    let _e62 = _e61.xy;
    let _e65 = bitcast<vec2<u32>>(vec2<i32>(floor(_e62)));
    let _e67 = j.P6_;
    let _e96 = bitcast<i32>((((((_e65.y >> bitcast<u32>(5u)) * (((_e67 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e65.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e65.x & 28u) << bitcast<u32>(5u)) + ((_e65.y & 28u) << bitcast<u32>(2i)))) + (((_e65.y & 3u) << bitcast<u32>(2i)) + (_e65.x & 3u))));
    let _e99 = R4_.r2_[_e96];
    let _e101 = (_e99 >> bitcast<u32>(17u));
    let _e102 = G0_1;
    if (_e101 == _e102) {
        phi_1317_ = _e99;
    } else {
        phi_1317_ = ((_e102 << bitcast<u32>(17u)) + 65536u);
    }
    let _e108 = phi_1317_;
    let _e109 = n1_1;
    R4_.r2_[_e96] = (_e108 + bitcast<u32>(i32(round((_e109 * 2048f)))));
    phi_1371_ = 0u;
    phi_1367_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e101 != _e102) {
        let _e119 = ((f32((_e99 & 131071u)) * 0.00048828125f) + -32f);
        let _e122 = VC.r2_[_e101];
        phi_1321_ = _e119;
        if ((_e122.x & 768u) != 0u) {
            let _e126 = abs(_e119);
            phi_899_ = aj;
            if aj {
                phi_899_ = ((_e122.x & 512u) != 0u);
            }
            let _e130 = phi_899_;
            phi_1322_ = _e126;
            if _e130 {
                phi_1322_ = (1f - abs(((fract((_e126 * 0.5f)) * 2f) + -1f)));
            }
            let _e138 = phi_1322_;
            phi_1321_ = _e138;
        }
        let _e140 = phi_1321_;
        let _e141 = clamp(_e140, 0f, 1f);
        phi_1325_ = _e141;
        if Wi {
            let _e143 = (_e122.x >> bitcast<u32>(16u));
            phi_1326_ = _e141;
            if (_e143 != 0u) {
                let _e147 = m0_.r2_[_e96];
                if (_e143 == (_e147 >> bitcast<u32>(16i))) {
                    phi_1323_ = min(_e141, unpack2x16float(_e147).x);
                } else {
                    phi_1323_ = 0f;
                }
                let _e155 = phi_1323_;
                phi_1326_ = _e155;
            }
            let _e157 = phi_1326_;
            phi_1325_ = _e157;
        }
        let _e159 = phi_1325_;
        phi_936_ = Xi;
        if Xi {
            phi_936_ = ((_e122.x & 1024u) != 0u);
        }
        let _e163 = phi_936_;
        phi_1328_ = _e159;
        if _e163 {
            let _e164 = (_e101 * 8u);
            let _e168 = JB.r2_[(_e164 + 2u)];
            let _e179 = JB.r2_[(_e164 + 3u)];
            let _e184 = _e179.zw;
            let _e186 = ((abs(((mat2x2<f32>(vec2<f32>(_e168.x, _e168.y), vec2<f32>(_e168.z, _e168.w)) * _e62) + _e179.xy)) * _e184) - _e184);
            phi_1328_ = min(_e159, clamp((min(_e186.x, _e186.y) + 0.5f), 0f, 1f));
        }
        let _e194 = phi_1328_;
        let _e195 = (_e122.x & 15u);
        if (_e195 <= 1u) {
            let _e205 = (Wi && (_e195 == 0u));
            phi_1356_ = 0u;
            if _e205 {
                phi_1356_ = (_e122.y | pack2x16float(vec2<f32>(_e194, 0f)));
            }
            let _e210 = phi_1356_;
            phi_1355_ = _e210;
            phi_1352_ = select(unpack4x8unorm(_e122.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e205));
        } else {
            let _e213 = (_e101 * 8u);
            let _e216 = JB.r2_[_e213];
            let _e227 = JB.r2_[(_e213 + 1u)];
            let _e230 = ((mat2x2<f32>(vec2<f32>(_e216.x, _e216.y), vec2<f32>(_e216.z, _e216.w)) * _e62) + _e227.xy);
            let _e236 = j.L8_;
            let _e238 = j.M8_;
            if (f32(_e195) == 2f) {
                phi_1327_ = _e230.x;
            } else {
                phi_1327_ = length(_e230);
            }
            let _e248 = phi_1327_;
            let _e254 = textureSampleLevel(XC, N8_, vec2<f32>(((clamp(_e248, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e227.z < 0f))) + ((max(0f, _e227.z) * 0.001953125f) + 0.0009765625f)), ((_e227.w * _e236) + _e238)), 0f);
            phi_1354_ = _e254;
            if !((Yi && (((_e122.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e258 = (_e254.xyz * _e254.w);
                phi_1354_ = vec4<f32>(_e258.x, _e258.y, _e258.z, (_e254.w * abs(bitcast<f32>(_e122.y))));
            }
            let _e268 = phi_1354_;
            phi_1355_ = 0u;
            phi_1352_ = _e268;
        }
        let _e270 = phi_1355_;
        let _e272 = phi_1352_;
        phi_1371_ = _e270;
        phi_1367_ = (_e272 * _e194);
    }
    let _e275 = phi_1371_;
    let _e277 = phi_1367_;
    let _e278 = _e277.xyz;
    let _e281 = j.F3_;
    let _e283 = j.G3_;
    if (dj && (_e277.w != 0f)) {
        phi_1368_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e61.x) + (0.00583715f * _e61.y))))) * _e281) + _e283)) + _e278);
    } else {
        phi_1368_ = _e278;
    }
    let _e299 = phi_1368_;
    let _e305 = vec4<f32>(_e299.x, _e277.y, _e277.z, _e277.w);
    let _e311 = vec4<f32>(_e305.x, _e299.y, _e305.z, _e305.w);
    L1_ = vec4<f32>(_e311.x, _e311.y, _e299.z, _e311.w);
    if (_e275 != 0u) {
        m0_.r2_[_e96] = _e275;
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
