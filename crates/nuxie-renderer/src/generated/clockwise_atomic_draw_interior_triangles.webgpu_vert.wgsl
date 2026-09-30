struct Hg {
    g2_: array<vec4<u32>>,
}

struct kf {
    g2_: array<vec2<u32>>,
}

struct TB {
    tc: f32,
    Cd: f32,
    Hf: f32,
    If: f32,
    n6_: u32,
    Nb: u32,
    tf: u32,
    uf: u32,
    U7_: vec4<i32>,
    hh: vec2<f32>,
    Dd: vec2<f32>,
    f2_: u32,
    lh: f32,
    c6_: u32,
    W2_: f32,
    Ed: f32,
    nf: u32,
    F3_: f32,
    G3_: f32,
    Fd: f32,
    eh: u32,
    Mb: u32,
    Zb: f32,
    ac: f32,
}

struct lf {
    g2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Ig {
    g2_: array<vec4<u32>>,
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

@id(0) override Hh: bool = true;
@id(2) override Jh: bool = true;
@id(1) override Ih: bool = true;
@id(8) override Ph: bool = true;

@group(0) @binding(2)
var<storage> OB: Hg;
var<private> gl_VertexIndex_1: i32;
var<private> JB_1: vec3<f32>;
var<private> j1_: f32;
@group(0) @binding(3)
var<storage> CD: kf;
var<private> D0_: f32;
@group(0) @binding(0)
var<uniform> j: TB;
var<private> Y1_: vec2<f32>;
var<private> g1_: f32;
@group(0) @binding(4)
var<storage> PB: lf;
var<private> O0_: vec4<f32>;
var<private> X1_: vec4<f32>;
var<private> C2_: vec3<f32>;
var<private> k3_: vec2<u32>;
var<private> v4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var JC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> HD: Ig;
@group(3) @binding(9)
var ca: sampler;

fn main_1() {
    var phi_890_: f32;
    var phi_891_: u32;
    var phi_892_: f32;
    var phi_893_: f32;
    var phi_718_: bool;
    var phi_894_: vec4<f32>;
    var phi_906_: vec4<f32>;
    var phi_907_: vec4<f32>;
    var phi_908_: f32;
    var phi_472_: bool;

    let _e52 = JB_1;
    let _e55 = (bitcast<u32>(_e52.z) & 65535u);
    let _e61 = (_e55 * 4u);
    let _e64 = OB.g2_[_e61];
    let _e65 = bitcast<vec4<f32>>(_e64);
    let _e76 = OB.g2_[(_e61 + 1u)];
    let _e80 = ((mat2x2<f32>(vec2<f32>(_e65.x, _e65.y), vec2<f32>(_e65.z, _e65.w)) * _e52.xy) + bitcast<vec2<f32>>(_e76.xy));
    j1_ = f32((bitcast<i32>(_e52.z) >> bitcast<u32>(16i)));
    let _e83 = CD.g2_[_e55];
    let _e85 = j.c6_;
    if (_e55 == 0u) {
        phi_890_ = 0f;
    } else {
        phi_890_ = unpack2x16float(((_e55 + 1023u) * _e85)).x;
    }
    let _e92 = phi_890_;
    D0_ = _e92;
    if ((_e83.x & 512u) != 0u) {
        let _e96 = D0_;
        D0_ = -(_e96);
    }
    let _e98 = (_e83.x & 15u);
    if Hh {
        let _e99 = (_e98 == 0u);
        if _e99 {
            phi_891_ = _e83.y;
        } else {
            phi_891_ = _e83.x;
        }
        let _e102 = phi_891_;
        let _e104 = (_e102 >> bitcast<u32>(16i));
        if (_e104 == 0u) {
            phi_892_ = 0f;
        } else {
            phi_892_ = unpack2x16float(((_e104 + 1023u) * _e85)).x;
        }
        let _e111 = phi_892_;
        phi_893_ = _e111;
        if _e99 {
            phi_893_ = -(_e111);
        }
        let _e114 = phi_893_;
        Y1_[0u] = _e114;
    }
    if Jh {
        g1_ = f32(((_e83.x >> bitcast<u32>(4i)) & 15u));
    }
    if Ih {
        let _e120 = (_e55 * 8u);
        let _e124 = PB.g2_[(_e120 + 2u)];
        let _e129 = vec2<f32>(_e124.x, _e124.y);
        let _e130 = vec2<f32>(_e124.z, _e124.w);
        let _e135 = PB.g2_[(_e120 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e140 = (abs(_e129) + abs(_e130));
                let _e142 = (_e140.x != 0f);
                phi_718_ = _e142;
                if _e142 {
                    phi_718_ = (_e140.y != 0f);
                }
                let _e146 = phi_718_;
                if _e146 {
                    let _e150 = ((mat2x2<f32>(_e129, _e130) * _e80) + _e135.xy);
                    let _e151 = -(_e150);
                    let _e157 = (vec2<f32>(1f, 1f) / _e140).xyxy;
                    phi_894_ = (((vec4<f32>(_e150.x, _e150.y, _e151.x, _e151.y) * _e157) + _e157) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_894_ = _e135.xyxy;
                    break;
                }
            }
        }
        let _e162 = phi_894_;
        O0_ = _e162;
    }
    if (_e98 == 1u) {
        X1_ = unpack4x8unorm(_e83.y);
    } else {
        if (Hh && (_e98 == 0u)) {
            let _e244 = (_e83.x >> bitcast<u32>(16i));
            if (_e244 == 0u) {
                phi_908_ = 0f;
            } else {
                phi_908_ = unpack2x16float(((_e244 + 1023u) * _e85)).x;
            }
            let _e251 = phi_908_;
            Y1_[1u] = _e251;
        } else {
            let _e166 = (_e55 * 8u);
            let _e169 = PB.g2_[_e166];
            let _e180 = PB.g2_[(_e166 + 1u)];
            let _e189 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e83.y));
            let _e191 = ((mat2x2<f32>(vec2<f32>(_e169.x, _e169.y), vec2<f32>(_e169.z, _e169.w)) * _e80) + _e180.xy);
            if (_e180.z > 0.9f) {
                phi_906_ = vec4<f32>(_e189.x, _e189.y, 2f, _e189.w);
            } else {
                phi_906_ = vec4<f32>(_e189.x, _e189.y, _e180.w, _e189.w);
            }
            let _e206 = phi_906_;
            if (f32(_e98) == 2f) {
                let _e232 = vec4<f32>(_e191.x, _e206.y, _e206.z, _e206.w);
                phi_907_ = vec4<f32>(_e232.x, 0f, _e232.z, _e232.w);
            } else {
                let _e214 = vec4<f32>(_e206.x, _e206.y, -(_e206.z), _e206.w);
                let _e220 = vec4<f32>(_e191.x, _e214.y, _e214.z, _e214.w);
                phi_907_ = vec4<f32>(_e220.x, _e191.y, _e220.z, _e220.w);
            }
            let _e239 = phi_907_;
            X1_ = _e239;
            let _e241 = X1_[3u];
            X1_[3u] = -(_e241);
        }
    }
    phi_472_ = Ph;
    if Ph {
        phi_472_ = ((_e83.x & 2048u) != 0u);
    }
    let _e258 = phi_472_;
    if _e258 {
        let _e259 = (_e55 * 8u);
        let _e263 = PB.g2_[(_e259 + 4u)];
        let _e274 = PB.g2_[(_e259 + 5u)];
        let _e277 = ((mat2x2<f32>(vec2<f32>(_e263.x, _e263.y), vec2<f32>(_e263.z, _e263.w)) * _e80) + _e274.xy);
        C2_ = vec3<f32>(_e277.x, _e277.y, (1f + _e274.z));
    } else {
        C2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e284 = j.Hf;
    let _e286 = j.If;
    let _e298 = OB.g2_[(_e61 + 3u)];
    k3_ = _e298.xy;
    v4_ = (_e80 + bitcast<vec2<f32>>(_e298.zw));
    unnamed.gl_Position = vec4<f32>(((_e80.x * _e284) - 1f), ((_e80.y * _e286) - sign(_e286)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) JB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    JB_1 = JB;
    main_1();
    let _e16 = j1_;
    let _e17 = D0_;
    let _e18 = Y1_;
    let _e19 = g1_;
    let _e20 = O0_;
    let _e21 = X1_;
    let _e22 = C2_;
    let _e23 = k3_;
    let _e24 = v4_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23, _e24, _e25);
}
