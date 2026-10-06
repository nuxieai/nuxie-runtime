struct Gf {
    k2_: array<vec2<u32>>,
}

struct m0he {
    k2_: array<u32>,
}

struct Hf {
    k2_: array<vec4<f32>>,
}

struct UB {
    Rc: f32,
    Ud: f32,
    dg: f32,
    eg: f32,
    A6_: u32,
    Y9_: u32,
    Pf: u32,
    Qf: u32,
    i8_: vec4<i32>,
    Nh: vec2<f32>,
    Vd: vec2<f32>,
    j2_: u32,
    Rh: f32,
    U4_: u32,
    a3_: f32,
    Wd: f32,
    Jf: u32,
    M3_: f32,
    N3_: f32,
    Xd: f32,
    Kh: u32,
    X9_: u32,
    xc: f32,
    yc: f32,
}

struct L4he {
    k2_: array<u32>,
}

struct L4he_1 {
    k2_: array<atomic<u32>>,
}

@id(7) override ti: bool = true;
@id(4) override qi: bool = true;
@id(0) override mi: bool = true;
@id(1) override ni: bool = true;
@id(2) override oi: bool = true;
@id(3) override pi: bool = true;

@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(3) @binding(9)
var xa: sampler;
@group(0) @binding(3)
var<storage> WC: Gf;
@group(2) @binding(1)
var<storage, read_write> m0_: m0he;
@group(0) @binding(4)
var<storage> JB: Hf;
var<private> gl_FragCoord_1: vec4<f32>;
@group(0) @binding(0)
var<uniform> j: UB;
@group(0) @binding(8)
var ED: texture_2d<f32>;
@group(3) @binding(8)
var ia: sampler;
var<private> S_1: vec4<f32>;
var<private> F0_1: u32;
@group(2) @binding(3)
var<storage, read_write> L4_: L4he_1;
var<private> L1_: vec4<f32>;
@group(1) @binding(11)
var CC: texture_2d<f32>;
@group(1) @binding(13)
var r5_: sampler;

