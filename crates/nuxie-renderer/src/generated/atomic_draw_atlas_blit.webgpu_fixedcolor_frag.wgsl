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
@group(0) @binding(10)
var ED: texture_2d<f32>;
@group(3) @binding(10)
var R9_: sampler;
var<private> F2_1: vec2<f32>;
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
    var phi_840_: bool;
    var phi_1220_: f32;
    var phi_1219_: f32;
    var phi_1221_: f32;
    var phi_1224_: f32;
    var phi_1223_: f32;
    var phi_877_: bool;
    var phi_1226_: f32;
    var phi_1252_: u32;
    var phi_1225_: f32;
    var phi_1250_: vec4<f32>;
    var phi_1251_: u32;
    var phi_1248_: vec4<f32>;
    var phi_1263_: vec3<f32>;

    let _e61 = gl_FragCoord_1;
    let _e62 = _e61.xy;
    let _e65 = bitcast<vec2<u32>>(vec2<i32>(floor(_e62)));
    let _e67 = j.n6_;
    let _e96 = bitcast<i32>((((((_e65.y >> bitcast<u32>(5u)) * (((_e67 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e65.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e65.x & 28u) << bitcast<u32>(5u)) + ((_e65.y & 28u) << bitcast<u32>(2i)))) + (((_e65.y & 3u) << bitcast<u32>(2i)) + (_e65.x & 3u))));
    let _e99 = A4_.g2_[_e96];
    let _e101 = (_e99 >> bitcast<u32>(17u));
    let _e102 = D0_1;
    let _e106 = F2_1;
    let _e107 = textureSampleLevel(ED, R9_, _e106, 0f);
    A4_.g2_[_e96] = (((_e102 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e107.x, 0f, 1f) * 2048f)))));
    let _e118 = ((f32((_e99 & 131071u)) * 0.00048828125f) + -32f);
    let _e121 = CD.g2_[_e101];
    phi_1219_ = _e118;
    if ((_e121.x & 768u) != 0u) {
        let _e125 = abs(_e118);
        phi_840_ = Ih;
        if Ih {
            phi_840_ = ((_e121.x & 512u) != 0u);
        }
        let _e129 = phi_840_;
        phi_1220_ = _e125;
        if _e129 {
            phi_1220_ = (1f - abs(((fract((_e125 * 0.5f)) * 2f) + -1f)));
        }
        let _e137 = phi_1220_;
        phi_1219_ = _e137;
    }
    let _e139 = phi_1219_;
    let _e140 = clamp(_e139, 0f, 1f);
    phi_1223_ = _e140;
    if Eh {
        let _e142 = (_e121.x >> bitcast<u32>(16u));
        phi_1224_ = _e140;
        if (_e142 != 0u) {
            let _e146 = i0_.g2_[_e96];
            if (_e142 == (_e146 >> bitcast<u32>(16i))) {
                phi_1221_ = min(_e140, unpack2x16float(_e146).x);
            } else {
                phi_1221_ = 0f;
            }
            let _e154 = phi_1221_;
            phi_1224_ = _e154;
        }
        let _e156 = phi_1224_;
        phi_1223_ = _e156;
    }
    let _e158 = phi_1223_;
    phi_877_ = Fh;
    if Fh {
        phi_877_ = ((_e121.x & 1024u) != 0u);
    }
    let _e162 = phi_877_;
    phi_1226_ = _e158;
    if _e162 {
        let _e163 = (_e101 * 8u);
        let _e167 = PB.g2_[(_e163 + 2u)];
        let _e178 = PB.g2_[(_e163 + 3u)];
        let _e183 = _e178.zw;
        let _e185 = ((abs(((mat2x2<f32>(vec2<f32>(_e167.x, _e167.y), vec2<f32>(_e167.z, _e167.w)) * _e62) + _e178.xy)) * _e183) - _e183);
        phi_1226_ = min(_e158, clamp((min(_e185.x, _e185.y) + 0.5f), 0f, 1f));
    }
    let _e193 = phi_1226_;
    let _e194 = (_e121.x & 15u);
    if (_e194 <= 1u) {
        let _e204 = (Eh && (_e194 == 0u));
        phi_1252_ = 0u;
        if _e204 {
            phi_1252_ = (_e121.y | pack2x16float(vec2<f32>(_e193, 0f)));
        }
        let _e209 = phi_1252_;
        phi_1251_ = _e209;
        phi_1248_ = select(unpack4x8unorm(_e121.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e204));
    } else {
        let _e212 = (_e101 * 8u);
        let _e215 = PB.g2_[_e212];
        let _e226 = PB.g2_[(_e212 + 1u)];
        let _e229 = ((mat2x2<f32>(vec2<f32>(_e215.x, _e215.y), vec2<f32>(_e215.z, _e215.w)) * _e62) + _e226.xy);
        if (_e194 == 2u) {
            phi_1225_ = _e229.x;
        } else {
            phi_1225_ = length(_e229);
        }
        let _e234 = phi_1225_;
        let _e241 = bitcast<f32>(_e121.y);
        let _e244 = j.Zb;
        let _e247 = j.ac;
        let _e250 = textureSampleLevel(DD, M9_, vec2<f32>(((clamp(_e234, 0f, 1f) * _e226.z) + _e226.w), ((floor(_e241) * _e244) + _e247)), 0f);
        phi_1250_ = _e250;
        if !((Gh && (((_e121.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e254 = (_e250.xyz * _e250.w);
            phi_1250_ = vec4<f32>(_e254.x, _e254.y, _e254.z, (_e250.w * (fract(_e241) * 1.0039216f)));
        }
        let _e263 = phi_1250_;
        phi_1251_ = 0u;
        phi_1248_ = _e263;
    }
    let _e265 = phi_1251_;
    let _e267 = phi_1248_;
    let _e268 = (_e267 * _e193);
    let _e269 = _e268.xyz;
    let _e272 = j.F3_;
    let _e274 = j.G3_;
    if (Lh && (_e268.w != 0f)) {
        phi_1263_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e61.x) + (0.00583715f * _e61.y))))) * _e272) + _e274)) + _e269);
    } else {
        phi_1263_ = _e269;
    }
    let _e290 = phi_1263_;
    let _e296 = vec4<f32>(_e290.x, _e268.y, _e268.z, _e268.w);
    let _e302 = vec4<f32>(_e296.x, _e290.y, _e296.z, _e296.w);
    F1_ = vec4<f32>(_e302.x, _e302.y, _e290.z, _e302.w);
    if (_e265 != 0u) {
        i0_.g2_[_e96] = _e265;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) D0_: u32, @location(0) F2_: vec2<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    D0_1 = D0_;
    F2_1 = F2_;
    main_1();
    let _e7 = F1_;
    return _e7;
}
