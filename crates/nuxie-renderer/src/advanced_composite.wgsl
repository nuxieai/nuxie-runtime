struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

struct Uniforms {
    blend_mode: u32,
    padding0: u32,
    padding1: u32,
    padding2: u32,
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var destination_texture: texture_2d<f32>;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn unmultiply(color: vec4<f32>) -> vec3<f32> {
    return select(vec3<f32>(0.0), color.rgb / color.a, color.a != 0.0);
}

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.3, 0.59, 0.11));
}

fn set_luminance(color: vec3<f32>, luminance_color: vec3<f32>) -> vec3<f32> {
    let target_luminance = luminance(luminance_color);
    let delta = color - vec3<f32>(luminance(color));
    let limits = vec2<f32>(target_luminance, 1.0 - target_luminance) /
        max(vec2<f32>(0.000062), vec2<f32>(-min(delta.x, min(delta.y, delta.z)), max(delta.x, max(delta.y, delta.z))));
    return delta * min(1.0, min(limits.x, limits.y)) + vec3<f32>(target_luminance);
}

fn set_luminance_saturation(
    hue_color: vec3<f32>,
    saturation_color: vec3<f32>,
    luminance_color: vec3<f32>,
) -> vec3<f32> {
    let target_saturation = max(saturation_color.x, max(saturation_color.y, saturation_color.z)) -
        min(saturation_color.x, min(saturation_color.y, saturation_color.z));
    let biased_hue = hue_color - vec3<f32>(min(hue_color.x, min(hue_color.y, hue_color.z)));
    let source_saturation = max(biased_hue.x, max(biased_hue.y, biased_hue.z));
    let scaled_hue = biased_hue * (target_saturation / max(0.000062, source_saturation));
    return set_luminance(scaled_hue, luminance_color);
}

fn advanced_coefficients(source: vec3<f32>, destination_premul: vec4<f32>, mode: u32) -> vec3<f32> {
    let destination = unmultiply(destination_premul);
    var result = source;
    switch mode {
        case 1u: {
            result = source + destination - source * destination;
        }
        case 2u: {
            let product = source * destination;
            result = 2.0 * select(product, source + destination - product - vec3<f32>(0.5), destination > vec3<f32>(0.5));
        }
        case 3u: {
            result = min(source, destination);
        }
        case 4u: {
            result = max(source, destination);
        }
        case 5u: {
            let clamped_destination = clamp(destination_premul.rgb, vec3<f32>(0.0), vec3<f32>(destination_premul.a));
            let denominator = clamp(vec3<f32>(1.0) - source, vec3<f32>(0.0), vec3<f32>(1.0)) * destination_premul.a;
            result = select(min(vec3<f32>(1.0), clamped_destination / denominator), sign(clamped_destination), denominator == vec3<f32>(0.0));
        }
        case 6u: {
            let clamped_source = clamp(source, vec3<f32>(0.0), vec3<f32>(1.0));
            var clamped_destination = vec4<f32>(clamp(destination_premul.rgb, vec3<f32>(0.0), vec3<f32>(destination_premul.a)), destination_premul.a);
            if clamped_destination.a == 0.0 {
                clamped_destination.a = 1.0;
            }
            let numerator = clamped_destination.a - clamped_destination.rgb;
            result = vec3<f32>(1.0) - select(
                min(vec3<f32>(1.0), numerator / (clamped_source * clamped_destination.a)),
                sign(numerator),
                clamped_source == vec3<f32>(0.0),
            );
        }
        case 7u: {
            let product = source * destination;
            result = 2.0 * select(product, source + destination - product - vec3<f32>(0.5), source > vec3<f32>(0.5));
        }
        case 8u: {
            var factors = vec3<f32>(0.0);
            for (var index = 0u; index < 3u; index += 1u) {
                if source[index] <= 0.5 {
                    factors[index] = 1.0 - destination[index];
                } else if destination[index] <= 0.25 {
                    factors[index] = (16.0 * destination[index] - 12.0) * destination[index] + 3.0;
                } else {
                    factors[index] = inverseSqrt(destination[index]) - 1.0;
                }
            }
            result = destination + destination * (2.0 * source - vec3<f32>(1.0)) * factors;
        }
        case 9u: {
            result = abs(destination - source);
        }
        case 10u: {
            result = source + destination - 2.0 * source * destination;
        }
        case 11u: {
            result = source * destination;
        }
        case 12u: {
            result = set_luminance_saturation(clamp(source, vec3<f32>(0.0), vec3<f32>(1.0)), destination, destination);
        }
        case 13u: {
            result = set_luminance_saturation(destination, clamp(source, vec3<f32>(0.0), vec3<f32>(1.0)), destination);
        }
        case 14u: {
            result = set_luminance(clamp(source, vec3<f32>(0.0), vec3<f32>(1.0)), destination);
        }
        case 15u: {
            result = set_luminance(destination, clamp(source, vec3<f32>(0.0), vec3<f32>(1.0)));
        }
        default: {}
    }
    return result;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let coordinate = vec2<i32>(floor(input.position.xy));
    let source_premul = textureLoad(source_texture, coordinate, 0);
    let destination_premul = textureLoad(destination_texture, coordinate, 0);
    if source_premul.a == 0.0 {
        return destination_premul;
    }
    let source = unmultiply(source_premul);
    let coefficients = advanced_coefficients(source, destination_premul, uniforms.blend_mode);
    let advanced_color = mix(source, coefficients, vec3<f32>(destination_premul.a));
    let blended_source = vec4<f32>(advanced_color * source_premul.a, source_premul.a);
    return blended_source + destination_premul * (1.0 - source_premul.a);
}
