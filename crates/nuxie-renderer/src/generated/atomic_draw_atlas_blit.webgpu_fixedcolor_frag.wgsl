struct ug {
    v2_: array<vec2<u32>>,
}

struct m0cf {
    v2_: array<u32>,
}

struct vg {
    v2_: array<vec4<f32>>,
}

struct WB {
    Fd: f32,
    Pe: f32,
    Sg: f32,
    Tg: f32,
    T6_: u32,
    Fa: u32,
    Eg: u32,
    Fg: u32,
    J8_: vec4<i32>,
    Ki: vec2<f32>,
    Qe: vec2<f32>,
    r2_: u32,
    Oi: f32,
    z6_: u32,
    i3_: f32,
    Re: f32,
    xg: u32,
    H3_: f32,
    I3_: f32,
    Se: f32,
    Hi: u32,
    Ea: u32,
    Q8_: f32,
    R8_: f32,
}

struct W4cf {
    v2_: array<u32>,
}

@id(8) override rj: bool = true;
@id(9) override sj: bool = true;
@id(5) override oj: bool = true;
@id(0) override jj: bool = true;
@id(1) override kj: bool = true;
@id(2) override lj: bool = true;

@group(0) @binding(3)
var<storage> XC: ug;
@group(2) @binding(1)
var<storage, read_write> m0_: m0cf;
@group(0) @binding(4)
var<storage> JB: vg;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: WB;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(3) @binding(8)
var S8_: sampler;
@group(2) @binding(3)
var<storage, read_write> W4_: W4cf;
var<private> I0_1: u32;
@group(0) @binding(10)
var JD: texture_2d<f32>;
@group(3) @binding(10)
var Ta: sampler;
var<private> U2_1: vec2<f32>;
var<private> Q1_: vec4<f32>;
@group(3) @binding(9)
var eb: sampler;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(1) @binding(11)
var RB: texture_2d<f32>;
@group(1) @binding(13)
var v4_: sampler;

