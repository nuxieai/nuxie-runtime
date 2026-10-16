struct hi {
    r2_: array<vec4<u32>>,
}

struct lg {
    r2_: array<vec2<u32>>,
}

struct VB {
    yd: f32,
    Ee: f32,
    Jg: f32,
    Kg: f32,
    S6_: u32,
    Ea: u32,
    vg: u32,
    wg: u32,
    H8_: vec4<i32>,
    Ci: vec2<f32>,
    Fe: vec2<f32>,
    q2_: u32,
    Gi: f32,
    w6_: u32,
    i3_: f32,
    Ge: f32,
    og: u32,
    G3_: f32,
    H3_: f32,
    He: f32,
    zi: u32,
    Da: u32,
    O8_: f32,
    P8_: f32,
}

struct mg {
    r2_: array<vec4<f32>>,
}

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    gl_ClipDistance: array<f32, 1>,
    gl_CullDistance: array<f32, 1>,
}

struct ii {
    r2_: array<vec4<u32>>,
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

@id(0) override aj: bool = true;
@id(2) override cj: bool = true;
@id(1) override bj: bool = true;
@id(8) override ij: bool = true;

@group(0) @binding(2)
var<storage> KB: hi;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> o1_: f32;
@group(0) @binding(3)
var<storage> XC: lg;
var<private> G0_: f32;
@group(0) @binding(0)
var<uniform> j: VB;
var<private> i2_: vec2<f32>;
var<private> Q0_: f32;
@group(0) @binding(4)
var<storage> JB: mg;
var<private> V0_: vec4<f32>;
var<private> P0_: vec4<f32>;
var<private> U0_: vec3<f32>;
var<private> A3_: vec2<u32>;
var<private> L4_: vec2<f32>;
var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 1>(), array<f32, 1>());
@group(0) @binding(7)
var UB: texture_2d<u32>;
@group(0) @binding(9)
var AD: texture_2d<f32>;
@group(0) @binding(5)
var<storage> DD: ii;
@group(3) @binding(9)
var db: sampler;

