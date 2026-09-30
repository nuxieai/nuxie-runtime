struct lf {
    g2_: array<vec2<u32>>,
}

struct i0Td {
    g2_: array<u32>,
}

struct mf {
    g2_: array<vec4<f32>>,
}

struct AC {
    tc: f32,
    Dd: f32,
    Hf: f32,
    If: f32,
    q6_: u32,
    Qb: u32,
    tf: u32,
    uf: u32,
    X7_: vec4<i32>,
    eh: vec2<f32>,
    Ed: vec2<f32>,
    f2_: u32,
    ih: f32,
    f6_: u32,
    U2_: f32,
    Fd: f32,
    of_: u32,
    F3_: f32,
    G3_: f32,
    Gd: f32,
    bh: u32,
    Pb: u32,
}

struct z4Td {
    g2_: array<u32>,
}

@id(7) override Lh: bool = true;
@id(4) override Ih: bool = true;
@id(0) override Eh: bool = true;
@id(1) override Fh: bool = true;
@id(2) override Gh: bool = true;

@group(0) @binding(3)
var<storage> CD: lf;
@group(2) @binding(1)
var<storage, read_write> i0_: i0Td;
@group(0) @binding(4)
var<storage> PB: mf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(8)
var DD: texture_2d<f32>;
@group(3) @binding(8)
var P9_: sampler;
@group(0) @binding(0)
var<uniform> j: AC;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;
var<private> c2_1: vec2<f32>;
var<private> N0_1: vec4<f32>;
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td;
var<private> B3_1: u32;
var<private> J1_1: vec4<f32>;
var<private> E1_: vec4<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
var<private> C1_1: u32;

