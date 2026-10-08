struct jg {
    v2_: array<vec2<u32>>,
}

struct m0Pe {
    v2_: array<u32>,
}

struct kg {
    v2_: array<vec4<f32>>,
}

struct VB {
    vd: f32,
    Ce: f32,
    Gg: f32,
    Hg: f32,
    L6_: u32,
    xa: u32,
    sg: u32,
    tg: u32,
    C8_: vec4<i32>,
    Bi: vec2<f32>,
    De: vec2<f32>,
    r2_: u32,
    Fi: f32,
    p6_: u32,
    h3_: f32,
    Ee: f32,
    mg: u32,
    E3_: f32,
    F3_: f32,
    Fe: f32,
    yi: u32,
    wa: u32,
    cd: f32,
    g7_: f32,
    Db: f32,
}

struct P4Pe {
    v2_: array<u32>,
}

@id(7) override fj: bool = true;
@id(4) override cj: bool = true;
@id(0) override Yi: bool = true;
@id(1) override Zi: bool = true;
@id(2) override aj: bool = true;

@group(0) @binding(3)
var<storage> WC: jg;
@group(2) @binding(1)
var<storage, read_write> m0_: m0Pe;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: VB;
@group(0) @binding(8)
var YC: texture_2d<f32>;
@group(3) @binding(8)
var I8_: sampler;
@group(2) @binding(3)
var<storage, read_write> P4_: P4Pe;
var<private> G0_1: u32;
var<private> o1_1: f32;
var<private> N1_: vec4<f32>;
@group(3) @binding(9)
var Va: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;

