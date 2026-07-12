struct Rf {
    X1_: array<vec4<u32>>,
}

struct Be {
    X1_: array<vec2<u32>>,
}

struct NB {
    Ub: f32,
    dd: f32,
    Xe: f32,
    Ye: f32,
    q5_: u32,
    ug: u32,
    Je: u32,
    Ke: u32,
    U7_: vec4<i32>,
    rg: vec2<f32>,
    ed: vec2<f32>,
    W1_: u32,
    vg: f32,
    Y5_: u32,
    P2_: f32,
    fd: f32,
    Ee: u32,
    y3_: f32,
    z3_: f32,
    gd: f32,
    og: u32,
}

struct Ce {
    X1_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct Sf {
    X1_: array<vec4<u32>>,
}

struct VertexOutput {
    @location(1) @interpolate(flat) member: f32,
    @location(3) @interpolate(flat) member_1: f32,
    @location(4) @interpolate(flat) member_2: vec2<f32>,
    @location(6) @interpolate(flat) member_3: f32,
    @location(5) member_4: vec4<f32>,
    @location(0) member_5: vec4<f32>,
    @location(7) @interpolate(flat) member_6: vec2<u32>,
    @location(8) member_7: vec2<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

@id(0) override Mg: bool = true;
@id(2) override Og: bool = true;
@id(1) override Ng: bool = true;

@group(0) @binding(3) 
var<storage> MB: Rf;
var<private> gl_VertexIndex_1: i32;
var<private> KB_1: vec3<f32>;
var<private> j1_: f32;
@group(0) @binding(4) 
var<storage> TC: Be;
var<private> z0_: f32;
@group(0) @binding(0) 
var<uniform> k: NB;
var<private> S1_: vec2<f32>;
var<private> Z1_: f32;
@group(0) @binding(5) 
var<storage> PB: Ce;
var<private> N0_: vec4<f32>;
var<private> i1_: vec4<f32>;
var<private> e3_: vec2<u32>;
var<private> j4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(8) 
var DC: texture_2d<u32>;
@group(0) @binding(10) 
var QC: texture_2d<f32>;
@group(0) @binding(6) 
var<storage> XC: Sf;
@group(3) @binding(10) 
var T9_: sampler;

fn main_1() {
    var phi_780_: f32;
    var phi_781_: u32;
    var phi_782_: f32;
    var phi_783_: f32;
    var phi_680_: bool;
    var phi_784_: vec4<f32>;
    var phi_785_: f32;

    let _e46 = KB_1;
    let _e49 = (bitcast<u32>(_e46.z) & 65535u);
    let _e55 = (_e49 * 4u);
    let _e58 = MB.X1_[_e55];
    let _e59 = bitcast<vec4<f32>>(_e58);
    let _e67 = (_e55 + 1u);
    let _e70 = MB.X1_[_e67];
    let _e74 = ((mat2x2<f32>(vec2<f32>(_e59.x, _e59.y), vec2<f32>(_e59.z, _e59.w)) * _e46.xy) + bitcast<vec2<f32>>(_e70.xy));
    j1_ = f32((bitcast<i32>(_e46.z) >> bitcast<u32>(16i)));
    let _e77 = TC.X1_[_e49];
    let _e79 = k.Y5_;
    if (_e49 == 0u) {
        phi_780_ = 0f;
    } else {
        phi_780_ = unpack2x16float(((_e49 + 1023u) * _e79)).x;
    }
    let _e86 = phi_780_;
    z0_ = _e86;
    if ((_e77.x & 512u) != 0u) {
        let _e90 = z0_;
        z0_ = -(_e90);
    }
    let _e92 = (_e77.x & 15u);
    if Mg {
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
        S1_[0u] = _e108;
    }
    if Og {
        Z1_ = f32(((_e77.x >> bitcast<u32>(4i)) & 15u));
    }
    if Ng {
        let _e117 = PB.X1_[(_e55 + 2u)];
        let _e122 = vec2<f32>(_e117.x, _e117.y);
        let _e123 = vec2<f32>(_e117.z, _e117.w);
        let _e128 = PB.X1_[(_e55 + 3u)];
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
        N0_ = _e155;
    }
    if (_e92 == 1u) {
        i1_ = unpack4x8unorm(_e77.y);
    } else {
        if (Mg && (_e92 == 0u)) {
            let _e205 = (_e77.x >> bitcast<u32>(16i));
            if (_e205 == 0u) {
                phi_785_ = 0f;
            } else {
                phi_785_ = unpack2x16float(((_e205 + 1023u) * _e79)).x;
            }
            let _e212 = phi_785_;
            S1_[1u] = _e212;
        } else {
            let _e161 = PB.X1_[_e55];
            let _e171 = PB.X1_[_e67];
            let _e174 = ((mat2x2<f32>(vec2<f32>(_e161.x, _e161.y), vec2<f32>(_e161.z, _e161.w)) * _e74) + _e171.xy);
            let _e175 = (_e92 == 2u);
            if (_e175 || (_e92 == 3u)) {
                i1_[3u] = -(bitcast<f32>(_e77.y));
                if (_e171.z > 0.9f) {
                    i1_[2u] = 2f;
                } else {
                    i1_[2u] = _e171.w;
                }
                if _e175 {
                    i1_[1u] = 0f;
                    i1_[0u] = _e174.x;
                } else {
                    let _e195 = i1_[2u];
                    i1_[2u] = -(_e195);
                    i1_[0u] = _e174.x;
                    i1_[1u] = _e174.y;
                }
            } else {
                i1_ = vec4<f32>(_e174.x, _e174.y, bitcast<f32>(_e77.y), (-2f - _e171.z));
            }
        }
    }
    let _e217 = k.Xe;
    let _e219 = k.Ye;
    let _e231 = MB.X1_[(_e55 + 3u)];
    e3_ = _e231.xy;
    j4_ = (_e74 + bitcast<vec2<f32>>(_e231.zw));
    unnamed.gl_Position = vec4<f32>(((_e74.x * _e217) - 1f), ((_e74.y * _e219) - sign(_e219)), 0f, 1f);
    return;
}

@vertex 
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) KB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    KB_1 = KB;
    main_1();
    let _e15 = j1_;
    let _e16 = z0_;
    let _e17 = S1_;
    let _e18 = Z1_;
    let _e19 = N0_;
    let _e20 = i1_;
    let _e21 = e3_;
    let _e22 = j4_;
    let _e23 = unnamed.gl_Position;
    return VertexOutput(_e15, _e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23);
}
