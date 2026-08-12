// language: metal2.4
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct VertexOutput {
    metal::float4 position;
};
struct Uniforms {
    uint blend_mode;
    uint padding0_;
    uint padding1_;
    uint padding2_;
};

metal::float3 unmultiply(
    metal::float4 color
) {
    return (color.w != 0.0) ? (color.xyz / metal::float3(color.w)) : metal::float3(0.0);
}

float luminance(
    metal::float3 color_1
) {
    return metal::dot(color_1, metal::float3(0.3, 0.59, 0.11));
}

metal::float3 set_luminance(
    metal::float3 color_2,
    metal::float3 luminance_color
) {
    float _e2 = luminance(luminance_color);
    float _e3 = luminance(color_2);
    metal::float3 delta = color_2 - metal::float3(_e3);
    metal::float2 limits = metal::float2(_e2, 1.0 - _e2) / metal::max(metal::float2(0.000062), metal::float2(-(metal::min(delta.x, metal::min(delta.y, delta.z))), metal::max(delta.x, metal::max(delta.y, delta.z))));
    return (delta * metal::min(1.0, metal::min(limits.x, limits.y))) + metal::float3(_e2);
}

metal::float3 set_luminance_saturation(
    metal::float3 hue_color,
    metal::float3 saturation_color,
    metal::float3 luminance_color_1
) {
    float target_saturation = metal::max(saturation_color.x, metal::max(saturation_color.y, saturation_color.z)) - metal::min(saturation_color.x, metal::min(saturation_color.y, saturation_color.z));
    metal::float3 biased_hue = hue_color - metal::float3(metal::min(hue_color.x, metal::min(hue_color.y, hue_color.z)));
    float source_saturation = metal::max(biased_hue.x, metal::max(biased_hue.y, biased_hue.z));
    metal::float3 scaled_hue = biased_hue * (target_saturation / metal::max(0.000062, source_saturation));
    metal::float3 _e30 = set_luminance(scaled_hue, luminance_color_1);
    return _e30;
}