fn main_1() {
    var phi_960_: bool;
    var phi_1431_: f32;
    var phi_1430_: f32;
    var phi_1432_: f32;
    var phi_1435_: f32;
    var phi_1434_: f32;
    var phi_997_: bool;
    var phi_1439_: f32;
    var phi_1436_: f32;
    var phi_1437_: f32;
    var phi_1438_: vec2<f32>;
    var phi_1465_: vec4<f32>;
    var phi_1467_: u32;
    var phi_1466_: u32;
    var phi_1463_: vec4<f32>;
    var phi_1480_: vec3<f32>;

    let _e66 = gl_FragCoord_1;
    let _e67 = _e66.xy;
    let _e70 = bitcast<vec2<u32>>(vec2<i32>(floor(_e67)));
    let _e72 = j.T6_;
    let _e101 = bitcast<i32>((((((_e70.y >> bitcast<u32>(5u)) * (((_e72 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e70.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e70.x & 28u) << bitcast<u32>(5u)) + ((_e70.y & 28u) << bitcast<u32>(2i)))) + (((_e70.y & 3u) << bitcast<u32>(2i)) + (_e70.x & 3u))));
    let _e104 = W4_.v2_[_e101];
    let _e106 = (_e104 >> bitcast<u32>(17u));
    let _e107 = I0_1;
    let _e111 = U2_1;
    let _e112 = textureSampleLevel(JD, Ta, _e111, 0f);
    W4_.v2_[_e101] = (((_e107 << bitcast<u32>(17u)) + 65536u) + bitcast<u32>(i32(round((clamp(_e112.x, 0f, 1f) * 2048f)))));
    let _e123 = ((f32((_e104 & 131071u)) * 0.00048828125f) + -32f);
    let _e126 = XC.v2_[_e106];
    phi_1430_ = _e123;
    if ((_e126.x & 768u) != 0u) {
        let _e130 = abs(_e123);
        phi_960_ = oj;
        if oj {
            phi_960_ = ((_e126.x & 512u) != 0u);
        }
        let _e134 = phi_960_;
        phi_1431_ = _e130;
        if _e134 {
            phi_1431_ = (1f - abs(((fract((_e130 * 0.5f)) * 2f) + -1f)));
        }
        let _e142 = phi_1431_;
        phi_1430_ = _e142;
    }
    let _e144 = phi_1430_;
    let _e145 = clamp(_e144, 0f, 1f);
    phi_1434_ = _e145;
    if jj {
        let _e147 = (_e126.x >> bitcast<u32>(16u));
        phi_1435_ = _e145;
        if (_e147 != 0u) {
            let _e151 = m0_.v2_[_e101];
            if (_e147 == (_e151 >> bitcast<u32>(16i))) {
                phi_1432_ = min(_e145, unpack2x16float(_e151).x);
            } else {
                phi_1432_ = 0f;
            }
            let _e159 = phi_1432_;
            phi_1435_ = _e159;
        }
        let _e161 = phi_1435_;
        phi_1434_ = _e161;
    }
    let _e163 = phi_1434_;
    phi_997_ = kj;
    if kj {
        phi_997_ = ((_e126.x & 1024u) != 0u);
    }
    let _e167 = phi_997_;
    phi_1439_ = _e163;
    if _e167 {
        let _e168 = (_e106 * 8u);
        let _e172 = JB.v2_[(_e168 + 2u)];
        let _e183 = JB.v2_[(_e168 + 3u)];
        let _e188 = _e183.zw;
        let _e190 = ((abs(((mat2x2<f32>(vec2<f32>(_e172.x, _e172.y), vec2<f32>(_e172.z, _e172.w)) * _e67) + _e183.xy)) * _e188) - _e188);
        phi_1439_ = min(_e163, clamp((min(_e190.x, _e190.y) + 0.5f), 0f, 1f));
    }
    let _e198 = phi_1439_;
    let _e199 = (_e126.x & 15u);
    if (_e199 <= 1u) {
        let _e285 = (jj && (_e199 == 0u));
        phi_1467_ = 0u;
        if _e285 {
            phi_1467_ = (_e126.y | pack2x16float(vec2<f32>(_e198, 0f)));
        }
        let _e290 = phi_1467_;
        phi_1466_ = _e290;
        phi_1463_ = select(unpack4x8unorm(_e126.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e285));
    } else {
        let _e206 = (_e106 * 8u);
        let _e209 = JB.v2_[_e206];
        let _e220 = JB.v2_[(_e206 + 1u)];
        let _e223 = ((mat2x2<f32>(vec2<f32>(_e209.x, _e209.y), vec2<f32>(_e209.z, _e209.w)) * _e67) + _e220.xy);
        let _e227 = f32(_e199);
        let _e229 = j.Q8_;
        let _e231 = j.R8_;
        let _e236 = ((_e220.w * _e229) + _e231);
        switch bitcast<i32>(0u) {
            default: {
                if (sj && (_e227 == 4f)) {
                    let _e254 = normalize(vec2<f32>(_e223.x, -(_e223.y)));
                    let _e257 = acos(clamp(_e254.x, -1f, 1f));
                    if (_e254.y >= 0f) {
                        phi_1437_ = _e257;
                    } else {
                        phi_1437_ = -(_e257);
                    }
                    let _e262 = phi_1437_;
                    phi_1438_ = vec2<f32>((_e262 * 0.15915494f), _e236);
                    break;
                } else {
                    if (_e227 == 2f) {
                        phi_1436_ = _e223.x;
                    } else {
                        phi_1436_ = length(_e223);
                    }
                    let _e244 = phi_1436_;
                    phi_1438_ = vec2<f32>(((clamp(_e244, 0f, 1f) * select(0.001953125f, 0.9980469f, (_e220.z < 0f))) + ((max(0f, _e220.z) * 0.001953125f) + 0.0009765625f)), _e236);
                    break;
                }
            }
        }
        let _e266 = phi_1438_;
        let _e267 = textureSampleLevel(ZC, S8_, _e266, 0f);
        phi_1465_ = _e267;
        if !((lj && (((_e126.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
            let _e271 = (_e267.xyz * _e267.w);
            phi_1465_ = vec4<f32>(_e271.x, _e271.y, _e271.z, (_e267.w * abs(bitcast<f32>(_e126.y))));
        }
        let _e281 = phi_1465_;
        phi_1466_ = 0u;
        phi_1463_ = _e281;
    }
    let _e294 = phi_1466_;
    let _e296 = phi_1463_;
    let _e297 = (_e296 * _e198);
    let _e298 = _e297.xyz;
    let _e301 = j.H3_;
    let _e303 = j.I3_;
    if (rj && (_e297.w != 0f)) {
        phi_1480_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e66.x) + (0.00583715f * _e66.y))))) * _e301) + _e303)) + _e298);
    } else {
        phi_1480_ = _e298;
    }
    let _e319 = phi_1480_;
    let _e325 = vec4<f32>(_e319.x, _e297.y, _e297.z, _e297.w);
    let _e331 = vec4<f32>(_e325.x, _e319.y, _e325.z, _e325.w);
    Q1_ = vec4<f32>(_e331.x, _e331.y, _e319.z, _e331.w);
    if (_e294 != 0u) {
        m0_.v2_[_e101] = _e294;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(1) @interpolate(flat, either) I0_: u32, @location(0) U2_: vec2<f32>) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    I0_1 = I0_;
    U2_1 = U2_;
    main_1();
    let _e7 = Q1_;
    return _e7;
}
