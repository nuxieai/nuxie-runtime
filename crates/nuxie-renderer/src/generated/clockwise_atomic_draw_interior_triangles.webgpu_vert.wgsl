struct lg {
    d2_: array<vec4<u32>>,
}

struct Re {
    d2_: array<vec2<u32>>,
}

struct BC {
    jc: f32,
    sd: f32,
    of_: f32,
    pf: f32,
    p6_: u32,
    Pg: u32,
    Ze: u32,
    af: u32,
    U7_: vec4<i32>,
    Lg: vec2<f32>,
    td: vec2<f32>,
    c2_: u32,
    Qg: f32,
    d6_: u32,
    R2_: f32,
    ud: f32,
    Ue: u32,
    A3_: f32,
    B3_: f32,
    vd: f32,
    Ig: u32,
}

struct Se {
    d2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct mg {
    d2_: array<vec4<u32>>,
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

@id(0) override jh: bool = true;
@id(2) override lh: bool = true;
@id(1) override kh: bool = true;
@id(8) override rh: bool = true;

@group(0) @binding(2)
var<storage> PB: lg;
var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> i1_: f32;
@group(0) @binding(3)
var<storage> AD: Re;
var<private> B0_: f32;
@group(0) @binding(0)
var<uniform> m: BC;
var<private> V1_: vec2<f32>;
var<private> f2_: f32;
@group(0) @binding(4)
var<storage> QB: Se;
var<private> M0_: vec4<f32>;
var<private> f1_: vec4<f32>;
var<private> A2_: vec3<f32>;
var<private> f3_: vec2<u32>;
var<private> o4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var KC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ED: mg;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_816_: f32;
    var phi_817_: u32;
    var phi_818_: f32;
    var phi_819_: f32;
    var phi_703_: bool;
    var phi_820_: vec4<f32>;
    var phi_821_: f32;
    var phi_458_: bool;

    let _e51 = KB_1;
    let _e54 = (bitcast<u32>(_e51.z) & 65535u);
    let _e60 = (_e54 * 4u);
    let _e63 = PB.d2_[_e60];
    let _e64 = bitcast<vec4<f32>>(_e63);
    let _e75 = PB.d2_[(_e60 + 1u)];
    let _e79 = ((mat2x2<f32>(vec2<f32>(_e64.x, _e64.y), vec2<f32>(_e64.z, _e64.w)) * _e51.xy) + bitcast<vec2<f32>>(_e75.xy));
    i1_ = f32((bitcast<i32>(_e51.z) >> bitcast<u32>(16i)));
    let _e82 = AD.d2_[_e54];
    let _e84 = m.d6_;
    if (_e54 == 0u) {
        phi_816_ = 0f;
    } else {
        phi_816_ = unpack2x16float(((_e54 + 1023u) * _e84)).x;
    }
    let _e91 = phi_816_;
    B0_ = _e91;
    if ((_e82.x & 512u) != 0u) {
        let _e95 = B0_;
        B0_ = -(_e95);
    }
    let _e97 = (_e82.x & 15u);
    if jh {
        let _e98 = (_e97 == 0u);
        if _e98 {
            phi_817_ = _e82.y;
        } else {
            phi_817_ = _e82.x;
        }
        let _e101 = phi_817_;
        let _e103 = (_e101 >> bitcast<u32>(16i));
        if (_e103 == 0u) {
            phi_818_ = 0f;
        } else {
            phi_818_ = unpack2x16float(((_e103 + 1023u) * _e84)).x;
        }
        let _e110 = phi_818_;
        phi_819_ = _e110;
        if _e98 {
            phi_819_ = -(_e110);
        }
        let _e113 = phi_819_;
        V1_[0u] = _e113;
    }
    if lh {
        f2_ = f32(((_e82.x >> bitcast<u32>(4i)) & 15u));
    }
    if kh {
        let _e119 = (_e54 * 8u);
        let _e123 = QB.d2_[(_e119 + 2u)];
        let _e128 = vec2<f32>(_e123.x, _e123.y);
        let _e129 = vec2<f32>(_e123.z, _e123.w);
        let _e134 = QB.d2_[(_e119 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e139 = (abs(_e128) + abs(_e129));
                let _e141 = (_e139.x != 0f);
                phi_703_ = _e141;
                if _e141 {
                    phi_703_ = (_e139.y != 0f);
                }
                let _e145 = phi_703_;
                if _e145 {
                    let _e149 = ((mat2x2<f32>(_e128, _e129) * _e79) + _e134.xy);
                    let _e150 = -(_e149);
                    let _e156 = (vec2<f32>(1f, 1f) / _e139).xyxy;
                    phi_820_ = (((vec4<f32>(_e149.x, _e149.y, _e150.x, _e150.y) * _e156) + _e156) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_820_ = _e134.xyxy;
                    break;
                }
            }
        }
        let _e161 = phi_820_;
        M0_ = _e161;
    }
    if (_e97 == 1u) {
        f1_ = unpack4x8unorm(_e82.y);
    } else {
        if (jh && (_e97 == 0u)) {
            let _e204 = (_e82.x >> bitcast<u32>(16i));
            if (_e204 == 0u) {
                phi_821_ = 0f;
            } else {
                phi_821_ = unpack2x16float(((_e204 + 1023u) * _e84)).x;
            }
            let _e211 = phi_821_;
            V1_[1u] = _e211;
        } else {
            let _e165 = (_e54 * 8u);
            let _e168 = QB.d2_[_e165];
            let _e179 = QB.d2_[(_e165 + 1u)];
            let _e182 = ((mat2x2<f32>(vec2<f32>(_e168.x, _e168.y), vec2<f32>(_e168.z, _e168.w)) * _e79) + _e179.xy);
            f1_[3u] = -(bitcast<f32>(_e82.y));
            if (_e179.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e179.w;
            }
            if (_e97 == 2u) {
                f1_[1u] = 0f;
                f1_[0u] = _e182.x;
            } else {
                let _e194 = f1_[2u];
                f1_[2u] = -(_e194);
                f1_[0u] = _e182.x;
                f1_[1u] = _e182.y;
            }
        }
    }
    phi_458_ = rh;
    if rh {
        phi_458_ = ((_e82.x & 2048u) != 0u);
    }
    let _e218 = phi_458_;
    if _e218 {
        let _e219 = (_e54 * 8u);
        let _e223 = QB.d2_[(_e219 + 4u)];
        let _e234 = QB.d2_[(_e219 + 5u)];
        let _e237 = ((mat2x2<f32>(vec2<f32>(_e223.x, _e223.y), vec2<f32>(_e223.z, _e223.w)) * _e79) + _e234.xy);
        A2_ = vec3<f32>(_e237.x, _e237.y, (1f + _e234.z));
    } else {
        A2_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e244 = m.of_;
    let _e246 = m.pf;
    let _e258 = PB.d2_[(_e60 + 3u)];
    f3_ = _e258.xy;
    o4_ = (_e79 + bitcast<vec2<f32>>(_e258.zw));
    unnamed.gl_Position = vec4<f32>(((_e79.x * _e244) - 1f), ((_e79.y * _e246) - sign(_e246)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e16 = i1_;
    let _e17 = B0_;
    let _e18 = V1_;
    let _e19 = f2_;
    let _e20 = M0_;
    let _e21 = f1_;
    let _e22 = A2_;
    let _e23 = f3_;
    let _e24 = o4_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23, _e24, _e25);
}
