#include <metal_stdlib>
using namespace metal;

// This deliberately small diagnostic pipeline proves the native adapter seam,
// offline metallib loading, submission, and oracle readback. Production path
// coverage replaces it with the mechanically ported upstream Rive pipelines.
vertex float4 nuxie_tracer_solid_vertex(
    uint vertex_id [[vertex_id]],
    constant float2* positions [[buffer(0)]],
    constant float2& viewport [[buffer(1)]])
{
    float2 normalized = positions[vertex_id] / viewport;
    return float4(normalized.x * 2.0 - 1.0, 1.0 - normalized.y * 2.0, 0.0, 1.0);
}

fragment float4 nuxie_tracer_solid_fragment(constant float4& color [[buffer(0)]])
{
    return color;
}
