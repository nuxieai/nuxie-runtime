struct dh {
    j2_: array<vec4<u32>>,
}

struct Bf {
    j2_: array<vec2<u32>>,
}

struct UB {
    Qc: f32,
    Td: f32,
    Yf: f32,
    Zf: f32,
    z6_: u32,
    X9_: u32,
    Kf: u32,
    Lf: u32,
    i8_: vec4<i32>,
    Ch: vec2<f32>,
    Ud: vec2<f32>,
    i2_: u32,
    Gh: f32,
    T4_: u32,
    c3_: f32,
    Vd: f32,
    Ef: u32,
    M3_: f32,
    N3_: f32,
    Wd: f32,
    zh: u32,
    W9_: u32,
    wc: f32,
    xc: f32,
}

struct Cf {
    j2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct eh {
    j2_: array<vec4<u32>>,
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

@id(0) override bi: bool = true;
@id(2) override di: bool = true;
@id(1) override ci: bool = true;
@id(8) override ji: bool = true;

@group(0) @binding(2)
var<storage> LB: dh;
var<private> gl_VertexIndex_1: i32;
var<private> MB_1: vec3<f32>;
var<private> m1_: f32;
@group(0) @binding(3)
var<storage> XC: Bf;
var<private> F0_: f32;
@group(0) @binding(0)
var<uniform> j: UB;
var<private> l1_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: Cf;
var<private> R0_: vec4<f32>;
var<private> a1_: vec4<f32>;
var<private> F1_: vec3<f32>;
var<private> q3_: vec2<u32>;
var<private> F4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var TB: texture_2d<u32>;
@group(0) @binding(9)
var ZC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> AD: eh;
@group(3) @binding(9)
var wa: sampler;

fn main_1() {
    var phi_889_: f32;
    var phi_890_: u32;
    var phi_891_: f32;
    var phi_892_: f32;
    var phi_717_: bool;
    var phi_893_: vec4<f32>;
    var phi_905_: vec4<f32>;
    var phi_906_: vec4<f32>;
    var phi_907_: f32;
    var phi_472_: bool;

    let _e52 = MB_1;
    let _e55 = (bitcast<u32>(_e52.z) & 65535u);
    let _e61 = (_e55 * 4u);
    let _e64 = LB.j2_[_e61];
    let _e65 = bitcast<vec4<f32>>(_e64);
    let _e76 = LB.j2_[(_e61 + 1u)];
    let _e80 = ((mat2x2<f32>(vec2<f32>(_e65.x, _e65.y), vec2<f32>(_e65.z, _e65.w)) * _e52.xy) + bitcast<vec2<f32>>(_e76.xy));
    m1_ = f32((bitcast<i32>(_e52.z) >> bitcast<u32>(16i)));
    let _e83 = XC.j2_[_e55];
    let _e85 = j.T4_;
    if (_e55 == 0u) {
        phi_889_ = 0f;
    } else {
        phi_889_ = unpack2x16float(((_e55 + 1023u) * _e85)).x;
    }
    let _e92 = phi_889_;
    F0_ = _e92;
    if ((_e83.x & 512u) != 0u) {
        let _e96 = F0_;
        F0_ = -(_e96);
    }
    let _e98 = (_e83.x & 15u);
    if bi {
        let _e99 = (_e98 == 0u);
        if _e99 {
            phi_890_ = _e83.y;
        } else {
            phi_890_ = _e83.x;
        }
        let _e102 = phi_890_;
        let _e104 = (_e102 >> bitcast<u32>(16i));
        if (_e104 == 0u) {
            phi_891_ = 0f;
        } else {
            phi_891_ = unpack2x16float(((_e104 + 1023u) * _e85)).x;
        }
        let _e111 = phi_891_;
        phi_892_ = _e111;
        if _e99 {
            phi_892_ = -(_e111);
        }
        let _e114 = phi_892_;
        l1_[0u] = _e114;
    }
    if di {
        Q0_ = f32(((_e83.x >> bitcast<u32>(4i)) & 15u));
    }
    if ci {
        let _e120 = (_e55 * 8u);
        let _e124 = JB.j2_[(_e120 + 2u)];
        let _e129 = vec2<f32>(_e124.x, _e124.y);
        let _e130 = vec2<f32>(_e124.z, _e124.w);
        let _e135 = JB.j2_[(_e120 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e140 = (abs(_e129) + abs(_e130));
                let _e142 = (_e140.x != 0f);
                phi_717_ = _e142;
                if _e142 {
                    phi_717_ = (_e140.y != 0f);
                }
                let _e146 = phi_717_;
                if _e146 {
                    let _e150 = ((mat2x2<f32>(_e129, _e130) * _e80) + _e135.xy);
                    let _e151 = -(_e150);
                    let _e157 = (vec2<f32>(1f, 1f) / _e140).xyxy;
                    phi_893_ = (((vec4<f32>(_e150.x, _e150.y, _e151.x, _e151.y) * _e157) + _e157) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_893_ = _e135.xyxy;
                    break;
                }
            }
        }
        let _e162 = phi_893_;
        R0_ = _e162;
    }
    if (_e98 == 1u) {
        a1_ = unpack4x8unorm(_e83.y);
    } else {
        if (bi && (_e98 == 0u)) {
            let _e244 = (_e83.x >> bitcast<u32>(16i));
            if (_e244 == 0u) {
                phi_907_ = 0f;
            } else {
                phi_907_ = unpack2x16float(((_e244 + 1023u) * _e85)).x;
            }
            let _e251 = phi_907_;
            l1_[1u] = _e251;
        } else {
            let _e166 = (_e55 * 8u);
            let _e169 = JB.j2_[_e166];
            let _e180 = JB.j2_[(_e166 + 1u)];
            let _e189 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e83.y));
            let _e191 = ((mat2x2<f32>(vec2<f32>(_e169.x, _e169.y), vec2<f32>(_e169.z, _e169.w)) * _e80) + _e180.xy);
            if (_e180.z > 0.9f) {
                phi_905_ = vec4<f32>(_e189.x, _e189.y, 2f, _e189.w);
            } else {
                phi_905_ = vec4<f32>(_e189.x, _e189.y, _e180.w, _e189.w);
            }
            let _e206 = phi_905_;
            if (f32(_e98) == 2f) {
                let _e232 = vec4<f32>(_e191.x, _e206.y, _e206.z, _e206.w);
                phi_906_ = vec4<f32>(_e232.x, 0f, _e232.z, _e232.w);
            } else {
                let _e214 = vec4<f32>(_e206.x, _e206.y, -(_e206.z), _e206.w);
                let _e220 = vec4<f32>(_e191.x, _e214.y, _e214.z, _e214.w);
                phi_906_ = vec4<f32>(_e220.x, _e191.y, _e220.z, _e220.w);
            }
            let _e239 = phi_906_;
            a1_ = _e239;
            let _e241 = a1_[3u];
            a1_[3u] = -(_e241);
        }
    }
    phi_472_ = ji;
    if ji {
        phi_472_ = ((_e83.x & 2048u) != 0u);
    }
    let _e258 = phi_472_;
    if _e258 {
        let _e259 = (_e55 * 8u);
        let _e263 = JB.j2_[(_e259 + 4u)];
        let _e274 = JB.j2_[(_e259 + 5u)];
        let _e277 = ((mat2x2<f32>(vec2<f32>(_e263.x, _e263.y), vec2<f32>(_e263.z, _e263.w)) * _e80) + _e274.xy);
        F1_ = vec3<f32>(_e277.x, _e277.y, (1f + _e274.z));
    } else {
        F1_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e284 = j.Yf;
    let _e286 = j.Zf;
    let _e298 = LB.j2_[(_e61 + 3u)];
    q3_ = _e298.xy;
    F4_ = (_e80 + bitcast<vec2<f32>>(_e298.zw));
    unnamed.gl_Position = vec4<f32>(((_e80.x * _e284) - 1f), ((_e80.y * _e286) - sign(_e286)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) MB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    MB_1 = MB;
    main_1();
    let _e16 = m1_;
    let _e17 = F0_;
    let _e18 = l1_;
    let _e19 = Q0_;
    let _e20 = R0_;
    let _e21 = a1_;
    let _e22 = F1_;
    let _e23 = q3_;
    let _e24 = F4_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23, _e24, _e25);
}
