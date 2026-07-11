struct Be {
    X1_: array<vec2<u32>>,
}

struct d0qd {
    X1_: array<u32>,
}

struct Ce {
    X1_: array<vec4<f32>>,
}

struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct p4qd {
    X1_: array<u32>,
}

struct LC {
    r9_: vec4<f32>,
    c2_: vec2<f32>,
    x4_: f32,
    ki: f32,
    k2_: vec4<f32>,
    D2_: vec2<f32>,
    V0_: u32,
    n2_: u32,
    Z6_: u32,
}

@id(7) override Tg: bool = true;
@id(4) override Qg: bool = true;
@id(0) override Mg: bool = true;
@id(1) override Ng: bool = true;

@group(0) @binding(4) 
var<storage> TC: Be;
@group(2) @binding(1) 
var<storage, read_write> d0_: d0qd;
@group(0) @binding(5) 
var<storage> PB: Ce;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(9) 
var DD: texture_2d<f32>;
@group(3) @binding(9) 
var Bb: sampler;
@group(0) @binding(0) 
var<uniform> k: NB;
@group(1) @binding(12) 
var AC: texture_2d<f32>;
@group(1) @binding(14) 
var R5_: sampler;
var<private> U0_1: vec2<f32>;
var<private> N0_1: vec4<f32>;
@group(2) @binding(3) 
var<storage, read_write> p4_: p4qd;
@group(0) @binding(2) 
var<uniform> A0_: LC;
var<private> l1_: vec4<f32>;
@group(3) @binding(10) 
var T9_: sampler;
@group(0) @binding(10) 
var QC: texture_2d<f32>;

