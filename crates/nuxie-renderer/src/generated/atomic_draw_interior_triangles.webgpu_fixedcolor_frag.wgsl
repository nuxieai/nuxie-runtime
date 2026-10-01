struct Ef {
    k2_: array<vec2<u32>>,
}

struct m0ge {
    k2_: array<u32>,
}

struct Ff {
    k2_: array<vec4<f32>>,
}

struct UB {
    Qc: f32,
    Td: f32,
    bg: f32,
    cg: f32,
    A6_: u32,
    X9_: u32,
    Nf: u32,
    Of: u32,
    j8_: vec4<i32>,
    Lh: vec2<f32>,
    Ud: vec2<f32>,
    j2_: u32,
    Ph: f32,
    T4_: u32,
    a3_: f32,
    Vd: f32,
    Hf: u32,
    L3_: f32,
    M3_: f32,
    Wd: f32,
    Ih: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct K4ge {
    k2_: array<u32>,
}

@id(7) override ri: bool = true;
@id(4) override oi: bool = true;
@id(0) override ki: bool = true;
@id(1) override li: bool = true;
@id(2) override mi: bool = true;

@group(0) @binding(3)
var<storage> WC: Ef;
@group(2) @binding(1)
var<storage, read_write> m0_: m0ge;
@group(0) @binding(4)
var<storage> JB: Ff;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var ha: sampler;
@group(2) @binding(3)
var<storage, read_write> K4_: K4ge;
var<private> F0_1: u32;
var<private> m1_1: f32;
var<private> K1_: vec4<f32>;
@group(3) @binding(9)
var wa: sampler;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(1) @binding(13)
var r5_: sampler;

fn main_1() {
    var phi_1210_: u32;
    var phi_833_: bool;
    var phi_1215_: f32;
    var phi_1214_: f32;
    var phi_1216_: f32;
    var phi_1219_: f32;
    var phi_1218_: f32;
    var phi_870_: bool;
    var phi_1221_: f32;
    var phi_1249_: u32;
    var phi_1220_: f32;
    var phi_1247_: vec4<f32>;
    var phi_1248_: u32;
    var phi_1245_: vec4<f32>;
    var phi_1264_: u32;
    var phi_1260_: vec4<f32>;
    var phi_1261_: vec3<f32>;

    let _e59 = gl_FragCoord_1;
    let _e60 = _e59.xy;
    let _e63 = bitcast<vec2<u32>>(vec2<i32>(floor(_e60)));
    let _e65 = j.A6_;
    let _e94 = bitcast<i32>((((((_e63.y >> bitcast<u32>(5u)) * (((_e65 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e63.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e63.x & 28u) << bitcast<u32>(5u)) + ((_e63.y & 28u) << bitcast<u32>(2i)))) + (((_e63.y & 3u) << bitcast<u32>(2i)) + (_e63.x & 3u))));
    let _e97 = K4_.k2_[_e94];
    let _e99 = (_e97 >> bitcast<u32>(17u));
    let _e100 = F0_1;
    if (_e99 == _e100) {
        phi_1210_ = _e97;
    } else {
        phi_1210_ = ((_e100 << bitcast<u32>(17u)) + 65536u);
    }
    let _e106 = phi_1210_;
    let _e107 = m1_1;
    K4_.k2_[_e94] = (_e106 + bitcast<u32>(i32(round((_e107 * 2048f)))));
    phi_1264_ = 0u;
    phi_1260_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e99 != _e100) {
        let _e117 = ((f32((_e97 & 131071u)) * 0.00048828125f) + -32f);
        let _e120 = WC.k2_[_e99];
        phi_1214_ = _e117;
        if ((_e120.x & 768u) != 0u) {
            let _e124 = abs(_e117);
            phi_833_ = oi;
            if oi {
                phi_833_ = ((_e120.x & 512u) != 0u);
            }
            let _e128 = phi_833_;
            phi_1215_ = _e124;
            if _e128 {
                phi_1215_ = (1f - abs(((fract((_e124 * 0.5f)) * 2f) + -1f)));
            }
            let _e136 = phi_1215_;
            phi_1214_ = _e136;
        }
        let _e138 = phi_1214_;
        let _e139 = clamp(_e138, 0f, 1f);
        phi_1218_ = _e139;
        if ki {
            let _e141 = (_e120.x >> bitcast<u32>(16u));
            phi_1219_ = _e139;
            if (_e141 != 0u) {
                let _e145 = m0_.k2_[_e94];
                if (_e141 == (_e145 >> bitcast<u32>(16i))) {
                    phi_1216_ = min(_e139, unpack2x16float(_e145).x);
                } else {
                    phi_1216_ = 0f;
                }
                let _e153 = phi_1216_;
                phi_1219_ = _e153;
            }
            let _e155 = phi_1219_;
            phi_1218_ = _e155;
        }
        let _e157 = phi_1218_;
        phi_870_ = li;
        if li {
            phi_870_ = ((_e120.x & 1024u) != 0u);
        }
        let _e161 = phi_870_;
        phi_1221_ = _e157;
        if _e161 {
            let _e162 = (_e99 * 8u);
            let _e166 = JB.k2_[(_e162 + 2u)];
            let _e177 = JB.k2_[(_e162 + 3u)];
            let _e182 = _e177.zw;
            let _e184 = ((abs(((mat2x2<f32>(vec2<f32>(_e166.x, _e166.y), vec2<f32>(_e166.z, _e166.w)) * _e60) + _e177.xy)) * _e182) - _e182);
            phi_1221_ = min(_e157, clamp((min(_e184.x, _e184.y) + 0.5f), 0f, 1f));
        }
        let _e192 = phi_1221_;
        let _e193 = (_e120.x & 15u);
        if (_e193 <= 1u) {
            let _e203 = (ki && (_e193 == 0u));
            phi_1249_ = 0u;
            if _e203 {
                phi_1249_ = (_e120.y | pack2x16float(vec2<f32>(_e192, 0f)));
            }
            let _e208 = phi_1249_;
            phi_1248_ = _e208;
            phi_1245_ = select(unpack4x8unorm(_e120.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e203));
        } else {
            let _e211 = (_e99 * 8u);
            let _e214 = JB.k2_[_e211];
            let _e225 = JB.k2_[(_e211 + 1u)];
            let _e228 = ((mat2x2<f32>(vec2<f32>(_e214.x, _e214.y), vec2<f32>(_e214.z, _e214.w)) * _e60) + _e225.xy);
            if (_e193 == 2u) {
                phi_1220_ = _e228.x;
            } else {
                phi_1220_ = length(_e228);
            }
            let _e233 = phi_1220_;
            let _e240 = bitcast<f32>(_e120.y);
            let _e243 = j.wc;
            let _e246 = j.xc;
            let _e249 = textureSampleLevel(ED, ha, vec2<f32>(((clamp(_e233, 0f, 1f) * _e225.z) + _e225.w), ((floor(_e240) * _e243) + _e246)), 0f);
            phi_1247_ = _e249;
            if !((mi && (((_e120.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e253 = (_e249.xyz * _e249.w);
                phi_1247_ = vec4<f32>(_e253.x, _e253.y, _e253.z, (_e249.w * (fract(_e240) * 1.0039216f)));
            }
            let _e262 = phi_1247_;
            phi_1248_ = 0u;
            phi_1245_ = _e262;
        }
        let _e264 = phi_1248_;
        let _e266 = phi_1245_;
        phi_1264_ = _e264;
        phi_1260_ = (_e266 * _e192);
    }
    let _e269 = phi_1264_;
    let _e271 = phi_1260_;
    let _e272 = _e271.xyz;
    let _e275 = j.L3_;
    let _e277 = j.M3_;
    if (ri && (_e271.w != 0f)) {
        phi_1261_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e59.x) + (0.00583715f * _e59.y))))) * _e275) + _e277)) + _e272);
    } else {
        phi_1261_ = _e272;
    }
    let _e293 = phi_1261_;
    let _e299 = vec4<f32>(_e293.x, _e271.y, _e271.z, _e271.w);
    let _e305 = vec4<f32>(_e299.x, _e293.y, _e299.z, _e299.w);
    K1_ = vec4<f32>(_e305.x, _e305.y, _e293.z, _e305.w);
    if (_e269 != 0u) {
        m0_.k2_[_e94] = _e269;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) F0_: u32, @location(0) @interpolate(flat, either) m1_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    F0_1 = F0_;
    m1_1 = m1_;
    main_1();
    let _e7 = K1_;
    return _e7;
}