fn main_1() {
    var phi_1212_: u32;
    var phi_835_: bool;
    var phi_1217_: f32;
    var phi_1216_: f32;
    var phi_1218_: f32;
    var phi_1221_: f32;
    var phi_1220_: f32;
    var phi_872_: bool;
    var phi_1223_: f32;
    var phi_1251_: u32;
    var phi_1222_: f32;
    var phi_1249_: vec4<f32>;
    var phi_1250_: u32;
    var phi_1247_: vec4<f32>;
    var phi_1266_: u32;
    var phi_1262_: vec4<f32>;
    var phi_1263_: vec3<f32>;

    let _e59 = gl_FragCoord_1;
    let _e60 = _e59.xy;
    let _e63 = bitcast<vec2<u32>>(vec2<i32>(floor(_e60)));
    let _e65 = j.L6_;
    let _e94 = bitcast<i32>((((((_e63.y >> bitcast<u32>(5u)) * (((_e65 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e63.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e63.x & 28u) << bitcast<u32>(5u)) + ((_e63.y & 28u) << bitcast<u32>(2i)))) + (((_e63.y & 3u) << bitcast<u32>(2i)) + (_e63.x & 3u))));
    let _e97 = P4_.v2_[_e94];
    let _e99 = (_e97 >> bitcast<u32>(17u));
    let _e100 = G0_1;
    if (_e99 == _e100) {
        phi_1212_ = _e97;
    } else {
        phi_1212_ = ((_e100 << bitcast<u32>(17u)) + 65536u);
    }
    let _e106 = phi_1212_;
    let _e107 = o1_1;
    P4_.v2_[_e94] = (_e106 + bitcast<u32>(i32(round((_e107 * 2048f)))));
    phi_1266_ = 0u;
    phi_1262_ = vec4<f32>(0f, 0f, 0f, 0f);
    if (_e99 != _e100) {
        let _e117 = ((f32((_e97 & 131071u)) * 0.00048828125f) + -32f);
        let _e120 = WC.v2_[_e99];
        phi_1216_ = _e117;
        if ((_e120.x & 768u) != 0u) {
            let _e124 = abs(_e117);
            phi_835_ = cj;
            if cj {
                phi_835_ = ((_e120.x & 512u) != 0u);
            }
            let _e128 = phi_835_;
            phi_1217_ = _e124;
            if _e128 {
                phi_1217_ = (1f - abs(((fract((_e124 * 0.5f)) * 2f) + -1f)));
            }
            let _e136 = phi_1217_;
            phi_1216_ = _e136;
        }
        let _e138 = phi_1216_;
        let _e139 = clamp(_e138, 0f, 1f);
        phi_1220_ = _e139;
        if Yi {
            let _e141 = (_e120.x >> bitcast<u32>(16u));
            phi_1221_ = _e139;
            if (_e141 != 0u) {
                let _e145 = m0_.v2_[_e94];
                if (_e141 == (_e145 >> bitcast<u32>(16i))) {
                    phi_1218_ = min(_e139, unpack2x16float(_e145).x);
                } else {
                    phi_1218_ = 0f;
                }
                let _e153 = phi_1218_;
                phi_1221_ = _e153;
            }
            let _e155 = phi_1221_;
            phi_1220_ = _e155;
        }
        let _e157 = phi_1220_;
        phi_872_ = Zi;
        if Zi {
            phi_872_ = ((_e120.x & 1024u) != 0u);
        }
        let _e161 = phi_872_;
        phi_1223_ = _e157;
        if _e161 {
            let _e162 = (_e99 * 8u);
            let _e166 = JB.v2_[(_e162 + 2u)];
            let _e177 = JB.v2_[(_e162 + 3u)];
            let _e182 = _e177.zw;
            let _e184 = ((abs(((mat2x2<f32>(vec2<f32>(_e166.x, _e166.y), vec2<f32>(_e166.z, _e166.w)) * _e60) + _e177.xy)) * _e182) - _e182);
            phi_1223_ = min(_e157, clamp((min(_e184.x, _e184.y) + 0.5f), 0f, 1f));
        }
        let _e192 = phi_1223_;
        let _e193 = (_e120.x & 15u);
        if (_e193 <= 1u) {
            let _e203 = (Yi && (_e193 == 0u));
            phi_1251_ = 0u;
            if _e203 {
                phi_1251_ = (_e120.y | pack2x16float(vec2<f32>(_e192, 0f)));
            }
            let _e208 = phi_1251_;
            phi_1250_ = _e208;
            phi_1247_ = select(unpack4x8unorm(_e120.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e203));
        } else {
            let _e211 = (_e99 * 8u);
            let _e214 = JB.v2_[_e211];
            let _e225 = JB.v2_[(_e211 + 1u)];
            let _e228 = ((mat2x2<f32>(vec2<f32>(_e214.x, _e214.y), vec2<f32>(_e214.z, _e214.w)) * _e60) + _e225.xy);
            if (_e193 == 2u) {
                phi_1222_ = _e228.x;
            } else {
                phi_1222_ = length(_e228);
            }
            let _e233 = phi_1222_;
            let _e240 = bitcast<f32>(_e120.y);
            let _e243 = j.cd;
            let _e246 = j.g7_;
            let _e249 = textureSampleLevel(YC, I8_, vec2<f32>(((clamp(_e233, 0f, 1f) * _e225.z) + _e225.w), ((floor(_e240) * _e243) + _e246)), 0f);
            phi_1249_ = _e249;
            if !((aj && (((_e120.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e253 = (_e249.xyz * _e249.w);
                phi_1249_ = vec4<f32>(_e253.x, _e253.y, _e253.z, (_e249.w * (fract(_e240) * 1.0039216f)));
            }
            let _e262 = phi_1249_;
            phi_1250_ = 0u;
            phi_1247_ = _e262;
        }
        let _e264 = phi_1250_;
        let _e266 = phi_1247_;
        phi_1266_ = _e264;
        phi_1262_ = (_e266 * _e192);
    }
    let _e269 = phi_1266_;
    let _e271 = phi_1262_;
    let _e272 = _e271.xyz;
    let _e275 = j.E3_;
    let _e277 = j.F3_;
    if (fj && (_e271.w != 0f)) {
        phi_1263_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e59.x) + (0.00583715f * _e59.y))))) * _e275) + _e277)) + _e272);
    } else {
        phi_1263_ = _e272;
    }
    let _e293 = phi_1263_;
    let _e299 = vec4<f32>(_e293.x, _e271.y, _e271.z, _e271.w);
    let _e305 = vec4<f32>(_e299.x, _e293.y, _e299.z, _e299.w);
    N1_ = vec4<f32>(_e305.x, _e305.y, _e293.z, _e305.w);
    if (_e269 != 0u) {
        m0_.v2_[_e94] = _e269;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32, @location(0) @interpolate(flat, either) o1_: f32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    G0_1 = G0_;
    o1_1 = o1_;
    main_1();
    let _e7 = N1_;
    return _e7;
}
