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
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;
var<private> m2_1: vec2<f32>;
var<private> W0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> P4_: P4Pe;
var<private> R3_1: u32;
var<private> U1_1: vec4<f32>;
var<private> N1_: vec4<f32>;
@group(3) @binding(9)
var Va: sampler;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
var<private> K1_1: u32;

fn main_1() {
    var phi_1335_: f32;
    var phi_892_: bool;
    var phi_1279_: f32;
    var phi_1278_: f32;
    var phi_1280_: f32;
    var phi_1283_: f32;
    var phi_1282_: f32;
    var phi_929_: bool;
    var phi_1285_: f32;
    var phi_1314_: u32;
    var phi_1284_: f32;
    var phi_1312_: vec4<f32>;
    var phi_1313_: u32;
    var phi_1310_: vec4<f32>;
    var phi_674_: bool;
    var phi_1326_: u32;
    var phi_1342_: f32;
    var phi_1360_: f32;
    var phi_1365_: vec3<f32>;

    let _e61 = gl_FragCoord_1;
    let _e62 = _e61.xy;
    let _e65 = bitcast<vec2<u32>>(vec2<i32>(floor(_e62)));
    let _e67 = j.L6_;
    let _e96 = bitcast<i32>((((((_e65.y >> bitcast<u32>(5u)) * (((_e67 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e65.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e65.x & 28u) << bitcast<u32>(5u)) + ((_e65.y & 28u) << bitcast<u32>(2i)))) + (((_e65.y & 3u) << bitcast<u32>(2i)) + (_e65.x & 3u))));
    let _e97 = m2_1;
    let _e98 = textureSample(TB, S4_, _e97);
    phi_1335_ = 1f;
    if Zi {
        let _e99 = W0_1;
        let _e102 = min(_e99.xy, _e99.zw);
        phi_1335_ = clamp(min(_e102.x, _e102.y), 0f, 1f);
    }
    let _e108 = phi_1335_;
    let _e111 = P4_.v2_[_e96];
    let _e113 = (_e111 >> bitcast<u32>(17u));
    let _e117 = ((f32((_e111 & 131071u)) * 0.00048828125f) + -32f);
    let _e120 = WC.v2_[_e113];
    phi_1278_ = _e117;
    if ((_e120.x & 768u) != 0u) {
        let _e124 = abs(_e117);
        phi_892_ = cj;
        if cj {
            phi_892_ = ((_e120.x & 512u) != 0u);
        }
        let _e128 = phi_892_;
        phi_1279_ = _e124;
        if _e128 {
            phi_1279_ = (1f - abs(((fract((_e124 * 0.5f)) * 2f) + -1f)));
        }
        let _e136 = phi_1279_;
        phi_1278_ = _e136;
    }
    let _e138 = phi_1278_;
    let _e139 = clamp(_e138, 0f, 1f);
    phi_1282_ = _e139;
    if Yi {
        let _e141 = (_e120.x >> bitcast<u32>(16u));
        phi_1283_ = _e139;
        if (_e141 != 0u) {
            let _e145 = m0_.v2_[_e96];
            if (_e141 == (_e145 >> bitcast<u32>(16i))) {
                phi_1280_ = min(_e139, unpack2x16float(_e145).x);
            } else {
                phi_1280_ = 0f;
            }
            let _e153 = phi_1280_;
            phi_1283_ = _e153;
        }
        let _e155 = phi_1283_;
        phi_1282_ = _e155;
    }
    let _e157 = phi_1282_;
    phi_929_ = Zi;
    if Zi {
        phi_929_ = ((_e120.x & 1024u) != 0u);
    }
    let _e161 = phi_929_;
    phi_1285_ = _e157;
    if _e161 {
        let _e162 = (_e113 * 8u);
        let _e166 = JB.v2_[(_e162 + 2u)];
        let _e177 = JB.v2_[(_e162 + 3u)];
        let _e182 = _e177.zw;
        let _e184 = ((abs(((mat2x2<f32>(vec2<f32>(_e166.x, _e166.y), vec2<f32>(_e166.z, _e166.w)) * _e62) + _e177.xy)) * _e182) - _e182);
        phi_1285_ = min(_e157, clamp((min(_e184.x, _e184.y) + 0.5f), 0f, 1f));
    }
    let _e192 = phi_1285_;
    let _e193 = (_e120.x & 15u);
    if (_e193 <= 1u) {
        let _e203 = (Yi && (_e193 == 0u));
        phi_1314_ = 0u;
        if _e203 {
            phi_1314_ = (_e120.y | pack2x16float(vec2<f32>(_e192, 0f)));
        }
        let _e208 = phi_1314_;
        phi_1313_ = _e208;
        phi_1310_ = select(unpack4x8unorm(_e120.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e203));
    } else {
        let _e211 = (_e113 * 8u);
        let _e214 = JB.v2_[_e211];
        let _e225 = JB.v2_[(_e211 + 1u)];
        let _e228 = ((mat2x2<f32>(vec2<f32>(_e214.x, _e214.y), vec2<f32>(_e214.z, _e214.w)) * _e62) + _e225.xy);
        if (_e193 == 2u) {
            phi_1284_ = _e228.x;
        } else {
            phi_1284_ = length(_e228);
        }
        let _e233 = phi_1284_;
        let _e240 = bitcast<f32>(_e120.y);
        let _e243 = j.cd;
        let _e246 = j.g7_;
        let _e249 = textureSampleLevel(YC, I8_, vec2<f32>(((clamp(_e233, 0f, 1f) * _e225.z) + _e225.w), ((floor(_e240) * _e243) + _e246)), 0f);
        phi_1312_ = _e249;
        if !((aj && (((_e120.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e253 = (_e249.xyz * _e249.w);
            phi_1312_ = vec4<f32>(_e253.x, _e253.y, _e253.z, (_e249.w * (fract(_e240) * 1.0039216f)));
        }
        let _e262 = phi_1312_;
        phi_1313_ = 0u;
        phi_1310_ = _e262;
    }
    let _e264 = phi_1313_;
    let _e266 = phi_1310_;
    phi_674_ = Yi;
    if Yi {
        let _e268 = R3_1;
        phi_674_ = (_e268 != 0u);
    }
    let _e271 = phi_674_;
    phi_1360_ = _e108;
    if _e271 {
        if (_e264 != 0u) {
            phi_1326_ = _e264;
        } else {
            let _e275 = m0_.v2_[_e96];
            phi_1326_ = _e275;
        }
        let _e277 = phi_1326_;
        let _e278 = R3_1;
        if (_e278 == (_e277 >> bitcast<u32>(16i))) {
            phi_1342_ = min(_e108, unpack2x16float(_e277).x);
        } else {
            phi_1342_ = 0f;
        }
        let _e286 = phi_1342_;
        phi_1360_ = _e286;
    }
    let _e288 = phi_1360_;
    let _e289 = U1_1;
    let _e291 = ((_e98 * _e289) * _e288);
    let _e295 = (((_e266 * _e192) * (1f - _e291.w)) + _e291);
    let _e296 = _e295.xyz;
    let _e299 = j.E3_;
    let _e301 = j.F3_;
    if (fj && (_e295.w != 0f)) {
        phi_1365_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e61.x) + (0.00583715f * _e61.y))))) * _e299) + _e301)) + _e296);
    } else {
        phi_1365_ = _e296;
    }
    let _e317 = phi_1365_;
    let _e323 = vec4<f32>(_e317.x, _e295.y, _e295.z, _e295.w);
    let _e329 = vec4<f32>(_e323.x, _e317.y, _e323.z, _e323.w);
    N1_ = vec4<f32>(_e329.x, _e329.y, _e317.z, _e329.w);
    if (_e264 != 0u) {
        m0_.v2_[_e96] = _e264;
    }
    P4_.v2_[_e96] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) m2_: vec2<f32>, @location(1) W0_: vec4<f32>, @location(4) @interpolate(flat, either) R3_: u32, @location(3) @interpolate(flat, either) U1_: vec4<f32>, @location(5) @interpolate(flat, either) K1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    m2_1 = m2_;
    W0_1 = W0_;
    R3_1 = R3_;
    U1_1 = U1_;
    K1_1 = K1_;
    main_1();
    let _e13 = N1_;
    return _e13;
}
