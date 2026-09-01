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

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
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
var QB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> m: DC;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> D2_: vec2<f32>;
@group(0) @binding(3)
var BD: texture_2d<u32>;
var<private> K3_: f32;
var<private> e2_: f32;
@group(0) @binding(4)
var RB: texture_2d<f32>;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var MC: texture_2d<u32>;
@group(0) @binding(9)
var YC: texture_2d<f32>;
@group(0) @binding(5)
var FD: texture_2d<u32>;
@group(3) @binding(9)
var aa: sampler;

fn main_1() {
    var phi_690_: u32;
    var phi_691_: f32;
    var phi_692_: f32;
    var phi_693_: vec4<f32>;
    var phi_429_: bool;

    let _e45 = LB_1;
    let _e47 = bitcast<u32>(_e45.z);
    let _e48 = (_e47 & 65535u);
    let _e50 = ((_e48 * 4u) + 2u);
    let _e57 = textureLoad(QB, vec2<i32>(bitcast<i32>((_e50 & 255u)), bitcast<i32>((_e50 >> bitcast<u32>(8i)))), 0i);
    let _e59 = _e45.xy;
    let _e61 = bitcast<vec3<f32>>(_e57.yzw);
    let _e67 = m.Hg;
    D2_ = (((_e59 * _e61.x) + _e61.yz) * _e67);
    let _e75 = textureLoad(BD, vec2<i32>(bitcast<i32>((_e47 & 255u)), bitcast<i32>((_e48 >> bitcast<u32>(8i)))), 0i);
    let _e77 = (_e75.x & 15u);
    if eh {
        let _e78 = (_e77 == 0u);
        if _e78 {
            phi_690_ = _e75.y;
        } else {
            phi_690_ = _e75.x;
        }
        let _e81 = phi_690_;
        let _e83 = (_e81 >> bitcast<u32>(16i));
        let _e85 = m.c6_;
        if (_e83 == 0u) {
            phi_691_ = 0f;
        } else {
            phi_691_ = unpack2x16float(((_e83 + 1023u) * _e85)).x;
        }
        let _e92 = phi_691_;
        phi_692_ = _e92;
        if _e78 {
            phi_692_ = -(_e92);
        }
        let _e95 = phi_692_;
        K3_ = _e95;
    }
    if gh {
        e2_ = f32(((_e75.x >> bitcast<u32>(4i)) & 15u));
    }
    if (_e77 == 1u) {
        let _e150 = unpack4x8unorm(_e75.y);
        if gh {
            phi_693_ = _e150;
        } else {
            let _e153 = (_e150.xyz * _e150.w);
            let _e159 = vec4<f32>(_e153.x, _e150.y, _e150.z, _e150.w);
            let _e165 = vec4<f32>(_e159.x, _e153.y, _e159.z, _e159.w);
            phi_693_ = vec4<f32>(_e165.x, _e165.y, _e153.z, _e165.w);
        }
        let _e173 = phi_693_;
        f1_ = _e173;
    } else {
        let _e101 = (_e48 * 8u);
        let _e108 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e101 & 255u)), bitcast<i32>((_e101 >> bitcast<u32>(8i)))), 0i);
        let _e116 = (_e101 + 1u);
        let _e123 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e116 & 255u)), bitcast<i32>((_e116 >> bitcast<u32>(8i)))), 0i);
        let _e126 = ((mat2x2<f32>(vec2<f32>(_e108.x, _e108.y), vec2<f32>(_e108.z, _e108.w)) * _e59) + _e123.xy);
        let _e127 = (_e77 == 2u);
        if (_e127 || (_e77 == 3u)) {
            f1_[3u] = -(bitcast<f32>(_e75.y));
            if (_e123.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e123.w;
            }
            if _e127 {
                f1_[1u] = 0f;
                f1_[0u] = _e126.x;
            } else {
                let _e140 = f1_[2u];
                f1_[2u] = -(_e140);
                f1_[0u] = _e126.x;
                f1_[1u] = _e126.y;
            }
        }
    }
    phi_429_ = mh;
    if mh {
        phi_429_ = ((_e75.x & 2048u) != 0u);
    }
    let _e177 = phi_429_;
    if _e177 {
        let _e178 = (_e48 * 8u);
        let _e179 = (_e178 + 4u);
        let _e186 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e179 & 255u)), bitcast<i32>((_e179 >> bitcast<u32>(8i)))), 0i);
        let _e194 = (_e178 + 5u);
        let _e201 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e194 & 255u)), bitcast<i32>((_e194 >> bitcast<u32>(8i)))), 0i);
        let _e204 = ((mat2x2<f32>(vec2<f32>(_e186.x, _e186.y), vec2<f32>(_e186.z, _e186.w)) * _e59) + _e201.xy);
        A2_ = vec3<f32>(_e204.x, _e204.y, (1f + _e201.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e211 = m.jf;
    let _e213 = m.kf;
    let _e221 = vec4<f32>(((_e45.x * _e211) - 1f), ((_e45.y * _e213) - sign(_e213)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e221.x, _e221.y, (1f - (f32(_e57.x) * 0.000061035156f)), _e221.w);
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
