struct hg {
    c2_: array<vec4<u32>>,
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

struct Me {
    c2_: array<vec2<u32>>,
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
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat, either) member_1: f32,
    @location(6) @interpolate(flat, either) member_2: f32,
    @location(0) member_3: vec4<f32>,
    @location(9) member_4: vec3<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override eh: bool = true;
@id(2) override gh: bool = true;
@id(8) override mh: bool = true;

@group(0) @binding(2)
var<storage> QB: hg;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> D2_: vec2<f32>;
@group(0) @binding(3)
var<storage> BD: Me;
var<private> K3_: f32;
var<private> e2_: f32;
@group(0) @binding(4)
var<storage> RB: Ne;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
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
    var phi_632_: u32;
    var phi_633_: f32;
    var phi_634_: f32;
    var phi_635_: vec4<f32>;
    var phi_388_: bool;

    let _e43 = LB_1;
    let _e46 = (bitcast<u32>(_e43.z) & 65535u);
    let _e51 = QB.c2_[((_e46 * 4u) + 2u)];
    let _e53 = _e43.xy;
    let _e55 = bitcast<vec3<f32>>(_e51.yzw);
    let _e61 = m.Hg;
    D2_ = (((_e53 * _e55.x) + _e55.yz) * _e61);
    let _e65 = BD.c2_[_e46];
    let _e67 = (_e65.x & 15u);
    if eh {
        let _e68 = (_e67 == 0u);
        if _e68 {
            phi_632_ = _e65.y;
        } else {
            phi_632_ = _e65.x;
        }
        let _e71 = phi_632_;
        let _e73 = (_e71 >> bitcast<u32>(16i));
        let _e75 = m.c6_;
        if (_e73 == 0u) {
            phi_633_ = 0f;
        } else {
            phi_633_ = unpack2x16float(((_e73 + 1023u) * _e75)).x;
        }
        let _e82 = phi_633_;
        phi_634_ = _e82;
        if _e68 {
            phi_634_ = -(_e82);
        }
        let _e85 = phi_634_;
        K3_ = _e85;
    }
    if gh {
        e2_ = f32(((_e65.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e67 == 1u) {
        let _e132 = unpack4x8unorm(_e65.y);
        if gh {
            phi_635_ = _e132;
        } else {
            let _e135 = (_e132.xyz * _e132.w);
            let _e141 = vec4<f32>(_e135.x, _e132.y, _e132.z, _e132.w);
            let _e147 = vec4<f32>(_e141.x, _e135.y, _e141.z, _e141.w);
            phi_635_ = vec4<f32>(_e147.x, _e147.y, _e135.z, _e147.w);
        }
        let _e155 = phi_635_;
        f1_ = _e155;
    } else {
        let _e91 = (_e46 * 8u);
        let _e94 = RB.c2_[_e91];
        let _e105 = RB.c2_[(_e91 + 1u)];
        let _e108 = ((mat2x2<f32>(vec2<f32>(_e94.x, _e94.y), vec2<f32>(_e94.z, _e94.w)) * _e53) + _e105.xy);
        let _e109 = (_e67 == 2u);
        if (_e109 || (_e67 == 3u)) {
            f1_[3u] = -(bitcast<f32>(_e65.y));
            if (_e105.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e105.w;
            }
            if _e109 {
                f1_[1u] = 0f;
                f1_[0u] = _e108.x;
            } else {
                let _e122 = f1_[2u];
                f1_[2u] = -(_e122);
                f1_[0u] = _e108.x;
                f1_[1u] = _e108.y;
            }
        }
    }
    phi_388_ = mh;
    if mh {
        phi_388_ = ((_e65.x & 2048u) != 0u);
    }
    let _e159 = phi_388_;
    if _e159 {
        let _e160 = (_e46 * 8u);
        let _e164 = RB.c2_[(_e160 + 4u)];
        let _e175 = RB.c2_[(_e160 + 5u)];
        let _e178 = ((mat2x2<f32>(vec2<f32>(_e164.x, _e164.y), vec2<f32>(_e164.z, _e164.w)) * _e53) + _e175.xy);
        A2_ = vec3<f32>(_e178.x, _e178.y, (1f + _e175.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e185 = m.jf;
    let _e187 = m.kf;
    let _e195 = vec4<f32>(((_e43.x * _e185) - 1f), ((_e43.y * _e187) - sign(_e187)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e195.x, _e195.y, (1f - (f32(_e51.x) * 0.000061035156f)), _e195.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e12 = D2_;
    let _e13 = K3_;
    let _e14 = e2_;
    let _e15 = f1_;
    let _e16 = A2_;
    let _e17 = unnamed.gl_Position;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
