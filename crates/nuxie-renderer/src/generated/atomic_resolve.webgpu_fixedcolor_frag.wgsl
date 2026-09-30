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
@group(2) @binding(3)
var<storage, read_write> z4_: z4Td;
var<private> E1_: vec4<f32>;
@group(3) @binding(9)
var ga: sampler;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(1) @binding(11)
var GC: texture_2d<f32>;
@group(1) @binding(13)
var Y5_: sampler;

fn main_1() {
    var phi_699_: bool;
    var phi_1018_: f32;
    var phi_1017_: f32;
    var phi_1019_: f32;
    var phi_1022_: f32;
    var phi_1021_: f32;
    var phi_736_: bool;
    var phi_1035_: f32;
    var phi_1023_: f32;
    var phi_1040_: vec4<f32>;
    var phi_1038_: vec4<f32>;
    var phi_1041_: vec3<f32>;

    let _e52 = gl_FragCoord_1;
    let _e53 = _e52.xy;
    let _e56 = bitcast<vec2<u32>>(vec2<i32>(floor(_e53)));
    let _e58 = j.q6_;
    let _e87 = bitcast<i32>((((((_e56.y >> bitcast<u32>(5u)) * (((_e58 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e56.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e56.x & 28u) << bitcast<u32>(5u)) + ((_e56.y & 28u) << bitcast<u32>(2i)))) + (((_e56.y & 3u) << bitcast<u32>(2i)) + (_e56.x & 3u))));
    let _e90 = z4_.g2_[_e87];
    let _e94 = ((f32((_e90 & 131071u)) * 0.00048828125f) + -32f);
    let _e96 = (_e90 >> bitcast<u32>(17u));
    let _e99 = CD.g2_[_e96];
    phi_1017_ = _e94;
    if ((_e99.x & 768u) != 0u) {
        let _e103 = abs(_e94);
        phi_699_ = Ih;
        if Ih {
            phi_699_ = ((_e99.x & 512u) != 0u);
        }
        let _e107 = phi_699_;
        phi_1018_ = _e103;
        if _e107 {
            phi_1018_ = (1f - abs(((fract((_e103 * 0.5f)) * 2f) + -1f)));
        }
        let _e115 = phi_1018_;
        phi_1017_ = _e115;
    }
    let _e117 = phi_1017_;
    let _e118 = clamp(_e117, 0f, 1f);
    phi_1021_ = _e118;
    if Eh {
        let _e120 = (_e99.x >> bitcast<u32>(16u));
        phi_1022_ = _e118;
        if (_e120 != 0u) {
            let _e124 = i0_.g2_[_e87];
            if (_e120 == (_e124 >> bitcast<u32>(16i))) {
                phi_1019_ = min(_e118, unpack2x16float(_e124).x);
            } else {
                phi_1019_ = 0f;
            }
            let _e132 = phi_1019_;
            phi_1022_ = _e132;
        }
        let _e134 = phi_1022_;
        phi_1021_ = _e134;
    }
    let _e136 = phi_1021_;
    phi_736_ = Fh;
    if Fh {
        phi_736_ = ((_e99.x & 1024u) != 0u);
    }
    let _e140 = phi_736_;
    phi_1035_ = _e136;
    if _e140 {
        let _e141 = (_e96 * 8u);
        let _e145 = PB.g2_[(_e141 + 2u)];
        let _e156 = PB.g2_[(_e141 + 3u)];
        let _e161 = _e156.zw;
        let _e163 = ((abs(((mat2x2<f32>(vec2<f32>(_e145.x, _e145.y), vec2<f32>(_e145.z, _e145.w)) * _e53) + _e156.xy)) * _e161) - _e161);
        phi_1035_ = min(_e136, clamp((min(_e163.x, _e163.y) + 0.5f), 0f, 1f));
    }
    let _e171 = phi_1035_;
    let _e172 = (_e99.x & 15u);
    if (_e172 <= 1u) {
        phi_1038_ = select(unpack4x8unorm(_e99.y), vec4<f32>(0f, 0f, 0f, 0f), vec4((Eh && (_e172 == 0u))));
    } else {
        let _e185 = (_e96 * 8u);
        let _e188 = PB.g2_[_e185];
        let _e199 = PB.g2_[(_e185 + 1u)];
        let _e202 = ((mat2x2<f32>(vec2<f32>(_e188.x, _e188.y), vec2<f32>(_e188.z, _e188.w)) * _e53) + _e199.xy);
        if (_e172 == 2u) {
            phi_1023_ = _e202.x;
        } else {
            phi_1023_ = length(_e202);
        }
        let _e207 = phi_1023_;
        let _e216 = textureSampleLevel(DD, P9_, vec2<f32>(((clamp(_e207, 0f, 1f) * _e199.z) + _e199.w), bitcast<f32>(_e99.y)), 0f);
        phi_1040_ = _e216;
        if !((Gh && (((_e99.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e220 = (_e216.xyz * _e216.w);
            let _e226 = vec4<f32>(_e220.x, _e216.y, _e216.z, _e216.w);
            let _e232 = vec4<f32>(_e226.x, _e220.y, _e226.z, _e226.w);
            phi_1040_ = vec4<f32>(_e232.x, _e232.y, _e220.z, _e232.w);
        }
        let _e240 = phi_1040_;
        phi_1038_ = _e240;
    }
    let _e242 = phi_1038_;
    let _e243 = (_e242 * _e171);
    let _e244 = _e243.xyz;
    let _e247 = j.F3_;
    let _e249 = j.G3_;
    if (Lh && (_e243.w != 0f)) {
        phi_1041_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e52.x) + (0.00583715f * _e52.y))))) * _e247) + _e249)) + _e244);
    } else {
        phi_1041_ = _e244;
    }
    let _e265 = phi_1041_;
    let _e271 = vec4<f32>(_e265.x, _e243.y, _e243.z, _e243.w);
    let _e277 = vec4<f32>(_e271.x, _e265.y, _e271.z, _e271.w);
    E1_ = vec4<f32>(_e277.x, _e277.y, _e265.z, _e277.w);
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e3 = E1_;
    return _e3;
}
