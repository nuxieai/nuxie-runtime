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
var<private> S4_1: f32;
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
    var phi_1251_: f32;
    var phi_848_: bool;
    var phi_1198_: f32;
    var phi_1197_: f32;
    var phi_1199_: f32;
    var phi_1202_: f32;
    var phi_1201_: f32;
    var phi_885_: bool;
    var phi_1204_: f32;
    var phi_1231_: u32;
    var phi_1203_: f32;
    var phi_1230_: u32;
    var phi_1228_: vec4<f32>;
    var phi_639_: bool;
    var phi_1242_: u32;
    var phi_1257_: f32;
    var phi_1258_: f32;
    var phi_1279_: vec3<f32>;

    let _e57 = gl_FragCoord_1;
    let _e58 = _e57.xy;
    let _e61 = bitcast<vec2<u32>>(vec2<i32>(floor(_e58)));
    let _e63 = k.q5_;
    let _e92 = bitcast<i32>((((((_e61.y >> bitcast<u32>(5u)) * (((_e63 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e61.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e61.x & 28u) << bitcast<u32>(5u)) + ((_e61.y & 28u) << bitcast<u32>(2i)))) + (((_e61.y & 3u) << bitcast<u32>(2i)) + (_e61.x & 3u))));
    let _e93 = U0_1;
    let _e94 = textureSample(AC, R5_, _e93);
    let _e95 = S4_1;
    let _e96 = min(_e95, 1f);
    phi_1251_ = _e96;
    if Ng {
        let _e97 = N0_1;
        let _e100 = min(_e97.xy, _e97.zw);
        phi_1251_ = clamp(min(_e100.x, _e100.y), 0f, _e96);
    }
    let _e106 = phi_1251_;
    let _e109 = p4_.X1_[_e92];
    let _e111 = (_e109 >> bitcast<u32>(17u));
    let _e115 = ((f32((_e109 & 131071u)) * 0.00048828125f) + -32f);
    let _e118 = TC.X1_[_e111];
    phi_1197_ = _e115;
    if ((_e118.x & 768u) != 0u) {
        let _e122 = abs(_e115);
        phi_848_ = Qg;
        if Qg {
            phi_848_ = ((_e118.x & 512u) != 0u);
        }
        let _e126 = phi_848_;
        phi_1198_ = _e122;
        if _e126 {
            phi_1198_ = (1f - abs(((fract((_e122 * 0.5f)) * 2f) + -1f)));
        }
        let _e134 = phi_1198_;
        phi_1197_ = _e134;
    }
    let _e136 = phi_1197_;
    let _e137 = clamp(_e136, 0f, 1f);
    phi_1201_ = _e137;
    if Mg {
        let _e139 = (_e118.x >> bitcast<u32>(16u));
        phi_1202_ = _e137;
        if (_e139 != 0u) {
            let _e143 = d0_.X1_[_e92];
            if (_e139 == (_e143 >> bitcast<u32>(16i))) {
                phi_1199_ = min(_e137, unpack2x16float(_e143).x);
            } else {
                phi_1199_ = 0f;
            }
            let _e151 = phi_1199_;
            phi_1202_ = _e151;
        }
        let _e153 = phi_1202_;
        phi_1201_ = _e153;
    }
    let _e155 = phi_1201_;
    phi_885_ = Ng;
    if Ng {
        phi_885_ = ((_e118.x & 1024u) != 0u);
    }
    let _e159 = phi_885_;
    phi_1204_ = _e155;
    if _e159 {
        let _e160 = (_e111 * 4u);
        let _e164 = PB.X1_[(_e160 + 2u)];
        let _e175 = PB.X1_[(_e160 + 3u)];
        let _e180 = _e175.zw;
        let _e182 = ((abs(((mat2x2<f32>(vec2<f32>(_e164.x, _e164.y), vec2<f32>(_e164.z, _e164.w)) * _e58) + _e175.xy)) * _e180) - _e180);
        phi_1204_ = min(_e155, clamp((min(_e182.x, _e182.y) + 0.5f), 0f, 1f));
    }
    let _e190 = phi_1204_;
    let _e191 = (_e118.x & 15u);
    if (_e191 <= 1u) {
        let _e196 = (Mg && (_e191 == 0u));
        phi_1231_ = 0u;
        if _e196 {
            phi_1231_ = (_e118.y | pack2x16float(vec2<f32>(_e190, 0f)));
        }
        let _e201 = phi_1231_;
        phi_1230_ = _e201;
        phi_1228_ = select(unpack4x8unorm(_e118.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e196));
    } else {
        let _e204 = (_e111 * 4u);
        let _e207 = PB.X1_[_e204];
        let _e218 = PB.X1_[(_e204 + 1u)];
        let _e221 = ((mat2x2<f32>(vec2<f32>(_e207.x, _e207.y), vec2<f32>(_e207.z, _e207.w)) * _e58) + _e218.xy);
        if (_e191 == 2u) {
            phi_1203_ = _e221.x;
        } else {
            phi_1203_ = length(_e221);
        }
        let _e226 = phi_1203_;
        let _e235 = textureSampleLevel(DD, Bb, vec2<f32>(((clamp(_e226, 0f, 1f) * _e218.z) + _e218.w), bitcast<f32>(_e118.y)), 0f);
        phi_1230_ = 0u;
        phi_1228_ = _e235;
    }
    let _e237 = phi_1230_;
    let _e239 = phi_1228_;
    let _e241 = (_e239.w * _e190);
    let _e243 = (_e239.xyz * _e241);
    phi_639_ = Mg;
    if Mg {
        let _e249 = A0_.V0_;
        phi_639_ = (_e249 != 0u);
    }
    let _e252 = phi_639_;
    phi_1258_ = _e106;
    if _e252 {
        if (_e237 != 0u) {
            phi_1242_ = _e237;
        } else {
            let _e256 = d0_.X1_[_e92];
            phi_1242_ = _e256;
        }
        let _e258 = phi_1242_;
        let _e260 = A0_.V0_;
        if (_e260 == (_e258 >> bitcast<u32>(16i))) {
            phi_1257_ = min(_e106, unpack2x16float(_e258).x);
        } else {
            phi_1257_ = 0f;
        }
        let _e268 = phi_1257_;
        phi_1258_ = _e268;
    }
    let _e270 = phi_1258_;
    let _e272 = A0_.x4_;
    let _e274 = (_e94 * (_e270 * _e272));
    let _e278 = ((vec4<f32>(_e243.x, _e243.y, _e243.z, _e241) * (1f - _e274.w)) + _e274);
    let _e279 = _e278.xyz;
    let _e281 = k.y3_;
    let _e283 = k.z3_;
    if Tg {
        phi_1279_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e57.x) + (0.00583715f * _e57.y))))) * _e281) + _e283)) + _e279);
    } else {
        phi_1279_ = _e279;
    }
    let _e297 = phi_1279_;
    let _e303 = vec4<f32>(_e297.x, _e278.y, _e278.z, _e278.w);
    let _e309 = vec4<f32>(_e303.x, _e297.y, _e303.z, _e303.w);
    l1_ = vec4<f32>(_e309.x, _e309.y, _e297.z, _e309.w);
    if (_e237 != 0u) {
        d0_.X1_[_e92] = _e237;
    }
    p4_.X1_[_e92] = 65536u;
    return;
}

@fragment 
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) U0_: vec2<f32>, @location(1) S4_: f32, @location(2) N0_: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    U0_1 = U0_;
    S4_1 = S4_;
    N0_1 = N0_;
    main_1();
    let _e9 = l1_;
    return _e9;
}
