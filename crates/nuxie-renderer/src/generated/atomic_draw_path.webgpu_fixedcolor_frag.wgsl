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

struct T4Ue_1 {
    v2_: array<atomic<u32>>,
}

@id(8) override nj: bool = true;
@id(9) override oj: bool = true;
@id(5) override kj: bool = true;
@id(0) override fj: bool = true;
@id(1) override gj: bool = true;
@id(2) override hj: bool = true;
@id(3) override ij: bool = true;

@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(3) @binding(9)
var db: sampler;
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
var<private> T_1: vec4<f32>;
var<private> G0_1: u32;
@group(2) @binding(3)
var<storage, read_write> T4_: T4Ue_1;
var<private> Q1_: vec4<f32>;
@group(1) @binding(11)
var UB: texture_2d<f32>;
@group(1) @binding(13)
var W4_: sampler;

fn main_1() {
    var phi_942_: bool;
    var phi_955_: bool;
    var phi_1950_: f32;
    var phi_1958_: f32;
    var phi_1966_: f32;
    var phi_1965_: f32;
    var phi_1452_: bool;
    var phi_1969_: f32;
    var phi_1968_: f32;
    var phi_1970_: f32;
    var phi_1973_: f32;
    var phi_1972_: f32;
    var phi_1489_: bool;
    var phi_1977_: f32;
    var phi_1974_: f32;
    var phi_1975_: f32;
    var phi_1976_: vec2<f32>;
    var phi_2015_: vec4<f32>;
    var phi_2017_: u32;
    var phi_2016_: u32;
    var phi_2013_: vec4<f32>;
    var phi_2035_: u32;
    var phi_2030_: vec4<f32>;
    var phi_2032_: vec3<f32>;

    let _e82 = gl_FragCoord_1;
    let _e83 = _e82.xy;
    let _e86 = bitcast<vec2<u32>>(vec2<i32>(floor(_e83)));
    let _e88 = j.T6_;
    let _e117 = bitcast<i32>((((((_e86.y >> bitcast<u32>(5u)) * (((_e88 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e86.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e86.x & 28u) << bitcast<u32>(5u)) + ((_e86.y & 28u) << bitcast<u32>(2i)))) + (((_e86.y & 3u) << bitcast<u32>(2i)) + (_e86.x & 3u))));
    phi_942_ = ij;
    if ij {
        let _e118 = T_1;
        phi_942_ = (_e118.x < -1.5f);
    }
    let _e122 = phi_942_;
    if _e122 {
        let _e195 = T_1;
        let _e199 = textureSampleLevel(AD, db, vec2<f32>((3f + _e195.x), 0f), 0f);
        let _e205 = textureSampleLevel(AD, db, vec2<f32>((1f - _e195.y), 0f), 0f);
        phi_1965_ = ((1f - _e199.x) - _e205.x);
    } else {
        phi_955_ = ij;
        if ij {
            let _e123 = T_1;
            phi_955_ = (_e123.y < -1.5f);
        }
        let _e127 = phi_955_;
        if _e127 {
            let _e135 = T_1;
            let _e138 = max(_e135.w, 0f);
            if (_e135.z >= 0f) {
                let _e141 = textureSampleLevel(AD, db, vec2<f32>(_e138, 0f), 0f);
                phi_1950_ = _e141.x;
            } else {
                phi_1950_ = 0f;
            }
            let _e144 = phi_1950_;
            phi_1958_ = _e144;
            if (abs(_e135.z) < 1000f) {
                let _e151 = (-2f - _e135.y);
                let _e153 = ((_e151 - _e138) * 0.5984134f);
                let _e156 = (vec4(_e138) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e153));
                let _e162 = ((_e156 * -(_e135.z)) + vec4(((_e151 * _e135.z) + (abs(_e135.x) - 0.25f))));
                let _e165 = textureSampleLevel(AD, db, vec2<f32>(_e162.x, 0f), 0f);
                let _e168 = textureSampleLevel(AD, db, vec2<f32>(_e162.y, 0f), 0f);
                let _e171 = textureSampleLevel(AD, db, vec2<f32>(_e162.z, 0f), 0f);
                let _e174 = textureSampleLevel(AD, db, vec2<f32>(_e162.w, 0f), 0f);
                let _e180 = (_e156 * 5.0959306f);
                phi_1958_ = (_e144 + (dot(vec4<f32>(_e165.x, _e168.x, _e171.x, _e174.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e180) * (_e180 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e153));
            }
            let _e189 = phi_1958_;
            phi_1966_ = (_e189 * sign(_e135.x));
        } else {
            let _e129 = T_1[0u];
            let _e131 = T_1[1u];
            phi_1966_ = min(min(_e129, abs(_e131)), 1f);
        }
        let _e194 = phi_1966_;
        phi_1965_ = _e194;
    }
    let _e209 = phi_1965_;
    let _e213 = u32(round(((_e209 * 2048f) + 65536f)));
    let _e214 = G0_1;
    let _e217 = ((_e214 << bitcast<u32>(17u)) | _e213);
    let _e220 = atomicMax((&T4_.v2_[_e117]), _e217);
    let _e222 = (_e220 >> bitcast<u32>(17u));
    if (_e222 == _e214) {
        let _e402 = T_1;
        if (_e402.y < 0f) {
            let _e409 = atomicAdd((&T4_.v2_[_e117]), ((_e213 + (_e220 - max(_e217, _e220))) - 65536u));
        }
        phi_2035_ = 0u;
        phi_2030_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e227 = ((f32((_e220 & 131071u)) * 0.00048828125f) + -32f);
        let _e230 = XC.v2_[_e222];
        phi_1968_ = _e227;
        if ((_e230.x & 768u) != 0u) {
            let _e234 = abs(_e227);
            phi_1452_ = kj;
            if kj {
                phi_1452_ = ((_e230.x & 512u) != 0u);
            }
            let _e238 = phi_1452_;
            phi_1969_ = _e234;
            if _e238 {
                phi_1969_ = (1f - abs(((fract((_e234 * 0.5f)) * 2f) + -1f)));
            }
            let _e246 = phi_1969_;
            phi_1968_ = _e246;
        }
        let _e248 = phi_1968_;
        let _e249 = clamp(_e248, 0f, 1f);
        phi_1972_ = _e249;
        if fj {
            let _e251 = (_e230.x >> bitcast<u32>(16u));
            phi_1973_ = _e249;
            if (_e251 != 0u) {
                let _e255 = m0_.v2_[_e117];
                if (_e251 == (_e255 >> bitcast<u32>(16i))) {
                    phi_1970_ = min(_e249, unpack2x16float(_e255).x);
                } else {
                    phi_1970_ = 0f;
                }
                let _e263 = phi_1970_;
                phi_1973_ = _e263;
            }
            let _e265 = phi_1973_;
            phi_1972_ = _e265;
        }
        let _e267 = phi_1972_;
        phi_1489_ = gj;
        if gj {
            phi_1489_ = ((_e230.x & 1024u) != 0u);
        }
        let _e271 = phi_1489_;
        phi_1977_ = _e267;
        if _e271 {
            let _e272 = (_e222 * 8u);
            let _e276 = JB.v2_[(_e272 + 2u)];
            let _e287 = JB.v2_[(_e272 + 3u)];
            let _e292 = _e287.zw;
            let _e294 = ((abs(((mat2x2<f32>(vec2<f32>(_e276.x, _e276.y), vec2<f32>(_e276.z, _e276.w)) * _e83) + _e287.xy)) * _e292) - _e292);
            phi_1977_ = min(_e267, clamp((min(_e294.x, _e294.y) + 0.5f), 0f, 1f));
        }
        let _e302 = phi_1977_;
        let _e303 = (_e230.x & 15u);
        if (_e303 <= 1u) {
            let _e389 = (fj && (_e303 == 0u));
            phi_2017_ = 0u;
            if _e389 {
                phi_2017_ = (_e230.y | pack2x16float(vec2<f32>(_e302, 0f)));
            }
            let _e394 = phi_2017_;
            phi_2016_ = _e394;
            phi_2013_ = select(unpack4x8unorm(_e230.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e389));
        } else {
            let _e310 = (_e222 * 8u);
            let _e313 = JB.v2_[_e310];
            let _e324 = JB.v2_[(_e310 + 1u)];
            let _e327 = ((mat2x2<f32>(vec2<f32>(_e313.x, _e313.y), vec2<f32>(_e313.z, _e313.w)) * _e83) + _e324.xy);
            let _e331 = f32(_e303);
            let _e333 = j.P8_;
            let _e335 = j.Q8_;
            let _e340 = ((_e324.w * _e333) + _e335);
            switch bitcast<i32>(0u) {
                default: {
                    if (oj && (_e331 == 4f)) {
                        let _e358 = normalize(vec2<f32>(_e327.x, -(_e327.y)));
                        let _e361 = acos(clamp(_e358.x, -1f, 1f));
                        if (_e358.y >= 0f) {
                            phi_1975_ = _e361;
                        } else {
                            phi_1975_ = -(_e361);
                        }
                        let _e366 = phi_1975_;
                        phi_1976_ = vec2<f32>((_e366 * 0.15915494f), _e340);
                        break;
                    } else {
                        if (_e331 == 2f) {
                            phi_1974_ = _e327.x;
                        } else {
                            phi_1974_ = length(_e327);
                        }
                        let _e348 = phi_1974_;
                        phi_1976_ = vec2<f32>(((clamp(_e348, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e324.z < 0f))) + ((max(0f, _e324.z) * 0.001953125f) + 0.0009765625f)), _e340);
                        break;
                    }
                }
            }
            let _e370 = phi_1976_;
            let _e371 = textureSampleLevel(ZC, R8_, _e370, 0f);
            phi_2015_ = _e371;
            if !((hj && (((_e230.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e375 = (_e371.xyz * _e371.w);
                phi_2015_ = vec4<f32>(_e375.x, _e375.y, _e375.z, (_e371.w * abs(bitcast<f32>(_e230.y))));
            }
            let _e385 = phi_2015_;
            phi_2016_ = 0u;
            phi_2013_ = _e385;
        }
        let _e398 = phi_2016_;
        let _e400 = phi_2013_;
        phi_2035_ = _e398;
        phi_2030_ = (_e400 * _e302);
    }
    let _e411 = phi_2035_;
    let _e413 = phi_2030_;
    let _e414 = _e413.xyz;
    let _e417 = j.G3_;
    let _e419 = j.H3_;
    if (nj && (_e413.w != 0f)) {
        phi_2032_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e82.x) + (0.00583715f * _e82.y))))) * _e417) + _e419)) + _e414);
    } else {
        phi_2032_ = _e414;
    }
    let _e435 = phi_2032_;
    let _e441 = vec4<f32>(_e435.x, _e413.y, _e413.z, _e413.w);
    let _e447 = vec4<f32>(_e441.x, _e435.y, _e441.z, _e441.w);
    Q1_ = vec4<f32>(_e447.x, _e447.y, _e435.z, _e447.w);
    if (_e411 != 0u) {
        m0_.v2_[_e117] = _e411;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) T: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    T_1 = T;
    G0_1 = G0_;
    main_1();
    let _e7 = Q1_;
    return _e7;
}