fn main_1() {
    var phi_1283_: f32;
    var phi_865_: bool;
    var phi_1227_: f32;
    var phi_1226_: f32;
    var phi_1228_: f32;
    var phi_1231_: f32;
    var phi_1230_: f32;
    var phi_902_: bool;
    var phi_1233_: f32;
    var phi_1262_: u32;
    var phi_1232_: f32;
    var phi_1260_: vec4<f32>;
    var phi_1261_: u32;
    var phi_1258_: vec4<f32>;
    var phi_650_: bool;
    var phi_1274_: u32;
    var phi_1290_: f32;
    var phi_1308_: f32;
    var phi_1313_: vec3<f32>;

    let _e58 = gl_FragCoord_1;
    let _e59 = _e58.xy;
    let _e62 = bitcast<vec2<u32>>(vec2<i32>(floor(_e59)));
    let _e64 = j.q6_;
    let _e93 = bitcast<i32>((((((_e62.y >> bitcast<u32>(5u)) * (((_e64 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e62.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e62.x & 28u) << bitcast<u32>(5u)) + ((_e62.y & 28u) << bitcast<u32>(2i)))) + (((_e62.y & 3u) << bitcast<u32>(2i)) + (_e62.x & 3u))));
    let _e94 = c2_1;
    let _e95 = textureSample(GC, Y5_, _e94);
    phi_1283_ = 1f;
    if Fh {
        let _e96 = N0_1;
        let _e99 = min(_e96.xy, _e96.zw);
        phi_1283_ = clamp(min(_e99.x, _e99.y), 0f, 1f);
    }
    let _e105 = phi_1283_;
    let _e108 = z4_.g2_[_e93];
    let _e110 = (_e108 >> bitcast<u32>(17u));
    let _e114 = ((f32((_e108 & 131071u)) * 0.00048828125f) + -32f);
    let _e117 = CD.g2_[_e110];
    phi_1226_ = _e114;
    if ((_e117.x & 768u) != 0u) {
        let _e121 = abs(_e114);
        phi_865_ = Ih;
        if Ih {
            phi_865_ = ((_e117.x & 512u) != 0u);
        }
        let _e125 = phi_865_;
        phi_1227_ = _e121;
        if _e125 {
            phi_1227_ = (1f - abs(((fract((_e121 * 0.5f)) * 2f) + -1f)));
        }
        let _e133 = phi_1227_;
        phi_1226_ = _e133;
    }
    let _e135 = phi_1226_;
    let _e136 = clamp(_e135, 0f, 1f);
    phi_1230_ = _e136;
    if Eh {
        let _e138 = (_e117.x >> bitcast<u32>(16u));
        phi_1231_ = _e136;
        if (_e138 != 0u) {
            let _e142 = i0_.g2_[_e93];
            if (_e138 == (_e142 >> bitcast<u32>(16i))) {
                phi_1228_ = min(_e136, unpack2x16float(_e142).x);
            } else {
                phi_1228_ = 0f;
            }
            let _e150 = phi_1228_;
            phi_1231_ = _e150;
        }
        let _e152 = phi_1231_;
        phi_1230_ = _e152;
    }
    let _e154 = phi_1230_;
    phi_902_ = Fh;
    if Fh {
        phi_902_ = ((_e117.x & 1024u) != 0u);
    }
    let _e158 = phi_902_;
    phi_1233_ = _e154;
    if _e158 {
        let _e159 = (_e110 * 8u);
        let _e163 = PB.g2_[(_e159 + 2u)];
        let _e174 = PB.g2_[(_e159 + 3u)];
        let _e179 = _e174.zw;
        let _e181 = ((abs(((mat2x2<f32>(vec2<f32>(_e163.x, _e163.y), vec2<f32>(_e163.z, _e163.w)) * _e59) + _e174.xy)) * _e179) - _e179);
        phi_1233_ = min(_e154, clamp((min(_e181.x, _e181.y) + 0.5f), 0f, 1f));
    }
    let _e189 = phi_1233_;
    let _e190 = (_e117.x & 15u);
    if (_e190 <= 1u) {
        let _e200 = (Eh && (_e190 == 0u));
        phi_1262_ = 0u;
        if _e200 {
            phi_1262_ = (_e117.y | pack2x16float(vec2<f32>(_e189, 0f)));
        }
        let _e205 = phi_1262_;
        phi_1261_ = _e205;
        phi_1258_ = select(unpack4x8unorm(_e117.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e200));
    } else {
        let _e208 = (_e110 * 8u);
        let _e211 = PB.g2_[_e208];
        let _e222 = PB.g2_[(_e208 + 1u)];
        let _e225 = ((mat2x2<f32>(vec2<f32>(_e211.x, _e211.y), vec2<f32>(_e211.z, _e211.w)) * _e59) + _e222.xy);
        if (_e190 == 2u) {
            phi_1232_ = _e225.x;
        } else {
            phi_1232_ = length(_e225);
        }
        let _e230 = phi_1232_;
        let _e239 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e230, 0f, 1f) * _e222.z) + _e222.w), bitcast<f32>(_e117.y)), 0f);
        phi_1260_ = _e239;
        if !((Gh && (((_e117.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e243 = (_e239.xyz * _e239.w);
            let _e249 = vec4<f32>(_e243.x, _e239.y, _e239.z, _e239.w);
            let _e255 = vec4<f32>(_e249.x, _e243.y, _e249.z, _e249.w);
            phi_1260_ = vec4<f32>(_e255.x, _e255.y, _e243.z, _e255.w);
        }
        let _e263 = phi_1260_;
        phi_1261_ = 0u;
        phi_1258_ = _e263;
    }
    let _e265 = phi_1261_;
    let _e267 = phi_1258_;
    phi_650_ = Eh;
    if Eh {
        let _e269 = B3_1;
        phi_650_ = (_e269 != 0u);
    }
    let _e272 = phi_650_;
    phi_1308_ = _e105;
    if _e272 {
        if (_e265 != 0u) {
            phi_1274_ = _e265;
        } else {
            let _e276 = i0_.g2_[_e93];
            phi_1274_ = _e276;
        }
        let _e278 = phi_1274_;
        let _e279 = B3_1;
        if (_e279 == (_e278 >> bitcast<u32>(16i))) {
            phi_1290_ = min(_e105, unpack2x16float(_e278).x);
        } else {
            phi_1290_ = 0f;
        }
        let _e287 = phi_1290_;
        phi_1308_ = _e287;
    }
    let _e289 = phi_1308_;
    let _e290 = J1_1;
    let _e292 = ((_e95 * _e290) * _e289);
    let _e296 = (((_e267 * _e189) * (1f - _e292.w)) + _e292);
    let _e297 = _e296.xyz;
    let _e300 = j.F3_;
    let _e302 = j.G3_;
    if (Lh && (_e296.w != 0f)) {
        phi_1313_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e58.x) + (0.00583715f * _e58.y))))) * _e300) + _e302)) + _e297);
    } else {
        phi_1313_ = _e297;
    }
    let _e318 = phi_1313_;
    let _e324 = vec4<f32>(_e318.x, _e296.y, _e296.z, _e296.w);
    let _e330 = vec4<f32>(_e324.x, _e318.y, _e324.z, _e324.w);
    E1_ = vec4<f32>(_e330.x, _e330.y, _e318.z, _e330.w);
    if (_e265 != 0u) {
        i0_.g2_[_e93] = _e265;
    }
    z4_.g2_[_e93] = 65536u;
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) c2_: vec2<f32>, @location(1) N0_: vec4<f32>, @location(4) @interpolate(flat, either) B3_: u32, @location(3) @interpolate(flat, either) J1_: vec4<f32>, @location(5) @interpolate(flat, either) C1_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    c2_1 = c2_;
    N0_1 = N0_;
    B3_1 = B3_;
    J1_1 = J1_;
    C1_1 = C1_;
    main_1();
    let _e13 = E1_;
    return _e13;
}
