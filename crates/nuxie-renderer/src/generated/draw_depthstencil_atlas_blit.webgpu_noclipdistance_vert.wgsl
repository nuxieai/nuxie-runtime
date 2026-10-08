struct ci {
    v2_: array<vec4<u32>>,
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

struct gg {
    v2_: array<vec2<u32>>,
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
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Ui: bool = true;
@id(2) override Wi: bool = true;
@id(8) override cj: bool = true;

@group(0) @binding(2)
var<storage> KB: ci;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> T2_: vec2<f32>;
@group(0) @binding(3)
var<storage> WC: gg;
var<private> e4_: f32;
var<private> P0_: f32;
@group(0) @binding(4)
var<storage> JB: hg;
var<private> O0_: vec4<f32>;
var<private> V0_: vec3<f32>;
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
    var phi_731_: u32;
    var phi_732_: f32;
    var phi_733_: f32;
    var phi_740_: vec4<f32>;
    var phi_741_: vec4<f32>;
    var phi_404_: bool;
    var phi_742_: f32;

    let _e50 = LB_1;
    let _e53 = (bitcast<u32>(_e50.z) & 65535u);
    let _e58 = KB.v2_[((_e53 * 4u) + 2u)];
    let _e60 = _e50.xy;
    let _e62 = bitcast<vec3<f32>>(_e58.yzw);
    let _e68 = j.xi;
    T2_ = (((_e60 * _e62.x) + _e62.yz) * _e68);
    let _e72 = WC.v2_[_e53];
    let _e74 = (_e72.x & 15u);
    if Ui {
        let _e75 = (_e74 == 0u);
        if _e75 {
            phi_731_ = _e72.y;
        } else {
            phi_731_ = _e72.x;
        }
        let _e78 = phi_731_;
        let _e80 = (_e78 >> bitcast<u32>(16i));
        let _e82 = j.p6_;
        if (_e80 == 0u) {
            phi_732_ = 0f;
        } else {
            phi_732_ = unpack2x16float(((_e80 + 1023u) * _e82)).x;
        }
        let _e89 = phi_732_;
        phi_733_ = _e89;
        if _e75 {
            phi_733_ = -(_e89);
        }
        let _e92 = phi_733_;
        e4_ = _e92;
    }
    if Wi {
        P0_ = f32(((_e72.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e74 == 1u) {
        O0_ = unpack4x8unorm(_e72.y);
    } else {
        let _e98 = (_e53 * 8u);
        let _e101 = JB.v2_[_e98];
        let _e112 = JB.v2_[(_e98 + 1u)];
        let _e121 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e72.y));
        let _e123 = ((mat2x2<f32>(vec2<f32>(_e101.x, _e101.y), vec2<f32>(_e101.z, _e101.w)) * _e60) + _e112.xy);
        if (_e112.z > 0.9f) {
            phi_740_ = vec4<f32>(_e121.x, _e121.y, 2f, _e121.w);
        } else {
            phi_740_ = vec4<f32>(_e121.x, _e121.y, _e112.w, _e121.w);
        }
        let _e138 = phi_740_;
        if (f32(_e74) == 2f) {
            let _e164 = vec4<f32>(_e123.x, _e138.y, _e138.z, _e138.w);
            phi_741_ = vec4<f32>(_e164.x, 0f, _e164.z, _e164.w);
        } else {
            let _e146 = vec4<f32>(_e138.x, _e138.y, -(_e138.z), _e138.w);
            let _e152 = vec4<f32>(_e123.x, _e146.y, _e146.z, _e146.w);
            phi_741_ = vec4<f32>(_e152.x, _e123.y, _e152.z, _e152.w);
        }
        let _e171 = phi_741_;
        O0_ = _e171;
        let _e173 = O0_[3u];
        O0_[3u] = -(_e173);
    }
    phi_404_ = cj;
    if cj {
        phi_404_ = ((_e72.x & 2048u) != 0u);
    }
    let _e180 = phi_404_;
    if _e180 {
        let _e181 = (_e53 * 8u);
        let _e185 = JB.v2_[(_e181 + 4u)];
        let _e196 = JB.v2_[(_e181 + 5u)];
        let _e199 = ((mat2x2<f32>(vec2<f32>(_e185.x, _e185.y), vec2<f32>(_e185.z, _e185.w)) * _e60) + _e196.xy);
        phi_742_ = (1f + _e196.z);
        if ((_e72.x & 4096u) != 0u) {
            phi_742_ = (-1f - f32(((_e72.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e210 = phi_742_;
        V0_ = vec3<f32>(_e199.x, _e199.y, _e210);
    } else {
        V0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e215 = j.Dg;
    let _e217 = j.Eg;
    let _e225 = vec4<f32>(((_e50.x * _e215) - 1f), ((_e50.y * _e217) - sign(_e217)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e225.x, _e225.y, ((f32(((_e58.x << bitcast<u32>(8u)) | 255u)) * 0.000000059604645f) + 0.000000029802322f), _e225.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e12 = T2_;
    let _e13 = e4_;
    let _e14 = P0_;
    let _e15 = O0_;
    let _e16 = V0_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