fn main_1() {
    var phi_833_: bool;
    var phi_846_: bool;
    var phi_1738_: f32;
    var phi_1746_: f32;
    var phi_1754_: f32;
    var phi_1753_: f32;
    var phi_1331_: bool;
    var phi_1757_: f32;
    var phi_1756_: f32;
    var phi_1758_: f32;
    var phi_1761_: f32;
    var phi_1760_: f32;
    var phi_1368_: bool;
    var phi_1763_: f32;
    var phi_1801_: u32;
    var phi_1762_: f32;
    var phi_1799_: vec4<f32>;
    var phi_1800_: u32;
    var phi_1797_: vec4<f32>;
    var phi_1817_: u32;
    var phi_1812_: vec4<f32>;
    var phi_1814_: vec3<f32>;

    let _e77 = gl_FragCoord_1;
    let _e78 = _e77.xy;
    let _e81 = bitcast<vec2<u32>>(vec2<i32>(floor(_e78)));
    let _e83 = j.A6_;
    let _e112 = bitcast<i32>((((((_e81.y >> bitcast<u32>(5u)) * (((_e83 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e81.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e81.x & 28u) << bitcast<u32>(5u)) + ((_e81.y & 28u) << bitcast<u32>(2i)))) + (((_e81.y & 3u) << bitcast<u32>(2i)) + (_e81.x & 3u))));
    phi_833_ = pi;
    if pi {
        let _e113 = S_1;
        phi_833_ = (_e113.x < -1.5f);
    }
    let _e117 = phi_833_;
    if _e117 {
        let _e118 = S_1;
        let _e122 = textureSampleLevel(YC, xa, vec2<f32>((3f + _e118.x), 0f), 0f);
        let _e128 = textureSampleLevel(YC, xa, vec2<f32>((1f - _e118.y), 0f), 0f);
        phi_1753_ = ((1f - _e122.x) - _e128.x);
    } else {
        phi_846_ = pi;
        if pi {
            let _e131 = S_1;
            phi_846_ = (_e131.y < -1.5f);
        }
        let _e135 = phi_846_;
        if _e135 {
            let _e136 = S_1;
            let _e139 = max(_e136.w, 0f);
            if (_e136.z >= 0f) {
                let _e142 = textureSampleLevel(YC, xa, vec2<f32>(_e139, 0f), 0f);
                phi_1738_ = _e142.x;
            } else {
                phi_1738_ = 0f;
            }
            let _e145 = phi_1738_;
            phi_1746_ = _e145;
            if (abs(_e136.z) < 1000f) {
                let _e152 = (-2f - _e136.y);
                let _e154 = ((_e152 - _e139) * 0.5984134f);
                let _e157 = (vec4(_e139) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e154));
                let _e163 = ((_e157 * -(_e136.z)) + vec4(((_e152 * _e136.z) + (abs(_e136.x) - 0.25f))));
                let _e166 = textureSampleLevel(YC, xa, vec2<f32>(_e163.x, 0f), 0f);
                let _e169 = textureSampleLevel(YC, xa, vec2<f32>(_e163.y, 0f), 0f);
                let _e172 = textureSampleLevel(YC, xa, vec2<f32>(_e163.z, 0f), 0f);
                let _e175 = textureSampleLevel(YC, xa, vec2<f32>(_e163.w, 0f), 0f);
                let _e181 = (_e157 * 5.0959306f);
                phi_1746_ = (_e145 + (dot(vec4<f32>(_e166.x, _e169.x, _e172.x, _e175.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e181) * (_e181 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e154));
            }
            let _e190 = phi_1746_;
            phi_1754_ = (_e190 * sign(_e136.x));
        } else {
            let _e195 = S_1[0u];
            let _e197 = S_1[1u];
            phi_1754_ = min(min(_e195, abs(_e197)), 1f);
        }
        let _e202 = phi_1754_;
        phi_1753_ = _e202;
    }
    let _e204 = phi_1753_;
    let _e208 = u32(round(((_e204 * 2048f) + 65536f)));
    let _e209 = F0_1;
    let _e212 = ((_e209 << bitcast<u32>(17u)) | _e208);
    let _e215 = atomicMax((&L4_.k2_[_e112]), _e212);
    let _e217 = (_e215 >> bitcast<u32>(17u));
    if (_e217 == _e209) {
        let _e219 = S_1;
        if (_e219.y < 0f) {
            let _e226 = atomicAdd((&L4_.k2_[_e112]), ((_e208 + (_e215 - max(_e212, _e215))) - 65536u));
        }
        phi_1817_ = 0u;
        phi_1812_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e230 = ((f32((_e215 & 131071u)) * 0.00048828125f) + -32f);
        let _e233 = WC.k2_[_e217];
        phi_1756_ = _e230;
        if ((_e233.x & 768u) != 0u) {
            let _e237 = abs(_e230);
            phi_1331_ = qi;
            if qi {
                phi_1331_ = ((_e233.x & 512u) != 0u);
            }
            let _e241 = phi_1331_;
            phi_1757_ = _e237;
            if _e241 {
                phi_1757_ = (1f - abs(((fract((_e237 * 0.5f)) * 2f) + -1f)));
            }
            let _e249 = phi_1757_;
            phi_1756_ = _e249;
        }
        let _e251 = phi_1756_;
        let _e252 = clamp(_e251, 0f, 1f);
        phi_1760_ = _e252;
        if mi {
            let _e254 = (_e233.x >> bitcast<u32>(16u));
            phi_1761_ = _e252;
            if (_e254 != 0u) {
                let _e258 = m0_.k2_[_e112];
                if (_e254 == (_e258 >> bitcast<u32>(16i))) {
                    phi_1758_ = min(_e252, unpack2x16float(_e258).x);
                } else {
                    phi_1758_ = 0f;
                }
                let _e266 = phi_1758_;
                phi_1761_ = _e266;
            }
            let _e268 = phi_1761_;
            phi_1760_ = _e268;
        }
        let _e270 = phi_1760_;
        phi_1368_ = ni;
        if ni {
            phi_1368_ = ((_e233.x & 1024u) != 0u);
        }
        let _e274 = phi_1368_;
        phi_1763_ = _e270;
        if _e274 {
            let _e275 = (_e217 * 8u);
            let _e279 = JB.k2_[(_e275 + 2u)];
            let _e290 = JB.k2_[(_e275 + 3u)];
            let _e295 = _e290.zw;
            let _e297 = ((abs(((mat2x2<f32>(vec2<f32>(_e279.x, _e279.y), vec2<f32>(_e279.z, _e279.w)) * _e78) + _e290.xy)) * _e295) - _e295);
            phi_1763_ = min(_e270, clamp((min(_e297.x, _e297.y) + 0.5f), 0f, 1f));
        }
        let _e305 = phi_1763_;
        let _e306 = (_e233.x & 15u);
        if (_e306 <= 1u) {
            let _e316 = (mi && (_e306 == 0u));
            phi_1801_ = 0u;
            if _e316 {
                phi_1801_ = (_e233.y | pack2x16float(vec2<f32>(_e305, 0f)));
            }
            let _e321 = phi_1801_;
            phi_1800_ = _e321;
            phi_1797_ = select(unpack4x8unorm(_e233.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e316));
        } else {
            let _e324 = (_e217 * 8u);
            let _e327 = JB.k2_[_e324];
            let _e338 = JB.k2_[(_e324 + 1u)];
            let _e341 = ((mat2x2<f32>(vec2<f32>(_e327.x, _e327.y), vec2<f32>(_e327.z, _e327.w)) * _e78) + _e338.xy);
            if (_e306 == 2u) {
                phi_1762_ = _e341.x;
            } else {
                phi_1762_ = length(_e341);
            }
            let _e346 = phi_1762_;
            let _e353 = bitcast<f32>(_e233.y);
            let _e356 = j.xc;
            let _e359 = j.yc;
            let _e362 = textureSampleLevel(ED, ia, vec2<f32>(((clamp(_e346, 0f, 1f) * _e338.z) + _e338.w), ((floor(_e353) * _e356) + _e359)), 0f);
            phi_1799_ = _e362;
            if !((oi && (((_e233.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e366 = (_e362.xyz * _e362.w);
                phi_1799_ = vec4<f32>(_e366.x, _e366.y, _e366.z, (_e362.w * (fract(_e353) * 1.0039216f)));
            }
            let _e375 = phi_1799_;
            phi_1800_ = 0u;
            phi_1797_ = _e375;
        }
        let _e377 = phi_1800_;
        let _e379 = phi_1797_;
        phi_1817_ = _e377;
        phi_1812_ = (_e379 * _e305);
    }
    let _e382 = phi_1817_;
    let _e384 = phi_1812_;
    let _e385 = _e384.xyz;
    let _e388 = j.M3_;
    let _e390 = j.N3_;
    if (ti && (_e384.w != 0f)) {
        phi_1814_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e77.x) + (0.00583715f * _e77.y))))) * _e388) + _e390)) + _e385);
    } else {
        phi_1814_ = _e385;
    }
    let _e406 = phi_1814_;
    let _e412 = vec4<f32>(_e406.x, _e384.y, _e384.z, _e384.w);
    let _e418 = vec4<f32>(_e412.x, _e406.y, _e412.z, _e412.w);
    L1_ = vec4<f32>(_e418.x, _e418.y, _e406.z, _e418.w);
    if (_e382 != 0u) {
        m0_.k2_[_e112] = _e382;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) S: vec4<f32>, @location(1) @interpolate(flat, either) F0_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    S_1 = S;
    F0_1 = F0_;
    main_1();
    let _e7 = L1_;
    return _e7;
}
