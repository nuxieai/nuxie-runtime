#version 450
// The immutable sampler's Y'CbCr conversion turns the decoder's YUV into RGB
// with the buffer's own matrix and range; video frames are opaque.
layout(set = 0, binding = 0) uniform sampler2D frame;

layout(location = 0) in vec2 source_uv;
layout(location = 0) out vec4 color;

void main() {
    color = vec4(texture(frame, source_uv).rgb, 1.0);
}
