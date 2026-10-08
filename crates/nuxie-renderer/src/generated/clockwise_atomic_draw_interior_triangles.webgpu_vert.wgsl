struct gi {
    v2_: array<vec4<u32>>,
}

struct jg {
    v2_: array<vec2<u32>>,
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

struct kg {
    v2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct hi {
    v2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) @interpolate(flat, either) member: f32,
    @location(3) @interpolate(flat, either) member_1: f32,
    @location(4) @interpolate(flat, either) member_2: vec2<f32>,
    @location(6) @interpolate(flat, either) member_3: f32,
    @location(5) member_4: vec4<f32>,
    @location(0) member_5: vec4<f32>,
    @location(9) member_6: vec3<f32>,
    @location(7) @interpolate(flat, either) member_7: vec2<u32>,
    @location(8) member_8: vec2<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Yi: bool = true;
@id(2) override aj: bool = true;
@id(1) override Zi: bool = true;
@id(8) override gj: bool = true;

@group(0) @binding(2)
var<storage> KB: gi;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> o1_: f32;
@group(0) @binding(3)
var<storage> WC: jg;
var<private> G0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> j2_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: kg;
var<private> W0_: vec4<f32>;
var<private> P0_: vec4<f32>;
var<private> V0_: vec3<f32>;
var<private> y3_: vec2<u32>;
var<private> J4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> CD: hi;
@group(3) @binding(9)
var Va: sampler;

fn main_1() {
    var phi_913_: f32;
    var phi_914_: u32;
    var phi_915_: f32;
    var phi_916_: f32;
    var phi_739_: bool;
    var phi_917_: vec4<f32>;
    var phi_929_: vec4<f32>;
    var phi_930_: vec4<f32>;
    var phi_931_: f32;
    var phi_472_: bool;
    var phi_932_: f32;

    let _e56 = LB_1;
    let _e59 = (bitcast<u32>(_e56.z) & 65535u);
    let _e65 = (_e59 * 4u);
    let _e68 = KB.v2_[_e65];
    let _e69 = bitcast<vec4<f32>>(_e68);
    let _e80 = KB.v2_[(_e65 + 1u)];
    let _e84 = ((mat2x2<f32>(vec2<f32>(_e69.x, _e69.y), vec2<f32>(_e69.z, _e69.w)) * _e56.xy) + bitcast<vec2<f32>>(_e80.xy));
    o1_ = f32((bitcast<i32>(_e56.z) >> bitcast<u32>(16i)));
    let _e87 = WC.v2_[_e59];
    let _e89 = j.p6_;
    if (_e59 == 0u) {
        phi_913_ = 0f;
    } else {
        phi_913_ = unpack2x16float(((_e59 + 1023u) * _e89)).x;
    }
    let _e96 = phi_913_;
    G0_ = _e96;
    if ((_e87.x & 512u) != 0u) {
        let _e100 = G0_;
        G0_ = -(_e100);
    }
    let _e102 = (_e87.x & 15u);
    if Yi {
        let _e103 = (_e102 == 0u);
        if _e103 {
            phi_914_ = _e87.y;
        } else {
            phi_914_ = _e87.x;
        }
        let _e106 = phi_914_;
        let _e108 = (_e106 >> bitcast<u32>(16i));
        if (_e108 == 0u) {
            phi_915_ = 0f;
        } else {
            phi_915_ = unpack2x16float(((_e108 + 1023u) * _e89)).x;
        }
        let _e115 = phi_915_;
        phi_916_ = _e115;
        if _e103 {
            phi_916_ = -(_e115);
        }
        let _e118 = phi_916_;
        j2_[0u] = _e118;
    }
    if aj {
        Q0_ = f32(((_e87.x >> bitcast<u32>(4i)) & 15u));
    }
    if Zi {
        let _e124 = (_e59 * 8u);
        let _e128 = JB.v2_[(_e124 + 2u)];
        let _e133 = vec2<f32>(_e128.x, _e128.y);
        let _e134 = vec2<f32>(_e128.z, _e128.w);
        let _e139 = JB.v2_[(_e124 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e144 = (abs(_e133) + abs(_e134));
                let _e146 = (_e144.x != 0f);
                phi_739_ = _e146;
                if _e146 {
                    phi_739_ = (_e144.y != 0f);
                }
                let _e150 = phi_739_;
                if _e150 {
                    let _e154 = ((mat2x2<f32>(_e133, _e134) * _e84) + _e139.xy);
                    let _e155 = -(_e154);
                    let _e161 = (vec2<f32>(1f, 1f) / _e144).xyxy;
                    phi_917_ = (((vec4<f32>(_e154.x, _e154.y, _e155.x, _e155.y) * _e161) + _e161) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_917_ = _e139.xyxy;
                    break;
                }
            }
        }
        let _e166 = phi_917_;
        W0_ = _e166;
    }
    if (_e102 == 1u) {
        P0_ = unpack4x8unorm(_e87.y);
    } else {
        if (Yi && (_e102 == 0u)) {
            let _e248 = (_e87.x >> bitcast<u32>(16i));
            if (_e248 == 0u) {
                phi_931_ = 0f;
            } else {
                phi_931_ = unpack2x16float(((_e248 + 1023u) * _e89)).x;
            }
            let _e255 = phi_931_;
            j2_[1u] = _e255;
        } else {
            let _e170 = (_e59 * 8u);
            let _e173 = JB.v2_[_e170];
            let _e184 = JB.v2_[(_e170 + 1u)];
            let _e193 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e87.y));
            let _e195 = ((mat2x2<f32>(vec2<f32>(_e173.x, _e173.y), vec2<f32>(_e173.z, _e173.w)) * _e84) + _e184.xy);
            if (_e184.z > 0.9f) {
                phi_929_ = vec4<f32>(_e193.x, _e193.y, 2f, _e193.w);
            } else {
                phi_929_ = vec4<f32>(_e193.x, _e193.y, _e184.w, _e193.w);
            }
            let _e210 = phi_929_;
            if (f32(_e102) == 2f) {
                let _e236 = vec4<f32>(_e195.x, _e210.y, _e210.z, _e210.w);
                phi_930_ = vec4<f32>(_e236.x, 0f, _e236.z, _e236.w);
            } else {
                let _e218 = vec4<f32>(_e210.x, _e210.y, -(_e210.z), _e210.w);
                let _e224 = vec4<f32>(_e195.x, _e218.y, _e218.z, _e218.w);
                phi_930_ = vec4<f32>(_e224.x, _e195.y, _e224.z, _e224.w);
            }
            let _e243 = phi_930_;
            P0_ = _e243;
            let _e245 = P0_[3u];
            P0_[3u] = -(_e245);
        }
    }
    phi_472_ = gj;
    if gj {
        phi_472_ = ((_e87.x & 2048u) != 0u);
    }
    let _e262 = phi_472_;
    if _e262 {
        let _e263 = (_e59 * 8u);
        let _e267 = JB.v2_[(_e263 + 4u)];
        let _e278 = JB.v2_[(_e263 + 5u)];
        let _e281 = ((mat2x2<f32>(vec2<f32>(_e267.x, _e267.y), vec2<f32>(_e267.z, _e267.w)) * _e84) + _e278.xy);
        phi_932_ = (1f + _e278.z);
        if ((_e87.x & 4096u) != 0u) {
            phi_932_ = (-1f - f32(((_e87.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e292 = phi_932_;
        V0_ = vec3<f32>(_e281.x, _e281.y, _e292);
    } else {
        V0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e297 = j.Gg;
    let _e299 = j.Hg;
    let _e311 = KB.v2_[(_e65 + 3u)];
    y3_ = _e311.xy;
    J4_ = (_e84 + bitcast<vec2<f32>>(_e311.zw));
    unnamed.gl_Position = vec4<f32>(((_e84.x * _e297) - 1f), ((_e84.y * _e299) - sign(_e299)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e16 = o1_;
    let _e17 = G0_;
    let _e18 = j2_;
    let _e19 = Q0_;
    let _e20 = W0_;
    let _e21 = P0_;
    let _e22 = V0_;
    let _e23 = y3_;
    let _e24 = J4_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23, _e24, _e25);
}
