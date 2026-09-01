struct hg {
    c2_: array<vec4<u32>>,
}

struct Me {
    c2_: array<vec2<u32>>,
}

struct DC {
    gc: f32,
    qd: f32,
    jf: f32,
    kf: f32,
    o6_: u32,
    Lg: u32,
    Ue: u32,
    Ve: u32,
    T7_: vec4<i32>,
    Hg: vec2<f32>,
    rd: vec2<f32>,
    a2_: u32,
    Mg: f32,
    c6_: u32,
    R2_: f32,
    sd: f32,
    Pe: u32,
    B3_: f32,
    C3_: f32,
    td: f32,
    Eg: u32,
}

struct Ne {
    c2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct ig {
    c2_: array<vec4<u32>>,
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

@id(0) override eh: bool = true;
@id(2) override gh: bool = true;
@id(1) override fh: bool = true;
@id(8) override mh: bool = true;

@group(0) @binding(2)
var<storage> QB: hg;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> i1_: f32;
@group(0) @binding(3)
var<storage> BD: Me;
var<private> B0_: f32;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> U1_: vec2<f32>;
var<private> e2_: f32;
@group(0) @binding(4)
var<storage> RB: Ne;
var<private> M0_: vec4<f32>;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
var<private> f3_: vec2<u32>;
var<private> n4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> FD: ig;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_823_: f32;
    var phi_824_: u32;
    var phi_825_: f32;
    var phi_826_: f32;
    var phi_710_: bool;
    var phi_827_: vec4<f32>;
    var phi_828_: f32;
    var phi_465_: bool;

    let _e51 = LB_1;
    let _e54 = (bitcast<u32>(_e51.z) & 65535u);
    let _e60 = (_e54 * 4u);
    let _e63 = QB.c2_[_e60];
    let _e64 = bitcast<vec4<f32>>(_e63);
    let _e75 = QB.c2_[(_e60 + 1u)];
    let _e79 = ((mat2x2<f32>(vec2<f32>(_e64.x, _e64.y), vec2<f32>(_e64.z, _e64.w)) * _e51.xy) + bitcast<vec2<f32>>(_e75.xy));
    i1_ = f32((bitcast<i32>(_e51.z) >> bitcast<u32>(16i)));
    let _e82 = BD.c2_[_e54];
    let _e84 = m.c6_;
    if (_e54 == 0u) {
        phi_823_ = 0f;
    } else {
        phi_823_ = unpack2x16float(((_e54 + 1023u) * _e84)).x;
    }
    let _e91 = phi_823_;
    B0_ = _e91;
    if ((_e82.x & 512u) != 0u) {
        let _e95 = B0_;
        B0_ = -(_e95);
    }
    let _e97 = (_e82.x & 15u);
    if eh {
        let _e98 = (_e97 == 0u);
        if _e98 {
            phi_824_ = _e82.y;
        } else {
            phi_824_ = _e82.x;
        }
        let _e101 = phi_824_;
        let _e103 = (_e101 >> bitcast<u32>(16i));
        if (_e103 == 0u) {
            phi_825_ = 0f;
        } else {
            phi_825_ = unpack2x16float(((_e103 + 1023u) * _e84)).x;
        }
        let _e110 = phi_825_;
        phi_826_ = _e110;
        if _e98 {
            phi_826_ = -(_e110);
        }
        let _e113 = phi_826_;
        U1_[0u] = _e113;
    }
    if gh {
        e2_ = f32(((_e82.x >> bitcast<u32>(4i)) & 15u));
    }
    if fh {
        let _e119 = (_e54 * 8u);
        let _e123 = RB.c2_[(_e119 + 2u)];
        let _e128 = vec2<f32>(_e123.x, _e123.y);
        let _e129 = vec2<f32>(_e123.z, _e123.w);
        let _e134 = RB.c2_[(_e119 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e139 = (abs(_e128) + abs(_e129));
                let _e141 = (_e139.x != 0f);
                phi_710_ = _e141;
                if _e141 {
                    phi_710_ = (_e139.y != 0f);
                }
                let _e145 = phi_710_;
                if _e145 {
                    let _e149 = ((mat2x2<f32>(_e128, _e129) * _e79) + _e134.xy);
                    let _e150 = -(_e149);
                    let _e156 = (vec2<f32>(1f, 1f) / _e139).xyxy;
                    phi_827_ = (((vec4<f32>(_e149.x, _e149.y, _e150.x, _e150.y) * _e156) + _e156) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_827_ = _e134.xyxy;
                    break;
                }
            }
        }
        let _e161 = phi_827_;
        M0_ = _e161;
    }
    if (_e97 == 1u) {
        f1_ = unpack4x8unorm(_e82.y);
    } else {
        if (eh && (_e97 == 0u)) {
            let _e206 = (_e82.x >> bitcast<u32>(16i));
            if (_e206 == 0u) {
                phi_828_ = 0f;
            } else {
                phi_828_ = unpack2x16float(((_e206 + 1023u) * _e84)).x;
            }
            let _e213 = phi_828_;
            U1_[1u] = _e213;
        } else {
            let _e165 = (_e54 * 8u);
            let _e168 = RB.c2_[_e165];
            let _e179 = RB.c2_[(_e165 + 1u)];
            let _e182 = ((mat2x2<f32>(vec2<f32>(_e168.x, _e168.y), vec2<f32>(_e168.z, _e168.w)) * _e79) + _e179.xy);
            let _e183 = (_e97 == 2u);
            if (_e183 || (_e97 == 3u)) {
                f1_[3u] = -(bitcast<f32>(_e82.y));
                if (_e179.z > 0.9f) {
                    f1_[2u] = 2f;
                } else {
                    f1_[2u] = _e179.w;
                }
                if _e183 {
                    f1_[1u] = 0f;
                    f1_[0u] = _e182.x;
                } else {
                    let _e196 = f1_[2u];
                    f1_[2u] = -(_e196);
                    f1_[0u] = _e182.x;
                    f1_[1u] = _e182.y;
                }
            }
        }
    }
    phi_465_ = mh;
    if mh {
        phi_465_ = ((_e82.x & 2048u) != 0u);
    }
    let _e220 = phi_465_;
    if _e220 {
        let _e221 = (_e54 * 8u);
        let _e225 = RB.c2_[(_e221 + 4u)];
        let _e236 = RB.c2_[(_e221 + 5u)];
        let _e239 = ((mat2x2<f32>(vec2<f32>(_e225.x, _e225.y), vec2<f32>(_e225.z, _e225.w)) * _e79) + _e236.xy);
        A2_ = vec3<f32>(_e239.x, _e239.y, (1f + _e236.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e246 = m.jf;
    let _e248 = m.kf;
    let _e260 = QB.c2_[(_e60 + 3u)];
    f3_ = _e260.xy;
    n4_ = (_e79 + bitcast<vec2<f32>>(_e260.zw));
    unnamed.gl_Position = vec4<f32>(((_e79.x * _e246) - 1f), ((_e79.y * _e248) - sign(_e248)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e16 = i1_;
    let _e17 = B0_;
    let _e18 = U1_;
    let _e19 = e2_;
    let _e20 = M0_;
    let _e21 = f1_;
    let _e22 = A2_;
    let _e23 = f3_;
    let _e24 = n4_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23, _e24, _e25);
}
