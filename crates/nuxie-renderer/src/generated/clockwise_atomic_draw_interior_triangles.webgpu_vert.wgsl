struct Dg {
    e2_: array<vec4<u32>>,
}

struct kf {
    e2_: array<vec2<u32>>,
}

struct BC {
    sc: f32,
    Cd: f32,
    Gf: f32,
    Hf: f32,
    o6_: u32,
    Pb: u32,
    sf: u32,
    tf: u32,
    V7_: vec4<i32>,
    dh: vec2<f32>,
    Dd: vec2<f32>,
    d2_: u32,
    hh: f32,
    d6_: u32,
    T2_: f32,
    Ed: f32,
    nf: u32,
    C3_: f32,
    D3_: f32,
    Fd: f32,
    ah: u32,
    Ob: u32,
}

struct lf {
    e2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Eg {
    e2_: array<vec4<u32>>,
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

@id(0) override Dh: bool = true;
@id(2) override Fh: bool = true;
@id(1) override Eh: bool = true;
@id(8) override Lh: bool = true;

@group(0) @binding(2)
var<storage> PB: Dg;
var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> h1_: f32;
@group(0) @binding(3)
var<storage> DD: kf;
var<private> C0_: f32;
@group(0) @binding(0)
var<uniform> l: BC;
var<private> W1_: vec2<f32>;
var<private> g2_: f32;
@group(0) @binding(4)
var<storage> QB: lf;
var<private> M0_: vec4<f32>;
var<private> V1_: vec4<f32>;
var<private> B2_: vec3<f32>;
var<private> g3_: vec2<u32>;
var<private> p4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ID: Eg;
@group(3) @binding(9)
var fa: sampler;

fn main_1() {
    var phi_913_: f32;
    var phi_914_: u32;
    var phi_915_: f32;
    var phi_916_: f32;
    var phi_734_: bool;
    var phi_917_: vec4<f32>;
    var phi_929_: vec4<f32>;
    var phi_930_: vec4<f32>;
    var phi_931_: f32;
    var phi_932_: vec4<f32>;
    var phi_488_: bool;

    let _e52 = KB_1;
    let _e55 = (bitcast<u32>(_e52.z) & 65535u);
    let _e61 = (_e55 * 4u);
    let _e64 = PB.e2_[_e61];
    let _e65 = bitcast<vec4<f32>>(_e64);
    let _e76 = PB.e2_[(_e61 + 1u)];
    let _e80 = ((mat2x2<f32>(vec2<f32>(_e65.x, _e65.y), vec2<f32>(_e65.z, _e65.w)) * _e52.xy) + bitcast<vec2<f32>>(_e76.xy));
    h1_ = f32((bitcast<i32>(_e52.z) >> bitcast<u32>(16i)));
    let _e83 = DD.e2_[_e55];
    let _e85 = l.d6_;
    if (_e55 == 0u) {
        phi_913_ = 0f;
    } else {
        phi_913_ = unpack2x16float(((_e55 + 1023u) * _e85)).x;
    }
    let _e92 = phi_913_;
    C0_ = _e92;
    if ((_e83.x & 512u) != 0u) {
        let _e96 = C0_;
        C0_ = -(_e96);
    }
    let _e98 = (_e83.x & 15u);
    if Dh {
        let _e99 = (_e98 == 0u);
        if _e99 {
            phi_914_ = _e83.y;
        } else {
            phi_914_ = _e83.x;
        }
        let _e102 = phi_914_;
        let _e104 = (_e102 >> bitcast<u32>(16i));
        if (_e104 == 0u) {
            phi_915_ = 0f;
        } else {
            phi_915_ = unpack2x16float(((_e104 + 1023u) * _e85)).x;
        }
        let _e111 = phi_915_;
        phi_916_ = _e111;
        if _e99 {
            phi_916_ = -(_e111);
        }
        let _e114 = phi_916_;
        W1_[0u] = _e114;
    }
    if Fh {
        g2_ = f32(((_e83.x >> bitcast<u32>(4i)) & 15u));
    }
    if Eh {
        let _e120 = (_e55 * 8u);
        let _e124 = QB.e2_[(_e120 + 2u)];
        let _e129 = vec2<f32>(_e124.x, _e124.y);
        let _e130 = vec2<f32>(_e124.z, _e124.w);
        let _e135 = QB.e2_[(_e120 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e140 = (abs(_e129) + abs(_e130));
                let _e142 = (_e140.x != 0f);
                phi_734_ = _e142;
                if _e142 {
                    phi_734_ = (_e140.y != 0f);
                }
                let _e146 = phi_734_;
                if _e146 {
                    let _e150 = ((mat2x2<f32>(_e129, _e130) * _e80) + _e135.xy);
                    let _e151 = -(_e150);
                    let _e157 = (vec2<f32>(1f, 1f) / _e140).xyxy;
                    phi_917_ = (((vec4<f32>(_e150.x, _e150.y, _e151.x, _e151.y) * _e157) + _e157) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_917_ = _e135.xyxy;
                    break;
                }
            }
        }
        let _e162 = phi_917_;
        M0_ = _e162;
    }
    if (_e98 == 1u) {
        let _e254 = unpack4x8unorm(_e83.y);
        if Fh {
            phi_932_ = _e254;
        } else {
            let _e257 = (_e254.xyz * _e254.w);
            let _e263 = vec4<f32>(_e257.x, _e254.y, _e254.z, _e254.w);
            let _e269 = vec4<f32>(_e263.x, _e257.y, _e263.z, _e263.w);
            phi_932_ = vec4<f32>(_e269.x, _e269.y, _e257.z, _e269.w);
        }
        let _e277 = phi_932_;
        V1_ = _e277;
    } else {
        if (Dh && (_e98 == 0u)) {
            let _e244 = (_e83.x >> bitcast<u32>(16i));
            if (_e244 == 0u) {
                phi_931_ = 0f;
            } else {
                phi_931_ = unpack2x16float(((_e244 + 1023u) * _e85)).x;
            }
            let _e251 = phi_931_;
            W1_[1u] = _e251;
        } else {
            let _e166 = (_e55 * 8u);
            let _e169 = QB.e2_[_e166];
            let _e180 = QB.e2_[(_e166 + 1u)];
            let _e189 = vec4<f32>(vec4<f32>().x, vec4<f32>().y, vec4<f32>().z, bitcast<f32>(_e83.y));
            let _e191 = ((mat2x2<f32>(vec2<f32>(_e169.x, _e169.y), vec2<f32>(_e169.z, _e169.w)) * _e80) + _e180.xy);
            if (_e180.z > 0.9f) {
                phi_929_ = vec4<f32>(_e189.x, _e189.y, 2f, _e189.w);
            } else {
                phi_929_ = vec4<f32>(_e189.x, _e189.y, _e180.w, _e189.w);
            }
            let _e206 = phi_929_;
            if (f32(_e98) == 2f) {
                let _e232 = vec4<f32>(_e191.x, _e206.y, _e206.z, _e206.w);
                phi_930_ = vec4<f32>(_e232.x, 0f, _e232.z, _e232.w);
            } else {
                let _e214 = vec4<f32>(_e206.x, _e206.y, -(_e206.z), _e206.w);
                let _e220 = vec4<f32>(_e191.x, _e214.y, _e214.z, _e214.w);
                phi_930_ = vec4<f32>(_e220.x, _e191.y, _e220.z, _e220.w);
            }
            let _e239 = phi_930_;
            V1_ = _e239;
            let _e241 = V1_[3u];
            V1_[3u] = -(_e241);
        }
    }
    phi_488_ = Lh;
    if Lh {
        phi_488_ = ((_e83.x & 2048u) != 0u);
    }
    let _e281 = phi_488_;
    if _e281 {
        let _e282 = (_e55 * 8u);
        let _e286 = QB.e2_[(_e282 + 4u)];
        let _e297 = QB.e2_[(_e282 + 5u)];
        let _e300 = ((mat2x2<f32>(vec2<f32>(_e286.x, _e286.y), vec2<f32>(_e286.z, _e286.w)) * _e80) + _e297.xy);
        B2_ = vec3<f32>(_e300.x, _e300.y, (1f + _e297.z));
    } else {
        B2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e307 = l.Gf;
    let _e309 = l.Hf;
    let _e321 = PB.e2_[(_e61 + 3u)];
    g3_ = _e321.xy;
    p4_ = (_e80 + bitcast<vec2<f32>>(_e321.zw));
    unnamed.gl_Position = vec4<f32>(((_e80.x * _e307) - 1f), ((_e80.y * _e309) - sign(_e309)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e16 = h1_;
    let _e17 = C0_;
    let _e18 = W1_;
    let _e19 = g2_;
    let _e20 = M0_;
    let _e21 = V1_;
    let _e22 = B2_;
    let _e23 = g3_;
    let _e24 = p4_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23, _e24, _e25);
}