fn main_1() {
    var phi_1246_: f32;
    var phi_843_: bool;
    var phi_1193_: f32;
    var phi_1192_: f32;
    var phi_1194_: f32;
    var phi_1197_: f32;
    var phi_1196_: f32;
    var phi_880_: bool;
    var phi_1199_: f32;
    var phi_1226_: u32;
    var phi_1198_: f32;
    var phi_1225_: u32;
    var phi_1223_: vec4<f32>;
    var phi_634_: bool;
    var phi_1237_: u32;
    var phi_1252_: f32;
    var phi_1253_: f32;
    var phi_1274_: vec3<f32>;

    let _e56 = gl_FragCoord_1;
    let _e57 = _e56.xy;
    let _e60 = bitcast<vec2<u32>>(vec2<i32>(floor(_e57)));
    let _e62 = k.q5_;
    let _e91 = bitcast<i32>((((((_e60.y >> bitcast<u32>(5u)) * (((_e62 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e60.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e60.x & 28u) << bitcast<u32>(5u)) + ((_e60.y & 28u) << bitcast<u32>(2i)))) + (((_e60.y & 3u) << bitcast<u32>(2i)) + (_e60.x & 3u))));
    let _e92 = U0_1;
    let _e93 = textureSample(AC, R5_, _e92);
    phi_1246_ = 1f;
    if Ng {
        let _e94 = N0_1;
        let _e97 = min(_e94.xy, _e94.zw);
        phi_1246_ = clamp(min(_e97.x, _e97.y), 0f, 1f);
    }
    let _e103 = phi_1246_;
    let _e106 = p4_.X1_[_e91];
    let _e108 = (_e106 >> bitcast<u32>(17u));
    let _e112 = ((f32((_e106 & 131071u)) * 0.00048828125f) + -32f);
    let _e115 = TC.X1_[_e108];
    phi_1192_ = _e112;
    if ((_e115.x & 768u) != 0u) {
        let _e119 = abs(_e112);
        phi_843_ = Qg;
        if Qg {
            phi_843_ = ((_e115.x & 512u) != 0u);
        }
        let _e123 = phi_843_;
        phi_1193_ = _e119;
        if _e123 {
            phi_1193_ = (1f - abs(((fract((_e119 * 0.5f)) * 2f) + -1f)));
        }
        let _e131 = phi_1193_;
        phi_1192_ = _e131;
    }
    let _e133 = phi_1192_;
    let _e134 = clamp(_e133, 0f, 1f);
    phi_1196_ = _e134;
    if Mg {
        let _e136 = (_e115.x >> bitcast<u32>(16u));
        phi_1197_ = _e134;
        if (_e136 != 0u) {
            let _e140 = d0_.X1_[_e91];
            if (_e136 == (_e140 >> bitcast<u32>(16i))) {
                phi_1194_ = min(_e134, unpack2x16float(_e140).x);
            } else {
                phi_1194_ = 0f;
            }
            let _e148 = phi_1194_;
            phi_1197_ = _e148;
        }
        let _e150 = phi_1197_;
        phi_1196_ = _e150;
    }
    let _e152 = phi_1196_;
    phi_880_ = Ng;
    if Ng {
        phi_880_ = ((_e115.x & 1024u) != 0u);
    }
    let _e156 = phi_880_;
    phi_1199_ = _e152;
    if _e156 {
        let _e157 = (_e108 * 4u);
        let _e161 = PB.X1_[(_e157 + 2u)];
        let _e172 = PB.X1_[(_e157 + 3u)];
        let _e177 = _e172.zw;
        let _e179 = ((abs(((mat2x2<f32>(vec2<f32>(_e161.x, _e161.y), vec2<f32>(_e161.z, _e161.w)) * _e57) + _e172.xy)) * _e177) - _e177);
        phi_1199_ = min(_e152, clamp((min(_e179.x, _e179.y) + 0.5f), 0f, 1f));
    }
    let _e187 = phi_1199_;
    let _e188 = (_e115.x & 15u);
    if (_e188 <= 1u) {
        let _e193 = (Mg && (_e188 == 0u));
        phi_1226_ = 0u;
        if _e193 {
            phi_1226_ = (_e115.y | pack2x16float(vec2<f32>(_e187, 0f)));
        }
        let _e198 = phi_1226_;
        phi_1225_ = _e198;
        phi_1223_ = select(unpack4x8unorm(_e115.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e193));
    } else {
        let _e201 = (_e108 * 4u);
        let _e204 = PB.X1_[_e201];
        let _e215 = PB.X1_[(_e201 + 1u)];
        let _e218 = ((mat2x2<f32>(vec2<f32>(_e204.x, _e204.y), vec2<f32>(_e204.z, _e204.w)) * _e57) + _e215.xy);
        if (_e188 == 2u) {
            phi_1198_ = _e218.x;
        } else {
            phi_1198_ = length(_e218);
        }
        let _e223 = phi_1198_;
        let _e232 = textureSampleLevel(DD, Bb, vec2<f32>(((clamp(_e223, 0f, 1f) * _e215.z) + _e215.w), bitcast<f32>(_e115.y)), 0f);
        phi_1225_ = 0u;
        phi_1223_ = _e232;
    }
    let _e234 = phi_1225_;
    let _e236 = phi_1223_;
    let _e238 = (_e236.w * _e187);
    let _e240 = (_e236.xyz * _e238);
    phi_634_ = Mg;
    if Mg {
        let _e246 = A0_.V0_;
        phi_634_ = (_e246 != 0u);
    }
    let _e249 = phi_634_;
    phi_1253_ = _e103;
    if _e249 {
        if (_e234 != 0u) {
            phi_1237_ = _e234;
        } else {
            let _e253 = d0_.X1_[_e91];
            phi_1237_ = _e253;
        }
        let _e255 = phi_1237_;
        let _e257 = A0_.V0_;
        if (_e257 == (_e255 >> bitcast<u32>(16i))) {
            phi_1252_ = min(_e103, unpack2x16float(_e255).x);
        } else {
            phi_1252_ = 0f;
        }
        let _e265 = phi_1252_;
        phi_1253_ = _e265;
    }
    let _e267 = phi_1253_;
    let _e269 = A0_.x4_;
    let _e271 = (_e93 * (_e267 * _e269));
    let _e275 = ((vec4<f32>(_e240.x, _e240.y, _e240.z, _e238) * (1f - _e271.w)) + _e271);
    let _e276 = _e275.xyz;
    let _e278 = k.y3_;
    let _e280 = k.z3_;
    if Tg {
        phi_1274_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e56.x) + (0.00583715f * _e56.y))))) * _e278) + _e280)) + _e276);
    } else {
        phi_1274_ = _e276;
    }
    let _e294 = phi_1274_;
    let _e300 = vec4<f32>(_e294.x, _e275.y, _e275.z, _e275.w);
    let _e306 = vec4<f32>(_e300.x, _e294.y, _e300.z, _e300.w);
    l1_ = vec4<f32>(_e306.x, _e306.y, _e294.z, _e306.w);
    if (_e234 != 0u) {
        d0_.X1_[_e91] = _e234;
    }
    p4_.X1_[_e91] = 65536u;
    return;
}

@fragment 
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) U0_: vec2<f32>, @location(1) N0_: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    U0_1 = U0_;
    N0_1 = N0_;
    main_1();
    let _e7 = l1_;
    return _e7;
}
