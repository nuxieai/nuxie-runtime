struct ci {
    v2_: array<vec4<u32>>,
}

struct gg {
    v2_: array<vec2<u32>>,
}

struct VB {
    td: f32,
    ze: f32,
    Dg: f32,
    Eg: f32,
    L6_: u32,
    va: u32,
    pg: u32,
    qg: u32,
    B8_: vec4<i32>,
    xi: vec2<f32>,
    Ae: vec2<f32>,
    r2_: u32,
    Bi: f32,
    p6_: u32,
    h3_: f32,
    Be: f32,
    jg: u32,
    E3_: f32,
    F3_: f32,
    Ce: f32,
    ui: u32,
    ua: u32,
    ad: f32,
    g7_: f32,
    Bb: f32,
}

struct hg {
    v2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct di {
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

@id(0) override Ui: bool = true;
@id(2) override Wi: bool = true;
@id(1) override Vi: bool = true;
@id(8) override cj: bool = true;

@group(0) @binding(2)
var<storage> KB: ci;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> o1_: f32;
@group(0) @binding(3)
var<storage> WC: gg;
var<private> G0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> j2_: vec2<f32>;
var<private> P0_: f32;
@group(0) @binding(4)
var<storage> JB: hg;
var<private> W0_: vec4<f32>;
var<private> O0_: vec4<f32>;
var<private> V0_: vec3<f32>;
var<private> y3_: vec2<u32>;
var<private> J4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> BD: di;
@group(3) @binding(9)
var Ta: sampler;

fn main_1() {
    var phi_912_: f32;
    var phi_913_: u32;
    var phi_914_: f32;
    var phi_915_: f32;
    var phi_738_: bool;
    var phi_916_: vec4<f32>;
    var phi_928_: vec4<f32>;
    var phi_929_: vec4<f32>;
    var phi_930_: f32;
    var phi_472_: bool;
    var phi_931_: f32;

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
        phi_912_ = 0f;
    } else {
        phi_912_ = unpack2x16float(((_e59 + 1023u) * _e89)).x;
    }
    let _e96 = phi_912_;
    G0_ = _e96;
    if ((_e87.x & 512u) != 0u) {
        let _e100 = G0_;
        G0_ = -(_e100);
    }
    let _e102 = (_e87.x & 15u);
    if Ui {
        let _e103 = (_e102 == 0u);
        if _e103 {
            phi_913_ = _e87.y;
        } else {
            phi_913_ = _e87.x;
        }
        let _e106 = phi_913_;
        let _e108 = (_e106 >> bitcast<u32>(16i));
        if (_e108 == 0u) {
            phi_914_ = 0f;
        } else {
            phi_914_ = unpack2x16float(((_e108 + 1023u) * _e89)).x;
        }
        let _e115 = phi_914_;
        phi_915_ = _e115;
        if _e103 {
            phi_915_ = -(_e115);
        }
        let _e118 = phi_915_;
        j2_[0u] = _e118;
    }
    if Wi {
        P0_ = f32(((_e87.x >> bitcast<u32>(4i)) & 15u));
    }
    if Vi {
        let _e124 = (_e59 * 8u);
        let _e128 = JB.v2_[(_e124 + 2u)];
        let _e133 = vec2<f32>(_e128.x, _e128.y);
        let _e134 = vec2<f32>(_e128.z, _e128.w);
        let _e139 = JB.v2_[(_e124 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e144 = (abs(_e133) + abs(_e134));
                let _e146 = (_e144.x != 0f);
                phi_738_ = _e146;
                if _e146 {
                    phi_738_ = (_e144.y != 0f);
                }
                let _e150 = phi_738_;
                if _e150 {
                    let _e154 = ((mat2x2<f32>(_e133, _e134) * _e84) + _e139.xy);
                    let _e155 = -(_e154);
                    let _e161 = (vec2<f32>(1f, 1f) / _e144).xyxy;
                    phi_916_ = (((vec4<f32>(_e154.x, _e154.y, _e155.x, _e155.y) * _e161) + _e161) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_916_ = _e139.xyxy;
                    break;
                }
            }
        }
        let _e166 = phi_916_;
        W0_ = _e166;
    }
    if (_e102 == 1u) {
        O0_ = unpack4x8unorm(_e87.y);
    } else {
        if (Ui && (_e102 == 0u)) {
            let _e248 = (_e87.x >> bitcast<u32>(16i));
            if (_e248 == 0u) {
                phi_930_ = 0f;
            } else {
                phi_930_ = unpack2x16float(((_e248 + 1023u) * _e89)).x;
            }
            let _e255 = phi_930_;
            j2_[1u] = _e255;
        } else {
            let _e170 = (_e59 * 8u);
            let _e173 = JB.v2_[_e170];
            let _e184 = JB.v2_[(_e170 + 1u)];
            let _e193 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e87.y));
            let _e195 = ((mat2x2<f32>(vec2<f32>(_e173.x, _e173.y), vec2<f32>(_e173.z, _e173.w)) * _e84) + _e184.xy);
            if (_e184.z > 0.9f) {
                phi_928_ = vec4<f32>(_e193.x, _e193.y, 2f, _e193.w);
            } else {
                phi_928_ = vec4<f32>(_e193.x, _e193.y, _e184.w, _e193.w);
            }
            let _e210 = phi_928_;
            if (f32(_e102) == 2f) {
                let _e236 = vec4<f32>(_e195.x, _e210.y, _e210.z, _e210.w);
                phi_929_ = vec4<f32>(_e236.x, 0f, _e236.z, _e236.w);
            } else {
                let _e218 = vec4<f32>(_e210.x, _e210.y, -(_e210.z), _e210.w);
                let _e224 = vec4<f32>(_e195.x, _e218.y, _e218.z, _e218.w);
                phi_929_ = vec4<f32>(_e224.x, _e195.y, _e224.z, _e224.w);
            }
            let _e243 = phi_929_;
            O0_ = _e243;
            let _e245 = O0_[3u];
            O0_[3u] = -(_e245);
        }
    }
    phi_472_ = cj;
    if cj {
        phi_472_ = ((_e87.x & 2048u) != 0u);
    }
    let _e262 = phi_472_;
    if _e262 {
        let _e263 = (_e59 * 8u);
        let _e267 = JB.v2_[(_e263 + 4u)];
        let _e278 = JB.v2_[(_e263 + 5u)];
        let _e281 = ((mat2x2<f32>(vec2<f32>(_e267.x, _e267.y), vec2<f32>(_e267.z, _e267.w)) * _e84) + _e278.xy);
        phi_931_ = (1f + _e278.z);
        if ((_e87.x & 4096u) != 0u) {
            phi_931_ = (-1f - f32(((_e87.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e292 = phi_931_;
        V0_ = vec3<f32>(_e281.x, _e281.y, _e292);
    } else {
        V0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e297 = j.Dg;
    let _e299 = j.Eg;
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
    let _e19 = P0_;
    let _e20 = W0_;
    let _e21 = O0_;
    let _e22 = V0_;
    let _e23 = y3_;
    let _e24 = J4_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23, _e24, _e25);
}