fn main_1() {
    var phi_933_: f32;
    var phi_934_: u32;
    var phi_935_: f32;
    var phi_936_: f32;
    var phi_751_: bool;
    var phi_937_: vec4<f32>;
    var phi_949_: f32;
    var phi_951_: f32;
    var phi_484_: bool;
    var phi_952_: f32;

    let _e57 = LB_1;
    let _e60 = (bitcast<u32>(_e57.z) & 65535u);
    let _e66 = (_e60 * 4u);
    let _e69 = KB.r2_[_e66];
    let _e70 = bitcast<vec4<f32>>(_e69);
    let _e81 = KB.r2_[(_e66 + 1u)];
    let _e85 = ((mat2x2<f32>(vec2<f32>(_e70.x, _e70.y), vec2<f32>(_e70.z, _e70.w)) * _e57.xy) + bitcast<vec2<f32>>(_e81.xy));
    o1_ = f32((bitcast<i32>(_e57.z) >> bitcast<u32>(16i)));
    let _e88 = XC.r2_[_e60];
    let _e90 = j.w6_;
    if (_e60 == 0u) {
        phi_933_ = 0f;
    } else {
        phi_933_ = unpack2x16float(((_e60 + 1023u) * _e90)).x;
    }
    let _e97 = phi_933_;
    G0_ = _e97;
    if ((_e88.x & 512u) != 0u) {
        let _e101 = G0_;
        G0_ = -(_e101);
    }
    let _e103 = (_e88.x & 15u);
    if aj {
        let _e104 = (_e103 == 0u);
        if _e104 {
            phi_934_ = _e88.y;
        } else {
            phi_934_ = _e88.x;
        }
        let _e107 = phi_934_;
        let _e109 = (_e107 >> bitcast<u32>(16i));
        if (_e109 == 0u) {
            phi_935_ = 0f;
        } else {
            phi_935_ = unpack2x16float(((_e109 + 1023u) * _e90)).x;
        }
        let _e116 = phi_935_;
        phi_936_ = _e116;
        if _e104 {
            phi_936_ = -(_e116);
        }
        let _e119 = phi_936_;
        i2_[0u] = _e119;
    }
    if cj {
        Q0_ = f32(((_e88.x >> bitcast<u32>(4i)) & 15u));
    }
    if bj {
        let _e125 = (_e60 * 8u);
        let _e129 = JB.r2_[(_e125 + 2u)];
        let _e134 = vec2<f32>(_e129.x, _e129.y);
        let _e135 = vec2<f32>(_e129.z, _e129.w);
        let _e140 = JB.r2_[(_e125 + 3u)];
        switch bitcast<i32>(0u) {
            default: {
                let _e145 = (abs(_e134) + abs(_e135));
                let _e147 = (_e145.x != 0f);
                phi_751_ = _e147;
                if _e147 {
                    phi_751_ = (_e145.y != 0f);
                }
                let _e151 = phi_751_;
                if _e151 {
                    let _e155 = ((mat2x2<f32>(_e134, _e135) * _e85) + _e140.xy);
                    let _e156 = -(_e155);
                    let _e162 = (vec2<f32>(1f, 1f) / _e145).xyxy;
                    phi_937_ = (((vec4<f32>(_e155.x, _e155.y, _e156.x, _e156.y) * _e162) + _e162) + vec4<f32>(0.5f, 0.5f, 0.5f, 0.5f));
                    break;
                } else {
                    phi_937_ = _e140.xyxy;
                    break;
                }
            }
        }
        let _e167 = phi_937_;
        V0_ = _e167;
    }
    if (_e103 == 1u) {
        P0_ = unpack4x8unorm(_e88.y);
    } else {
        if (aj && (_e103 == 0u)) {
            let _e210 = (_e88.x >> bitcast<u32>(16i));
            if (_e210 == 0u) {
                phi_951_ = 0f;
            } else {
                phi_951_ = unpack2x16float(((_e210 + 1023u) * _e90)).x;
            }
            let _e217 = phi_951_;
            i2_[1u] = _e217;
        } else {
            let _e171 = (_e60 * 8u);
            let _e174 = JB.r2_[_e171];
            let _e185 = JB.r2_[(_e171 + 1u)];
            let _e190 = ((mat2x2<f32>(vec2<f32>(_e174.x, _e174.y), vec2<f32>(_e174.z, _e174.w)) * _e85) + _e185.xy);
            let _e201 = ((_e185.w + (f32(_e103) * 0.125f)) + (max(_e185.z, 0f) * 0.00024414063f));
            if (_e185.z < 0f) {
                phi_949_ = -(_e201);
            } else {
                phi_949_ = _e201;
            }
            let _e204 = phi_949_;
            P0_ = vec4<f32>(_e190.x, _e190.y, _e204, (-0.75f - round((bitcast<f32>(_e88.y) * 255f))));
        }
    }
    phi_484_ = ij;
    if ij {
        phi_484_ = ((_e88.x & 2048u) != 0u);
    }
    let _e224 = phi_484_;
    if _e224 {
        let _e225 = (_e60 * 8u);
        let _e229 = JB.r2_[(_e225 + 4u)];
        let _e240 = JB.r2_[(_e225 + 5u)];
        let _e243 = ((mat2x2<f32>(vec2<f32>(_e229.x, _e229.y), vec2<f32>(_e229.z, _e229.w)) * _e85) + _e240.xy);
        phi_952_ = (1f + _e240.z);
        if ((_e88.x & 4096u) != 0u) {
            phi_952_ = (-1f - f32(((_e88.x & 24576u) >> bitcast<u32>(13u))));
        }
        let _e254 = phi_952_;
        U0_ = vec3<f32>(_e243.x, _e243.y, _e254);
    } else {
        U0_ = vec3<f32>(0f, 0f, 0f);
    }
    let _e259 = j.Jg;
    let _e261 = j.Kg;
    let _e273 = KB.r2_[(_e66 + 3u)];
    A3_ = _e273.xy;
    L4_ = (_e85 + bitcast<vec2<f32>>(_e273.zw));
    unnamed.gl_Position = vec4<f32>(((_e85.x * _e259) - 1f), ((_e85.y * _e261) - sign(_e261)), 0f, 1f);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e16 = o1_;
    let _e17 = G0_;
    let _e18 = i2_;
    let _e19 = Q0_;
    let _e20 = V0_;
    let _e21 = P0_;
    let _e22 = U0_;
    let _e23 = A3_;
    let _e24 = L4_;
    let _e25 = unnamed.gl_Position;
    return VertexOutput(_e16, _e17, _e18, _e19, _e20, _e21, _e22, _e23, _e24, _e25);
}
