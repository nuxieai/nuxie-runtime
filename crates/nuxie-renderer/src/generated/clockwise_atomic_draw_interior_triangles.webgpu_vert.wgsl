struct nd {
    z2_: array<vec4<u32>>,
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

struct Cg {
    z2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct xi {
    z2_: array<vec4<u32>>,
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

@id(0) override uj: bool = true;
@id(2) override wj: bool = true;
@id(1) override vj: bool = true;
@id(4) override yj: bool = true;

@group(0) @binding(2)
var<storage> JB: nd;
var<private> gl_VertexIndex_1: i32;
var<private> MB_1: vec3<f32>;
var<private> r1_: f32;
var<private> J0_: f32;
@group(0) @binding(0)
var<uniform> j: RB;
var<private> l2_: vec2<f32>;
var<private> S0_: f32;
@group(0) @binding(3)
var<storage> KB: Cg;
var<private> Y0_: vec4<f32>;
var<private> N0_: vec4<f32>;
var<private> O0_: vec3<f32>;
var<private> D3_: vec2<u32>;
var<private> V4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(6)
var WB: texture_2d<u32>;
@group(0) @binding(8)
var ZC: texture_2d<f32>;
@group(0) @binding(4)
var<storage> CD: xi;
@group(3) @binding(8)
var ib: sampler;

fn main_1() {
    var phi_932_: f32;
    var phi_933_: u32;
    var phi_934_: f32;
    var phi_935_: f32;
    var phi_750_: bool;
    var phi_936_: vec4<f32>;
    var phi_948_: f32;
    var phi_950_: f32;
    var phi_482_: bool;
    var phi_951_: f32;

    let _e56 = MB_1;
    let _e59 = (bitcast<u32>(_e56.z) & 65535u);
    let _e60 = (_e59 * 4u);
    let _e64 = JB.z2_[(_e60 + 1u)];
    let _e72 = JB.z2_[_e60];
    let _e73 = bitcast<vec4<f32>>(_e72);
    let _e84 = ((mat2x2<f32>(vec2<f32>(_e73.x, _e73.y), vec2<f32>(_e73.z, _e73.w)) * _e56.xy) + bitcast<vec2<f32>>(_e64.xy));
    r1_ = f32((bitcast<i32>(_e56.z) >> bitcast<u32>(16i)));
    let _e86 = j.z6_;
    if (_e59 == 0u) {
        phi_932_ = 0f;
    } else {
        phi_932_ = unpack2x16float(((_e59 + 1023u) * _e86)).x;
    }
    let _e93 = phi_932_;
    J0_ = _e93;
    if ((_e64.z & 512u) != 0u) {
        let _e97 = J0_;
        J0_ = -(_e97);
    }
    let _e99 = (_e64.z & 15u);
    if uj {
        let _e100 = (_e99 == 0u);
        if _e100 {
            phi_933_ = _e64.w;
        } else {
            phi_933_ = _e64.z;
        }
        let _e103 = phi_933_;
        let _e105 = (_e103 >> bitcast<u32>(16i));
        if (_e105 == 0u) {
            phi_934_ = 0f;
        } else {
            phi_934_ = unpack2x16float(((_e105 + 1023u) * _e86)).x;
        }
        let _e112 = phi_934_;
        phi_935_ = _e112;
        if _e100 {
            phi_935_ = -(_e112);
        }
        let _e115 = phi_935_;
        l2_[0u] = _e115;
    }
    if wj {
        S0_ = f32(((_e64.z >> bitcast<u32>(4i)) & 15u));
    }
    if vj {
        let _e121 = (_e59 * 8u);
        let _e125 = KB.z2_[(_e121 + 2u)];
        let _e130 = vec2<f32>(_e125.x, _e125.y);
        let _e131 = vec2<f32>(_e125.z, _e125.w);
        let _e136 = KB.z2_[(_e121 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e141 = (abs(_e130) + abs(_e131));
                let _e143 = (_e141.x != 0f);
                phi_750_ = _e143;
                if _e143 {
                    phi_750_ = (_e141.y != 0f);
                }
                let _e147 = phi_750_;
                if _e147 {
                    let _e151 = ((mat2x2<f32>(_e130, _e131) * _e84) + _e136.xy);
                    let _e152 = -(_e151);
                    let _e158 = (vec2<f32>(1f, 1f) / _e141).xyxy;
                    phi_936_ = (((vec4<f32>(_e151.x, _e151.y, _e152.x, _e152.y) * _e158) + _e158) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_936_ = _e136.xyxy;
                    break;
                }
            }
        }
        let _e163 = phi_936_;
        Y0_ = _e163;
    }
    if (_e99 == 1u) {
        N0_ = unpack4x8unorm(_e64.w);
    } else {
        if (uj && (_e99 == 0u)) {
            let _e206 = (_e64.z >> bitcast<u32>(16i));
            if (_e206 == 0u) {
                phi_950_ = 0f;
            } else {
                phi_950_ = unpack2x16float(((_e206 + 1023u) * _e86)).x;
            }
            let _e213 = phi_950_;
            l2_[1u] = _e213;
        } else {
            let _e167 = (_e59 * 8u);
            let _e170 = KB.z2_[_e167];
            let _e181 = KB.z2_[(_e167 + 1u)];
            let _e186 = ((mat2x2<f32>(vec2<f32>(_e170.x, _e170.y), vec2<f32>(_e170.z, _e170.w)) * _e84) + _e181.xy);
            let _e197 = ((_e181.w + (f32(_e99) * 0.125f)) + (max(_e181.z, 0f) * 0.00024414063f));
            if (_e181.z < 0f) {
                phi_948_ = -(_e197);
            } else {
                phi_948_ = _e197;
            }
            let _e200 = phi_948_;
            N0_ = vec4<f32>(_e186.x, _e186.y, _e200, (-0.75f - round((bitcast<f32>(_e64.w) * 255f))));
        }
    }
    phi_482_ = yj;
    if yj {
        phi_482_ = ((_e64.z & 2048u) != 0u);
    }
    let _e220 = phi_482_;
    if _e220 {
        let _e221 = (_e59 * 8u);
        let _e225 = KB.z2_[(_e221 + 4u)];
        let _e236 = KB.z2_[(_e221 + 5u)];
        let _e239 = ((mat2x2<f32>(vec2<f32>(_e225.x, _e225.y), vec2<f32>(_e225.z, _e225.w)) * _e84) + _e236.xy);
        phi_951_ = (1f + _e236.z);
        if ((_e64.z & 4096u) != 0u) {
            phi_951_ = (-1f - f32(((_e64.z & 24576u) >> bitcast<u32>(13u))));
        }
        let _e250 = phi_951_;
        O0_ = vec3<f32>(_e239.x, _e239.y, _e250);
    } else {
        O0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e255 = j.ah;
    let _e257 = j.bh;
    let _e269 = JB.z2_[(_e60 + 3u)];
    D3_ = _e269.xy;
    V4_ = (_e84 + bitcast<vec2<f32>>(_e269.zw));
    unnamed.gl_Position = vec4<f32>(((_e84.x * _e255) - 1f), ((_e84.y * _e257) - sign(_e257)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) MB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    MB_1 = MB;
    main_1();
    let _e16 = r1_;
    let _e17 = J0_;
    let _e18 = l2_;
    let _e19 = S0_;
    let _e20 = Y0_;
    let _e21 = N0_;
    let _e22 = O0_;
    let _e23 = D3_;
    let _e24 = V4_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23, _e24, _e25);
}
