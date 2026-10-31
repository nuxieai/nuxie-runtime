struct nd {
    z2_: array<vec4<u32>>,
}

struct n0hf {
    z2_: array<u32>,
}

struct Cg {
    z2_: array<vec4<f32>>,
}

struct RB {
    Kd: f32,
    Ue: f32,
    ah: f32,
    bh: f32,
    V5_: u32,
    Ia: u32,
    Lg: u32,
    Ng: u32,
    M8_: vec4<i32>,
    Ri: vec2<f32>,
    Ve: vec2<f32>,
    y2_: u32,
    Vi: f32,
    z6_: u32,
    n3_: f32,
    We: f32,
    Eg: u32,
    K3_: f32,
    L3_: f32,
    Xe: f32,
    Oi: u32,
    Ha: u32,
    T8_: f32,
    U8_: f32,
}

struct d5hf {
    z2_: array<u32>,
}

@id(8) override Cj: bool = true;
@id(9) override Dj: bool = true;
@id(5) override zj: bool = true;
@id(0) override uj: bool = true;
@id(1) override vj: bool = true;
@id(2) override wj: bool = true;

@group(0) @binding(2)
var<storage> JB: nd;
@group(2) @binding(1)
var<storage, read_write> n0_: n0hf;
@group(0) @binding(3)
var<storage> KB: Cg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: RB;
@group(0) @binding(7)
var YC: texture_2d<f32>;
@group(3) @binding(7)
var V8_: sampler;
@group(2) @binding(3)
var<storage, read_write> d5_: d5hf;
var<private> J0_1: u32;
@group(0) @binding(9)
var ID: texture_2d<f32>;
@group(3) @binding(9)
var Xa: sampler;
var<private> W2_1: vec2<f32>;
var<private> S1_: vec4<f32>;
@group(3) @binding(8)
var ib: sampler;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(1) @binding(10)
var SB: texture_2d<f32>;
@group(1) @binding(12)
var x4_: sampler;

