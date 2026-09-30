struct kf {
    g2_: array<vec2<u32>>,
}

struct i0Sd {
    g2_: array<u32>,
}

struct lf {
    g2_: array<vec4<f32>>,
}

struct SB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    eh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    ih: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    bh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct A4Sd {
    g2_: array<u32>,
}

@id(7) override Lh: bool = true;
@id(4) override Ih: bool = true;
@id(0) override Eh: bool = true;
@id(1) override Fh: bool = true;
@id(2) override Gh: bool = true;

@group(0) @binding(3)
var<storage> CD: kf;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Sd;
@group(0) @binding(4)
var<storage> PB: lf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: SB;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var M9_: sampler;
@group(2) @binding(3)
var<storage, read_write> A4_: A4Sd;
var<private> D0_1: u32;
var<private> j1_1: f32;
var<private> F1_: vec4<f32>;
@group(3) @binding(9)
var ca: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var W5_: sampler;

fn main_1() {
    var phi_1211_: u32;
    var phi_834_: bool;
    var phi_1216_: f32;
    var phi_1215_: f32;
    var phi_1217_: f32;
    var phi_1220_: f32;
    var phi_1219_: f32;
    var phi_871_: bool;
    var phi_1222_: f32;
    var phi_1250_: u32;
    var phi_1221_: f32;
    var phi_1248_: vec4<f32>;
    var phi_1249_: u32;
    var phi_1246_: vec4<f32>;
    var phi_1265_: u32;
    var phi_1261_: vec4<f32>;
    var phi_1262_: vec3<f32>;

    let _e59 = gl_FragCoord_1;
    let _e60 = _e59.xy;
    let _e63 = bitcast<vec2<u32>>(vec2<i32>(floor(_e60)));
    let _e65 = j.n6_;
    let _e94 = bitcast<i32>((((((_e63.y >> bitcast<u32>(5u)) * (((_e65 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e63.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e63.x & 28u) << bitcast<u32>(5u)) + ((_e63.y & 28u) << bitcast<u32>(2i)))) + (((_e63.y & 3u) << bitcast<u32>(2i)) + (_e63.x & 3u))));
    let _e97 = A4_.g2_[_e94];
    let _e99 = (_e97 >> bitcast<u32>(17u));
    let _e100 = D0_1;
    if (_e99 == _e100) {
        phi_1211_ = _e97;
    } else {
        phi_1211_ = ((_e100 << bitcast<u32>(17u)) + 65536u);
    }
    let _e106 = phi_1211_;
    let _e107 = j1_1;
    A4_.g2_[_e94] = (_e106 + bitcast<u32>(i32(round((_e107 * 2048f)))));
    phi_1265_ = 0u;
    phi_1261_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e99 != _e100) {
        let _e117 = ((f32((_e97 & 131071u)) * 0.00048828125f) + -32f);
        let _e120 = CD.g2_[_e99];
        phi_1215_ = _e117;
        if ((_e120.x & 768u) != 0u) {
            let _e124 = abs(_e117);
            phi_834_ = Ih;
            if Ih {
                phi_834_ = ((_e120.x & 512u) != 0u);
            }
            let _e128 = phi_834_;
            phi_1216_ = _e124;
            if _e128 {
                phi_1216_ = (1f - abs(((fract((_e124 * 0.5f)) * 2f) + -1f)));
            }
            let _e136 = phi_1216_;
            phi_1215_ = _e136;
        }
        let _e138 = phi_1215_;
        let _e139 = clamp(_e138, 0f, 1f);
        phi_1219_ = _e139;
        if Eh {
            let _e141 = (_e120.x >> bitcast<u32>(16u));
            phi_1220_ = _e139;
            if (_e141 != 0u) {
                let _e145 = i0_.g2_[_e94];
                if (_e141 == (_e145 >> bitcast<u32>(16i))) {
                    phi_1217_ = min(_e139, unpack2x16float(_e145).x);
                } else {
                    phi_1217_ = 0f;
                }
                let _e153 = phi_1217_;
                phi_1220_ = _e153;
            }
            let _e155 = phi_1220_;
            phi_1219_ = _e155;
        }
        let _e157 = phi_1219_;
        phi_871_ = Fh;
        if Fh {
            phi_871_ = ((_e120.x & 1024u) != 0u);
        }
        let _e161 = phi_871_;
        phi_1222_ = _e157;
        if _e161 {
            let _e162 = (_e99 * 8u);
            let _e166 = PB.g2_[(_e162 + 2u)];
            let _e177 = PB.g2_[(_e162 + 3u)];
            let _e182 = _e177.zw;
            let _e184 = ((abs(((mat2x2<f32>(vec2<f32>(_e166.x, _e166.y), vec2<f32>(_e166.z, _e166.w)) * _e60) + _e177.xy)) * _e182) - _e182);
            phi_1222_ = min(_e157, clamp((min(_e184.x, _e184.y) + 0.5f), 0f, 1f));
        }
        let _e192 = phi_1222_;
        let _e193 = (_e120.x & 15u);
        if (_e193 <= 1u) {
            let _e203 = (Eh && (_e193 == 0u));
            phi_1250_ = 0u;
            if _e203 {
                phi_1250_ = (_e120.y | pack2x16float(vec2<f32>(_e192, 0f)));
            }
            let _e208 = phi_1250_;
            phi_1249_ = _e208;
            phi_1246_ = select(unpack4x8unorm(_e120.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e203));
        } else {
            let _e211 = (_e99 * 8u);
            let _e214 = PB.g2_[_e211];
            let _e225 = PB.g2_[(_e211 + 1u)];
            let _e228 = ((mat2x2<f32>(vec2<f32>(_e214.x, _e214.y), vec2<f32>(_e214.z, _e214.w)) * _e60) + _e225.xy);
            if (_e193 == 2u) {
                phi_1221_ = _e228.x;
            } else {
                phi_1221_ = length(_e228);
            }
            let _e233 = phi_1221_;
            let _e240 = bitcast<f32>(_e120.y);
            let _e243 = j.Zb;
            let _e246 = j.ac;
            let _e249 = textureSampleLevel(DD, M9_, vec2<f32>(((clamp(_e233, 0f, 1f) * _e225.z) + _e225.w), ((floor(_e240) * _e243) + _e246)), 0f);
            phi_1248_ = _e249;
            if !((Gh && (((_e120.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e253 = (_e249.xyz * _e249.w);
                phi_1248_ = vec4<f32>(_e253.x, _e253.y, _e253.z, (_e249.w * (fract(_e240) * 1.0039216f)));
            }
            let _e262 = phi_1248_;
            phi_1249_ = 0u;
            phi_1246_ = _e262;
        }
        let _e264 = phi_1249_;
        let _e266 = phi_1246_;
        phi_1265_ = _e264;
        phi_1261_ = (_e266 * _e192);
    }
    let _e269 = phi_1265_;
    let _e271 = phi_1261_;
    let _e272 = _e271.xyz;
    let _e275 = j.F3_;
    let _e277 = j.G3_;
    if (Lh && (_e271.w != 0f)) {
        phi_1262_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e59.x) + (0.00583715f * _e59.y))))) * _e275) + _e277)) + _e272);
    } else {
        phi_1262_ = _e272;
    }
    let _e293 = phi_1262_;
    let _e299 = vec4<f32>(_e293.x, _e271.y, _e271.z, _e271.w);
    let _e305 = vec4<f32>(_e299.x, _e293.y, _e299.z, _e299.w);
    F1_ = vec4<f32>(_e305.x, _e305.y, _e293.z, _e305.w);
    if (_e269 != 0u) {
        i0_.g2_[_e94] = _e269;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) D0_: u32, @location(0) @interpolate(flat, either) j1_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    D0_1 = D0_;
    j1_1 = j1_;
    main_1();
    let _e7 = F1_;
    return _e7;
}
