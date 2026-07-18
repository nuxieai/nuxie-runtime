enable clip_distances;

struct gl_PerVertex {
    @builtin(position) gl_Position: vec4<f32>,
    gl_PointSize: f32,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    gl_CullDistance: array<f32, 1>,
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

struct VertexOutput {
    @builtin(position) gl_Position: vec4<f32>,
    @builtin(clip_distances) gl_ClipDistance: array<f32, 4>,
    @location(1) member: vec2<f32>,
    @location(4) @interpolate(flat) member_1: f32,
    @location(6) @interpolate(flat) member_2: f32,
    @location(0) member_3: vec4<f32>,
}

@id(0) override Ug: bool = true;
@id(2) override Wg: bool = true;
@id(1) override Vg: bool = true;

var<private> unnamed: gl_PerVertex = gl_PerVertex(vec4<f32>(0f, 0f, 0f, 1f), 1f, array<f32, 4>(), array<f32, 1>());
@group(0) @binding(2)
var PB: texture_2d<u32>;
@group(0) @binding(0)
var<uniform> m: CC;
var<private> gl_VertexIndex_1: i32;
var<private> LB_1: vec3<f32>;
var<private> B2_: vec2<f32>;
@group(0) @binding(3)
var AD: texture_2d<u32>;
var<private> H3_: f32;
var<private> e2_: f32;
@group(0) @binding(4)
var RB: texture_2d<f32>;
var<private> f1_: vec4<f32>;
@group(0) @binding(7)
var LC: texture_2d<u32>;
@group(0) @binding(9)
var XC: texture_2d<f32>;
@group(0) @binding(5)
var ED: texture_2d<u32>;
@group(3) @binding(9)
var Z9_: sampler;

fn main_1() {
    var phi_721_: u32;
    var phi_722_: f32;
    var phi_723_: f32;
    var phi_724_: vec4<f32>;

    let _e44 = LB_1;
    let _e46 = bitcast<u32>(_e44.z);
    let _e47 = (_e46 & 65535u);
    let _e48 = (_e47 * 4u);
    let _e49 = (_e48 + 2u);
    let _e55 = vec2<i32>(bitcast<i32>((_e49 & 127u)), bitcast<i32>((_e49 >> bitcast<u32>(7i))));
    let _e56 = textureLoad(PB, _e55, 0i);
    let _e58 = _e44.xy;
    let _e60 = bitcast<vec3<f32>>(_e56.yzw);
    let _e66 = m.xg;
    B2_ = (((_e58 * _e60.x) + _e60.yz) * _e66);
    let _e74 = textureLoad(AD, vec2<i32>(bitcast<i32>((_e46 & 127u)), bitcast<i32>((_e47 >> bitcast<u32>(7i)))), 0i);
    let _e76 = (_e74.x & 15u);
    if Ug {
        let _e77 = (_e76 == 0u);
        if _e77 {
            phi_721_ = _e74.y;
        } else {
            phi_721_ = _e74.x;
        }
        let _e80 = phi_721_;
        let _e82 = (_e80 >> bitcast<u32>(16i));
        let _e84 = m.Z5_;
        if (_e82 == 0u) {
            phi_722_ = 0f;
        } else {
            phi_722_ = unpack2x16float(((_e82 + 1023u) * _e84)).x;
        }
        let _e91 = phi_722_;
        phi_723_ = _e91;
        if _e77 {
            phi_723_ = -(_e91);
        }
        let _e94 = phi_723_;
        H3_ = _e94;
    }
    if Wg {
        e2_ = f32(((_e74.x >> bitcast<u32>(4i)) & 15u));
    }
    if Vg {
        let _e99 = textureLoad(RB, _e55, 0i);
        let _e107 = (_e48 + 3u);
        let _e114 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e107 & 127u)), bitcast<i32>((_e107 >> bitcast<u32>(7i)))), 0i);
        if any((_e99 != vec4<f32>(0f, 0f, 0f, 0f))) {
            let _e129 = ((mat2x2<f32>(vec2<f32>(_e99.x, _e99.y), vec2<f32>(_e99.z, _e99.w)) * _e58) + _e114.xy);
            unnamed.gl_ClipDistance[0i] = (_e129.x + 1f);
            unnamed.gl_ClipDistance[1i] = (_e129.y + 1f);
            unnamed.gl_ClipDistance[2i] = (1f - _e129.x);
            unnamed.gl_ClipDistance[3i] = (1f - _e129.y);
        } else {
            let _e119 = (_e114.x - 0.5f);
            unnamed.gl_ClipDistance[3i] = _e119;
            unnamed.gl_ClipDistance[2i] = _e119;
            unnamed.gl_ClipDistance[1i] = _e119;
            unnamed.gl_ClipDistance[0i] = _e119;
        }
    }
    if (_e76 == 1u) {
        let _e200 = unpack4x8unorm(_e74.y);
        if Wg {
            phi_724_ = _e200;
        } else {
            let _e203 = (_e200.xyz * _e200.w);
            let _e209 = vec4<f32>(_e203.x, _e200.y, _e200.z, _e200.w);
            let _e215 = vec4<f32>(_e209.x, _e203.y, _e209.z, _e209.w);
            phi_724_ = vec4<f32>(_e215.x, _e215.y, _e203.z, _e215.w);
        }
        let _e223 = phi_724_;
        f1_ = _e223;
    } else {
        let _e151 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e48 & 127u)), bitcast<i32>((_e48 >> bitcast<u32>(7i)))), 0i);
        let _e159 = (_e48 + 1u);
        let _e166 = textureLoad(RB, vec2<i32>(bitcast<i32>((_e159 & 127u)), bitcast<i32>((_e159 >> bitcast<u32>(7i)))), 0i);
        let _e169 = ((mat2x2<f32>(vec2<f32>(_e151.x, _e151.y), vec2<f32>(_e151.z, _e151.w)) * _e58) + _e166.xy);
        let _e170 = (_e76 == 2u);
        if (_e170 || (_e76 == 3u)) {
            f1_[3u] = -(bitcast<f32>(_e74.y));
            if (_e166.z > 0.9f) {
                f1_[2u] = 2f;
            } else {
                f1_[2u] = _e166.w;
            }
            if _e170 {
                f1_[1u] = 0f;
                f1_[0u] = _e169.x;
            } else {
                let _e190 = f1_[2u];
                f1_[2u] = -(_e190);
                f1_[0u] = _e169.x;
                f1_[1u] = _e169.y;
            }
        } else {
            f1_ = vec4<f32>(_e169.x, _e169.y, bitcast<f32>(_e74.y), (-2f - _e166.z));
        }
    }
    let _e225 = m.bf;
    let _e227 = m.cf;
    let _e235 = vec4<f32>(((_e44.x * _e225) - 1f), ((_e44.y * _e227) - sign(_e227)), 0f, 1f);
    unnamed.gl_Position = vec4<f32>(_e235.x, _e235.y, (1f - (f32(_e56.x) * 0.000061035156f)), _e235.w);
    return;
}

@vertex
fn main(@builtin(vertex_index) gl_VertexIndex: u32, @location(0) LB: vec3<f32>) -> VertexOutput {
    gl_VertexIndex_1 = i32(gl_VertexIndex);
    LB_1 = LB;
    main_1();
    let _e12 = unnamed.gl_Position;
    let _e13 = unnamed.gl_ClipDistance;
    let _e14 = B2_;
    let _e15 = H3_;
    let _e16 = e2_;
    let _e17 = f1_;
    return VertexOutput(_e12, _e13, _e14, _e15, _e16, _e17);
}