fn main_1() {
    var phi_963_: bool;
    var phi_1427_: f32;
    var phi_1426_: f32;
    var phi_1428_: f32;
    var phi_1431_: f32;
    var phi_1430_: f32;
    var phi_1000_: bool;
    var phi_1435_: f32;
    var phi_1432_: f32;
    var phi_1433_: f32;
    var phi_1434_: vec2<f32>;
    var phi_1461_: vec4<f32>;
    var phi_1463_: u32;
    var phi_1462_: u32;
    var phi_1459_: vec4<f32>;
    var phi_1476_: vec3<f32>;

    let _e67 = gl_FragCoord_1;
    let _e68 = _e67.xy;
    let _e71 = bitcast<vec2<u32>>(vec2<i32>(floor(_e68)));
    let _e73 = j.V5_;
    let _e102 = bitcast<i32>((((((_e71.y >> bitcast<u32>(5u)) * (((_e73 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e71.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e71.x & 28u) << bitcast<u32>(5u)) + ((_e71.y & 28u) << bitcast<u32>(2i)))) + (((_e71.y & 3u) << bitcast<u32>(2i)) + (_e71.x & 3u))));
    let _e105 = d5_.z2_[_e102];
    let _e107 = (_e105 >> bitcast<u32>(17u));
    let _e108 = J0_1;
    let _e112 = W2_1;
    let _e113 = textureSampleLevel(ID, Xa, _e112, 0f);
    d5_.z2_[_e102] = (((_e108 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e113.x, 0f, 1f) * 2048f)))));
    let _e124 = ((f32((_e105 & 131071u)) * 0.00048828125f) + -32f);
    let _e129 = JB.z2_[((_e107 * 4u) + 1u)];
    phi_1426_ = _e124;
    if ((_e129.z & 768u) != 0u) {
        let _e133 = abs(_e124);
        phi_963_ = zj;
        if zj {
            phi_963_ = ((_e129.z & 512u) != 0u);
        }
        let _e137 = phi_963_;
        phi_1427_ = _e133;
        if _e137 {
            phi_1427_ = (1f - abs(((fract((_e133 * 0.5f)) * 2f) + -1f)));
        }
        let _e145 = phi_1427_;
        phi_1426_ = _e145;
    }
    let _e147 = phi_1426_;
    let _e148 = clamp(_e147, 0f, 1f);
    phi_1430_ = _e148;
    if uj {
        let _e150 = (_e129.z >> bitcast<u32>(16u));
        phi_1431_ = _e148;
        if (_e150 != 0u) {
            let _e154 = n0_.z2_[_e102];
            if (_e150 == (_e154 >> bitcast<u32>(16i))) {
                phi_1428_ = min(_e148, unpack2x16float(_e154).x);
            } else {
                phi_1428_ = 0f;
            }
            let _e162 = phi_1428_;
            phi_1431_ = _e162;
        }
        let _e164 = phi_1431_;
        phi_1430_ = _e164;
    }
    let _e166 = phi_1430_;
    phi_1000_ = vj;
    if vj {
        phi_1000_ = ((_e129.z & 1024u) != 0u);
    }
    let _e170 = phi_1000_;
    phi_1435_ = _e166;
    if _e170 {
        let _e171 = (_e107 * 8u);
        let _e175 = KB.z2_[(_e171 + 2u)];
        let _e186 = KB.z2_[(_e171 + 3u)];
        let _e191 = _e186.zw;
        let _e193 = ((abs(((mat2x2<f32>(vec2<f32>(_e175.x, _e175.y), vec2<f32>(_e175.z, _e175.w)) * _e68) + _e186.xy)) * _e191) - _e191);
        phi_1435_ = min(_e166, clamp((min(_e193.x, _e193.y) + 0.5f), 0f, 1f));
    }
    let _e201 = phi_1435_;
    let _e202 = (_e129.z & 15u);
    if (_e202 <= 1u) {
        let _e284 = (uj && (_e202 == 0u));
        phi_1463_ = 0u;
        if _e284 {
            phi_1463_ = (_e129.w | pack2x16float(vec2<f32>(_e201, 0f)));
        }
        let _e289 = phi_1463_;
        phi_1462_ = _e289;
        phi_1459_ = select(unpack4x8unorm(_e129.w), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e284));
    } else {
        let _e209 = (_e107 * 8u);
        let _e212 = KB.z2_[_e209];
        let _e223 = KB.z2_[(_e209 + 1u)];
        let _e226 = ((mat2x2<f32>(vec2<f32>(_e212.x, _e212.y), vec2<f32>(_e212.z, _e212.w)) * _e68) + _e223.xy);
        let _e230 = f32(_e202);
        let _e232 = j.T8_;
        let _e234 = j.U8_;
        let _e239 = ((_e223.w * _e232) + _e234);
        switch bitcast<i32>(0u) {
            default: {
                if (Dj && (_e230 == 4f)) {
                    let _e253 = normalize(_e226);
                    let _e256 = acos(clamp(_e253.x, -1f, 1f));
                    if (_e253.y >= 0f) {
                        phi_1433_ = _e256;
                    } else {
                        phi_1433_ = -(_e256);
                    }
                    let _e261 = phi_1433_;
                    phi_1434_ = vec2<f32>((_e261 * 0.15915494f), _e239);
                    break;
                } else {
                    if (_e230 == 2f) {
                        phi_1432_ = _e226.x;
                    } else {
                        phi_1432_ = length(_e226);
                    }
                    let _e247 = phi_1432_;
                    phi_1434_ = vec2<f32>(((clamp(_e247, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e223.z < 0f))) + ((max(0f, _e223.z) * 0.001953125f) + 0.0009765625f)), _e239);
                    break;
                }
            }
        }
        let _e265 = phi_1434_;
        let _e266 = textureSampleLevel(YC, V8_, _e265, 0f);
        phi_1461_ = _e266;
        if !((wj && (((_e129.z >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e270 = (_e266.xyz * _e266.w);
            phi_1461_ = vec4<f32>(_e270.x, _e270.y, _e270.z, (_e266.w * abs(bitcast<f32>(_e129.w))));
        }
        let _e280 = phi_1461_;
        phi_1462_ = 0u;
        phi_1459_ = _e280;
    }
    let _e293 = phi_1462_;
    let _e295 = phi_1459_;
    let _e296 = (_e295 * _e201);
    let _e297 = _e296.xyz;
    let _e300 = j.K3_;
    let _e302 = j.L3_;
    if (Cj && (_e296.w != 0f)) {
        phi_1476_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e67.x) + (0.00583715f * _e67.y))))) * _e300) + _e302)) + _e297);
    } else {
        phi_1476_ = _e297;
    }
    let _e318 = phi_1476_;
    let _e324 = vec4<f32>(_e318.x, _e296.y, _e296.z, _e296.w);
    let _e330 = vec4<f32>(_e324.x, _e318.y, _e324.z, _e324.w);
    S1_ = vec4<f32>(_e330.x, _e330.y, _e318.z, _e330.w);
    if (_e293 != 0u) {
        n0_.z2_[_e102] = _e293;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) J0_: u32, @location(0) W2_: vec2<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    J0_1 = J0_;
    W2_1 = W2_;
    main_1();
    let _e7 = S1_;
    return _e7;
}
