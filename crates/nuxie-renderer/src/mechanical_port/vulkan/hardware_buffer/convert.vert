#version 450
// Fullscreen triangle over the target texture, top-left origin. Each output
// point maps to the source frame through an affine transform that applies the
// frame's crop and rotation.
layout(push_constant) uniform SourceTransform {
    vec4 s;
    vec4 t;
} source_transform;

layout(location = 0) out vec2 source_uv;

void main() {
    vec2 uv = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
    gl_Position = vec4(uv * 2.0 - 1.0, 0.0, 1.0);
    vec3 point = vec3(uv, 1.0);
    source_uv = vec2(dot(source_transform.s.xyz, point), dot(source_transform.t.xyz, point));
}