metal::float3 advanced_coefficients(
    metal::float3 source,
    metal::float4 destination_premul,
    uint mode
) {
    metal::float3 result = {};
    metal::float4 clamped_destination = {};
    metal::float3 factors = metal::float3(0.0);
    uint index = 0u;
    metal::float3 _e3 = unmultiply(destination_premul);
    result = source;
    switch(mode) {
        case 1u: {
            result = (source + _e3) - (source * _e3);
            break;
        }
        case 2u: {
            metal::float3 product = source * _e3;
            result = 2.0 * metal::select(product, ((source + _e3) - product) - metal::float3(0.5), _e3 > metal::float3(0.5));
            break;
        }
        case 3u: {
            result = metal::min(source, _e3);
            break;
        }
        case 4u: {
            result = metal::max(source, _e3);
            break;
        }
        case 5u: {
            metal::float3 clamped_destination_1 = metal::clamp(destination_premul.xyz, metal::float3(0.0), metal::float3(destination_premul.w));
            metal::float3 denominator = metal::clamp(metal::float3(1.0) - source, metal::float3(0.0), metal::float3(1.0)) * destination_premul.w;
            result = metal::select(metal::min(metal::float3(1.0), clamped_destination_1 / denominator), metal::sign(clamped_destination_1), denominator == metal::float3(0.0));
            break;
        }
        case 6u: {
            metal::float3 clamped_source = metal::clamp(source, metal::float3(0.0), metal::float3(1.0));
            clamped_destination = metal::float4(metal::clamp(destination_premul.xyz, metal::float3(0.0), metal::float3(destination_premul.w)), destination_premul.w);
            float _e62 = clamped_destination.w;
            if (_e62 == 0.0) {
                clamped_destination.w = 1.0;
            }
            float _e68 = clamped_destination.w;
            metal::float4 _e69 = clamped_destination;
            metal::float3 numerator = metal::float3(_e68) - _e69.xyz;
            float _e78 = clamped_destination.w;
            result = metal::float3(1.0) - metal::select(metal::min(metal::float3(1.0), numerator / (clamped_source * _e78)), metal::sign(numerator), clamped_source == metal::float3(0.0));
            break;
        }
        case 7u: {
            metal::float3 product_1 = source * _e3;
            result = 2.0 * metal::select(product_1, ((source + _e3) - product_1) - metal::float3(0.5), source > metal::float3(0.5));
            break;
        }
        case 8u: {
            uint2 loop_bound = uint2(4294967295u);
            bool loop_init = true;
            while(true) {
                if (metal::all(loop_bound == uint2(0u))) { break; }
                loop_bound -= uint2(loop_bound.y == 0u, 1u);
                if (!loop_init) {
                    uint _e142 = index;
                    index = _e142 + 1u;
                }
                loop_init = false;
                uint _e105 = index;
                if (_e105 < 3u) {
                } else {
                    break;
                }
                {
                    uint _e108 = index;
                    if (source[metal::min(unsigned(_e108), 2u)] <= 0.5) {
                        uint _e112 = index;
                        uint _e114 = index;
                        factors[metal::min(unsigned(_e112), 2u)] = 1.0 - _e3[metal::min(unsigned(_e114), 2u)];
                    } else {
                        uint _e118 = index;
                        if (_e3[metal::min(unsigned(_e118), 2u)] <= 0.25) {
                            uint _e122 = index;
                            uint _e124 = index;
                            uint _e130 = index;
                            factors[metal::min(unsigned(_e122), 2u)] = (((16.0 * _e3[metal::min(unsigned(_e124), 2u)]) - 12.0) * _e3[metal::min(unsigned(_e130), 2u)]) + 3.0;
                        } else {
                            uint _e135 = index;
                            uint _e137 = index;
                            factors[metal::min(unsigned(_e135), 2u)] = metal::rsqrt(_e3[metal::min(unsigned(_e137), 2u)]) - 1.0;
                        }
                    }
                }
            }
            metal::float3 _e151 = factors;
            result = _e3 + ((_e3 * ((2.0 * source) - metal::float3(1.0))) * _e151);
            break;
        }
        case 9u: {
            result = metal::abs(_e3 - source);
            break;
        }
        case 10u: {
            result = (source + _e3) - ((2.0 * source) * _e3);
            break;
        }
        case 11u: {
            result = source * _e3;
            break;
        }
        case 12u: {
            metal::float3 _e167 = set_luminance_saturation(metal::clamp(source, metal::float3(0.0), metal::float3(1.0)), _e3, _e3);
            result = _e167;
            break;
        }
        case 13u: {
            metal::float3 _e173 = set_luminance_saturation(_e3, metal::clamp(source, metal::float3(0.0), metal::float3(1.0)), _e3);
            result = _e173;
            break;
        }
        case 14u: {
            metal::float3 _e179 = set_luminance(metal::clamp(source, metal::float3(0.0), metal::float3(1.0)), _e3);
            result = _e179;
            break;
        }
        case 15u: {
            metal::float3 _e185 = set_luminance(_e3, metal::clamp(source, metal::float3(0.0), metal::float3(1.0)));
            result = _e185;
            break;
        }
        default: {
            break;
        }
    }
    metal::float3 _e186 = result;
    return _e186;
}
metal::int2 naga_f2i32(metal::float2 value) {
    return static_cast<metal::int2>(metal::clamp(value, -2147483600.0, 2147483500.0));
}


struct fragment_mainInput {
};
struct fragment_mainOutput {
    metal::float4 member [[color(0)]];
};
fragment fragment_mainOutput fragment_main(
  metal::float4 position [[position]]
, metal::texture2d<float, metal::access::sample> source_texture [[texture(0)]]
, metal::texture2d<float, metal::access::sample> destination_texture [[texture(1)]]
, constant Uniforms& uniforms [[buffer(0)]]
) {
    const VertexOutput input = { position };
    metal::int2 coordinate = naga_f2i32(metal::floor(input.position.xy));
    uint clamped_lod_e7 = metal::min(uint(0), source_texture.get_num_mip_levels() - 1);
    metal::float4 source_premul = source_texture.read(metal::min(metal::uint2(coordinate), metal::uint2(source_texture.get_width(clamped_lod_e7), source_texture.get_height(clamped_lod_e7)) - 1), clamped_lod_e7);
    uint clamped_lod_e10 = metal::min(uint(0), destination_texture.get_num_mip_levels() - 1);
    metal::float4 destination_premul_1 = destination_texture.read(metal::min(metal::uint2(coordinate), metal::uint2(destination_texture.get_width(clamped_lod_e10), destination_texture.get_height(clamped_lod_e10)) - 1), clamped_lod_e10);
    if (source_premul.w == 0.0) {
        return fragment_mainOutput { destination_premul_1 };
    }
    metal::float3 _e14 = unmultiply(source_premul);
    uint _e17 = uniforms.blend_mode;
    metal::float3 _e18 = advanced_coefficients(_e14, destination_premul_1, _e17);
    metal::float3 advanced_color = metal::mix(_e14, _e18, metal::float3(destination_premul_1.w));
    metal::float4 blended_source = metal::float4(advanced_color * source_premul.w, source_premul.w);
    return fragment_mainOutput { blended_source + (destination_premul_1 * (1.0 - source_premul.w)) };
}
