struct Xf {
    c2_: array<vec4<u32>>,
}

struct Fe {
    c2_: array<vec2<u32>>,
}

struct CC {
    bc: f32,
    kd: f32,
    bf: f32,
    cf: f32,
    m6_: u32,
    Bg: u32,
    Ne: u32,
    Oe: u32,
    Q7_: vec4<i32>,
    xg: vec2<f32>,
    ld: vec2<f32>,
    a2_: u32,
    Cg: f32,
    Z5_: u32,
    N2_: f32,
    md: f32,
    Ie: u32,
    y3_: f32,
    z3_: f32,
    nd: f32,
    ug: u32,
}

struct Ge {
    c2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Yf {
    c2_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) @interpolate(flat, either) member: f32,
    @location(3) @interpolate(flat, either) member_1: f32,
    @location(4) @interpolate(flat, either) member_2: vec2<f32>,
    @location(6) @interpolate(flat, either) member_3: f32,
    @location(5) member_4: vec4<f32>,
    @location(0) member_5: vec4<f32>,
    @location(7) @interpolate(flat, either) member_6: vec2<u32>,
    @location(8) member_7: vec2<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Ug: bool = true;
@id(2) override Wg: bool = true;
@id(1) override Vg: bool = true;

@group(0) @binding(2)
var<storage> PB: Xf;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> i1_: f32;
@group(0) @binding(3)
var<storage> AD: Fe;
var<private> A0_: f32;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> U1_: vec2<f32>;
var<private> e2_: f32;
@group(0) @binding(4)
var<storage> RB: Ge;
var<private> L0_: vec4<f32>;
var<private> f1_: vec4<f32>;
var<private> a3_: vec2<u32>;
var<private> l4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var<storage> ED: Yf;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_780_: f32;
    var phi_781_: u32;
    var phi_782_: f32;
    var phi_783_: f32;
    var phi_680_: bool;
    var phi_784_: vec4<f32>;
    var phi_785_: f32;

    let _e46 = LB_1;
    let _e49 = (bitcast<u32>(_e46.z) & 65535u);
    let _e55 = (_e49 * 4u);
    let _e58 = PB.c2_[_e55];
    let _e59 = bitcast<vec4<f32>>(_e58);
    let _e67 = (_e55 + 1u);
    let _e70 = PB.c2_[_e67];
    let _e74 = ((mat2x2<f32>(vec2<f32>(_e59.x, _e59.y), vec2<f32>(_e59.z, _e59.w)) * _e46.xy) + bitcast<vec2<f32>>(_e70.xy));
    i1_ = f32((bitcast<i32>(_e46.z) >> bitcast<u32>(16i)));
    let _e77 = AD.c2_[_e49];
    let _e79 = m.Z5_;
    if (_e49 == 0u) {
        phi_780_ = 0f;
    } else {
        phi_780_ = unpack2x16float(((_e49 + 1023u) * _e79)).x;
    }
    let _e86 = phi_780_;
    A0_ = _e86;
    if ((_e77.x & 512u) != 0u) {
        let _e90 = A0_;
        A0_ = -(_e90);
    }
    let _e92 = (_e77.x & 15u);
    if Ug {
        let _e93 = (_e92 == 0u);
        if _e93 {
            phi_781_ = _e77.y;
        } else {
            phi_781_ = _e77.x;
        }
        let _e96 = phi_781_;
        let _e98 = (_e96 >> bitcast<u32>(16i));
        if (_e98 == 0u) {
            phi_782_ = 0f;
        } else {
            phi_782_ = unpack2x16float(((_e98 + 1023u) * _e79)).x;
        }
        let _e105 = phi_782_;
        phi_783_ = _e105;
        if _e93 {
            phi_783_ = -(_e105);
        }
        let _e108 = phi_783_;
        U1_[0u] = _e108;
    }
    if Wg {
        e2_ = f32(((_e77.x >> bitcast<u32>(4i)) & 15u));
    }
    if Vg {
        let _e117 = RB.c2_[(_e55 + 2u)];
        let _e122 = vec2<f32>(_e117.x, _e117.y);
        let _e123 = vec2<f32>(_e117.z, _e117.w);
        let _e128 = RB.c2_[(_e55 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e133 = (abs(_e122) + abs(_e123));
                let _e135 = (_e133.x != 0f);
                phi_680_ = _e135;
                if _e135 {
                    phi_680_ = (_e133.y != 0f);
                }
                let _e139 = phi_680_;
                if _e139 {
                    let _e143 = ((mat2x2<f32>(_e122, _e123) * _e74) + _e128.xy);
                    let _e144 = -(_e143);
                    let _e150 = (vec2<f32>(1f, 1f) / _e133).xyxy;
                    phi_784_ = (((vec4<f32>(_e143.x, _e143.y, _e144.x, _e144.y) * _e150) + _e150) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_784_ = _e128.xyxy;
                    break;
                }
            }
        }
        let _e155 = phi_784_;
        L0_ = _e155;
    }
    if (_e92 == 1u) {
        f1_ = unpack4x8unorm(_e77.y);
    } else {
        if (Ug && (_e92 == 0u)) {
            let _e205 = (_e77.x >> bitcast<u32>(16i));
            if (_e205 == 0u) {
                phi_785_ = 0f;
            } else {
                phi_785_ = unpack2x16float(((_e205 + 1023u) * _e79)).x;
            }
            let _e212 = phi_785_;
            U1_[1u] = _e212;
        } else {
            let _e161 = RB.c2_[_e55];
            let _e171 = RB.c2_[_e67];
            let _e174 = ((mat2x2<f32>(vec2<f32>(_e161.x, _e161.y), vec2<f32>(_e161.z, _e161.w)) * _e74) + _e171.xy);
            let _e175 = (_e92 == 2u);
            if (_e175 || (_e92 == 3u)) {
                f1_[3u] = -(bitcast<f32>(_e77.y));
                if (_e171.z > 0.9f) {
                    f1_[2u] = 2f;
                } else {
                    f1_[2u] = _e171.w;
                }
                if _e175 {
                    f1_[1u] = 0f;
                    f1_[0u] = _e174.x;
                } else {
                    let _e195 = f1_[2u];
                    f1_[2u] = -(_e195);
                    f1_[0u] = _e174.x;
                    f1_[1u] = _e174.y;
                }
            } else {
                f1_ = vec4<f32>(_e174.x, _e174.y, bitcast<f32>(_e77.y), (-2f - _e171.z));
            }
        }
    }
    let _e217 = m.bf;
    let _e219 = m.cf;
    let _e231 = PB.c2_[(_e55 + 3u)];
    a3_ = _e231.xy;
    l4_ = (_e74 + bitcast<vec2<f32>>(_e231.zw));
    unnamed.gl_Position = vec4<f32>(((_e74.x * _e217) - 1f), ((_e74.y * _e219) - sign(_e219)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e15 = i1_;
    let _e16 = A0_;
    let _e17 = U1_;
    let _e18 = e2_;
    let _e19 = L0_;
    let _e20 = f1_;
    let _e21 = a3_;
    let _e22 = l4_;
    let _e23 = unnamed.gl_Position;
    return VertexOutput(_e15, _e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23);
}
