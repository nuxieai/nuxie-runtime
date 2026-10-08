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

struct P4Pe_1 {
    v2_: array<atomic<u32>>,
}

@id(7) override fj: bool = true;
@id(4) override cj: bool = true;
@id(0) override Yi: bool = true;
@id(1) override Zi: bool = true;
@id(2) override aj: bool = true;
@id(3) override bj: bool = true;

@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(3) @binding(9)
var Va: sampler;
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
var<private> S_1: vec4<f32>;
var<private> G0_1: u32;
@group(2) @binding(3)
var<storage, read_write> P4_: P4Pe_1;
var<private> N1_: vec4<f32>;
@group(1) @binding(11)
var TB: texture_2d<f32>;
@group(1) @binding(13)
var S4_: sampler;

fn main_1() {
    var phi_833_: bool;
    var phi_846_: bool;
    var phi_1740_: f32;
    var phi_1748_: f32;
    var phi_1756_: f32;
    var phi_1755_: f32;
    var phi_1333_: bool;
    var phi_1759_: f32;
    var phi_1758_: f32;
    var phi_1760_: f32;
    var phi_1763_: f32;
    var phi_1762_: f32;
    var phi_1370_: bool;
    var phi_1765_: f32;
    var phi_1803_: u32;
    var phi_1764_: f32;
    var phi_1801_: vec4<f32>;
    var phi_1802_: u32;
    var phi_1799_: vec4<f32>;
    var phi_1819_: u32;
    var phi_1814_: vec4<f32>;
    var phi_1816_: vec3<f32>;

    let _e77 = gl_FragCoord_1;
    let _e78 = _e77.xy;
    let _e81 = bitcast<vec2<u32>>(vec2<i32>(floor(_e78)));
    let _e83 = j.L6_;
    let _e112 = bitcast<i32>((((((_e81.y >> bitcast<u32>(5u)) * (((_e83 + 31u) & 4294967264u) << bitcast<u32>(5u))) + ((_e81.x >> bitcast<u32>(5u)) << bitcast<u32>(10u))) + (((_e81.x & 28u) << bitcast<u32>(5u)) + ((_e81.y & 28u) << bitcast<u32>(2i)))) + (((_e81.y & 3u) << bitcast<u32>(2i)) + (_e81.x & 3u))));
    phi_833_ = bj;
    if bj {
        let _e113 = S_1;
        phi_833_ = (_e113.x < -1.5f);
    }
    let _e117 = phi_833_;
    if _e117 {
        let _e118 = S_1;
        let _e122 = textureSampleLevel(ZC, Va, vec2<f32>((3f + _e118.x), 0f), 0f);
        let _e128 = textureSampleLevel(ZC, Va, vec2<f32>((1f - _e118.y), 0f), 0f);
        phi_1755_ = ((1f - _e122.x) - _e128.x);
    } else {
        phi_846_ = bj;
        if bj {
            let _e131 = S_1;
            phi_846_ = (_e131.y < -1.5f);
        }
        let _e135 = phi_846_;
        if _e135 {
            let _e136 = S_1;
            let _e139 = max(_e136.w, 0f);
            if (_e136.z >= 0f) {
                let _e142 = textureSampleLevel(ZC, Va, vec2<f32>(_e139, 0f), 0f);
                phi_1740_ = _e142.x;
            } else {
                phi_1740_ = 0f;
            }
            let _e145 = phi_1740_;
            phi_1748_ = _e145;
            if (abs(_e136.z) < 1000f) {
                let _e152 = (-2f - _e136.y);
                let _e154 = ((_e152 - _e139) * 0.5984134f);
                let _e157 = (vec4(_e139) + (vec4<f32>(0.20888568f, 0.62665707f, 1.0444285f, 1.4621998f) * _e154));
                let _e163 = ((_e157 * -(_e136.z)) + vec4(((_e152 * _e136.z) + (abs(_e136.x) - 0.25f))));
                let _e166 = textureSampleLevel(ZC, Va, vec2<f32>(_e163.x, 0f), 0f);
                let _e169 = textureSampleLevel(ZC, Va, vec2<f32>(_e163.y, 0f), 0f);
                let _e172 = textureSampleLevel(ZC, Va, vec2<f32>(_e163.z, 0f), 0f);
                let _e175 = textureSampleLevel(ZC, Va, vec2<f32>(_e163.w, 0f), 0f);
                let _e181 = (_e157 * 5.0959306f);
                phi_1748_ = (_e145 + (dot(vec4<f32>(_e166.x, _e169.x, _e172.x, _e175.x), exp2(((vec4<f32>(2.5479653f, 2.5479653f, 2.5479653f, 2.5479653f) - _e181) * (_e181 + vec4<f32>(-2.5479653f, -2.5479653f, -2.5479653f, -2.5479653f))))) * _e154));
            }
            let _e190 = phi_1748_;
            phi_1756_ = (_e190 * sign(_e136.x));
        } else {
            let _e195 = S_1[0u];
            let _e197 = S_1[1u];
            phi_1756_ = min(min(_e195, abs(_e197)), 1f);
        }
        let _e202 = phi_1756_;
        phi_1755_ = _e202;
    }
    let _e204 = phi_1755_;
    let _e208 = u32(round(((_e204 * 2048f) + 65536f)));
    let _e209 = G0_1;
    let _e212 = ((_e209 << bitcast<u32>(17u)) | _e208);
    let _e215 = atomicMax((&P4_.v2_[_e112]), _e212);
    let _e217 = (_e215 >> bitcast<u32>(17u));
    if (_e217 == _e209) {
        let _e219 = S_1;
        if (_e219.y < 0f) {
            let _e226 = atomicAdd((&P4_.v2_[_e112]), ((_e208 + (_e215 - max(_e212, _e215))) - 65536u));
        }
        phi_1819_ = 0u;
        phi_1814_ = vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let _e230 = ((f32((_e215 & 131071u)) * 0.00048828125f) + -32f);
        let _e233 = WC.v2_[_e217];
        phi_1758_ = _e230;
        if ((_e233.x & 768u) != 0u) {
            let _e237 = abs(_e230);
            phi_1333_ = cj;
            if cj {
                phi_1333_ = ((_e233.x & 512u) != 0u);
            }
            let _e241 = phi_1333_;
            phi_1759_ = _e237;
            if _e241 {
                phi_1759_ = (1f - abs(((fract((_e237 * 0.5f)) * 2f) + -1f)));
            }
            let _e249 = phi_1759_;
            phi_1758_ = _e249;
        }
        let _e251 = phi_1758_;
        let _e252 = clamp(_e251, 0f, 1f);
        phi_1762_ = _e252;
        if Yi {
            let _e254 = (_e233.x >> bitcast<u32>(16u));
            phi_1763_ = _e252;
            if (_e254 != 0u) {
                let _e258 = m0_.v2_[_e112];
                if (_e254 == (_e258 >> bitcast<u32>(16i))) {
                    phi_1760_ = min(_e252, unpack2x16float(_e258).x);
                } else {
                    phi_1760_ = 0f;
                }
                let _e266 = phi_1760_;
                phi_1763_ = _e266;
            }
            let _e268 = phi_1763_;
            phi_1762_ = _e268;
        }
        let _e270 = phi_1762_;
        phi_1370_ = Zi;
        if Zi {
            phi_1370_ = ((_e233.x & 1024u) != 0u);
        }
        let _e274 = phi_1370_;
        phi_1765_ = _e270;
        if _e274 {
            let _e275 = (_e217 * 8u);
            let _e279 = JB.v2_[(_e275 + 2u)];
            let _e290 = JB.v2_[(_e275 + 3u)];
            let _e295 = _e290.zw;
            let _e297 = ((abs(((mat2x2<f32>(vec2<f32>(_e279.x, _e279.y), vec2<f32>(_e279.z, _e279.w)) * _e78) + _e290.xy)) * _e295) - _e295);
            phi_1765_ = min(_e270, clamp((min(_e297.x, _e297.y) + 0.5f), 0f, 1f));
        }
        let _e305 = phi_1765_;
        let _e306 = (_e233.x & 15u);
        if (_e306 <= 1u) {
            let _e316 = (Yi && (_e306 == 0u));
            phi_1803_ = 0u;
            if _e316 {
                phi_1803_ = (_e233.y | pack2x16float(vec2<f32>(_e305, 0f)));
            }
            let _e321 = phi_1803_;
            phi_1802_ = _e321;
            phi_1799_ = select(unpack4x8unorm(_e233.y), vec4<f32>(0f, 0f, 0f, 0f), vec4(_e316));
        } else {
            let _e324 = (_e217 * 8u);
            let _e327 = JB.v2_[_e324];
            let _e338 = JB.v2_[(_e324 + 1u)];
            let _e341 = ((mat2x2<f32>(vec2<f32>(_e327.x, _e327.y), vec2<f32>(_e327.z, _e327.w)) * _e78) + _e338.xy);
            if (_e306 == 2u) {
                phi_1764_ = _e341.x;
            } else {
                phi_1764_ = length(_e341);
            }
            let _e346 = phi_1764_;
            let _e353 = bitcast<f32>(_e233.y);
            let _e356 = j.cd;
            let _e359 = j.g7_;
            let _e362 = textureSampleLevel(YC, I8_, vec2<f32>(((clamp(_e346, 0f, 1f) * _e338.z) + _e338.w), ((floor(_e353) * _e356) + _e359)), 0f);
            phi_1801_ = _e362;
            if !((aj && (((_e233.x >> bitcast<u32>(4i)) & 15u) != 0u))) {
                let _e366 = (_e362.xyz * _e362.w);
                phi_1801_ = vec4<f32>(_e366.x, _e366.y, _e366.z, (_e362.w * (fract(_e353) * 1.0039216f)));
            }
            let _e375 = phi_1801_;
            phi_1802_ = 0u;
            phi_1799_ = _e375;
        }
        let _e377 = phi_1802_;
        let _e379 = phi_1799_;
        phi_1819_ = _e377;
        phi_1814_ = (_e379 * _e305);
    }
    let _e382 = phi_1819_;
    let _e384 = phi_1814_;
    let _e385 = _e384.xyz;
    let _e388 = j.E3_;
    let _e390 = j.F3_;
    if (fj && (_e384.w != 0f)) {
        phi_1816_ = (vec3(((fract((52.982918f * fract(((0.06711056f * _e77.x) + (0.00583715f * _e77.y))))) * _e388) + _e390)) + _e385);
    } else {
        phi_1816_ = _e385;
    }
    let _e406 = phi_1816_;
    let _e412 = vec4<f32>(_e406.x, _e384.y, _e384.z, _e384.w);
    let _e418 = vec4<f32>(_e412.x, _e406.y, _e412.z, _e412.w);
    N1_ = vec4<f32>(_e418.x, _e418.y, _e406.z, _e418.w);
    if (_e382 != 0u) {
        m0_.v2_[_e112] = _e382;
    }
    return;
}

@fragment
fn main(@builtin(position) gl_FragCoord: vec4<f32>, @location(0) S: vec4<f32>, @location(1) @interpolate(flat, either) G0_: u32) -> @location(0) vec4<f32> {
    gl_FragCoord_1 = gl_FragCoord;
    S_1 = S;
    G0_1 = G0_;
    main_1();
    let _e7 = N1_;
    return _e7;
}
