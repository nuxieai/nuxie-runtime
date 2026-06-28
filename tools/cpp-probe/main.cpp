/*
 * Inspect graph-relevant state from the C++ Rive runtime as strict JSON.
 *
 * This is a parity oracle for the Rust port: it imports the same .riv bytes
 * through rive::File::import and dumps the object arena details that matter
 * for graph construction.
 */

#include <algorithm>
#include <cstdint>
#include <cmath>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <iterator>
#include <memory>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <vector>

#define private public
#include "rive/assets/manifest_asset.hpp"
#undef private
#define private public
#include "rive/shapes/vertex.hpp"
#undef private
#define protected public
#include "rive/shapes/paint/shape_paint.hpp"
#include "rive/shapes/shape_paint_container.hpp"
#undef protected
#define private public
#include "rive/shapes/paint/linear_gradient.hpp"
#include "rive/shapes/paint/target_effect.hpp"
#undef private
#define private public
#include "rive/artboard_component_list.hpp"
#undef private
#include "rive/artboard.hpp"
#include "rive/animation/blend_animation.hpp"
#include "rive/animation/blend_state.hpp"
#include "rive/animation/blend_state_transition.hpp"
#include "rive/animation/cubic_ease_interpolator.hpp"
#include "rive/animation/elastic_interpolator.hpp"
#include "rive/animation/keyed_object.hpp"
#include "rive/animation/keyed_property.hpp"
#include "rive/animation/keyframe.hpp"
#include "rive/animation/animation_state.hpp"
#include "rive/animation/layer_state.hpp"
#include "rive/animation/linear_animation.hpp"
#include "rive/animation/listener_action.hpp"
#include "rive/animation/listener_fire_event.hpp"
#include "rive/animation/listener_types/listener_input_type.hpp"
#include "rive/animation/listener_types/listener_input_type_viewmodel.hpp"
#include "rive/animation/scripted_listener_action.hpp"
#include "rive/animation/scripted_transition_condition.hpp"
#include "rive/animation/state_machine_fire_trigger.hpp"
#include "rive/animation/state_machine_fire_action.hpp"
#include "rive/animation/state_machine_fire_event.hpp"
#include "rive/animation/state_machine_layer_component.hpp"
#include "rive/animation/state_machine_component.hpp"
#include "rive/animation/state_machine_input.hpp"
#include "rive/animation/state_machine_layer.hpp"
#include "rive/animation/state_machine_listener.hpp"
#include "rive/animation/state_machine_listener_single.hpp"
#include "rive/animation/state_machine.hpp"
#include "rive/animation/state_transition.hpp"
#include "rive/animation/transition_condition.hpp"
#include "rive/assets/file_asset.hpp"
#include "rive/bones/skin.hpp"
#include "rive/bones/tendon.hpp"
#include "rive/component.hpp"
#include "rive/constraints/scrolling/scroll_constraint.hpp"
#include "rive/core.hpp"
#include "rive/core/field_types/core_bool_type.hpp"
#include "rive/core/field_types/core_color_type.hpp"
#include "rive/core/field_types/core_double_type.hpp"
#include "rive/core/field_types/core_string_type.hpp"
#include "rive/core/field_types/core_uint_type.hpp"
#include "rive/custom_property.hpp"
#include "rive/data_bind/converters/formula/formula_token.hpp"
#include "rive/generated/data_bind/converters/data_converter_formula_base.hpp"
#include "rive/data_bind/data_values/data_value_number.hpp"
#include "rive/viewmodel/viewmodel_instance_value.hpp"
#include "rive/viewmodel/viewmodel_value_dependent.hpp"
#include "rive/viewmodel/symbol_type.hpp"
#define protected public
#include "rive/viewmodel/viewmodel_instance_asset.hpp"
#undef protected
#define private public
#include "rive/data_bind/converters/data_converter_formula.hpp"
#undef private
#include "rive/data_bind/converters/formula/formula_token_argument_separator.hpp"
#include "rive/data_bind/converters/formula/formula_token_function.hpp"
#include "rive/data_bind/converters/formula/formula_token_input.hpp"
#include "rive/data_bind/converters/formula/formula_token_operation.hpp"
#include "rive/data_bind/converters/formula/formula_token_parenthesis_close.hpp"
#include "rive/data_bind/converters/formula/formula_token_parenthesis_open.hpp"
#include "rive/data_bind/converters/formula/formula_token_value.hpp"
#include "rive/data_bind/converters/data_converter_boolean_negate.hpp"
#include "rive/data_bind/converters/data_converter_interpolator.hpp"
#include "rive/data_bind/converters/data_converter_group.hpp"
#include "rive/data_bind/converters/data_converter_group_item.hpp"
#include "rive/data_bind/converters/data_converter_list_to_length.hpp"
#define private public
#include "rive/data_bind/converters/data_converter_number_to_list.hpp"
#include "rive/file.hpp"
#undef private
#include "rive/data_bind/converters/data_converter_operation.hpp"
#include "rive/data_bind/converters/data_converter_operation_value.hpp"
#include "rive/data_bind/converters/data_converter_operation_viewmodel.hpp"
#include "rive/data_bind/converters/data_converter_range_mapper.hpp"
#include "rive/data_bind/converters/data_converter_rounder.hpp"
#include "rive/data_bind/converters/data_converter_string_pad.hpp"
#include "rive/data_bind/converters/data_converter_string_remove_zeros.hpp"
#include "rive/data_bind/converters/data_converter_string_trim.hpp"
#include "rive/data_bind/converters/data_converter_system_degs_to_rads.hpp"
#include "rive/data_bind/converters/data_converter_system_normalizer.hpp"
#include "rive/data_bind/converters/data_converter_to_number.hpp"
#include "rive/data_bind/converters/data_converter_to_string.hpp"
#include "rive/data_bind/converters/data_converter_trigger.hpp"
#include "rive/data_bind/data_bind.hpp"
#define private public
#define protected public
#include "rive/data_bind/data_bind_context.hpp"
#undef protected
#undef private
#include "rive/data_bind/data_context.hpp"
#include "rive/data_bind/data_bind_path.hpp"
#include "rive/data_bind/data_values/data_value_artboard.hpp"
#include "rive/data_bind/data_values/data_value_asset_image.hpp"
#include "rive/data_bind/data_values/data_value_boolean.hpp"
#include "rive/data_bind/data_values/data_value_color.hpp"
#include "rive/data_bind/data_values/data_value_enum.hpp"
#include "rive/data_bind/data_values/data_value_string.hpp"
#include "rive/data_bind/data_values/data_value_symbol_list_index.hpp"
#include "rive/data_bind/data_values/data_value_trigger.hpp"
#include "rive/draw_rules.hpp"
#include "rive/draw_target.hpp"
#include "rive/drawable.hpp"
#include "rive/event.hpp"
#include "rive/foreground_layout_drawable.hpp"
#include "rive/generated/core_registry.hpp"
#include "rive/generated/event_base.hpp"
#include "rive/layout_component.hpp"
#include "rive/layout/axis.hpp"
#include "rive/layout/n_sliced_node.hpp"
#include "rive/layout/n_slicer.hpp"
#include "rive/layout/n_slicer_details.hpp"
#include "rive/layout/n_slicer_tile_mode.hpp"
#include "rive/nested_artboard.hpp"
#include "rive/refcnt.hpp"
#include "rive/scripted/scripted_object.hpp"
#include "rive/scripted/scripted_data_converter.hpp"
#include "rive/scripted/scripted_path_effect.hpp"
#include "rive/bones/weight.hpp"
#include "rive/shapes/clipping_shape.hpp"
#include "rive/shapes/mesh.hpp"
#include "rive/shapes/mesh_vertex.hpp"
#include "rive/shapes/path.hpp"
#include "rive/shapes/path_vertex.hpp"
#include "rive/shapes/paint/dash_path.hpp"
#include "rive/shapes/paint/feather.hpp"
#include "rive/shapes/paint/gradient_stop.hpp"
#include "rive/shapes/paint/group_effect.hpp"
#include "rive/shapes/paint/radial_gradient.hpp"
#include "rive/shapes/paint/solid_color.hpp"
#include "rive/shapes/paint/trim_path.hpp"
#include "rive/shapes/points_path.hpp"
#include "rive/shapes/shape.hpp"
#include "rive/text/text_input_cursor.hpp"
#include "rive/text/text_input_selected_text.hpp"
#include "rive/text/text_input_selection.hpp"
#include "rive/text/text_input_text.hpp"
#include "rive/text/text_style_paint.hpp"
#include "rive/viewmodel/data_enum.hpp"
#include "rive/viewmodel/data_enum_value.hpp"
#include "rive/viewmodel/viewmodel.hpp"
#include "rive/viewmodel/viewmodel_instance.hpp"
#include "rive/viewmodel/viewmodel_instance_artboard.hpp"
#include "rive/viewmodel/viewmodel_instance_asset_image.hpp"
#include "rive/viewmodel/viewmodel_instance_boolean.hpp"
#include "rive/viewmodel/viewmodel_instance_color.hpp"
#include "rive/viewmodel/viewmodel_instance_number.hpp"
#include "rive/viewmodel/viewmodel_instance_list.hpp"
#include "rive/viewmodel/viewmodel_instance_list_item.hpp"
#include "rive/viewmodel/viewmodel_instance_value.hpp"
#include "rive/viewmodel/viewmodel_instance_viewmodel.hpp"
#include "rive/viewmodel/viewmodel_instance_enum.hpp"
#include "rive/viewmodel/viewmodel_instance_string.hpp"
#include "rive/viewmodel/viewmodel_instance_symbol_list_index.hpp"
#include "rive/viewmodel/viewmodel_instance_trigger.hpp"
#include "rive/viewmodel/runtime/viewmodel_instance_artboard_runtime.hpp"
#include "rive/viewmodel/runtime/viewmodel_instance_asset_image_runtime.hpp"
#include "rive/viewmodel/runtime/viewmodel_instance_boolean_runtime.hpp"
#include "rive/viewmodel/runtime/viewmodel_instance_color_runtime.hpp"
#include "rive/viewmodel/runtime/viewmodel_instance_enum_runtime.hpp"
#include "rive/viewmodel/runtime/viewmodel_instance_runtime.hpp"
#include "rive/viewmodel/runtime/viewmodel_instance_list_runtime.hpp"
#include "rive/viewmodel/runtime/viewmodel_instance_number_runtime.hpp"
#include "rive/viewmodel/runtime/viewmodel_instance_string_runtime.hpp"
#include "rive/viewmodel/runtime/viewmodel_instance_trigger_runtime.hpp"
#include "rive/viewmodel/viewmodel_property_enum.hpp"
#include "rive/viewmodel/viewmodel_property.hpp"
#include "rive/script_input_viewmodel_property.hpp"
#include "rive/world_transform_component.hpp"
#include "utils/no_op_factory.hpp"

namespace
{
using LocalIds = std::unordered_map<const rive::Core*, size_t>;

struct ProbeOptions
{
    bool propertyValues = false;
    bool artboardPropertyValues = false;
    bool advanceArtboards = true;
    bool completeViewModelProperties = false;
    bool dataContextLookups = false;
};

std::vector<uint8_t> read_bytes(const char* path)
{
    std::ifstream file(path, std::ios::binary);
    if (!file)
    {
        throw std::runtime_error(std::string("failed to open ") + path);
    }

    std::vector<char> chars((std::istreambuf_iterator<char>(file)),
                            std::istreambuf_iterator<char>());
    return std::vector<uint8_t>(chars.begin(), chars.end());
}

const char* import_result_name(rive::ImportResult result)
{
    switch (result)
    {
        case rive::ImportResult::success:
            return "success";
        case rive::ImportResult::unsupportedVersion:
            return "unsupportedVersion";
        case rive::ImportResult::malformed:
            return "malformed";
    }
    return "unknown";
}

rive::rcp<rive::File> open_file(const char* path, rive::ImportResult* result)
{
    std::vector<uint8_t> bytes = read_bytes(path);
    static rive::NoOpFactory factory;
    return rive::File::import(bytes, &factory, result);
}

void write_json_string(std::ostream& out, const std::string& value)
{
    out << '"';
    for (unsigned char c : value)
    {
        switch (c)
        {
            case '"':
                out << "\\\"";
                break;
            case '\\':
                out << "\\\\";
                break;
            case '\b':
                out << "\\b";
                break;
            case '\f':
                out << "\\f";
                break;
            case '\n':
                out << "\\n";
                break;
            case '\r':
                out << "\\r";
                break;
            case '\t':
                out << "\\t";
                break;
            default:
                if (c < 0x20)
                {
                    out << "\\u" << std::hex << std::setw(4)
                        << std::setfill('0') << static_cast<int>(c)
                        << std::dec << std::setfill(' ');
                }
                else
                {
                    out << c;
                }
                break;
        }
    }
    out << '"';
}

void write_local_id_or_null(std::ostream& out,
                            const LocalIds& localIds,
                            const rive::Core* object)
{
    if (object == nullptr)
    {
        out << "null";
        return;
    }

    auto itr = localIds.find(object);
    if (itr == localIds.end())
    {
        out << "null";
        return;
    }

    out << itr->second;
}

void write_world_transform(std::ostream& out,
                           const rive::WorldTransformComponent* component)
{
    const rive::Mat2D& transform = component->worldTransform();
    out << '[';
    for (size_t i = 0; i < 6; ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        out << transform[i];
    }
    out << ']';
}

void write_component_fields(std::ostream& out,
                            const LocalIds& localIds,
                            const rive::Component* component)
{
    out << ",\"name\":";
    write_json_string(out, component->name());
    out << ",\"parentId\":" << component->parentId();
    out << ",\"parentLocal\":";
    write_local_id_or_null(out, localIds, component->parent());
    out << ",\"graphOrder\":" << component->graphOrder();

    out << ",\"worldTransform\":";
    if (component->is<rive::WorldTransformComponent>())
    {
        write_world_transform(out,
                              component->as<rive::WorldTransformComponent>());
    }
    else
    {
        out << "null";
    }
}

void write_file_artboard_index_or_null(std::ostream& out,
                                       const rive::File* file,
                                       const rive::Artboard* artboard)
{
    if (file == nullptr || artboard == nullptr)
    {
        out << "null";
        return;
    }

    for (size_t i = 0; i < file->artboardCount(); ++i)
    {
        if (file->artboard(i) == artboard)
        {
            out << i;
            return;
        }
    }

    out << "null";
}

void write_u32_vector_or_null(std::ostream& out,
                              const std::vector<uint32_t>* values)
{
    if (values == nullptr)
    {
        out << "null";
        return;
    }

    out << '[';
    for (size_t i = 0; i < values->size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        out << (*values)[i];
    }
    out << ']';
}

void write_data_bind_path_ids_or_null(std::ostream& out,
                                      rive::DataBindPath* dataBindPath)
{
    if (dataBindPath == nullptr)
    {
        out << "null";
        return;
    }

    write_u32_vector_or_null(out, &dataBindPath->path());
}

void write_data_bind_path_resolved_ids_or_null(std::ostream& out,
                                               rive::DataBindPath* dataBindPath)
{
    if (dataBindPath == nullptr)
    {
        out << "null";
        return;
    }

    write_u32_vector_or_null(out, &dataBindPath->resolvedPath());
}

void write_core_data_bind_path_ids_or_null(std::ostream& out,
                                           rive::Core* object)
{
    if (object == nullptr)
    {
        out << "null";
        return;
    }

    if (object->is<rive::NestedArtboard>())
    {
        write_data_bind_path_ids_or_null(
            out, object->as<rive::NestedArtboard>()->dataBindPath());
    }
    else if (object->is<rive::StateMachineListenerSingle>())
    {
        write_data_bind_path_ids_or_null(
            out, object->as<rive::StateMachineListenerSingle>()->dataBindPath());
    }
    else if (object->is<rive::ListenerInputTypeViewModel>())
    {
        write_data_bind_path_ids_or_null(
            out, object->as<rive::ListenerInputTypeViewModel>()->dataBindPath());
    }
    else if (object->is<rive::StateMachineFireTrigger>())
    {
        write_data_bind_path_ids_or_null(
            out, object->as<rive::StateMachineFireTrigger>()->dataBindPath());
    }
    else if (object->is<rive::ScriptInputViewModelProperty>())
    {
        write_data_bind_path_ids_or_null(
            out,
            object->as<rive::ScriptInputViewModelProperty>()->dataBindPath());
    }
    else
    {
        out << "null";
    }
}

void write_core_data_bind_resolved_path_ids_or_null(std::ostream& out,
                                                    rive::Core* object)
{
    if (object == nullptr)
    {
        out << "null";
        return;
    }

    if (object->is<rive::NestedArtboard>())
    {
        write_data_bind_path_resolved_ids_or_null(
            out, object->as<rive::NestedArtboard>()->dataBindPath());
    }
    else if (object->is<rive::StateMachineListenerSingle>())
    {
        write_data_bind_path_resolved_ids_or_null(
            out, object->as<rive::StateMachineListenerSingle>()->dataBindPath());
    }
    else if (object->is<rive::ListenerInputTypeViewModel>())
    {
        write_data_bind_path_resolved_ids_or_null(
            out, object->as<rive::ListenerInputTypeViewModel>()->dataBindPath());
    }
    else if (object->is<rive::StateMachineFireTrigger>())
    {
        write_data_bind_path_resolved_ids_or_null(
            out, object->as<rive::StateMachineFireTrigger>()->dataBindPath());
    }
    else if (object->is<rive::ScriptInputViewModelProperty>())
    {
        write_data_bind_path_resolved_ids_or_null(
            out,
            object->as<rive::ScriptInputViewModelProperty>()->dataBindPath());
    }
    else
    {
        out << "null";
    }
}

void write_scroll_physics_core_type_or_null(std::ostream& out,
                                            rive::Core* object)
{
    if (object != nullptr && object->is<rive::ScrollConstraint>() &&
        object->as<rive::ScrollConstraint>()->physics() != nullptr)
    {
        out << object->as<rive::ScrollConstraint>()->physics()->coreType();
    }
    else
    {
        out << "null";
    }
}

void write_registry_property_value(std::ostream& out,
                                   const rive::Core* object,
                                   uint32_t propertyKey,
                                   int fieldId)
{
    rive::Core* mutableObject = const_cast<rive::Core*>(object);
    out << "{\"key\":" << propertyKey << ",\"kind\":";
    switch (fieldId)
    {
        case rive::CoreUintType::id:
            write_json_string(out, "uint");
            out << ",\"value\":"
                << rive::CoreRegistry::getUint(mutableObject, propertyKey);
            break;
        case rive::CoreStringType::id:
            write_json_string(out, "string");
            out << ",\"value\":";
            write_json_string(out,
                              rive::CoreRegistry::getString(mutableObject,
                                                            propertyKey));
            break;
        case rive::CoreDoubleType::id:
        {
            float value =
                rive::CoreRegistry::getDouble(mutableObject, propertyKey);
            write_json_string(out, "double");
            out << ",\"value\":";
            if (std::isnan(value))
            {
                write_json_string(out, "nan");
            }
            else if (std::isinf(value))
            {
                write_json_string(out, value > 0.0f ? "inf" : "-inf");
            }
            else
            {
                out << value;
            }
            break;
        }
        case rive::CoreColorType::id:
            write_json_string(out, "color");
            out << ",\"value\":"
                << static_cast<uint32_t>(
                       rive::CoreRegistry::getColor(mutableObject,
                                                    propertyKey));
            break;
        case rive::CoreBoolType::id:
            write_json_string(out, "bool");
            out << ",\"value\":"
                << (rive::CoreRegistry::getBool(mutableObject, propertyKey)
                        ? "true"
                        : "false");
            break;
    }
    out << '}';
}

void write_registry_property_values(std::ostream& out, const rive::Core* object)
{
    rive::Core* mutableObject = const_cast<rive::Core*>(object);
    out << ",\"propertyValues\":[";
    bool first = true;
    for (uint32_t propertyKey = 1; propertyKey <= 65535; ++propertyKey)
    {
        if (!rive::CoreRegistry::objectSupportsProperty(mutableObject,
                                                        propertyKey))
        {
            continue;
        }

        int fieldId =
            rive::CoreRegistry::propertyFieldId(static_cast<int>(propertyKey));
        switch (fieldId)
        {
            case rive::CoreUintType::id:
            case rive::CoreStringType::id:
            case rive::CoreDoubleType::id:
            case rive::CoreColorType::id:
            case rive::CoreBoolType::id:
                break;
            default:
                continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_registry_property_value(out, object, propertyKey, fieldId);
    }
    out << ']';
}

void write_object(std::ostream& out,
                  const rive::File* file,
                  const LocalIds& localIds,
                  size_t localId,
                  rive::Core* object,
                  const ProbeOptions& options)
{
    if (object == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"localId\":" << localId;
    out << ",\"coreType\":" << object->coreType();
    out << ",\"isComponent\":"
        << (object->is<rive::Component>() ? "true" : "false");
    out << ",\"sourceArtboardIndex\":";
    if (object->is<rive::NestedArtboard>())
    {
        write_file_artboard_index_or_null(
            out, file, object->as<rive::NestedArtboard>()->sourceArtboard());
    }
    else
    {
        out << "null";
    }
    out << ",\"dataBindPathIds\":";
    write_core_data_bind_path_ids_or_null(out, object);
    out << ",\"resolvedDataBindPathIds\":";
    write_core_data_bind_resolved_path_ids_or_null(out, object);
    out << ",\"scrollPhysicsCoreType\":";
    write_scroll_physics_core_type_or_null(out, object);

    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, object);
    }

    if (object->is<rive::Component>())
    {
        write_component_fields(out, localIds, object->as<rive::Component>());
    }

    out << '}';
}

void write_component(std::ostream& out,
                     const LocalIds& localIds,
                     size_t localId,
                     const rive::Component* component)
{
    out << "{\"localId\":" << localId;
    out << ",\"coreType\":" << component->coreType();
    write_component_fields(out, localIds, component);
    out << '}';
}

void write_draw_target(std::ostream& out,
                       const LocalIds& localIds,
                       size_t localId,
                       const rive::DrawTarget* target)
{
    out << "{\"localId\":" << localId;
    out << ",\"drawableId\":" << target->drawableId();
    out << ",\"drawableLocal\":";
    write_local_id_or_null(out, localIds, target->drawable());
    out << ",\"placementValue\":" << target->placementValue();
    out << '}';
}

void write_draw_rules(std::ostream& out,
                      const LocalIds& localIds,
                      size_t localId,
                      const rive::DrawRules* rules)
{
    out << "{\"localId\":" << localId;
    out << ",\"drawTargetId\":" << rules->drawTargetId();
    out << ",\"activeTargetLocal\":";
    write_local_id_or_null(out, localIds, rules->activeTarget());
    out << '}';
}

bool drawable_has_clipping_shape(const rive::Drawable* drawable,
                                 const rive::ClippingShape* clippingShape)
{
    for (auto candidate : drawable->clippingShapes())
    {
        if (candidate == clippingShape)
        {
            return true;
        }
    }
    return false;
}

void write_clipping_shape(std::ostream& out,
                          const std::vector<rive::Core*>& objects,
                          const LocalIds& localIds,
                          size_t localId,
                          const rive::ClippingShape* clippingShape)
{
    out << "{\"localId\":" << localId;
    out << ",\"sourceId\":" << clippingShape->sourceId();
    out << ",\"sourceLocal\":";
    write_local_id_or_null(out, localIds, clippingShape->source());
    out << ",\"fillRule\":" << clippingShape->fillRule();
    out << ",\"isVisible\":" << (clippingShape->isVisible() ? "true" : "false");

    out << ",\"shapeLocals\":[";
    bool first = true;
    for (auto shape : clippingShape->shapes())
    {
        auto itr = localIds.find(shape);
        if (itr == localIds.end())
        {
            continue;
        }
        if (!first)
        {
            out << ',';
        }
        first = false;
        out << itr->second;
    }
    out << ']';

    out << ",\"clippedDrawableLocals\":[";
    first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        auto object = objects[i];
        if (object == nullptr || object == clippingShape ||
            !object->is<rive::Drawable>())
        {
            continue;
        }
        auto drawable = object->as<rive::Drawable>();
        if (!drawable_has_clipping_shape(drawable, clippingShape))
        {
            continue;
        }
        if (!first)
        {
            out << ',';
        }
        first = false;
        out << i;
    }
    out << "]}";
}

void write_mesh(std::ostream& out,
                const LocalIds& localIds,
                const std::vector<rive::Core*>& objects,
                size_t localId,
                rive::Mesh* mesh)
{
    out << "{\"localId\":" << localId;
    out << ",\"coreType\":" << mesh->coreType();

    out << ",\"vertices\":[";
    bool first = true;
    size_t vertexIndex = 0;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        auto object = objects[i];
        if (object == nullptr || !object->is<rive::MeshVertex>())
        {
            continue;
        }

        auto vertex = object->as<rive::MeshVertex>();
        if (vertex->parent() != mesh)
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;

        auto weight = vertex->m_Weight;
        out << "{\"index\":" << vertexIndex++;
        out << ",\"localId\":" << i;
        out << ",\"coreType\":" << vertex->coreType();
        out << ",\"weightLocal\":";
        write_local_id_or_null(out, localIds, weight);
        out << ",\"weightCoreType\":";
        if (weight == nullptr)
        {
            out << "null";
        }
        else
        {
            out << weight->coreType();
        }
        out << ",\"weightValues\":";
        if (weight == nullptr)
        {
            out << "null";
        }
        else
        {
            out << weight->values();
        }
        out << ",\"weightIndices\":";
        if (weight == nullptr)
        {
            out << "null";
        }
        else
        {
            out << weight->indices();
        }
        out << '}';
    }
    out << "]}";
}

void write_path(std::ostream& out,
                const LocalIds& localIds,
                const std::vector<rive::Core*>& objects,
                size_t localId,
                rive::Path* path)
{
    out << "{\"localId\":" << localId;
    out << ",\"coreType\":" << path->coreType();

    out << ",\"vertices\":[";
    bool first = true;
    size_t vertexIndex = 0;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        auto object = objects[i];
        if (object == nullptr || !object->is<rive::PathVertex>())
        {
            continue;
        }

        auto vertex = object->as<rive::PathVertex>();
        if (vertex->parent() != path)
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;

        auto weight = vertex->m_Weight;
        out << "{\"index\":" << vertexIndex++;
        out << ",\"localId\":" << i;
        out << ",\"coreType\":" << vertex->coreType();
        out << ",\"weightLocal\":";
        write_local_id_or_null(out, localIds, weight);
        out << ",\"weightCoreType\":";
        if (weight == nullptr)
        {
            out << "null";
        }
        else
        {
            out << weight->coreType();
        }
        out << ",\"weightValues\":";
        if (weight == nullptr)
        {
            out << "null";
        }
        else
        {
            out << weight->values();
        }
        out << ",\"weightIndices\":";
        if (weight == nullptr)
        {
            out << "null";
        }
        else
        {
            out << weight->indices();
        }
        out << '}';
    }
    out << "]}";
}

void write_n_slicer_axis(std::ostream& out,
                         const LocalIds& localIds,
                         size_t index,
                         rive::Axis* axis)
{
    out << "{\"index\":" << index;
    out << ",\"localId\":";
    write_local_id_or_null(out, localIds, axis);
    out << ",\"coreType\":" << axis->coreType();
    out << '}';
}

void write_n_slicer_tile_mode(std::ostream& out,
                              const LocalIds& localIds,
                              const std::vector<rive::Core*>& objects,
                              size_t index,
                              size_t parentLocalId,
                              int patchIndex,
                              rive::NSlicerTileModeType style)
{
    rive::NSlicerTileMode* tileMode = nullptr;
    for (auto object : objects)
    {
        if (object == nullptr || !object->is<rive::NSlicerTileMode>())
        {
            continue;
        }
        auto candidate = object->as<rive::NSlicerTileMode>();
        if (candidate->parentId() == parentLocalId &&
            static_cast<int>(candidate->patchIndex()) == patchIndex)
        {
            tileMode = candidate;
        }
    }

    out << "{\"index\":" << index;
    out << ",\"localId\":";
    write_local_id_or_null(out, localIds, tileMode);
    out << ",\"coreType\":";
    if (tileMode == nullptr)
    {
        out << "null";
    }
    else
    {
        out << tileMode->coreType();
    }
    out << ",\"patchIndex\":" << patchIndex;
    out << ",\"style\":" << static_cast<int>(style);
    out << '}';
}

void write_n_slicer_details(std::ostream& out,
                            const LocalIds& localIds,
                            const std::vector<rive::Core*>& objects,
                            size_t localId,
                            rive::Component* component,
                            rive::NSlicerDetails* details)
{
    out << "{\"localId\":" << localId;
    out << ",\"coreType\":" << component->coreType();

    out << ",\"xAxes\":[";
    const auto& xs = details->xs();
    for (size_t i = 0; i < xs.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_n_slicer_axis(out, localIds, i, xs[i]);
    }
    out << ']';

    out << ",\"yAxes\":[";
    const auto& ys = details->ys();
    for (size_t i = 0; i < ys.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_n_slicer_axis(out, localIds, i, ys[i]);
    }
    out << ']';

    std::vector<std::pair<int, rive::NSlicerTileModeType>> tileModes;
    for (auto entry : details->tileModes())
    {
        tileModes.push_back(entry);
    }
    std::sort(tileModes.begin(), tileModes.end(), [](const auto& a, const auto& b) {
        return a.first < b.first;
    });

    out << ",\"tileModes\":[";
    for (size_t i = 0; i < tileModes.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_n_slicer_tile_mode(out,
                                 localIds,
                                 objects,
                                 i,
                                 localId,
                                 tileModes[i].first,
                                 tileModes[i].second);
    }
    out << "]}";
}

template <typename T>
rive::Core* stroke_effect_core_as(rive::Core* object,
                                  rive::StrokeEffect* effect)
{
    if (object == nullptr || !object->is<T>())
    {
        return nullptr;
    }

    auto typed = object->as<T>();
    if (static_cast<rive::StrokeEffect*>(typed) == effect)
    {
        return typed;
    }
    return nullptr;
}

rive::Core* stroke_effect_core(const std::vector<rive::Core*>& objects,
                               rive::StrokeEffect* effect)
{
    for (auto object : objects)
    {
        if (auto core = stroke_effect_core_as<rive::DashPath>(object, effect))
        {
            return core;
        }
        if (auto core = stroke_effect_core_as<rive::TrimPath>(object, effect))
        {
            return core;
        }
        if (auto core = stroke_effect_core_as<rive::TargetEffect>(object, effect))
        {
            return core;
        }
        if (auto core = stroke_effect_core_as<rive::GroupEffect>(object, effect))
        {
            return core;
        }
        if (auto core =
                stroke_effect_core_as<rive::ScriptedPathEffect>(object, effect))
        {
            return core;
        }
    }

    return nullptr;
}

void write_stroke_effect(std::ostream& out,
                         const LocalIds& localIds,
                         const std::vector<rive::Core*>& objects,
                         size_t effectIndex,
                         rive::StrokeEffect* effect)
{
    auto core = stroke_effect_core(objects, effect);
    if (core == nullptr)
    {
        throw std::runtime_error("unknown StrokeEffect subclass in C++ probe");
    }

    rive::GroupEffect* targetGroupEffect = nullptr;
    if (core->is<rive::TargetEffect>())
    {
        targetGroupEffect = core->as<rive::TargetEffect>()->m_groupEffect;
    }

    out << "{\"index\":" << effectIndex;
    out << ",\"localId\":";
    write_local_id_or_null(out, localIds, core);
    out << ",\"coreType\":" << core->coreType();
    out << ",\"targetGroupEffectLocal\":";
    write_local_id_or_null(out, localIds, targetGroupEffect);
    out << ",\"targetGroupEffectCoreType\":";
    if (targetGroupEffect == nullptr)
    {
        out << "null";
    }
    else
    {
        out << targetGroupEffect->coreType();
    }
    out << '}';
}

void write_shape_paint(std::ostream& out,
                       const LocalIds& localIds,
                       const std::vector<rive::Core*>& objects,
                       size_t paintIndex,
                       rive::ShapePaint* paint)
{
    auto mutator = paint->m_PaintMutator == nullptr ? nullptr : paint->paint();
    out << "{\"index\":" << paintIndex;
    out << ",\"localId\":";
    write_local_id_or_null(out, localIds, paint);
    out << ",\"coreType\":" << paint->coreType();
    out << ",\"mutatorLocal\":";
    write_local_id_or_null(out, localIds, mutator);
    out << ",\"mutatorCoreType\":";
    if (mutator == nullptr)
    {
        out << "null";
    }
    else
    {
        out << mutator->coreType();
    }

    auto feather = paint->feather();
    out << ",\"featherLocal\":";
    write_local_id_or_null(out, localIds, feather);
    out << ",\"featherCoreType\":";
    if (feather == nullptr)
    {
        out << "null";
    }
    else
    {
        out << feather->coreType();
    }

    out << ",\"effects\":[";
    bool first = true;
    size_t effectIndex = 0;
    for (auto effect : *paint->effects())
    {
        if (!first)
        {
            out << ',';
        }
        first = false;
        write_stroke_effect(out, localIds, objects, effectIndex++, effect);
    }
    out << ']';

    out << ",\"gradientStops\":[";
    first = true;
    size_t stopIndex = 0;
    if (mutator != nullptr && mutator->is<rive::LinearGradient>())
    {
        auto gradient = mutator->as<rive::LinearGradient>();
        for (auto stop : gradient->m_stops)
        {
            if (!first)
            {
                out << ',';
            }
            first = false;
            out << "{\"index\":" << stopIndex++;
            out << ",\"localId\":";
            write_local_id_or_null(out, localIds, stop);
            out << ",\"coreType\":" << stop->coreType();
            out << '}';
        }
    }
    out << "]}";
}

void write_shape(std::ostream& out,
                 const LocalIds& localIds,
                 const std::vector<rive::Core*>& objects,
                 size_t localId,
                 rive::Shape* shape)
{
    out << "{\"localId\":" << localId;
    out << ",\"coreType\":" << shape->coreType();

    out << ",\"paths\":[";
    bool first = true;
    size_t pathIndex = 0;
    for (auto path : shape->paths())
    {
        if (!first)
        {
            out << ',';
        }
        first = false;
        out << "{\"index\":" << pathIndex++;
        out << ",\"localId\":";
        write_local_id_or_null(out, localIds, path);
        out << ",\"coreType\":" << path->coreType();
        out << '}';
    }
    out << ']';

    out << ",\"paints\":[";
    first = true;
    size_t paintIndex = 0;
    for (auto paint : shape->m_ShapePaints)
    {
        if (!first)
        {
            out << ',';
        }
        first = false;
        write_shape_paint(out, localIds, objects, paintIndex++, paint);
    }
    out << "]}";
}

rive::ShapePaintContainer* shape_paint_container_from(rive::Core* object)
{
    if (object == nullptr || !object->is<rive::Component>())
    {
        return nullptr;
    }

    switch (object->coreType())
    {
        case rive::Artboard::typeKey:
            return object->as<rive::Artboard>();
        case rive::LayoutComponent::typeKey:
            return object->as<rive::LayoutComponent>();
        case rive::Shape::typeKey:
            return object->as<rive::Shape>();
        case rive::TextStylePaint::typeKey:
            return object->as<rive::TextStylePaint>();
        case rive::ForegroundLayoutDrawable::typeKey:
            return object->as<rive::ForegroundLayoutDrawable>();
        case rive::TextInputCursor::typeKey:
            return object->as<rive::TextInputCursor>();
        case rive::TextInputSelection::typeKey:
            return object->as<rive::TextInputSelection>();
        case rive::TextInputText::typeKey:
            return object->as<rive::TextInputText>();
        case rive::TextInputSelectedText::typeKey:
            return object->as<rive::TextInputSelectedText>();
    }
    return nullptr;
}

void write_shape_paint_container(std::ostream& out,
                                 const LocalIds& localIds,
                                 const std::vector<rive::Core*>& objects,
                                 size_t localId,
                                 rive::Core* object,
                                 rive::ShapePaintContainer* container)
{
    out << "{\"localId\":" << localId;
    out << ",\"coreType\":" << object->coreType();
    out << ",\"paints\":[";
    bool first = true;
    size_t paintIndex = 0;
    for (auto paint : container->m_ShapePaints)
    {
        if (!first)
        {
            out << ',';
        }
        first = false;
        write_shape_paint(out, localIds, objects, paintIndex++, paint);
    }
    out << "]}";
}

void write_skin(std::ostream& out,
                const LocalIds& localIds,
                const std::vector<rive::Core*>& objects,
                size_t localId,
                rive::Skin* skin)
{
    out << "{\"localId\":" << localId;
    out << ",\"coreType\":" << skin->coreType();

    auto parent = skin->parent();
    bool parentIsSkinnable =
        parent != nullptr &&
        (parent->is<rive::Mesh>() || parent->is<rive::PointsPath>());
    out << ",\"skinnableLocal\":";
    if (parentIsSkinnable)
    {
        write_local_id_or_null(out, localIds, parent);
    }
    else
    {
        out << "null";
    }
    out << ",\"skinnableCoreType\":";
    if (parentIsSkinnable)
    {
        out << parent->coreType();
    }
    else
    {
        out << "null";
    }

    out << ",\"tendons\":[";
    bool first = true;
    size_t tendonIndex = 0;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        auto object = objects[i];
        if (object == nullptr || !object->is<rive::Tendon>())
        {
            continue;
        }

        auto tendon = object->as<rive::Tendon>();
        if (tendon->parent() != skin)
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;

        out << "{\"index\":" << tendonIndex++;
        out << ",\"localId\":";
        write_local_id_or_null(out, localIds, tendon);
        out << ",\"coreType\":" << tendon->coreType();
        out << ",\"boneLocal\":";
        write_local_id_or_null(out, localIds, tendon->bone());
        out << ",\"boneCoreType\":";
        if (tendon->bone() == nullptr)
        {
            out << "null";
        }
        else
        {
            out << tendon->bone()->coreType();
        }
        out << '}';
    }
    out << "]}";
}

void write_key_frame_or_null(std::ostream& out,
                             const rive::KeyFrame* keyFrame,
                             const ProbeOptions& options)
{
    if (keyFrame == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"coreType\":" << keyFrame->coreType();
    out << ",\"frame\":" << keyFrame->frame();
    out << ",\"seconds\":" << keyFrame->seconds();
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, keyFrame);
    }
    out << '}';
}

void write_keyed_property(std::ostream& out,
                          size_t index,
                          const rive::KeyedProperty* keyedProperty,
                          const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << keyedProperty->coreType();
    out << ",\"propertyKey\":" << keyedProperty->propertyKey();
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, keyedProperty);
    }
    out << ",\"firstKeyFrame\":";
    write_key_frame_or_null(out, keyedProperty->first(), options);
    out << '}';
}

void write_keyed_object(std::ostream& out,
                        size_t index,
                        const rive::KeyedObject* keyedObject,
                        const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << keyedObject->coreType();
    out << ",\"objectId\":" << keyedObject->objectId();
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, keyedObject);
    }
    out << ",\"keyedProperties\":[";
    for (size_t i = 0; i < keyedObject->numKeyedProperties(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_keyed_property(out, i, keyedObject->getProperty(i), options);
    }
    out << "]}";
}

void write_animation(std::ostream& out,
                     size_t index,
                     const rive::LinearAnimation* animation,
                     const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << animation->coreType();
    out << ",\"name\":";
    write_json_string(out, animation->name());
    out << ",\"fps\":" << animation->fps();
    out << ",\"duration\":" << animation->duration();
    out << ",\"speed\":" << animation->speed();
    out << ",\"loopValue\":" << animation->loopValue();
    out << ",\"workStart\":" << animation->workStart();
    out << ",\"workEnd\":" << animation->workEnd();
    out << ",\"enableWorkArea\":"
        << (animation->enableWorkArea() ? "true" : "false");
    out << ",\"quantize\":" << (animation->quantize() ? "true" : "false");
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, animation);
    }
    out << ",\"keyedObjects\":[";
    for (size_t i = 0; i < animation->numKeyedObjects(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_keyed_object(out, i, animation->getObject(i), options);
    }
    out << ']';
    out << '}';
}

void write_state_machine_component(std::ostream& out,
                                   size_t index,
                                   const rive::StateMachineComponent* component,
                                   const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << component->coreType();
    out << ",\"name\":";
    write_json_string(out, component->name());
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, component);
    }
}

void write_state_machine_input(std::ostream& out,
                               size_t index,
                               const rive::StateMachineInput* input,
                               const ProbeOptions& options)
{
    if (input == nullptr)
    {
        out << "null";
        return;
    }

    write_state_machine_component(out, index, input, options);
    out << '}';
}

void write_state_to_index_or_null(std::ostream& out,
                                  const rive::StateMachineLayer* layer,
                                  const rive::LayerState* state)
{
    if (layer == nullptr || state == nullptr)
    {
        out << "null";
        return;
    }

    for (size_t i = 0; i < layer->stateCount(); ++i)
    {
        if (layer->state(i) == state)
        {
            out << i;
            return;
        }
    }

    out << "null";
}

bool artboard_animation_index(const rive::Artboard* artboard,
                              const rive::LinearAnimation* animation,
                              size_t& index)
{
    if (artboard == nullptr || animation == nullptr)
    {
        return false;
    }

    for (size_t i = 0; i < artboard->animationCount(); ++i)
    {
        if (artboard->animation(i) == animation)
        {
            index = i;
            return true;
        }
    }

    return false;
}

bool blend_animation_index(const rive::BlendState* state,
                           const rive::BlendAnimation* animation,
                           size_t& index)
{
    if (state == nullptr || animation == nullptr)
    {
        return false;
    }

    const auto& animations = state->animations();
    for (size_t i = 0; i < animations.size(); ++i)
    {
        if (animations[i] == animation)
        {
            index = i;
            return true;
        }
    }

    return false;
}

void write_resolved_event_fields(std::ostream& out,
                                 const rive::Artboard* artboard,
                                 bool hasEventId,
                                 uint32_t eventId)
{
    out << ",\"eventId\":";
    if (hasEventId)
    {
        out << eventId;
    }
    else
    {
        out << "null";
    }

    const auto* objects = artboard == nullptr ? nullptr : &artboard->objects();
    rive::Core* coreEvent = hasEventId && objects != nullptr &&
                                    eventId < objects->size()
                                ? (*objects)[eventId]
                                : nullptr;
    if (coreEvent == nullptr || !coreEvent->isTypeOf(rive::EventBase::typeKey))
    {
        out << ",\"eventLocal\":null";
        out << ",\"eventCoreType\":null";
        return;
    }

    out << ",\"eventLocal\":" << eventId;
    out << ",\"eventCoreType\":" << coreEvent->coreType();
}

void write_transition_condition(std::ostream& out,
                                size_t index,
                                const rive::TransitionCondition* condition,
                                const ProbeOptions& options)
{
    if (condition == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"index\":" << index;
    out << ",\"coreType\":" << condition->coreType();
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, condition);
    }
    out << '}';
}

void write_state_machine_fire_action(
    std::ostream& out,
    const rive::Artboard* artboard,
    size_t index,
    const rive::StateMachineFireAction* action,
    const ProbeOptions& options)
{
    if (action == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"index\":" << index;
    out << ",\"coreType\":" << action->coreType();
    if (action->is<rive::StateMachineFireEvent>())
    {
        write_resolved_event_fields(
            out,
            artboard,
            true,
            action->as<rive::StateMachineFireEvent>()->eventId());
    }
    else
    {
        write_resolved_event_fields(out, artboard, false, 0);
    }
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, action);
    }
    out << '}';
}

void write_state_machine_fire_actions(
    std::ostream& out,
    const rive::Artboard* artboard,
    const rive::StateMachineLayerComponent* component,
    const ProbeOptions& options)
{
    const auto& fireActions = component->events();
    out << ",\"fireActionCount\":" << fireActions.size();
    out << ",\"fireActions\":[";
    for (size_t i = 0; i < fireActions.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_state_machine_fire_action(out, artboard, i, fireActions[i], options);
    }
    out << ']';
}

void write_listener_action(std::ostream& out,
                           const rive::Artboard* artboard,
                           size_t index,
                           const rive::ListenerAction* action,
                           const ProbeOptions& options);

void write_state_machine_layer_component_listener_actions(
    std::ostream& out,
    const rive::Artboard* artboard,
    const rive::StateMachineLayerComponent* component,
    const ProbeOptions& options);

void write_state_transition(std::ostream& out,
                            const rive::Artboard* artboard,
                            const rive::StateMachineLayer* layer,
                            const rive::LayerState* state,
                            size_t index,
                            const rive::StateTransition* transition,
                            const ProbeOptions& options)
{
    if (transition == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"index\":" << index;
    out << ",\"coreType\":" << transition->coreType();
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, transition);
    }
    out << ",\"stateToId\":" << transition->stateToId();
    out << ",\"stateToIndex\":";
    write_state_to_index_or_null(out, layer, transition->stateTo());
    out << ",\"stateToCoreType\":";
    if (transition->stateTo() == nullptr)
    {
        out << "null";
    }
    else
    {
        out << transition->stateTo()->coreType();
    }
    out << ",\"interpolatorId\":" << transition->interpolatorId();
    out << ",\"interpolatorCoreType\":";
    if (transition->interpolator() == nullptr)
    {
        out << "null";
    }
    else
    {
        out << transition->interpolator()->coreType();
    }
    out << ",\"exitBlendAnimationId\":";
    if (transition->is<rive::BlendStateTransition>())
    {
        out << transition->as<rive::BlendStateTransition>()
                   ->exitBlendAnimationId();
    }
    else
    {
        out << "null";
    }
    const rive::BlendState* blendState =
        state != nullptr && state->is<rive::BlendState>()
            ? state->as<rive::BlendState>()
            : nullptr;
    const rive::BlendAnimation* exitBlendAnimation =
        transition->is<rive::BlendStateTransition>()
            ? transition->as<rive::BlendStateTransition>()
                  ->exitBlendAnimation()
            : nullptr;
    size_t exitBlendAnimationIndex = 0;
    bool hasExitBlendAnimation = blend_animation_index(
        blendState, exitBlendAnimation, exitBlendAnimationIndex);
    out << ",\"exitBlendAnimationIndex\":";
    if (hasExitBlendAnimation)
    {
        out << exitBlendAnimationIndex;
    }
    else
    {
        out << "null";
    }
    out << ",\"exitBlendAnimationCoreType\":";
    if (hasExitBlendAnimation)
    {
        out << exitBlendAnimation->coreType();
    }
    else
    {
        out << "null";
    }
    const rive::LinearAnimation* exitAnimation =
        transition->is<rive::BlendStateTransition>()
            ? transition->as<rive::BlendStateTransition>()
                  ->exitTimeAnimation(state)
            : nullptr;
    size_t exitAnimationIndex = 0;
    bool hasExitAnimation =
        artboard_animation_index(artboard, exitAnimation, exitAnimationIndex);
    out << ",\"exitAnimationIndex\":";
    if (hasExitAnimation)
    {
        out << exitAnimationIndex;
    }
    else
    {
        out << "null";
    }
    out << ",\"exitAnimationCoreType\":";
    if (hasExitAnimation)
    {
        out << exitAnimation->coreType();
    }
    else
    {
        out << "null";
    }
    write_state_machine_fire_actions(out, artboard, transition, options);
    write_state_machine_layer_component_listener_actions(
        out, artboard, transition, options);
    out << ",\"conditionCount\":" << transition->conditionCount();
    out << ",\"conditions\":[";
    for (size_t i = 0; i < transition->conditionCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_transition_condition(out, i, transition->condition(i), options);
    }
    out << "]}";
}

void write_blend_animation(std::ostream& out,
                           const rive::Artboard* artboard,
                           size_t index,
                           const rive::BlendAnimation* animation,
                           const ProbeOptions& options)
{
    if (animation == nullptr)
    {
        out << "null";
        return;
    }

    size_t animationIndex = 0;
    bool hasAnimation =
        artboard_animation_index(artboard, animation->animation(), animationIndex);

    out << "{\"index\":" << index;
    out << ",\"coreType\":" << animation->coreType();
    out << ",\"animationId\":" << animation->animationId();
    out << ",\"animationIndex\":";
    if (hasAnimation)
    {
        out << animationIndex;
    }
    else
    {
        out << "null";
    }
    out << ",\"animationCoreType\":";
    if (hasAnimation)
    {
        out << animation->animation()->coreType();
    }
    else
    {
        out << "null";
    }
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, animation);
    }
    out << '}';
}

void write_layer_state(std::ostream& out,
                       const rive::Artboard* artboard,
                       const rive::StateMachineLayer* layer,
                       size_t index,
                       const rive::LayerState* state,
                       const ProbeOptions& options)
{
    if (state == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"index\":" << index;
    out << ",\"coreType\":" << state->coreType();
    out << ",\"animationId\":";
    if (state->is<rive::AnimationState>())
    {
        out << state->as<rive::AnimationState>()->animationId();
    }
    else
    {
        out << "null";
    }
    out << ",\"animationCoreType\":";
    if (state->is<rive::AnimationState>() &&
        state->as<rive::AnimationState>()->animation() != nullptr)
    {
        out << state->as<rive::AnimationState>()->animation()->coreType();
    }
    else
    {
        out << "null";
    }
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, state);
    }
    write_state_machine_fire_actions(out, artboard, state, options);
    write_state_machine_layer_component_listener_actions(out, artboard, state, options);
    out << ",\"transitionCount\":" << state->transitionCount();
    out << ",\"transitions\":[";
    for (size_t i = 0; i < state->transitionCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_state_transition(
            out, artboard, layer, state, i, state->transition(i), options);
    }
    out << ']';
    out << ",\"blendAnimations\":[";
    if (state->is<rive::BlendState>())
    {
        const auto& blendAnimations =
            state->as<rive::BlendState>()->animations();
        for (size_t i = 0; i < blendAnimations.size(); ++i)
        {
            if (i != 0)
            {
                out << ',';
            }
            write_blend_animation(out, artboard, i, blendAnimations[i], options);
        }
    }
    out << "]}";
}

void write_state_machine_layer(std::ostream& out,
                               const rive::Artboard* artboard,
                               size_t index,
                               const rive::StateMachineLayer* layer,
                               const ProbeOptions& options)
{
    if (layer == nullptr)
    {
        out << "null";
        return;
    }

    write_state_machine_component(out, index, layer, options);
    out << ",\"stateCount\":" << layer->stateCount();
    out << ",\"states\":[";
    for (size_t i = 0; i < layer->stateCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_layer_state(out, artboard, layer, i, layer->state(i), options);
    }
    out << ']';
    out << '}';
}

void write_listener_action(std::ostream& out,
                           const rive::Artboard* artboard,
                           size_t index,
                           const rive::ListenerAction* action,
                           const ProbeOptions& options)
{
    if (action == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"index\":" << index;
    out << ",\"coreType\":" << action->coreType();
    out << ",\"flags\":" << action->flags();
    if (action->is<rive::ListenerFireEvent>())
    {
        write_resolved_event_fields(
            out, artboard, true, action->as<rive::ListenerFireEvent>()->eventId());
    }
    else
    {
        write_resolved_event_fields(out, artboard, false, 0);
    }
    out << ",\"dataBindPathIds\":";
    write_core_data_bind_path_ids_or_null(out, const_cast<rive::ListenerAction*>(action));
    out << ",\"resolvedDataBindPathIds\":";
    write_core_data_bind_resolved_path_ids_or_null(
        out, const_cast<rive::ListenerAction*>(action));
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, action);
    }
    out << '}';
}

void write_state_machine_layer_component_listener_actions(
    std::ostream& out,
    const rive::Artboard* artboard,
    const rive::StateMachineLayerComponent* component,
    const ProbeOptions& options)
{
    const auto& actions = component->listenerActions();
    out << ",\"listenerActionCount\":" << actions.size();
    out << ",\"listenerActions\":[";
    for (size_t i = 0; i < actions.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_listener_action(out, artboard, i, actions[i].get(), options);
    }
    out << ']';
}

void write_listener_input_type(std::ostream& out,
                               size_t index,
                               const rive::ListenerInputType* inputType,
                               const ProbeOptions& options)
{
    if (inputType == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"index\":" << index;
    out << ",\"coreType\":" << inputType->coreType();
    out << ",\"listenerTypeValue\":" << inputType->listenerTypeValue();
    out << ",\"dataBindPathIds\":";
    write_core_data_bind_path_ids_or_null(
        out, const_cast<rive::ListenerInputType*>(inputType));
    out << ",\"resolvedDataBindPathIds\":";
    write_core_data_bind_resolved_path_ids_or_null(
        out, const_cast<rive::ListenerInputType*>(inputType));
    out << ",\"viewModelPathIdsBuffer\":";
    if (inputType->is<rive::ListenerInputTypeViewModel>() &&
        inputType->as<rive::ListenerInputTypeViewModel>()->dataBindPath() !=
            nullptr)
    {
        auto pathIds =
            inputType->as<rive::ListenerInputTypeViewModel>()
                ->viewModelPathIdsBuffer();
        write_u32_vector_or_null(out, &pathIds);
    }
    else
    {
        out << "null";
    }
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, inputType);
    }
    out << '}';
}

void write_state_machine_listener(std::ostream& out,
                                  const rive::Artboard* artboard,
                                  size_t index,
                                  const rive::StateMachineListener* listener,
                                  const ProbeOptions& options)
{
    if (listener == nullptr)
    {
        out << "null";
        return;
    }

    write_state_machine_component(out, index, listener, options);
    out << ",\"targetId\":" << listener->targetId();
    out << ",\"dataBindPathIds\":";
    write_core_data_bind_path_ids_or_null(
        out, const_cast<rive::StateMachineListener*>(listener));
    out << ",\"resolvedDataBindPathIds\":";
    write_core_data_bind_resolved_path_ids_or_null(
        out, const_cast<rive::StateMachineListener*>(listener));
    out << ",\"actionCount\":" << listener->actionCount();
    out << ",\"listenerInputTypeCount\":"
        << listener->listenerInputTypeCount();
    out << ",\"actions\":[";
    for (size_t i = 0; i < listener->actionCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_listener_action(out, artboard, i, listener->action(i), options);
    }
    out << ']';
    out << ",\"listenerInputTypes\":[";
    for (size_t i = 0; i < listener->listenerInputTypeCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_listener_input_type(out,
                                  i,
                                  listener->listenerInputType(i),
                                  options);
    }
    out << ']';
    out << '}';
}

void write_converter_interpolator_core_type(std::ostream& out,
                                            const char* fieldName,
                                            rive::DataConverter* converter)
{
    out << ",\"" << fieldName << "\":";
    rive::KeyFrameInterpolator* interpolator = nullptr;
    if (converter != nullptr && converter->is<rive::DataConverterRangeMapper>())
    {
        interpolator =
            converter->as<rive::DataConverterRangeMapper>()->interpolator();
    }
    else if (converter != nullptr &&
             converter->is<rive::DataConverterInterpolator>())
    {
        interpolator =
            converter->as<rive::DataConverterInterpolator>()->interpolator();
    }

    if (interpolator == nullptr)
    {
        out << "null";
    }
    else
    {
        out << interpolator->coreType();
    }
}

void write_converter_source_path_ids(std::ostream& out,
                                     const char* fieldName,
                                     rive::DataConverter* converter)
{
    out << ",\"" << fieldName << "\":";
    if (converter != nullptr &&
        converter->is<rive::DataConverterOperationViewModel>())
    {
        const auto& sourcePathIds =
            converter->as<rive::DataConverterOperationViewModel>()
                ->sourcePathIds();
        write_u32_vector_or_null(out, &sourcePathIds);
    }
    else
    {
        out << "null";
    }
}

bool data_converter_output_type(rive::DataConverter* converter,
                                rive::DataType& outputType,
                                std::vector<rive::DataConverter*>& visiting)
{
    if (converter == nullptr)
    {
        return false;
    }
    if (std::find(visiting.begin(), visiting.end(), converter) != visiting.end())
    {
        return false;
    }

    if (converter->is<rive::DataConverterGroup>())
    {
        visiting.push_back(converter);
        auto group = converter->as<rive::DataConverterGroup>();
        const auto& items = group->items();
        for (auto item = items.rbegin(); item != items.rend(); ++item)
        {
            rive::DataType itemOutputType = rive::DataType::none;
            if ((*item)->converter() == nullptr ||
                !data_converter_output_type((*item)->converter(),
                                            itemOutputType,
                                            visiting))
            {
                visiting.pop_back();
                return false;
            }
            if (itemOutputType != rive::DataType::input)
            {
                outputType = itemOutputType;
                visiting.pop_back();
                return true;
            }
        }
        outputType = rive::DataType::none;
        visiting.pop_back();
        return true;
    }

    outputType = converter->outputType();
    return true;
}

void write_converter_output_type(std::ostream& out,
                                 const char* fieldName,
                                 rive::DataConverter* converter)
{
    out << ",\"" << fieldName << "\":";
    rive::DataType outputType = rive::DataType::none;
    std::vector<rive::DataConverter*> visiting;
    if (!data_converter_output_type(converter, outputType, visiting))
    {
        out << "null";
        return;
    }
    out << static_cast<unsigned int>(outputType);
}

void write_data_bind_source_output_type(std::ostream& out,
                                        const char* fieldName,
                                        const rive::DataBind* dataBind)
{
    out << ",\"" << fieldName << "\":";
    if (dataBind == nullptr)
    {
        out << "null";
        return;
    }

    auto mutableDataBind = const_cast<rive::DataBind*>(dataBind);
    out << static_cast<unsigned int>(mutableDataBind->sourceOutputType());
}

void write_data_bind_output_type(std::ostream& out,
                                 const char* fieldName,
                                 const rive::DataBind* dataBind)
{
    out << ",\"" << fieldName << "\":";
    if (dataBind == nullptr)
    {
        out << "null";
        return;
    }

    rive::DataType converterOutputType = rive::DataType::none;
    std::vector<rive::DataConverter*> visiting;
    auto converter = dataBind->converter();
    if (converter != nullptr)
    {
        if (!data_converter_output_type(converter, converterOutputType, visiting))
        {
            out << "null";
            return;
        }
        if (converterOutputType != rive::DataType::input &&
            converterOutputType != rive::DataType::none)
        {
            out << static_cast<unsigned int>(converterOutputType);
            return;
        }
    }

    auto mutableDataBind = const_cast<rive::DataBind*>(dataBind);
    out << static_cast<unsigned int>(mutableDataBind->sourceOutputType());
}

void write_data_bind_lifecycle_flags(std::ostream& out,
                                     const rive::DataBind* dataBind)
{
    out << ",\"targetSupportsPush\":";
    if (dataBind == nullptr)
    {
        out << "null";
        out << ",\"usesPersistingList\":null";
        return;
    }

    auto mutableDataBind = const_cast<rive::DataBind*>(dataBind);
    auto targetSupportsPush = mutableDataBind->targetSupportsPush();
    out << (targetSupportsPush ? "true" : "false");
    out << ",\"usesPersistingList\":"
        << (mutableDataBind->toSource() && !targetSupportsPush ? "true"
                                                               : "false");
}

void write_converter_number_to_list_view_model(std::ostream& out,
                                               rive::DataConverter* converter)
{
    out << ",\"converterNumberToListViewModelCoreType\":";
    if (converter != nullptr && converter->is<rive::DataConverterNumberToList>())
    {
        auto numberToList = converter->as<rive::DataConverterNumberToList>();
        auto converterFile = numberToList->file();
        auto viewModel = converterFile == nullptr
                             ? nullptr
                             : converterFile->viewModel(numberToList->viewModelId());
        if (viewModel == nullptr)
        {
            out << "null";
        }
        else
        {
            out << viewModel->coreType();
        }
        out << ",\"converterNumberToListViewModelName\":";
        if (viewModel == nullptr)
        {
            out << "null";
        }
        else
        {
            write_json_string(out, viewModel->name());
        }
    }
    else
    {
        out << "null";
        out << ",\"converterNumberToListViewModelName\":null";
    }
}

void write_data_bind_context_source_path_ids(std::ostream& out,
                                             const char* fieldName,
                                             const rive::DataBind* dataBind)
{
    out << ",\"" << fieldName << "\":";
    if (dataBind != nullptr && dataBind->is<rive::DataBindContext>())
    {
        const auto& sourcePathIds =
            dataBind->as<rive::DataBindContext>()->sourcePathIds();
        write_u32_vector_or_null(out, &sourcePathIds);
    }
    else
    {
        out << "null";
    }
}

void write_data_bind_context_resolved_source_path_ids(
    std::ostream& out,
    const char* fieldName,
    const rive::DataBind* dataBind)
{
    out << ",\"" << fieldName << "\":";
    if (dataBind != nullptr && dataBind->is<rive::DataBindContext>())
    {
        auto context = const_cast<rive::DataBindContext*>(
            dataBind->as<rive::DataBindContext>());
        auto sourcePathIds = context->m_SourcePathIdsBuffer;
        auto isPathResolved = context->m_isPathResolved;
        context->resolvePath();
        write_u32_vector_or_null(out, &context->m_SourcePathIdsBuffer);
        context->m_SourcePathIdsBuffer = sourcePathIds;
        context->m_isPathResolved = isPathResolved;
    }
    else
    {
        out << "null";
    }
}

void write_converter_group_items(std::ostream& out,
                                 rive::DataConverter* converter)
{
    out << ",\"converterGroupItems\":[";
    if (converter != nullptr && converter->is<rive::DataConverterGroup>())
    {
        auto group = converter->as<rive::DataConverterGroup>();
        const auto& items = group->items();
        for (size_t i = 0; i < items.size(); ++i)
        {
            if (i != 0)
            {
                out << ',';
            }
            auto item = items[i];
            out << "{\"index\":" << i;
            out << ",\"coreType\":" << item->coreType();
            out << ",\"converterCoreType\":";
            if (item->converter() == nullptr)
            {
                out << "null";
            }
            else
            {
                out << item->converter()->coreType();
            }
            write_converter_output_type(out,
                                        "converterOutputType",
                                        item->converter());
            write_converter_interpolator_core_type(
                out,
                "interpolatorCoreType",
                item->converter());
            write_converter_source_path_ids(out,
                                            "converterSourcePathIds",
                                            item->converter());
            out << '}';
        }
    }
    out << ']';
}

void write_converter_formula_tokens(std::ostream& out,
                                    rive::DataConverter* converter,
                                    const ProbeOptions& options)
{
    out << ",\"converterFormulaTokens\":[";
    if (converter != nullptr && converter->is<rive::DataConverterFormula>())
    {
        auto formula = converter->as<rive::DataConverterFormula>();
        for (size_t i = 0; i < formula->m_outputQueue.size(); ++i)
        {
            if (i != 0)
            {
                out << ',';
            }
            auto token = formula->m_outputQueue[i];
            out << "{\"index\":" << i;
            out << ",\"coreType\":" << token->coreType();
            if (options.propertyValues && options.artboardPropertyValues)
            {
                write_registry_property_values(out, token);
            }
            out << '}';
        }
    }
    out << ']';
}

void write_state_machine_data_bind(std::ostream& out,
                                   size_t index,
                                   const rive::DataBind* dataBind,
                                   const ProbeOptions& options)
{
    if (dataBind == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"index\":" << index;
    out << ",\"coreType\":" << dataBind->coreType();
    out << ",\"propertyKey\":" << dataBind->propertyKey();
    out << ",\"flags\":" << dataBind->flags();
    out << ",\"converterId\":" << dataBind->converterId();
    out << ",\"converterCoreType\":";
    if (dataBind->converter() == nullptr)
    {
        out << "null";
    }
    else
    {
        out << dataBind->converter()->coreType();
    }
    write_converter_output_type(out, "converterOutputType", dataBind->converter());
    write_data_bind_source_output_type(
        out, "dataBindSourceOutputType", dataBind);
    write_data_bind_output_type(out, "dataBindOutputType", dataBind);
    write_data_bind_lifecycle_flags(out, dataBind);
    write_converter_interpolator_core_type(
        out,
        "converterInterpolatorCoreType",
        dataBind->converter());
    write_converter_source_path_ids(
        out, "converterSourcePathIds", dataBind->converter());
    write_converter_number_to_list_view_model(out, dataBind->converter());
    write_data_bind_context_source_path_ids(
        out, "dataBindContextSourcePathIds", dataBind);
    write_data_bind_context_resolved_source_path_ids(
        out, "resolvedDataBindContextSourcePathIds", dataBind);
    write_converter_group_items(out, dataBind->converter());
    write_converter_formula_tokens(out, dataBind->converter(), options);
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, dataBind);
    }
    out << '}';
}

void write_artboard_data_bind(std::ostream& out,
                              const LocalIds& localIds,
                              size_t index,
                              const rive::DataBind* dataBind,
                              const ProbeOptions& options)
{
    if (dataBind == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"index\":" << index;
    out << ",\"coreType\":" << dataBind->coreType();
    out << ",\"propertyKey\":" << dataBind->propertyKey();
    out << ",\"flags\":" << dataBind->flags();
    out << ",\"converterId\":" << dataBind->converterId();
    out << ",\"converterCoreType\":";
    if (dataBind->converter() == nullptr)
    {
        out << "null";
    }
    else
    {
        out << dataBind->converter()->coreType();
    }
    write_converter_output_type(out, "converterOutputType", dataBind->converter());
    write_data_bind_source_output_type(
        out, "dataBindSourceOutputType", dataBind);
    write_data_bind_output_type(out, "dataBindOutputType", dataBind);
    write_data_bind_lifecycle_flags(out, dataBind);
    write_converter_interpolator_core_type(
        out,
        "converterInterpolatorCoreType",
        dataBind->converter());
    write_converter_source_path_ids(
        out, "converterSourcePathIds", dataBind->converter());
    write_converter_number_to_list_view_model(out, dataBind->converter());
    write_data_bind_context_source_path_ids(
        out, "dataBindContextSourcePathIds", dataBind);
    write_data_bind_context_resolved_source_path_ids(
        out, "resolvedDataBindContextSourcePathIds", dataBind);
    write_converter_group_items(out, dataBind->converter());
    write_converter_formula_tokens(out, dataBind->converter(), options);
    out << ",\"targetCoreType\":";
    if (dataBind->target() == nullptr)
    {
        out << "null";
    }
    else
    {
        out << dataBind->target()->coreType();
    }
    out << ",\"targetLocal\":";
    write_local_id_or_null(out, localIds, dataBind->target());
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, dataBind);
    }
    out << '}';
}

uint16_t scripted_object_core_type(rive::ScriptedObject* object)
{
    switch (object->scriptProtocol())
    {
        case rive::ScriptProtocol::listenerAction:
            return static_cast<rive::ScriptedListenerAction*>(object)->coreType();
        case rive::ScriptProtocol::transitionCondition:
            return static_cast<rive::ScriptedTransitionCondition*>(object)
                ->coreType();
        default:
            return 0;
    }
}

void write_state_machine_scripted_object(std::ostream& out,
                                         size_t index,
                                         rive::ScriptedObject* object,
                                         const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << scripted_object_core_type(object);
    auto inputs = object->customProperties();
    out << ",\"inputCount\":" << inputs.size();
    out << ",\"inputs\":[";
    for (size_t inputIndex = 0; inputIndex < inputs.size(); ++inputIndex)
    {
        if (inputIndex != 0)
        {
            out << ',';
        }
        auto input = inputs[inputIndex];
        out << "{\"index\":" << inputIndex;
        out << ",\"coreType\":" << input->coreType();
        out << ",\"name\":";
        write_json_string(out, input->name());
        if (options.propertyValues && options.artboardPropertyValues)
        {
            write_registry_property_values(out, input);
        }
        out << '}';
    }
    out << ']';
    out << '}';
}

void write_state_machine(std::ostream& out,
                         const rive::Artboard* artboard,
                         size_t index,
                         const rive::StateMachine* stateMachine,
                         const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << stateMachine->coreType();
    out << ",\"name\":";
    write_json_string(out, stateMachine->name());
    out << ",\"layerCount\":" << stateMachine->layerCount();
    out << ",\"inputCount\":" << stateMachine->inputCount();
    out << ",\"listenerCount\":" << stateMachine->listenerCount();
    out << ",\"dataBindCount\":" << stateMachine->dataBindCount();
    auto scriptedObjects = stateMachine->scriptedObjects();
    out << ",\"scriptedObjectCount\":" << scriptedObjects.size();
    if (options.propertyValues && options.artboardPropertyValues)
    {
        write_registry_property_values(out, stateMachine);
    }
    out << ",\"layers\":[";
    for (size_t i = 0; i < stateMachine->layerCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_state_machine_layer(
            out, artboard, i, stateMachine->layer(i), options);
    }
    out << ']';
    out << ",\"inputs\":[";
    for (size_t i = 0; i < stateMachine->inputCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_state_machine_input(out, i, stateMachine->input(i), options);
    }
    out << ']';
    out << ",\"listeners\":[";
    for (size_t i = 0; i < stateMachine->listenerCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_state_machine_listener(
            out, artboard, i, stateMachine->listener(i), options);
    }
    out << ']';
    out << ",\"dataBinds\":[";
    for (size_t i = 0; i < stateMachine->dataBindCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_state_machine_data_bind(out, i, stateMachine->dataBind(i), options);
    }
    out << ']';
    out << ",\"scriptedObjects\":[";
    for (size_t i = 0; i < scriptedObjects.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_state_machine_scripted_object(out, i, scriptedObjects[i], options);
    }
    out << ']';
    out << '}';
}

void write_file_asset(std::ostream& out,
                      size_t index,
                      rive::FileAsset* asset,
                      const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << asset->coreType();
    out << ",\"name\":";
    write_json_string(out, asset->name());
    out << ",\"assetId\":" << asset->assetId();
    out << ",\"cdnBaseUrl\":";
    write_json_string(out, asset->cdnBaseUrl());
    out << ",\"cdnUuidString\":";
    write_json_string(out, asset->cdnUuidStr());
    out << ",\"fileExtension\":";
    write_json_string(out, asset->fileExtension());
    out << ",\"uniqueName\":";
    write_json_string(out, asset->uniqueName());
    out << ",\"uniqueFilename\":";
    write_json_string(out, asset->uniqueFilename());
    if (options.propertyValues)
    {
        write_registry_property_values(out, asset);
    }
    out << '}';
}

std::string missing_data_enum_key(
    const std::vector<rive::DataEnumValue*>& values)
{
    std::string missingKey = "__rive_probe_missing__";
    for (;;)
    {
        bool found = false;
        for (auto value : values)
        {
            if (value->key() == missingKey)
            {
                found = true;
                break;
            }
        }
        if (!found)
        {
            return missingKey;
        }
        missingKey += "_";
    }
}

void write_data_enum_lookup_arrays(std::ostream& out,
                                   rive::DataEnum* dataEnum,
                                   const char* keyLookupName,
                                   const char* indexLookupName)
{
    auto values = dataEnum == nullptr ? std::vector<rive::DataEnumValue*>()
                                      : dataEnum->values();
    auto missingKey = missing_data_enum_key(values);

    out << ",\"" << keyLookupName << "\":[";
    for (size_t i = 0; i < values.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        const auto key = values[i]->key();
        out << "{\"key\":";
        write_json_string(out, key);
        out << ",\"value\":";
        write_json_string(out, dataEnum->value(key));
        out << ",\"valueIndex\":" << dataEnum->valueIndex(key);
        out << '}';
    }
    if (!values.empty())
    {
        out << ',';
    }
    out << "{\"key\":";
    write_json_string(out, missingKey);
    out << ",\"value\":";
    auto missingKeyValue =
        dataEnum == nullptr ? std::string() : dataEnum->value(missingKey);
    write_json_string(out, missingKeyValue);
    out << ",\"valueIndex\":"
        << (dataEnum == nullptr ? -1 : dataEnum->valueIndex(missingKey));
    out << "}]";

    out << ",\"" << indexLookupName << "\":[";
    for (size_t i = 0; i < values.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        out << "{\"index\":" << i;
        out << ",\"value\":";
        write_json_string(out, dataEnum->value(static_cast<uint32_t>(i)));
        out << ",\"valueIndex\":"
            << dataEnum->valueIndex(static_cast<uint32_t>(i));
        out << '}';
    }
    if (!values.empty())
    {
        out << ',';
    }
    auto missingIndex = static_cast<uint32_t>(values.size());
    out << "{\"index\":" << missingIndex;
    out << ",\"value\":";
    auto missingIndexValue =
        dataEnum == nullptr ? std::string() : dataEnum->value(missingIndex);
    write_json_string(out, missingIndexValue);
    out << ",\"valueIndex\":"
        << (dataEnum == nullptr ? -1 : dataEnum->valueIndex(missingIndex));
    out << "}]";
}

void write_view_model_property(std::ostream& out,
                               size_t index,
                               rive::ViewModelProperty* property,
                               const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << property->coreType();
    out << ",\"name\":";
    write_json_string(out, property->name());
    if (options.propertyValues)
    {
        write_registry_property_values(out, property);
    }
    if (property->is<rive::ViewModelPropertyEnum>())
    {
        auto enumProperty = property->as<rive::ViewModelPropertyEnum>();
        write_data_enum_lookup_arrays(out,
                                      enumProperty->dataEnum(),
                                      "enumKeyLookups",
                                      "enumIndexLookups");
    }
    out << '}';
}

void write_view_model_instance_list_item(
    std::ostream& out,
    rive::File* file,
    size_t index,
    rive::ViewModelInstanceListItem* item,
    const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << item->coreType();
    out << ",\"viewModelId\":" << item->viewModelId();
    out << ",\"viewModelInstanceId\":" << item->viewModelInstanceId();
    out << ",\"referencedViewModelInstance\":";
    auto viewModelIndex = static_cast<size_t>(item->viewModelId());
    auto instanceIndex = static_cast<size_t>(item->viewModelInstanceId());
    auto viewModel = file != nullptr && viewModelIndex < file->viewModelCount()
                         ? file->viewModel(viewModelIndex)
                         : nullptr;
    auto instance = viewModel == nullptr ? nullptr : viewModel->instance(instanceIndex);
    if (instance == nullptr)
    {
        out << "null";
    }
    else
    {
        out << "{\"viewModelIndex\":" << viewModelIndex;
        out << ",\"instanceIndex\":" << instanceIndex;
        out << ",\"coreType\":" << instance->coreType();
        out << ",\"name\":";
        write_json_string(out, instance->name());
        out << ",\"viewModelId\":" << instance->viewModelId();
        out << '}';
    }
    if (options.propertyValues)
    {
        write_registry_property_values(out, item);
    }
    out << '}';
}

void write_view_model_instance_view_model_reference(
    std::ostream& out,
    rive::File* file,
    rive::ViewModelInstanceValue* value)
{
    out << ",\"referenceViewModelInstance\":";
    if (file == nullptr || !value->is<rive::ViewModelInstanceViewModel>())
    {
        out << "null";
        return;
    }

    auto reference =
        value->as<rive::ViewModelInstanceViewModel>()->referenceViewModelInstance();
    if (reference == nullptr)
    {
        out << "null";
        return;
    }

    for (size_t viewModelIndex = 0; viewModelIndex < file->viewModelCount();
         ++viewModelIndex)
    {
        auto viewModel = file->viewModel(viewModelIndex);
        if (viewModel == nullptr)
        {
            continue;
        }
        for (size_t instanceIndex = 0; instanceIndex < viewModel->instanceCount();
             ++instanceIndex)
        {
            auto instance = viewModel->instance(instanceIndex);
            if (instance != reference.get())
            {
                continue;
            }

            out << "{\"viewModelIndex\":" << viewModelIndex;
            out << ",\"instanceIndex\":" << instanceIndex;
            out << ",\"coreType\":" << instance->coreType();
            out << ",\"name\":";
            write_json_string(out, instance->name());
            out << ",\"viewModelId\":" << instance->viewModelId();
            out << '}';
            return;
        }
    }

    out << "null";
}

void write_view_model_instance_asset_snapshot_item(std::ostream& out,
                                                   size_t index,
                                                   rive::FileAsset* asset)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << asset->coreType();
    out << ",\"name\":";
    write_json_string(out, asset->name());
    out << ",\"assetId\":" << asset->assetId();
    out << ",\"cdnBaseUrl\":";
    write_json_string(out, asset->cdnBaseUrl());
    out << ",\"cdnUuidString\":";
    write_json_string(out, asset->cdnUuidStr());
    out << ",\"fileExtension\":";
    write_json_string(out, asset->fileExtension());
    out << ",\"uniqueName\":";
    write_json_string(out, asset->uniqueName());
    out << ",\"uniqueFilename\":";
    write_json_string(out, asset->uniqueFilename());
    out << '}';
}

void write_view_model_instance_asset_snapshot(
    std::ostream& out,
    rive::ViewModelInstanceValue* value)
{
    out << ",\"assetSnapshot\":[";
    if (value->is<rive::ViewModelInstanceAsset>())
    {
        const auto& assets =
            value->as<rive::ViewModelInstanceAsset>()->assets();
        for (size_t i = 0; i < assets.size(); ++i)
        {
            if (i != 0)
            {
                out << ',';
            }
            write_view_model_instance_asset_snapshot_item(
                out, i, assets[i].get());
        }
    }
    out << ']';

    out << ",\"resolvedAsset\":";
    if (!value->is<rive::ViewModelInstanceAsset>())
    {
        out << "null";
        return;
    }

    auto assetValue = value->as<rive::ViewModelInstanceAsset>();
    const auto& assets = assetValue->assets();
    auto assetIndex = static_cast<size_t>(assetValue->propertyValue());
    if (assetIndex >= assets.size())
    {
        out << "null";
        return;
    }
    write_view_model_instance_asset_snapshot_item(
        out, assetIndex, assets[assetIndex].get());
}

void write_manifest(std::ostream& out, rive::File* file)
{
    auto resolver = file->dataResolver();
    if (resolver == nullptr)
    {
        out << "null";
        return;
    }

    auto manifest = static_cast<rive::ManifestAsset*>(resolver);
    out << "{\"names\":[";
    std::vector<int> nameIds;
    for (const auto& entry : manifest->m_names)
    {
        nameIds.push_back(entry.first);
    }
    std::sort(nameIds.begin(), nameIds.end());
    for (size_t i = 0; i < nameIds.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        int id = nameIds[i];
        out << "{\"id\":" << id << ",\"value\":";
        write_json_string(out, manifest->m_names[id]);
        out << '}';
    }

    out << "],\"paths\":[";
    std::vector<int> pathIds;
    for (const auto& entry : manifest->m_paths)
    {
        pathIds.push_back(entry.first);
    }
    std::sort(pathIds.begin(), pathIds.end());
    for (size_t i = 0; i < pathIds.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        int id = pathIds[i];
        out << "{\"id\":" << id << ",\"path\":";
        write_u32_vector_or_null(out, &manifest->m_paths[id]);
        out << '}';
    }
    out << "]}";
}

void write_view_model_instance_value_runtime(
    std::ostream& out,
    rive::ViewModelInstanceValue* value)
{
    out << ",\"valueRuntime\":";
    if (value == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"dataType\":";
    if (value->is<rive::ViewModelInstanceNumber>())
    {
        rive::ViewModelInstanceNumberRuntime runtime(
            value->as<rive::ViewModelInstanceNumber>());
        out << static_cast<unsigned int>(runtime.dataType());
        out << ",\"numberValue\":" << runtime.value();
    }
    else if (value->is<rive::ViewModelInstanceString>())
    {
        rive::ViewModelInstanceStringRuntime runtime(
            value->as<rive::ViewModelInstanceString>());
        out << static_cast<unsigned int>(runtime.dataType());
        out << ",\"stringValue\":";
        write_json_string(out, runtime.value());
    }
    else if (value->is<rive::ViewModelInstanceBoolean>())
    {
        rive::ViewModelInstanceBooleanRuntime runtime(
            value->as<rive::ViewModelInstanceBoolean>());
        out << static_cast<unsigned int>(runtime.dataType());
        out << ",\"booleanValue\":"
            << (runtime.value() ? "true" : "false");
    }
    else if (value->is<rive::ViewModelInstanceColor>())
    {
        rive::ViewModelInstanceColorRuntime runtime(
            value->as<rive::ViewModelInstanceColor>());
        out << static_cast<unsigned int>(runtime.dataType());
        out << ",\"colorValue\":"
            << static_cast<uint32_t>(runtime.value());
    }
    else if (value->is<rive::ViewModelInstanceList>())
    {
        rive::ViewModelInstanceListRuntime runtime(
            value->as<rive::ViewModelInstanceList>());
        out << static_cast<unsigned int>(runtime.dataType());
        out << ",\"listSize\":" << runtime.size();
    }
    else if (value->is<rive::ViewModelInstanceEnum>())
    {
        rive::ViewModelInstanceEnumRuntime runtime(
            value->as<rive::ViewModelInstanceEnum>());
        out << static_cast<unsigned int>(runtime.dataType());
    }
    else if (value->is<rive::ViewModelInstanceTrigger>())
    {
        rive::ViewModelInstanceTriggerRuntime runtime(
            value->as<rive::ViewModelInstanceTrigger>());
        out << static_cast<unsigned int>(runtime.dataType());
        out << ",\"triggerCount\":"
            << value->as<rive::ViewModelInstanceTrigger>()->propertyValue();
    }
    else if (value->is<rive::ViewModelInstanceViewModel>())
    {
        out << static_cast<unsigned int>(rive::DataType::viewModel);
        out << ",\"viewModelIndex\":"
            << value->as<rive::ViewModelInstanceViewModel>()->propertyValue();
    }
    else if (value->is<rive::ViewModelInstanceSymbolListIndex>())
    {
        out << static_cast<unsigned int>(rive::DataType::symbolListIndex);
        out << ",\"integerValue\":"
            << value->as<rive::ViewModelInstanceSymbolListIndex>()
                   ->propertyValue();
    }
    else if (value->is<rive::ViewModelInstanceAssetImage>())
    {
        rive::ViewModelInstanceAssetImageRuntime runtime(
            value->as<rive::ViewModelInstanceAssetImage>());
        out << static_cast<unsigned int>(runtime.dataType());
        out << ",\"assetIndex\":"
            << value->as<rive::ViewModelInstanceAssetImage>()->propertyValue();
    }
    else if (value->is<rive::ViewModelInstanceArtboard>())
    {
        rive::ViewModelInstanceArtboardRuntime runtime(
            value->as<rive::ViewModelInstanceArtboard>());
        out << static_cast<unsigned int>(runtime.dataType());
        out << ",\"artboardIndex\":"
            << value->as<rive::ViewModelInstanceArtboard>()->propertyValue();
    }
    else
    {
        out << static_cast<unsigned int>(rive::DataType::none);
    }
    out << '}';
}

void write_view_model_instance_source_data_value(
    std::ostream& out,
    rive::ViewModelInstanceValue* value)
{
    out << ",\"sourceDataValue\":";
    if (value == nullptr)
    {
        out << "null";
        return;
    }

    out << "{\"dataType\":";
    if (value->is<rive::ViewModelInstanceNumber>())
    {
        out << static_cast<unsigned int>(rive::DataType::number);
        out << ",\"numberValue\":"
            << value->as<rive::ViewModelInstanceNumber>()->propertyValue();
    }
    else if (value->is<rive::ViewModelInstanceString>())
    {
        out << static_cast<unsigned int>(rive::DataType::string);
        out << ",\"stringValue\":";
        write_json_string(
            out, value->as<rive::ViewModelInstanceString>()->propertyValue());
    }
    else if (value->is<rive::ViewModelInstanceColor>())
    {
        out << static_cast<unsigned int>(rive::DataType::color);
        out << ",\"colorValue\":"
            << static_cast<uint32_t>(
                   value->as<rive::ViewModelInstanceColor>()->propertyValue());
    }
    else if (value->is<rive::ViewModelInstanceBoolean>())
    {
        out << static_cast<unsigned int>(rive::DataType::boolean);
        out << ",\"booleanValue\":"
            << (value->as<rive::ViewModelInstanceBoolean>()->propertyValue()
                    ? "true"
                    : "false");
    }
    else if (value->is<rive::ViewModelInstanceEnum>())
    {
        out << static_cast<unsigned int>(rive::DataType::enumType);
        auto enumValue = value->as<rive::ViewModelInstanceEnum>();
        out << ",\"integerValue\":" << enumValue->propertyValue();
        out << ",\"enumDataCoreType\":";
        auto property = enumValue->viewModelProperty();
        auto enumProperty =
            property != nullptr && property->is<rive::ViewModelPropertyEnum>()
                ? property->as<rive::ViewModelPropertyEnum>()
                : nullptr;
        auto dataEnum =
            enumProperty != nullptr ? enumProperty->dataEnum() : nullptr;
        if (dataEnum == nullptr)
        {
            out << "null";
        }
        else
        {
            out << dataEnum->coreType();
        }
    }
    else if (value->is<rive::ViewModelInstanceTrigger>())
    {
        out << static_cast<unsigned int>(rive::DataType::trigger);
        out << ",\"integerValue\":"
            << value->as<rive::ViewModelInstanceTrigger>()->propertyValue();
    }
    else if (value->is<rive::ViewModelInstanceList>())
    {
        out << static_cast<unsigned int>(rive::DataType::list);
        out << ",\"listSize\":"
            << value->as<rive::ViewModelInstanceList>()->listItems().size();
    }
    else if (value->is<rive::ViewModelInstanceSymbolListIndex>())
    {
        out << static_cast<unsigned int>(rive::DataType::symbolListIndex);
        out << ",\"integerValue\":"
            << value->as<rive::ViewModelInstanceSymbolListIndex>()
                   ->propertyValue();
    }
    else if (value->is<rive::ViewModelInstanceAssetImage>())
    {
        out << static_cast<unsigned int>(rive::DataType::assetImage);
        out << ",\"integerValue\":"
            << value->as<rive::ViewModelInstanceAssetImage>()->propertyValue();
    }
    else if (value->is<rive::ViewModelInstanceArtboard>())
    {
        out << static_cast<unsigned int>(rive::DataType::artboard);
        out << ",\"integerValue\":"
            << value->as<rive::ViewModelInstanceArtboard>()->propertyValue();
    }
    else if (value->is<rive::ViewModelInstanceViewModel>())
    {
        out << static_cast<unsigned int>(rive::DataType::viewModel);
    }
    else
    {
        out << static_cast<unsigned int>(rive::DataType::none);
    }
    out << '}';
}

void write_view_model_instance_value(std::ostream& out,
                                     rive::File* file,
                                     size_t index,
                                     rive::ViewModelInstanceValue* value,
                                     const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << value->coreType();
    out << ",\"viewModelPropertyId\":" << value->viewModelPropertyId();
    out << ",\"name\":";
    write_json_string(out, value->name());
    out << ",\"viewModelPropertyCoreType\":";
    if (value->viewModelProperty() == nullptr)
    {
        out << "null";
    }
    else
    {
        out << value->viewModelProperty()->coreType();
    }
    out << ",\"viewModelPropertyName\":";
    if (value->viewModelProperty() == nullptr)
    {
        out << "null";
    }
    else
    {
        write_json_string(out, value->viewModelProperty()->name());
    }
    out << ",\"enumRuntime\":";
    if (value->is<rive::ViewModelInstanceEnum>() &&
        value->viewModelProperty() != nullptr &&
        value->viewModelProperty()->is<rive::ViewModelPropertyEnum>() &&
        value->viewModelProperty()
                ->as<rive::ViewModelPropertyEnum>()
                ->dataEnum() != nullptr)
    {
        rive::ViewModelInstanceEnumRuntime runtime(
            value->as<rive::ViewModelInstanceEnum>());
        out << "{\"value\":";
        write_json_string(out, runtime.value());
        out << ",\"valueIndex\":" << runtime.valueIndex();
        out << ",\"values\":[";
        auto values = runtime.values();
        for (size_t i = 0; i < values.size(); ++i)
        {
            if (i != 0)
            {
                out << ',';
            }
            write_json_string(out, values[i]);
        }
        out << "],\"enumType\":";
        write_json_string(out, runtime.enumType());
        out << '}';
    }
    else
    {
        out << "null";
    }
    write_view_model_instance_value_runtime(out, value);
    write_view_model_instance_source_data_value(out, value);
    if (options.propertyValues)
    {
        write_registry_property_values(out, value);
    }
    write_view_model_instance_view_model_reference(out, file, value);
    write_view_model_instance_asset_snapshot(out, value);

    out << ",\"items\":[";
    if (value->is<rive::ViewModelInstanceList>())
    {
        auto items = value->as<rive::ViewModelInstanceList>()->listItems();
        for (size_t i = 0; i < items.size(); ++i)
        {
            if (i != 0)
            {
                out << ',';
            }
            write_view_model_instance_list_item(
                out, file, i, items[i].get(), options);
        }
    }
    out << "]}";
}

void write_view_model_instance(std::ostream& out,
                               rive::File* file,
                               size_t index,
                               rive::ViewModelInstance* instance,
                               const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << instance->coreType();
    out << ",\"name\":";
    write_json_string(out, instance->name());
    out << ",\"viewModelId\":" << instance->viewModelId();
    if (options.propertyValues)
    {
        write_registry_property_values(out, instance);
    }
    if (options.completeViewModelProperties && file != nullptr)
    {
        file->completeViewModelProperties(instance);
    }

    auto values = instance->propertyValues();
    out << ",\"values\":[";
    for (size_t i = 0; i < values.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_view_model_instance_value(out, file, i, values[i].get(), options);
    }
    out << ']';

    out << ",\"valueIdLookups\":[";
    auto writeValueIdLookup = [&](uint32_t propertyId) {
        auto lookup = instance->propertyValue(propertyId);
        out << "{\"propertyId\":" << propertyId;
        out << ",\"resultIndex\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            auto found = values.size();
            for (size_t valueIndex = 0; valueIndex < values.size(); ++valueIndex)
            {
                if (values[valueIndex].get() == lookup)
                {
                    found = valueIndex;
                    break;
                }
            }
            if (found == values.size())
            {
                out << "null";
            }
            else
            {
                out << found;
            }
        }
        out << ",\"coreType\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            out << lookup->coreType();
        }
        out << '}';
    };
    for (size_t i = 0; i < values.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        writeValueIdLookup(values[i]->viewModelPropertyId());
    }
    uint32_t missingPropertyId = 0xFFFFFFFFu;
    bool foundMissingPropertyId = true;
    while (foundMissingPropertyId && missingPropertyId > 0)
    {
        foundMissingPropertyId = false;
        for (size_t i = 0; i < values.size(); ++i)
        {
            if (values[i]->viewModelPropertyId() == missingPropertyId)
            {
                foundMissingPropertyId = true;
                --missingPropertyId;
                break;
            }
        }
    }
    if (!values.empty())
    {
        out << ',';
    }
    writeValueIdLookup(missingPropertyId);
    out << ']';

    out << ",\"valueNameLookups\":[";
    bool firstLookup = true;
    for (size_t i = 0; i < values.size(); ++i)
    {
        auto property = values[i]->viewModelProperty();
        if (property == nullptr)
        {
            continue;
        }
        if (!firstLookup)
        {
            out << ',';
        }
        firstLookup = false;
        auto lookup = instance->propertyValue(property->name());
        out << "{\"name\":";
        write_json_string(out, property->name());
        out << ",\"resultIndex\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            auto found = values.size();
            for (size_t valueIndex = 0; valueIndex < values.size(); ++valueIndex)
            {
                if (values[valueIndex].get() == lookup)
                {
                    found = valueIndex;
                    break;
                }
            }
            if (found == values.size())
            {
                out << "null";
            }
            else
            {
                out << found;
            }
        }
        out << ",\"coreType\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            out << lookup->coreType();
        }
        out << '}';
    }
    if (!firstLookup)
    {
        out << ',';
    }
    out << "{\"name\":\"__rive_probe_missing__\",\"resultIndex\":null,\"coreType\":null}";
    out << ']';

    out << ",\"valueSymbolLookups\":[";
    for (uint16_t symbol = 0; symbol <= 16; ++symbol)
    {
        if (symbol != 0)
        {
            out << ',';
        }
        auto lookup =
            instance->propertyValue(static_cast<rive::SymbolType>(symbol));
        out << "{\"symbol\":" << symbol;
        out << ",\"resultIndex\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            auto found = values.size();
            for (size_t valueIndex = 0; valueIndex < values.size(); ++valueIndex)
            {
                if (values[valueIndex].get() == lookup)
                {
                    found = valueIndex;
                    break;
                }
            }
            if (found == values.size())
            {
                out << "null";
            }
            else
            {
                out << found;
            }
        }
        out << ",\"coreType\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            out << lookup->coreType();
        }
        out << '}';
    }
    out << "]}";
}

void write_view_model(std::ostream& out,
                      rive::File* file,
                      size_t index,
                      rive::ViewModel* viewModel,
                      const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << viewModel->coreType();
    out << ",\"name\":";
    write_json_string(out, viewModel->name());
    if (options.propertyValues)
    {
        write_registry_property_values(out, viewModel);
    }

    auto properties = viewModel->properties();
    out << ",\"properties\":[";
    for (size_t i = 0; i < properties.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_view_model_property(out, i, properties[i], options);
    }
    out << ']';

    out << ",\"propertyNameLookups\":[";
    for (size_t i = 0; i < properties.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        auto lookup = viewModel->property(properties[i]->name());
        out << "{\"name\":";
        write_json_string(out, properties[i]->name());
        out << ",\"resultIndex\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            auto found = properties.size();
            for (size_t propertyIndex = 0; propertyIndex < properties.size();
                 ++propertyIndex)
            {
                if (properties[propertyIndex] == lookup)
                {
                    found = propertyIndex;
                    break;
                }
            }
            if (found == properties.size())
            {
                out << "null";
            }
            else
            {
                out << found;
            }
        }
        out << ",\"coreType\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            out << lookup->coreType();
        }
        out << '}';
    }
    if (!properties.empty())
    {
        out << ',';
    }
    out << "{\"name\":\"__rive_probe_missing__\",\"resultIndex\":null,\"coreType\":null}";
    out << ']';

    out << ",\"propertySymbolLookups\":[";
    for (uint16_t symbol = 0; symbol <= 16; ++symbol)
    {
        if (symbol != 0)
        {
            out << ',';
        }
        auto lookup = viewModel->property(static_cast<rive::SymbolType>(symbol));
        out << "{\"symbol\":" << symbol;
        out << ",\"resultIndex\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            auto found = properties.size();
            for (size_t propertyIndex = 0; propertyIndex < properties.size();
                 ++propertyIndex)
            {
                if (properties[propertyIndex] == lookup)
                {
                    found = propertyIndex;
                    break;
                }
            }
            if (found == properties.size())
            {
                out << "null";
            }
            else
            {
                out << found;
            }
        }
        out << ",\"coreType\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            out << lookup->coreType();
        }
        out << '}';
    }
    out << ']';

    auto instances = viewModel->instances();
    out << ",\"instances\":[";
    for (size_t i = 0; i < instances.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_view_model_instance(out, file, i, instances[i], options);
    }
    out << ']';

    out << ",\"defaultInstance\":";
    auto defaultInstance = viewModel->defaultInstance();
    if (defaultInstance == nullptr)
    {
        out << "null";
    }
    else
    {
        auto found = instances.size();
        for (size_t instanceIndex = 0; instanceIndex < instances.size();
             ++instanceIndex)
        {
            if (instances[instanceIndex] == defaultInstance)
            {
                found = instanceIndex;
                break;
            }
        }
        if (found == instances.size())
        {
            out << "null";
        }
        else
        {
            out << "{\"viewModelIndex\":" << index;
            out << ",\"instanceIndex\":" << found;
            out << ",\"coreType\":" << defaultInstance->coreType();
            out << ",\"name\":";
            write_json_string(out, defaultInstance->name());
            out << ",\"viewModelId\":" << defaultInstance->viewModelId();
            out << '}';
        }
    }

    out << ",\"instanceNameLookups\":[";
    for (size_t i = 0; i < instances.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        auto lookup = viewModel->instance(instances[i]->name());
        out << "{\"name\":";
        write_json_string(out, instances[i]->name());
        out << ",\"resultIndex\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            auto found = instances.size();
            for (size_t instanceIndex = 0; instanceIndex < instances.size();
                 ++instanceIndex)
            {
                if (instances[instanceIndex] == lookup)
                {
                    found = instanceIndex;
                    break;
                }
            }
            if (found == instances.size())
            {
                out << "null";
            }
            else
            {
                out << found;
            }
        }
        out << ",\"coreType\":";
        if (lookup == nullptr)
        {
            out << "null";
        }
        else
        {
            out << lookup->coreType();
        }
        out << '}';
    }
    if (!instances.empty())
    {
        out << ',';
    }
    out << "{\"name\":\"__rive_probe_missing__\",\"resultIndex\":null,\"coreType\":null}";
    out << "]}";
}

void write_data_enum_value(std::ostream& out,
                           size_t index,
                           rive::DataEnumValue* value,
                           const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << value->coreType();
    out << ",\"key\":";
    write_json_string(out, value->key());
    out << ",\"value\":";
    write_json_string(out, value->value());
    if (options.propertyValues)
    {
        write_registry_property_values(out, value);
    }
    out << '}';
}

void write_data_enum(std::ostream& out,
                     size_t index,
                     rive::DataEnum* dataEnum,
                     const ProbeOptions& options)
{
    out << "{\"index\":" << index;
    out << ",\"coreType\":" << dataEnum->coreType();
    out << ",\"name\":";
    write_json_string(out, dataEnum->enumName());
    if (options.propertyValues)
    {
        write_registry_property_values(out, dataEnum);
    }

    auto values = dataEnum->values();
    out << ",\"values\":[";
    for (size_t i = 0; i < values.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_data_enum_value(out, i, values[i], options);
    }
    out << ']';

    write_data_enum_lookup_arrays(out, dataEnum, "keyLookups", "indexLookups");
    out << '}';
}

void complete_view_model_properties(rive::File* file)
{
    for (size_t viewModelIndex = 0; viewModelIndex < file->viewModelCount();
         ++viewModelIndex)
    {
        auto viewModel = file->viewModel(viewModelIndex);
        if (viewModel == nullptr)
        {
            continue;
        }
        for (size_t instanceIndex = 0; instanceIndex < viewModel->instanceCount();
             ++instanceIndex)
        {
            auto instance = viewModel->instance(instanceIndex);
            if (instance != nullptr)
            {
                file->completeViewModelProperties(instance);
            }
        }
    }
}

void write_data_context_instance_ref(std::ostream& out,
                                     rive::File* file,
                                     rive::ViewModelInstance* instance)
{
    if (instance == nullptr)
    {
        out << "null";
        return;
    }

    for (size_t viewModelIndex = 0; viewModelIndex < file->viewModelCount();
         ++viewModelIndex)
    {
        auto viewModel = file->viewModel(viewModelIndex);
        if (viewModel == nullptr)
        {
            continue;
        }
        for (size_t instanceIndex = 0; instanceIndex < viewModel->instanceCount();
             ++instanceIndex)
        {
            auto candidate = viewModel->instance(instanceIndex);
            if (candidate != instance)
            {
                continue;
            }

            out << "{\"viewModelIndex\":" << viewModelIndex;
            out << ",\"instanceIndex\":" << instanceIndex;
            out << ",\"coreType\":" << instance->coreType();
            out << ",\"name\":";
            write_json_string(out, instance->name());
            out << ",\"viewModelId\":" << instance->viewModelId();
            out << '}';
            return;
        }
    }

    out << "null";
}

void write_data_context_value_ref(std::ostream& out,
                                  rive::File* file,
                                  rive::ViewModelInstanceValue* value)
{
    if (value == nullptr)
    {
        out << "null";
        return;
    }

    for (size_t viewModelIndex = 0; viewModelIndex < file->viewModelCount();
         ++viewModelIndex)
    {
        auto viewModel = file->viewModel(viewModelIndex);
        if (viewModel == nullptr)
        {
            continue;
        }
        for (size_t instanceIndex = 0; instanceIndex < viewModel->instanceCount();
             ++instanceIndex)
        {
            auto instance = viewModel->instance(instanceIndex);
            if (instance == nullptr)
            {
                continue;
            }
            auto values = instance->propertyValues();
            for (size_t valueIndex = 0; valueIndex < values.size(); ++valueIndex)
            {
                if (values[valueIndex].get() != value)
                {
                    continue;
                }

                out << "{\"viewModelIndex\":" << viewModelIndex;
                out << ",\"instanceIndex\":" << instanceIndex;
                out << ",\"valueIndex\":" << valueIndex;
                out << ",\"coreType\":" << value->coreType();
                out << ",\"viewModelPropertyId\":"
                    << value->viewModelPropertyId();
                out << ",\"name\":";
                write_json_string(out, value->name());
                out << '}';
                return;
            }
        }
    }

    out << "null";
}

bool data_context_manifest_name_id(rive::File* file,
                                   const std::string& name,
                                   uint32_t* id)
{
    auto resolver = file->dataResolver();
    if (resolver == nullptr)
    {
        return false;
    }
    auto manifest = static_cast<rive::ManifestAsset*>(resolver);
    for (const auto& entry : manifest->m_names)
    {
        if (entry.second == name)
        {
            *id = static_cast<uint32_t>(entry.first);
            return true;
        }
    }
    return false;
}

void write_data_context_lookup(std::ostream& out,
                               bool& first,
                               const char* kind,
                               size_t currentViewModelIndex,
                               size_t currentInstanceIndex,
                               bool hasParent,
                               size_t parentViewModelIndex,
                               size_t parentInstanceIndex,
                               const std::vector<uint32_t>& path,
                               rive::File* file,
                               rive::ViewModelInstanceValue* value,
                               rive::ViewModelInstance* instance)
{
    if (!first)
    {
        out << ',';
    }
    first = false;

    out << "{\"kind\":";
    write_json_string(out, kind);
    out << ",\"currentViewModelIndex\":" << currentViewModelIndex;
    out << ",\"currentInstanceIndex\":" << currentInstanceIndex;
    out << ",\"parentViewModelIndex\":";
    if (hasParent)
    {
        out << parentViewModelIndex;
    }
    else
    {
        out << "null";
    }
    out << ",\"parentInstanceIndex\":";
    if (hasParent)
    {
        out << parentInstanceIndex;
    }
    else
    {
        out << "null";
    }
    out << ",\"path\":";
    write_u32_vector_or_null(out, &path);
    out << ",\"value\":";
    write_data_context_value_ref(out, file, value);
    out << ",\"instance\":";
    write_data_context_instance_ref(out, file, instance);
    out << '}';
}

void collect_data_context_absolute_lookups(std::ostream& out,
                                           bool& first,
                                           rive::File* file,
                                           rive::ViewModelInstance* root,
                                           size_t rootViewModelIndex,
                                           size_t rootInstanceIndex,
                                           rive::ViewModelInstance* instance,
                                           std::vector<uint32_t> path,
                                           size_t depth)
{
    if (depth > 8 || instance == nullptr)
    {
        return;
    }

    rive::DataContext context(rive::ref_rcp(root));
    auto instanceResult = context.getViewModelInstance(path);
    write_data_context_lookup(out,
                              first,
                              "absoluteInstance",
                              rootViewModelIndex,
                              rootInstanceIndex,
                              false,
                              0,
                              0,
                              path,
                              file,
                              nullptr,
                              instanceResult.get());

    auto values = instance->propertyValues();
    for (auto& value : values)
    {
        std::vector<uint32_t> valuePath = path;
        valuePath.push_back(value->viewModelPropertyId());
        rive::DataContext propertyContext(rive::ref_rcp(root));
        write_data_context_lookup(out,
                                  first,
                                  "absoluteProperty",
                                  rootViewModelIndex,
                                  rootInstanceIndex,
                                  false,
                                  0,
                                  0,
                                  valuePath,
                                  file,
                                  propertyContext.getViewModelProperty(valuePath),
                                  nullptr);

        if (value->is<rive::ViewModelInstanceViewModel>())
        {
            auto reference =
                value->as<rive::ViewModelInstanceViewModel>()
                    ->referenceViewModelInstance();
            if (reference != nullptr)
            {
                rive::DataContext instanceContext(rive::ref_rcp(root));
                auto nestedInstance =
                    instanceContext.getViewModelInstance(valuePath);
                write_data_context_lookup(out,
                                          first,
                                          "absoluteInstance",
                                          rootViewModelIndex,
                                          rootInstanceIndex,
                                          false,
                                          0,
                                          0,
                                          valuePath,
                                          file,
                                          nullptr,
                                          nestedInstance.get());
                collect_data_context_absolute_lookups(out,
                                                      first,
                                                      file,
                                                      root,
                                                      rootViewModelIndex,
                                                      rootInstanceIndex,
                                                      reference.get(),
                                                      valuePath,
                                                      depth + 1);
            }
        }
    }
}

void collect_view_model_instance_property_from_path_lookups(
    std::ostream& out,
    bool& first,
    rive::File* file,
    rive::ViewModelInstance* root,
    size_t rootViewModelIndex,
    size_t rootInstanceIndex,
    rive::ViewModelInstance* instance,
    std::vector<uint32_t> path,
    size_t depth)
{
    if (depth > 8 || root == nullptr || instance == nullptr)
    {
        return;
    }

    auto values = instance->propertyValues();
    for (auto& value : values)
    {
        std::vector<uint32_t> valuePath = path;
        valuePath.push_back(value->viewModelPropertyId());
        std::vector<uint32_t> lookupPath = valuePath;
        auto property = root->propertyFromPath(&lookupPath, 0);
        write_data_context_lookup(out,
                                  first,
                                  "propertyFromPath",
                                  rootViewModelIndex,
                                  rootInstanceIndex,
                                  false,
                                  0,
                                  0,
                                  valuePath,
                                  file,
                                  property,
                                  nullptr);

        if (value->is<rive::ViewModelInstanceViewModel>())
        {
            auto reference =
                value->as<rive::ViewModelInstanceViewModel>()
                    ->referenceViewModelInstance();
            if (reference != nullptr)
            {
                collect_view_model_instance_property_from_path_lookups(
                    out,
                    first,
                    file,
                    root,
                    rootViewModelIndex,
                    rootInstanceIndex,
                    reference.get(),
                    valuePath,
                    depth + 1);
            }
        }
    }
}

void collect_data_context_relative_lookups(std::ostream& out,
                                           bool& first,
                                           rive::File* file,
                                           rive::ViewModelInstance* root,
                                           size_t rootViewModelIndex,
                                           size_t rootInstanceIndex,
                                           rive::ViewModelInstance* instance,
                                           std::vector<uint32_t> path,
                                           size_t depth)
{
    if (depth > 8 || instance == nullptr || file->dataResolver() == nullptr)
    {
        return;
    }

    auto values = instance->propertyValues();
    for (auto& value : values)
    {
        auto property = value->viewModelProperty();
        if (property == nullptr)
        {
            continue;
        }
        uint32_t nameId = 0;
        if (!data_context_manifest_name_id(file, property->name(), &nameId))
        {
            continue;
        }

        std::vector<uint32_t> valuePath = path;
        valuePath.push_back(nameId);
        rive::DataContext propertyContext(rive::ref_rcp(root));
        write_data_context_lookup(
            out,
            first,
            "relativeProperty",
            rootViewModelIndex,
            rootInstanceIndex,
            false,
            0,
            0,
            valuePath,
            file,
            propertyContext.getRelativeViewModelProperty(valuePath,
                                                         file->dataResolver()),
            nullptr);

        if (value->is<rive::ViewModelInstanceViewModel>())
        {
            auto reference =
                value->as<rive::ViewModelInstanceViewModel>()
                    ->referenceViewModelInstance();
            if (reference != nullptr)
            {
                rive::DataContext instanceContext(rive::ref_rcp(root));
                auto nestedInstance =
                    instanceContext.getRelativeViewModelInstance(
                        valuePath,
                        file->dataResolver());
                write_data_context_lookup(out,
                                          first,
                                          "relativeInstance",
                                          rootViewModelIndex,
                                          rootInstanceIndex,
                                          false,
                                          0,
                                          0,
                                          valuePath,
                                          file,
                                          nullptr,
                                          nestedInstance.get());
                collect_data_context_relative_lookups(out,
                                                      first,
                                                      file,
                                                      root,
                                                      rootViewModelIndex,
                                                      rootInstanceIndex,
                                                      reference.get(),
                                                      valuePath,
                                                      depth + 1);
            }
        }
    }
}

void write_data_context_parent_fallback_lookups(std::ostream& out,
                                                bool& first,
                                                rive::File* file)
{
    if (file->viewModelCount() < 2)
    {
        return;
    }

    for (size_t currentViewModelIndex = 0;
         currentViewModelIndex < file->viewModelCount();
         ++currentViewModelIndex)
    {
        auto currentViewModel = file->viewModel(currentViewModelIndex);
        if (currentViewModel == nullptr || currentViewModel->instanceCount() == 0)
        {
            continue;
        }
        auto currentInstance = currentViewModel->instance(0);

        for (size_t parentViewModelIndex = 0;
             parentViewModelIndex < file->viewModelCount();
             ++parentViewModelIndex)
        {
            if (parentViewModelIndex == currentViewModelIndex)
            {
                continue;
            }
            auto parentViewModel = file->viewModel(parentViewModelIndex);
            if (parentViewModel == nullptr ||
                parentViewModel->instanceCount() == 0)
            {
                continue;
            }
            auto parentInstance = parentViewModel->instance(0);
            auto parentValues = parentInstance->propertyValues();
            if (parentValues.empty())
            {
                continue;
            }

            auto parentContext = rive::make_rcp<rive::DataContext>(
                rive::ref_rcp(parentInstance));
            rive::DataContext context(rive::ref_rcp(currentInstance));
            context.parent(parentContext);

            std::vector<uint32_t> absolutePath = {
                parentInstance->viewModelId(),
                parentValues[0]->viewModelPropertyId(),
            };
            write_data_context_lookup(out,
                                      first,
                                      "absolutePropertyParentFallback",
                                      currentViewModelIndex,
                                      0,
                                      true,
                                      parentViewModelIndex,
                                      0,
                                      absolutePath,
                                      file,
                                      context.getViewModelProperty(absolutePath),
                                      nullptr);

            if (file->dataResolver() != nullptr &&
                parentValues[0]->viewModelProperty() != nullptr)
            {
                uint32_t nameId = 0;
                if (data_context_manifest_name_id(
                        file,
                        parentValues[0]->viewModelProperty()->name(),
                        &nameId))
                {
                    std::vector<uint32_t> relativePath = {nameId};
                    rive::DataContext relativeContext(
                        rive::ref_rcp(currentInstance));
                    relativeContext.parent(parentContext);
                    write_data_context_lookup(
                        out,
                        first,
                        "relativePropertyParentFallback",
                        currentViewModelIndex,
                        0,
                        true,
                        parentViewModelIndex,
                        0,
                        relativePath,
                        file,
                        relativeContext.getRelativeViewModelProperty(
                            relativePath,
                            file->dataResolver()),
                        nullptr);
                }
            }
            return;
        }
    }
}

void write_data_context_lookups(std::ostream& out,
                                rive::File* file,
                                const ProbeOptions& options)
{
    out << ",\"dataContextLookups\":[";
    bool first = true;
    if (options.dataContextLookups)
    {
        for (size_t viewModelIndex = 0; viewModelIndex < file->viewModelCount();
             ++viewModelIndex)
        {
            auto viewModel = file->viewModel(viewModelIndex);
            if (viewModel == nullptr)
            {
                continue;
            }
            for (size_t instanceIndex = 0;
                 instanceIndex < viewModel->instanceCount();
                 ++instanceIndex)
            {
                auto instance = viewModel->instance(instanceIndex);
                if (instance == nullptr)
                {
                    continue;
                }
                std::vector<uint32_t> absolutePath = {instance->viewModelId()};
                collect_data_context_absolute_lookups(out,
                                                      first,
                                                      file,
                                                      instance,
                                                      viewModelIndex,
                                                      instanceIndex,
                                                      instance,
                                                      absolutePath,
                                                      0);
                collect_view_model_instance_property_from_path_lookups(
                    out,
                    first,
                    file,
                    instance,
                    viewModelIndex,
                    instanceIndex,
                    instance,
                    {},
                    0);
                collect_data_context_relative_lookups(out,
                                                      first,
                                                      file,
                                                      instance,
                                                      viewModelIndex,
                                                      instanceIndex,
                                                      instance,
                                                      {},
                                                      0);
            }
        }
        write_data_context_parent_fallback_lookups(out, first, file);
    }
    out << ']';
}

void write_artboard_reference_or_null(std::ostream& out,
                                      rive::File* file,
                                      rive::Artboard* artboard)
{
    if (artboard == nullptr)
    {
        out << "null";
        return;
    }

    for (size_t artboardIndex = 0; artboardIndex < file->artboardCount();
         ++artboardIndex)
    {
        auto candidate = file->artboard(artboardIndex);
        if (candidate != artboard)
        {
            continue;
        }

        out << "{\"index\":" << artboardIndex;
        out << ",\"coreType\":" << artboard->coreType();
        out << ",\"name\":";
        write_json_string(out, artboard->name());
        out << ",\"viewModelId\":" << artboard->viewModelId();
        out << '}';
        return;
    }

    out << "null";
}

void write_artboard_component_list_item_artboards(
    std::ostream& out,
    rive::File* file,
    rive::ArtboardComponentList* componentList)
{
    out << '[';
    bool first = true;
    for (size_t viewModelIndex = 0; viewModelIndex < file->viewModelCount();
         ++viewModelIndex)
    {
        auto viewModel = file->viewModel(viewModelIndex);
        if (viewModel == nullptr)
        {
            continue;
        }
        for (size_t instanceIndex = 0;
             instanceIndex < viewModel->instanceCount();
             ++instanceIndex)
        {
            auto instance = viewModel->instance(instanceIndex);
            if (instance == nullptr)
            {
                continue;
            }
            auto values = instance->propertyValues();
            for (size_t valueIndex = 0; valueIndex < values.size(); ++valueIndex)
            {
                auto value = values[valueIndex].get();
                if (value == nullptr ||
                    !value->is<rive::ViewModelInstanceList>())
                {
                    continue;
                }

                auto list = value->as<rive::ViewModelInstanceList>();
                auto items = list->listItems();
                for (size_t itemIndex = 0; itemIndex < items.size(); ++itemIndex)
                {
                    auto item = items[itemIndex].get();
                    if (item == nullptr)
                    {
                        continue;
                    }

                    if (!first)
                    {
                        out << ',';
                    }
                    first = false;

                    out << "{\"ownerViewModelIndex\":" << viewModelIndex;
                    out << ",\"ownerInstanceIndex\":" << instanceIndex;
                    out << ",\"valueIndex\":" << valueIndex;
                    out << ",\"itemIndex\":" << itemIndex;
                    out << ",\"viewModelId\":" << item->viewModelId();
                    out << ",\"viewModelInstanceId\":"
                        << item->viewModelInstanceId();
                    out << ",\"artboard\":";

                    auto instanceCopy = file->createViewModelInstance(
                        static_cast<size_t>(item->viewModelId()),
                        static_cast<size_t>(item->viewModelInstanceId()));
                    rive::rcp<rive::ViewModelInstanceListItem> runtimeItem =
                        nullptr;
                    if (instanceCopy != nullptr)
                    {
                        runtimeItem = rive::rcp<rive::ViewModelInstanceListItem>(
                            file->viewModelInstanceListItem(instanceCopy,
                                                            nullptr));
                    }
                    auto artboard = runtimeItem == nullptr
                                        ? nullptr
                                        : componentList->findArtboard(runtimeItem);
                    write_artboard_reference_or_null(out, file, artboard);
                    out << '}';
                }
            }
        }
    }
    out << ']';
}

void write_artboard_component_list(std::ostream& out,
                                   size_t localId,
                                   rive::File* file,
                                   rive::ArtboardComponentList* componentList)
{
    out << "{\"localId\":" << localId;
    out << ",\"coreType\":" << componentList->coreType();
    out << ",\"mapRules\":[";
    std::vector<std::pair<int, int>> rules(componentList->m_artboardMapRules.begin(),
                                           componentList->m_artboardMapRules.end());
    std::sort(rules.begin(), rules.end());
    for (size_t i = 0; i < rules.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        out << "{\"viewModelId\":" << rules[i].first;
        out << ",\"artboardId\":" << rules[i].second;
        out << '}';
    }
    out << ']';
    out << ",\"itemArtboards\":";
    write_artboard_component_list_item_artboards(out, file, componentList);
    out << '}';
}

void write_artboard(std::ostream& out,
                    rive::File* file,
                    size_t index,
                    rive::Artboard* artboard,
                    const ProbeOptions& options)
{
    if (options.advanceArtboards)
    {
        artboard->advance(0.0f);
    }

    const auto& objects = artboard->objects();
    LocalIds localIds;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        if (objects[i] != nullptr)
        {
            localIds[objects[i]] = i;
        }
    }

    out << "{\"index\":" << index;
    out << ",\"name\":";
    write_json_string(out, artboard->name());
    out << ",\"width\":" << artboard->width();
    out << ",\"height\":" << artboard->height();
    out << ",\"objectCount\":" << objects.size();
    out << ",\"viewModel\":";
    auto viewModelIndex = static_cast<size_t>(artboard->viewModelId());
    auto viewModel = file != nullptr && viewModelIndex < file->viewModelCount()
                         ? file->viewModel(viewModelIndex)
                         : nullptr;
    if (viewModel == nullptr)
    {
        out << "null";
    }
    else
    {
        out << "{\"viewModelIndex\":" << viewModelIndex;
        out << ",\"coreType\":" << viewModel->coreType();
        out << ",\"name\":";
        write_json_string(out, viewModel->name());
        out << '}';
    }

    out << ",\"objects\":[";
    for (size_t i = 0; i < objects.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_object(out, file, localIds, i, objects[i], options);
    }
    out << ']';

    out << ",\"components\":[";
    bool first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        const rive::Core* object = objects[i];
        if (object == nullptr || !object->is<rive::Component>())
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_component(out, localIds, i, object->as<rive::Component>());
    }
    out << ']';

    out << ",\"artboardComponentLists\":[";
    first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        rive::Core* object = objects[i];
        if (object == nullptr || !object->is<rive::ArtboardComponentList>())
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_artboard_component_list(
            out, i, file, object->as<rive::ArtboardComponentList>());
    }
    out << ']';

    out << ",\"drawTargets\":[";
    first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        const rive::Core* object = objects[i];
        if (object == nullptr || !object->is<rive::DrawTarget>())
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_draw_target(out, localIds, i, object->as<rive::DrawTarget>());
    }
    out << ']';

    out << ",\"drawRules\":[";
    first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        const rive::Core* object = objects[i];
        if (object == nullptr || !object->is<rive::DrawRules>())
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_draw_rules(out, localIds, i, object->as<rive::DrawRules>());
    }
    out << ']';

    out << ",\"clippingShapes\":[";
    first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        const rive::Core* object = objects[i];
        if (object == nullptr || !object->is<rive::ClippingShape>())
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_clipping_shape(
            out, objects, localIds, i, object->as<rive::ClippingShape>());
    }
    out << ']';

    out << ",\"meshes\":[";
    first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        rive::Core* object = objects[i];
        if (object == nullptr || !object->is<rive::Mesh>())
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_mesh(out, localIds, objects, i, object->as<rive::Mesh>());
    }
    out << ']';

    out << ",\"paths\":[";
    first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        rive::Core* object = objects[i];
        if (object == nullptr || !object->is<rive::Path>())
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_path(out, localIds, objects, i, object->as<rive::Path>());
    }
    out << ']';

    out << ",\"shapes\":[";
    first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        rive::Core* object = objects[i];
        if (object == nullptr || !object->is<rive::Shape>())
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_shape(out, localIds, objects, i, object->as<rive::Shape>());
    }
    out << ']';

    out << ",\"nSlicerDetails\":[";
    first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        rive::Core* object = objects[i];
        if (object == nullptr || !object->is<rive::Component>())
        {
            continue;
        }
        auto component = object->as<rive::Component>();
        auto details = rive::NSlicerDetails::from(component);
        if (details == nullptr)
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_n_slicer_details(out, localIds, objects, i, component, details);
    }
    out << ']';

    out << ",\"shapePaintContainers\":[";
    first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        rive::Core* object = objects[i];
        auto container = shape_paint_container_from(object);
        if (container == nullptr || container->m_ShapePaints.empty())
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_shape_paint_container(out, localIds, objects, i, object, container);
    }
    out << ']';

    out << ",\"skins\":[";
    first = true;
    for (size_t i = 0; i < objects.size(); ++i)
    {
        rive::Core* object = objects[i];
        if (object == nullptr || !object->is<rive::Skin>())
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;
        write_skin(out, localIds, objects, i, object->as<rive::Skin>());
    }
    out << ']';

    out << ",\"animations\":[";
    for (size_t i = 0; i < artboard->animationCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_animation(out, i, artboard->animation(i), options);
    }
    out << ']';

    out << ",\"stateMachines\":[";
    for (size_t i = 0; i < artboard->stateMachineCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_state_machine(out, artboard, i, artboard->stateMachine(i), options);
    }
    out << ']';

    auto dataBinds = artboard->dataBinds();
    out << ",\"dataBinds\":[";
    for (size_t i = 0; i < dataBinds.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_artboard_data_bind(out, localIds, i, dataBinds[i], options);
    }
    out << "]}";
}

void write_file(std::ostream& out,
                const char* path,
                rive::File* file,
                const ProbeOptions& options)
{
    out << std::setprecision(9);
    out << "{\"path\":";
    write_json_string(out, path);
    out << ",\"artboardCount\":" << file->artboardCount();
    out << ",\"assets\":[";
    auto assets = file->assets();
    for (size_t i = 0; i < assets.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_file_asset(out, i, assets[i].get(), options);
    }
    out << ']';
    out << ",\"manifest\":";
    write_manifest(out, file);
    out << ",\"viewModels\":[";
    for (size_t i = 0; i < file->viewModelCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_view_model(out, file, i, file->viewModel(i), options);
    }
    out << ']';
    out << ",\"enums\":[";
    const auto& enums = file->enums();
    for (size_t i = 0; i < enums.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        write_data_enum(out, i, enums[i], options);
    }
    out << ']';
    write_data_context_lookups(out, file, options);
    out << ",\"artboards\":[";

    for (size_t i = 0; i < file->artboardCount(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }

        rive::Artboard* artboard = file->artboard(i);
        if (artboard == nullptr)
        {
            out << "null";
        }
        else
        {
            write_artboard(out, file, i, artboard, options);
        }
    }

    out << "]}\n";
}

void write_data_value(std::ostream& out, rive::DataValue* value)
{
    out << "{\"dataType\":";
    if (value == nullptr)
    {
        out << "null}";
        return;
    }

    if (value->is<rive::DataValueNumber>())
    {
        out << static_cast<unsigned int>(rive::DataType::number);
        out << ",\"numberValue\":" << std::setprecision(9)
            << value->as<rive::DataValueNumber>()->value();
    }
    else if (value->is<rive::DataValueString>())
    {
        out << static_cast<unsigned int>(rive::DataType::string);
        out << ",\"stringValue\":";
        write_json_string(out, value->as<rive::DataValueString>()->value());
    }
    else if (value->is<rive::DataValueBoolean>())
    {
        out << static_cast<unsigned int>(rive::DataType::boolean);
        out << ",\"booleanValue\":"
            << (value->as<rive::DataValueBoolean>()->value() ? "true"
                                                             : "false");
    }
    else if (value->is<rive::DataValueColor>())
    {
        out << static_cast<unsigned int>(rive::DataType::color);
        out << ",\"colorValue\":"
            << static_cast<uint32_t>(
                   value->as<rive::DataValueColor>()->value());
    }
    else if (value->is<rive::DataValueList>())
    {
        out << static_cast<unsigned int>(rive::DataType::list);
        auto list = value->as<rive::DataValueList>()->value();
        out << ",\"listSize\":" << list->size();
        out << ",\"listItems\":[";
        for (size_t i = 0; i < list->size(); ++i)
        {
            if (i != 0)
            {
                out << ',';
            }

            auto item = list->at(i);
            if (item.get() == nullptr ||
                item->viewModelInstance().get() == nullptr)
            {
                out << "null";
                continue;
            }

            auto instance = item->viewModelInstance().get();
            out << "{\"viewModelId\":" << instance->viewModelId();
            out << ",\"valueCoreTypes\":[";
            const auto& values = instance->propertyValues();
            for (size_t valueIndex = 0; valueIndex < values.size();
                 ++valueIndex)
            {
                if (valueIndex != 0)
                {
                    out << ',';
                }
                out << values[valueIndex]->coreType();
            }
            out << "]}";
        }
        out << ']';
    }
    else if (value->is<rive::DataValueEnum>())
    {
        out << static_cast<unsigned int>(rive::DataType::enumType);
        out << ",\"integerValue\":"
            << value->as<rive::DataValueEnum>()->value();
    }
    else if (value->is<rive::DataValueTrigger>())
    {
        out << static_cast<unsigned int>(rive::DataType::trigger);
        out << ",\"integerValue\":"
            << value->as<rive::DataValueTrigger>()->value();
    }
    else if (value->is<rive::DataValueSymbolListIndex>())
    {
        out << static_cast<unsigned int>(rive::DataType::symbolListIndex);
        out << ",\"integerValue\":"
            << value->as<rive::DataValueSymbolListIndex>()->value();
    }
    else if (value->is<rive::DataValueAssetImage>())
    {
        out << static_cast<unsigned int>(rive::DataType::assetImage);
        out << ",\"integerValue\":"
            << value->as<rive::DataValueAssetImage>()->value();
    }
    else if (value->is<rive::DataValueArtboard>())
    {
        out << static_cast<unsigned int>(rive::DataType::artboard);
        out << ",\"integerValue\":"
            << value->as<rive::DataValueArtboard>()->value();
    }
    else
    {
        out << static_cast<unsigned int>(rive::DataType::none);
    }
    out << '}';
}

void write_converter_sample_with_data_bind(std::ostream& out,
                                           bool& first,
                                           const char* label,
                                           rive::DataConverter* converter,
                                           rive::DataValue* input,
                                           rive::DataBind* dataBind)
{
    if (!first)
    {
        out << ',';
    }
    first = false;

    out << "{\"label\":";
    write_json_string(out, label);
    out << ",\"converterCoreType\":" << converter->coreType();
    out << ",\"output\":";
    write_data_value(out, converter->convert(input, dataBind));
    out << ",\"reverseOutput\":";
    write_data_value(out, converter->reverseConvert(input, dataBind));
    out << '}';
}

void write_converter_forward_sample(std::ostream& out,
                                    bool& first,
                                    const char* label,
                                    rive::DataConverter* converter,
                                    rive::DataValue* input)
{
    if (!first)
    {
        out << ',';
    }
    first = false;

    out << "{\"label\":";
    write_json_string(out, label);
    out << ",\"converterCoreType\":" << converter->coreType();
    out << ",\"output\":";
    write_data_value(out, converter->convert(input, nullptr));
    out << ",\"reverseOutput\":{\"dataType\":null}}";
}

void write_float_array(std::ostream& out, const std::vector<float>& values)
{
    out << '[';
    for (size_t i = 0; i < values.size(); ++i)
    {
        if (i != 0)
        {
            out << ',';
        }
        out << values[i];
    }
    out << ']';
}

void write_converter_sample_with_random_values(
    std::ostream& out,
    bool& first,
    const char* label,
    rive::DataConverter* converter,
    rive::DataValue* input,
    const std::vector<float>& randomValues,
    const std::vector<float>& reverseRandomValues)
{
    if (!first)
    {
        out << ',';
    }
    first = false;

    out << "{\"label\":";
    write_json_string(out, label);
    out << ",\"converterCoreType\":" << converter->coreType();
    out << ",\"randomValues\":";
    write_float_array(out, randomValues);
    out << ",\"reverseRandomValues\":";
    write_float_array(out, reverseRandomValues);
    out << ",\"output\":";
    write_data_value(out, converter->convert(input, nullptr));
    out << ",\"reverseOutput\":";
    write_data_value(out, converter->reverseConvert(input, nullptr));
    out << '}';
}

void write_stateful_converter_sample(std::ostream& out,
                                     bool& first,
                                     const char* label,
                                     uint16_t converterCoreType,
                                     rive::DataValue* output,
                                     rive::DataValue* reverseOutput)
{
    if (!first)
    {
        out << ',';
    }
    first = false;

    out << "{\"label\":";
    write_json_string(out, label);
    out << ",\"converterCoreType\":" << converterCoreType;
    out << ",\"output\":";
    write_data_value(out, output);
    out << ",\"reverseOutput\":";
    write_data_value(out, reverseOutput);
    out << '}';
}

void write_converter_sample(std::ostream& out,
                            bool& first,
                            const char* label,
                            rive::DataConverter* converter,
                            rive::DataValue* input)
{
    write_converter_sample_with_data_bind(
        out, first, label, converter, input, nullptr);
}

void write_converter_samples(std::ostream& out)
{
    struct ProbeToString : rive::DataConverterToString
    {
        void configure(uint32_t flags, uint32_t decimals)
        {
            m_parentDataBind = nullptr;
            m_Flags = flags;
            m_Decimals = decimals;
        }
        void configureColorFormat(const std::string& colorFormat)
        {
            m_parentDataBind = nullptr;
            m_ColorFormat = colorFormat;
        }
    };
    struct ProbeRounder : rive::DataConverterRounder
    {
        void configure(uint32_t decimals)
        {
            m_parentDataBind = nullptr;
            m_Decimals = decimals;
        }
    };
    struct ProbeStringTrim : rive::DataConverterStringTrim
    {
        void configure(uint32_t trimType)
        {
            m_parentDataBind = nullptr;
            m_TrimType = trimType;
        }
    };
    struct ProbeStringPad : rive::DataConverterStringPad
    {
        void configure(uint32_t length, const std::string& text, uint32_t padType)
        {
            m_parentDataBind = nullptr;
            m_Length = length;
            m_Text = text;
            m_PadType = padType;
        }
    };
    struct ProbeOperationValue : rive::DataConverterOperationValue
    {
        void configure(uint32_t operationType, float operationValue)
        {
            m_parentDataBind = nullptr;
            m_OperationType = operationType;
            m_OperationValue = operationValue;
        }
    };
    struct ProbeOperationViewModel : rive::DataConverterOperationViewModel
    {
        void configure(uint32_t operationType,
                       const std::vector<uint32_t>& sourcePathIds)
        {
            m_parentDataBind = nullptr;
            m_OperationType = operationType;
            m_SourcePathIdsBuffer = sourcePathIds;
        }
    };
    struct ProbeSystemDegsToRads : rive::DataConverterSystemDegsToRads
    {
        void configure(uint32_t operationType, float operationValue)
        {
            m_parentDataBind = nullptr;
            m_OperationType = operationType;
            m_OperationValue = operationValue;
        }
    };
    struct ProbeSystemNormalizer : rive::DataConverterSystemNormalizer
    {
        void configure(uint32_t operationType, float operationValue)
        {
            m_parentDataBind = nullptr;
            m_OperationType = operationType;
            m_OperationValue = operationValue;
        }
    };
    struct ProbeRangeMapper : rive::DataConverterRangeMapper
    {
        void configure(uint32_t interpolationType,
                       uint32_t flags,
                       float minInput,
                       float maxInput,
                       float minOutput,
                       float maxOutput)
        {
            m_parentDataBind = nullptr;
            m_interpolator = nullptr;
            m_InterpolationType = interpolationType;
            m_Flags = flags;
            m_MinInput = minInput;
            m_MaxInput = maxInput;
            m_MinOutput = minOutput;
            m_MaxOutput = maxOutput;
        }
    };
    struct ProbeInterpolator : rive::DataConverterInterpolator
    {
        void configure(float durationValue,
                       rive::KeyFrameInterpolator* keyFrameInterpolator = nullptr)
        {
            m_parentDataBind = nullptr;
            duration(durationValue);
            interpolator(keyFrameInterpolator);
        }
    };
    struct ProbeFormulaValue : rive::FormulaTokenValue
    {
        void configure(float operationValue)
        {
            m_OperationValue = operationValue;
        }
    };
    struct ProbeFormulaOperation : rive::FormulaTokenOperation
    {
        void configure(uint32_t operationType)
        {
            m_OperationType = operationType;
        }
    };
    struct ProbeFormulaFunction : rive::FormulaTokenFunction
    {
        void configure(uint32_t functionType)
        {
            m_FunctionType = functionType;
        }
    };

    out << "{\"samples\":[";
    bool first = true;

    rive::DataConverterBooleanNegate booleanNegate;
    rive::DataValueBoolean booleanTrue(true);
    write_converter_sample(
        out, first, "boolean_negate_true", &booleanNegate, &booleanTrue);

    rive::DataConverterTrigger trigger;
    rive::DataValueTrigger triggerValue(41);
    write_converter_sample(
        out, first, "trigger_increment", &trigger, &triggerValue);
    rive::DataValueSymbolListIndex symbolListIndex(5);
    write_converter_sample(out,
                           first,
                           "trigger_symbol_list_index_default",
                           &trigger,
                           &symbolListIndex);

    rive::DataConverterToNumber toNumber;
    rive::DataValueString numericString("123.5suffix");
    write_converter_sample(
        out, first, "to_number_string_prefix", &toNumber, &numericString);
    rive::DataValueString hexIntegerString("0x10tail");
    write_converter_sample(out,
                           first,
                           "to_number_hex_integer_prefix",
                           &toNumber,
                           &hexIntegerString);
    rive::DataValueString hexFloatString("-0x1.8p+2tail");
    write_converter_sample(out,
                           first,
                           "to_number_hex_float_prefix",
                           &toNumber,
                           &hexFloatString);
    std::string invalidUtf8NumericString("12.5", 4);
    invalidUtf8NumericString.push_back(static_cast<char>(0xff));
    invalidUtf8NumericString.append("tail");
    rive::DataValueString invalidUtf8Numeric(invalidUtf8NumericString);
    write_converter_forward_sample(out,
                                   first,
                                   "to_number_invalid_utf8_suffix",
                                   &toNumber,
                                   &invalidUtf8Numeric);
    rive::DataValueColor signedColor(static_cast<int>(0xff000000u));
    write_converter_sample(
        out, first, "to_number_color_signed", &toNumber, &signedColor);

    ProbeToString toString;
    toString.configure(1 | 2 | 4, 2);
    rive::DataValueNumber numberForString(12345.5f);
    write_converter_sample(
        out, first, "to_string_number_flags", &toString, &numberForString);

    rive::DataEnum dataEnum;
    auto firstEnumValue = new rive::DataEnumValue();
    firstEnumValue->key("first");
    firstEnumValue->value("");
    dataEnum.addValue(firstEnumValue);
    auto secondEnumValue = new rive::DataEnumValue();
    secondEnumValue->key("second");
    secondEnumValue->value("Second Label");
    dataEnum.addValue(secondEnumValue);
    rive::DataValueEnum enumValue(1, &dataEnum);
    write_converter_sample(
        out, first, "to_string_enum_value", &toString, &enumValue);

    ProbeToString colorToString;
    colorToString.configureColorFormat(
        "rgba(%r,%g,%b,%a)|#%R%G%B%A|hsl(%h,%s,%l)|%%|\\%|%x|%");
    rive::DataValueColor formattedColor(static_cast<int>(0xcc336699u));
    write_converter_sample(out,
                           first,
                           "to_string_color_format",
                           &colorToString,
                           &formattedColor);

    ProbeRounder rounder;
    rounder.configure(2);
    rive::DataValueNumber rounderInput(12.345f);
    write_converter_sample(
        out, first, "rounder_two_decimals", &rounder, &rounderInput);

    rive::DataConverterStringRemoveZeros removeZeros;
    rive::DataValueString zerosInput("42.5000");
    write_converter_sample(
        out, first, "string_remove_zeros", &removeZeros, &zerosInput);

    ProbeStringTrim trim;
    trim.configure(3);
    rive::DataValueString trimInput(" \ttrim me \n");
    write_converter_sample(out, first, "string_trim_all", &trim, &trimInput);

    ProbeStringPad pad;
    pad.configure(6, "ab", 0);
    rive::DataValueString padInput("xy");
    write_converter_sample(out, first, "string_pad_start", &pad, &padInput);

    ProbeOperationValue operationValue;
    operationValue.configure(2, 3.0f);
    rive::DataValueNumber operationInput(4.0f);
    write_converter_sample(out,
                           first,
                           "operation_value_multiply",
                           &operationValue,
                           &operationInput);

    auto operationViewModelInstance = rive::make_rcp<rive::ViewModelInstance>();
    operationViewModelInstance->viewModelId(7);
    auto operationViewModelValue = new rive::ViewModelInstanceNumber();
    operationViewModelValue->viewModelPropertyId(3);
    operationViewModelValue->propertyValue(2.5f);
    operationViewModelInstance->addValue(operationViewModelValue);
    auto operationDataContext =
        rive::make_rcp<rive::DataContext>(operationViewModelInstance);
    rive::DataBind operationDataBind;

    ProbeOperationViewModel operationViewModel;
    operationViewModel.configure(0, {7, 3});
    operationViewModel.bindFromContext(operationDataContext.get(),
                                       &operationDataBind);
    rive::DataValueNumber operationViewModelInput(4.0f);
    write_converter_sample(out,
                           first,
                           "operation_viewmodel_bound_add",
                           &operationViewModel,
                           &operationViewModelInput);

    ProbeOperationViewModel missingOperationViewModel;
    missingOperationViewModel.configure(0, {7, 99});
    missingOperationViewModel.bindFromContext(operationDataContext.get(),
                                             &operationDataBind);
    write_converter_sample(out,
                           first,
                           "operation_viewmodel_missing_source",
                           &missingOperationViewModel,
                           &operationViewModelInput);

    ProbeOperationViewModel unboundOperationViewModel;
    unboundOperationViewModel.configure(0, {7, 3});
    write_converter_sample(out,
                           first,
                           "operation_viewmodel_unbound_default",
                           &unboundOperationViewModel,
                           &operationViewModelInput);

    rive::DataBind toTargetDataBind;
    toTargetDataBind.flags(0);
    rive::DataBind toSourceDataBind;
    toSourceDataBind.flags(1);

    ProbeSystemDegsToRads degsToRads;
    degsToRads.configure(2, 3.14159265358979323846f / 180.0f);
    rive::DataValueNumber degreesInput(180.0f);
    write_converter_sample_with_data_bind(out,
                                          first,
                                          "system_degs_to_rads_to_target",
                                          &degsToRads,
                                          &degreesInput,
                                          &toTargetDataBind);
    rive::DataValueNumber radiansInput(3.14159265358979323846f);
    write_converter_sample_with_data_bind(out,
                                          first,
                                          "system_degs_to_rads_to_source",
                                          &degsToRads,
                                          &radiansInput,
                                          &toSourceDataBind);

    ProbeSystemNormalizer normalizer;
    normalizer.configure(3, 100.0f);
    rive::DataValueNumber percentInput(25.0f);
    write_converter_sample_with_data_bind(out,
                                          first,
                                          "system_normalizer_to_target",
                                          &normalizer,
                                          &percentInput,
                                          &toTargetDataBind);
    rive::DataValueNumber normalizedInput(0.25f);
    write_converter_sample_with_data_bind(out,
                                          first,
                                          "system_normalizer_to_source",
                                          &normalizer,
                                          &normalizedInput,
                                          &toSourceDataBind);

    rive::DataConverterListToLength listToLength;
    rive::DataValueList listValue;
    listValue.addItem(rive::make_rcp<rive::ViewModelInstanceListItem>());
    listValue.addItem(rive::make_rcp<rive::ViewModelInstanceListItem>());
    listValue.addItem(rive::make_rcp<rive::ViewModelInstanceListItem>());
    write_converter_sample(
        out, first, "list_to_length_three", &listToLength, &listValue);

    rive::DataConverterNumberToList numberToList;
    write_converter_sample(out,
                           first,
                           "number_to_list_passthrough",
                           &numberToList,
                           &listValue);
    rive::DataValueNumber numberToListInput(3.8f);
    write_converter_sample(out,
                           first,
                           "number_to_list_no_file_empty",
                           &numberToList,
                           &numberToListInput);

    ProbeRangeMapper rangeMapper;
    rangeMapper.configure(1, 0, 0.0f, 10.0f, 0.0f, 100.0f);
    rive::DataValueNumber rangeInput(5.0f);
    write_converter_sample(out, first, "range_mapper_linear", &rangeMapper, &rangeInput);

    rive::CubicEaseInterpolator rangeCubicEase;
    rangeCubicEase.x1(0.25f);
    rangeCubicEase.y1(0.1f);
    rangeCubicEase.x2(0.25f);
    rangeCubicEase.y2(1.0f);
    rangeCubicEase.initialize();

    ProbeRangeMapper rangeMapperCubic;
    rangeMapperCubic.configure(1, 0, 0.0f, 1.0f, 0.0f, 100.0f);
    rangeMapperCubic.interpolator(&rangeCubicEase);
    rive::DataValueNumber rangeCubicInput(0.25f);
    write_converter_sample(out,
                           first,
                           "range_mapper_cubic_ease",
                           &rangeMapperCubic,
                           &rangeCubicInput);

    rive::ElasticInterpolator rangeElasticEase;
    rangeElasticEase.easingValue(1);
    rangeElasticEase.amplitude(1.2f);
    rangeElasticEase.period(0.4f);
    rangeElasticEase.initialize();

    ProbeRangeMapper rangeMapperElastic;
    rangeMapperElastic.configure(1, 0, 0.0f, 1.0f, 0.0f, 100.0f);
    rangeMapperElastic.interpolator(&rangeElasticEase);
    rive::DataValueNumber rangeElasticInput(0.25f);
    write_converter_sample(out,
                           first,
                           "range_mapper_elastic_out",
                           &rangeMapperElastic,
                           &rangeElasticInput);

    ProbeRangeMapper rangeMapperModulo;
    rangeMapperModulo.configure(1, 4, 0.0f, 10.0f, 0.0f, 100.0f);
    rive::DataValueNumber rangeModuloInput(12.0f);
    write_converter_sample(out,
                           first,
                           "range_mapper_modulo",
                           &rangeMapperModulo,
                           &rangeModuloInput);

    ProbeRangeMapper rangeMapperReverse;
    rangeMapperReverse.configure(1, 8, 0.0f, 10.0f, 0.0f, 100.0f);
    rive::DataValueNumber rangeReverseInput(2.0f);
    write_converter_sample(out,
                           first,
                           "range_mapper_reverse",
                           &rangeMapperReverse,
                           &rangeReverseInput);

    ProbeRangeMapper rangeMapperHold;
    rangeMapperHold.configure(0, 0, 0.0f, 1.0f, 0.0f, 100.0f);
    rive::DataValueNumber rangeHoldInput(0.2f);
    write_converter_sample(
        out, first, "range_mapper_hold", &rangeMapperHold, &rangeHoldInput);

    rive::DataConverterInterpolator interpolatorNumber;
    rive::DataValueNumber interpolatorNumberInput(7.25f);
    write_converter_sample(out,
                           first,
                           "interpolator_number_first_run",
                           &interpolatorNumber,
                           &interpolatorNumberInput);

    rive::DataConverterInterpolator interpolatorColor;
    rive::DataValueColor interpolatorColorInput(static_cast<int>(0xff336699u));
    write_converter_sample(out,
                           first,
                           "interpolator_color_first_run",
                           &interpolatorColor,
                           &interpolatorColorInput);

    rive::DataConverterInterpolator interpolatorString;
    rive::DataValueString interpolatorStringInput("steady");
    write_converter_sample(out,
                           first,
                           "interpolator_string_passthrough",
                           &interpolatorString,
                           &interpolatorStringInput);

    ProbeInterpolator interpolatorNumberHalf;
    interpolatorNumberHalf.configure(1.0f);
    rive::DataValueNumber interpolatorStartNumber(0.0f);
    rive::DataValueNumber interpolatorTargetNumber(10.0f);
    interpolatorNumberHalf.convert(&interpolatorStartNumber, nullptr);
    interpolatorNumberHalf.advance(0.1f);
    interpolatorNumberHalf.advance(0.1f);
    interpolatorNumberHalf.convert(&interpolatorTargetNumber, nullptr);
    interpolatorNumberHalf.advance(0.5f);
    auto interpolatorNumberHalfOutput =
        interpolatorNumberHalf.convert(&interpolatorTargetNumber, nullptr);

    ProbeInterpolator interpolatorNumberHalfReverse;
    interpolatorNumberHalfReverse.configure(1.0f);
    interpolatorNumberHalfReverse.reverseConvert(&interpolatorStartNumber,
                                                 nullptr);
    interpolatorNumberHalfReverse.advance(0.1f);
    interpolatorNumberHalfReverse.advance(0.1f);
    interpolatorNumberHalfReverse.reverseConvert(&interpolatorTargetNumber,
                                                 nullptr);
    interpolatorNumberHalfReverse.advance(0.5f);
    auto interpolatorNumberHalfReverseOutput =
        interpolatorNumberHalfReverse.reverseConvert(&interpolatorTargetNumber,
                                                     nullptr);
    write_stateful_converter_sample(out,
                                    first,
                                    "interpolator_number_after_half_duration",
                                    interpolatorNumberHalf.coreType(),
                                    interpolatorNumberHalfOutput,
                                    interpolatorNumberHalfReverseOutput);

    ProbeInterpolator interpolatorRetarget;
    interpolatorRetarget.configure(1.0f);
    rive::DataValueNumber interpolatorRetargetTwenty(20.0f);
    interpolatorRetarget.convert(&interpolatorStartNumber, nullptr);
    interpolatorRetarget.advance(0.1f);
    interpolatorRetarget.advance(0.1f);
    interpolatorRetarget.convert(&interpolatorTargetNumber, nullptr);
    interpolatorRetarget.advance(0.5f);
    interpolatorRetarget.convert(&interpolatorRetargetTwenty, nullptr);
    interpolatorRetarget.advance(0.25f);
    auto interpolatorRetargetOutput =
        interpolatorRetarget.convert(&interpolatorRetargetTwenty, nullptr);

    ProbeInterpolator interpolatorRetargetReverse;
    interpolatorRetargetReverse.configure(1.0f);
    interpolatorRetargetReverse.reverseConvert(&interpolatorStartNumber,
                                               nullptr);
    interpolatorRetargetReverse.advance(0.1f);
    interpolatorRetargetReverse.advance(0.1f);
    interpolatorRetargetReverse.reverseConvert(&interpolatorTargetNumber,
                                               nullptr);
    interpolatorRetargetReverse.advance(0.5f);
    interpolatorRetargetReverse.reverseConvert(&interpolatorRetargetTwenty,
                                               nullptr);
    interpolatorRetargetReverse.advance(0.25f);
    auto interpolatorRetargetReverseOutput =
        interpolatorRetargetReverse.reverseConvert(&interpolatorRetargetTwenty,
                                                   nullptr);
    write_stateful_converter_sample(out,
                                    first,
                                    "interpolator_number_midflight_retarget",
                                    interpolatorRetarget.coreType(),
                                    interpolatorRetargetOutput,
                                    interpolatorRetargetReverseOutput);

    ProbeInterpolator interpolatorColorHalf;
    interpolatorColorHalf.configure(1.0f);
    rive::DataValueColor interpolatorStartColor(static_cast<int>(0xff000000u));
    rive::DataValueColor interpolatorTargetColor(static_cast<int>(0xffffffffu));
    interpolatorColorHalf.convert(&interpolatorStartColor, nullptr);
    interpolatorColorHalf.advance(0.1f);
    interpolatorColorHalf.advance(0.1f);
    interpolatorColorHalf.convert(&interpolatorTargetColor, nullptr);
    interpolatorColorHalf.advance(0.5f);
    auto interpolatorColorHalfOutput =
        interpolatorColorHalf.convert(&interpolatorTargetColor, nullptr);

    ProbeInterpolator interpolatorColorHalfReverse;
    interpolatorColorHalfReverse.configure(1.0f);
    interpolatorColorHalfReverse.reverseConvert(&interpolatorStartColor,
                                                nullptr);
    interpolatorColorHalfReverse.advance(0.1f);
    interpolatorColorHalfReverse.advance(0.1f);
    interpolatorColorHalfReverse.reverseConvert(&interpolatorTargetColor,
                                                nullptr);
    interpolatorColorHalfReverse.advance(0.5f);
    auto interpolatorColorHalfReverseOutput =
        interpolatorColorHalfReverse.reverseConvert(&interpolatorTargetColor,
                                                    nullptr);
    write_stateful_converter_sample(out,
                                    first,
                                    "interpolator_color_after_half_duration",
                                    interpolatorColorHalf.coreType(),
                                    interpolatorColorHalfOutput,
                                    interpolatorColorHalfReverseOutput);

    rive::CubicEaseInterpolator interpolatorCubicEase;
    interpolatorCubicEase.x1(0.25f);
    interpolatorCubicEase.y1(0.1f);
    interpolatorCubicEase.x2(0.25f);
    interpolatorCubicEase.y2(1.0f);
    interpolatorCubicEase.initialize();

    ProbeInterpolator interpolatorCubic;
    interpolatorCubic.configure(1.0f, &interpolatorCubicEase);
    rive::DataValueNumber interpolatorCubicTarget(100.0f);
    interpolatorCubic.convert(&interpolatorStartNumber, nullptr);
    interpolatorCubic.advance(0.1f);
    interpolatorCubic.advance(0.1f);
    interpolatorCubic.convert(&interpolatorCubicTarget, nullptr);
    interpolatorCubic.advance(0.25f);
    auto interpolatorCubicOutput =
        interpolatorCubic.convert(&interpolatorCubicTarget, nullptr);

    ProbeInterpolator interpolatorCubicReverse;
    interpolatorCubicReverse.configure(1.0f, &interpolatorCubicEase);
    interpolatorCubicReverse.reverseConvert(&interpolatorStartNumber, nullptr);
    interpolatorCubicReverse.advance(0.1f);
    interpolatorCubicReverse.advance(0.1f);
    interpolatorCubicReverse.reverseConvert(&interpolatorCubicTarget, nullptr);
    interpolatorCubicReverse.advance(0.25f);
    auto interpolatorCubicReverseOutput =
        interpolatorCubicReverse.reverseConvert(&interpolatorCubicTarget,
                                                nullptr);
    write_stateful_converter_sample(out,
                                    first,
                                    "interpolator_cubic_after_quarter_duration",
                                    interpolatorCubic.coreType(),
                                    interpolatorCubicOutput,
                                    interpolatorCubicReverseOutput);

    ProbeInterpolator groupStatefulInterpolator;
    groupStatefulInterpolator.configure(1.0f);
    ProbeOperationValue groupStatefulMultiply;
    groupStatefulMultiply.configure(2, 2.0f);
    rive::DataConverterGroup statefulGroup;
    auto statefulGroupInterpolatorItem = new rive::DataConverterGroupItem();
    statefulGroupInterpolatorItem->converter(&groupStatefulInterpolator);
    statefulGroup.addItem(statefulGroupInterpolatorItem);
    auto statefulGroupMultiplyItem = new rive::DataConverterGroupItem();
    statefulGroupMultiplyItem->converter(&groupStatefulMultiply);
    statefulGroup.addItem(statefulGroupMultiplyItem);
    statefulGroup.convert(&interpolatorStartNumber, nullptr);
    statefulGroup.advance(0.1f);
    statefulGroup.advance(0.1f);
    statefulGroup.convert(&interpolatorTargetNumber, nullptr);
    statefulGroup.advance(0.5f);
    auto statefulGroupOutput =
        statefulGroup.convert(&interpolatorTargetNumber, nullptr);

    ProbeInterpolator groupStatefulInterpolatorReverse;
    groupStatefulInterpolatorReverse.configure(1.0f);
    ProbeOperationValue groupStatefulMultiplyReverse;
    groupStatefulMultiplyReverse.configure(2, 2.0f);
    rive::DataConverterGroup statefulGroupReverse;
    auto statefulGroupInterpolatorReverseItem =
        new rive::DataConverterGroupItem();
    statefulGroupInterpolatorReverseItem->converter(
        &groupStatefulInterpolatorReverse);
    statefulGroupReverse.addItem(statefulGroupInterpolatorReverseItem);
    auto statefulGroupMultiplyReverseItem = new rive::DataConverterGroupItem();
    statefulGroupMultiplyReverseItem->converter(&groupStatefulMultiplyReverse);
    statefulGroupReverse.addItem(statefulGroupMultiplyReverseItem);
    statefulGroupReverse.reverseConvert(&interpolatorStartNumber, nullptr);
    statefulGroupReverse.advance(0.1f);
    statefulGroupReverse.advance(0.1f);
    statefulGroupReverse.reverseConvert(&interpolatorTargetNumber, nullptr);
    statefulGroupReverse.advance(0.5f);
    auto statefulGroupReverseOutput =
        statefulGroupReverse.reverseConvert(&interpolatorTargetNumber, nullptr);
    write_stateful_converter_sample(
        out,
        first,
        "group_interpolator_then_multiply_after_half_duration",
        statefulGroup.coreType(),
        statefulGroupOutput,
        statefulGroupReverseOutput);

    ProbeStringTrim groupTrim;
    groupTrim.configure(3);
    ProbeStringPad groupPad;
    groupPad.configure(4, "!", 1);
    rive::DataConverterGroup group;
    auto groupTrimItem = new rive::DataConverterGroupItem();
    groupTrimItem->converter(&groupTrim);
    group.addItem(groupTrimItem);
    auto groupPadItem = new rive::DataConverterGroupItem();
    groupPadItem->converter(&groupPad);
    group.addItem(groupPadItem);
    rive::DataValueString groupInput(" x ");
    write_converter_sample(out, first, "group_trim_then_pad", &group, &groupInput);

    rive::DataConverterFormula arithmeticFormula;
    arithmeticFormula.addToken(new rive::FormulaTokenInput());
    auto arithmeticAdd = new ProbeFormulaOperation();
    arithmeticAdd->configure(0);
    arithmeticFormula.addToken(arithmeticAdd);
    auto arithmeticThree = new ProbeFormulaValue();
    arithmeticThree->configure(3.0f);
    arithmeticFormula.addToken(arithmeticThree);
    auto arithmeticMultiply = new ProbeFormulaOperation();
    arithmeticMultiply->configure(2);
    arithmeticFormula.addToken(arithmeticMultiply);
    auto arithmeticTwo = new ProbeFormulaValue();
    arithmeticTwo->configure(2.0f);
    arithmeticFormula.addToken(arithmeticTwo);
    arithmeticFormula.calculateFormula();
    rive::DataValueNumber arithmeticInput(5.0f);
    write_converter_sample(out,
                           first,
                           "formula_arithmetic_precedence",
                           &arithmeticFormula,
                           &arithmeticInput);
    rive::DataValueString formulaStringInput("not a number");
    write_converter_sample(out,
                           first,
                           "formula_non_number_default",
                           &arithmeticFormula,
                           &formulaStringInput);

    rive::DataConverterFormula maxFormula;
    auto maxFunction = new ProbeFormulaFunction();
    maxFunction->configure(1);
    maxFormula.addToken(maxFunction);
    maxFormula.addToken(new rive::FormulaTokenInput());
    maxFormula.addToken(new rive::FormulaTokenArgumentSeparator());
    auto maxThree = new ProbeFormulaValue();
    maxThree->configure(3.0f);
    maxFormula.addToken(maxThree);
    maxFormula.addToken(new rive::FormulaTokenArgumentSeparator());
    auto maxEight = new ProbeFormulaValue();
    maxEight->configure(8.0f);
    maxFormula.addToken(maxEight);
    maxFormula.addToken(new rive::FormulaTokenParenthesisClose());
    maxFormula.calculateFormula();
    rive::DataValueNumber maxInput(4.0f);
    write_converter_sample(
        out, first, "formula_function_max", &maxFormula, &maxInput);

    rive::DataConverterFormula powFormula;
    auto powFunction = new ProbeFormulaFunction();
    powFunction->configure(6);
    powFormula.addToken(powFunction);
    powFormula.addToken(new rive::FormulaTokenInput());
    powFormula.addToken(new rive::FormulaTokenArgumentSeparator());
    auto powTwo = new ProbeFormulaValue();
    powTwo->configure(2.0f);
    powFormula.addToken(powTwo);
    powFormula.addToken(new rive::FormulaTokenParenthesisClose());
    powFormula.calculateFormula();
    rive::DataValueSymbolListIndex powInput(4);
    write_converter_sample(
        out, first, "formula_symbol_list_pow", &powFormula, &powInput);

    auto writeFormulaFunctionSample =
        [&](const char* label,
            uint32_t functionType,
            const std::vector<float>& values) {
            rive::DataConverterFormula formula;
            auto function = new ProbeFormulaFunction();
            function->configure(functionType);
            formula.addToken(function);
            for (size_t i = 0; i < values.size(); ++i)
            {
                if (i != 0)
                {
                    formula.addToken(new rive::FormulaTokenArgumentSeparator());
                }
                auto value = new ProbeFormulaValue();
                value->configure(values[i]);
                formula.addToken(value);
            }
            formula.addToken(new rive::FormulaTokenParenthesisClose());
            formula.calculateFormula();
            rive::DataValueNumber input(0.0f);
            write_converter_sample(out, first, label, &formula, &input);
        };

    writeFormulaFunctionSample(
        "formula_function_min", 0, std::vector<float>{9.0f, -2.0f});
    writeFormulaFunctionSample(
        "formula_function_round", 2, std::vector<float>{2.6f});
    writeFormulaFunctionSample(
        "formula_function_ceil", 3, std::vector<float>{2.1f});
    writeFormulaFunctionSample(
        "formula_function_floor", 4, std::vector<float>{2.9f});
    writeFormulaFunctionSample(
        "formula_function_sqrt", 5, std::vector<float>{9.0f});
    writeFormulaFunctionSample(
        "formula_function_exp", 7, std::vector<float>{1.0f});
    writeFormulaFunctionSample(
        "formula_function_log", 8, std::vector<float>{2.71828175f});
    writeFormulaFunctionSample(
        "formula_function_cosine", 9, std::vector<float>{0.0f});
    writeFormulaFunctionSample("formula_function_sine",
                               10,
                               std::vector<float>{1.57079637f});
    writeFormulaFunctionSample("formula_function_tangent",
                               11,
                               std::vector<float>{0.785398185f});
    writeFormulaFunctionSample(
        "formula_function_acosine", 12, std::vector<float>{0.5f});
    writeFormulaFunctionSample(
        "formula_function_asine", 13, std::vector<float>{0.5f});
    writeFormulaFunctionSample(
        "formula_function_atangent", 14, std::vector<float>{1.0f});
    writeFormulaFunctionSample("formula_function_atangent2",
                               15,
                               std::vector<float>{1.0f, 1.0f});
    std::srand(7);
    writeFormulaFunctionSample("formula_function_random_range",
                               16,
                               std::vector<float>{2.0f, 6.0f});
    std::srand(11);
    std::vector<float> randomAlwaysValues{
        static_cast<float>(std::rand()) / static_cast<float>(RAND_MAX),
        static_cast<float>(std::rand()) / static_cast<float>(RAND_MAX),
        static_cast<float>(std::rand()) / static_cast<float>(RAND_MAX),
        static_cast<float>(std::rand()) / static_cast<float>(RAND_MAX)};
    std::srand(11);
    rive::DataConverterFormula randomAlwaysFormula;
    randomAlwaysFormula.randomModeValue(1);
    auto randomAlwaysFirst = new ProbeFormulaFunction();
    randomAlwaysFirst->configure(16);
    randomAlwaysFormula.addToken(randomAlwaysFirst);
    auto randomAlwaysFirstLower = new ProbeFormulaValue();
    randomAlwaysFirstLower->configure(2.0f);
    randomAlwaysFormula.addToken(randomAlwaysFirstLower);
    randomAlwaysFormula.addToken(new rive::FormulaTokenArgumentSeparator());
    auto randomAlwaysFirstUpper = new ProbeFormulaValue();
    randomAlwaysFirstUpper->configure(6.0f);
    randomAlwaysFormula.addToken(randomAlwaysFirstUpper);
    randomAlwaysFormula.addToken(new rive::FormulaTokenParenthesisClose());
    auto randomAlwaysAdd = new ProbeFormulaOperation();
    randomAlwaysAdd->configure(0);
    randomAlwaysFormula.addToken(randomAlwaysAdd);
    auto randomAlwaysSecond = new ProbeFormulaFunction();
    randomAlwaysSecond->configure(16);
    randomAlwaysFormula.addToken(randomAlwaysSecond);
    auto randomAlwaysSecondLower = new ProbeFormulaValue();
    randomAlwaysSecondLower->configure(10.0f);
    randomAlwaysFormula.addToken(randomAlwaysSecondLower);
    randomAlwaysFormula.addToken(new rive::FormulaTokenArgumentSeparator());
    auto randomAlwaysSecondUpper = new ProbeFormulaValue();
    randomAlwaysSecondUpper->configure(20.0f);
    randomAlwaysFormula.addToken(randomAlwaysSecondUpper);
    randomAlwaysFormula.addToken(new rive::FormulaTokenParenthesisClose());
    randomAlwaysFormula.calculateFormula();
    rive::DataValueNumber randomAlwaysInput(0.0f);
    write_converter_sample_with_random_values(
        out,
        first,
        "formula_function_random_always_pair",
        &randomAlwaysFormula,
        &randomAlwaysInput,
        std::vector<float>{randomAlwaysValues[0], randomAlwaysValues[1]},
        std::vector<float>{randomAlwaysValues[2], randomAlwaysValues[3]});

    rive::DataConverterOperation operation;
    rive::DataValueNumber operationBaseInput(42.25f);
    write_converter_sample(out,
                           first,
                           "operation_base_passthrough",
                           &operation,
                           &operationBaseInput);

    rive::ScriptedDataConverter scriptedDataConverter;
    rive::DataValueString scriptedDataConverterInput("scripted");
    write_converter_sample(out,
                           first,
                           "scripted_data_converter_non_scripting_passthrough",
                           &scriptedDataConverter,
                           &scriptedDataConverterInput);

    out << "]}\n";
}

void write_number_to_list_samples(std::ostream& out, rive::File* file)
{
    out << "{\"samples\":[";
    bool first = true;
    for (size_t i = 0; i < file->m_DataConverters.size(); ++i)
    {
        auto converter = file->m_DataConverters[i];
        if (converter == nullptr ||
            !converter->is<rive::DataConverterNumberToList>())
        {
            continue;
        }

        if (!first)
        {
            out << ',';
        }
        first = false;

        rive::DataValueNumber input(2.8f);
        out << "{\"converterIndex\":" << i;
        out << ",\"converterCoreType\":" << converter->coreType();
        out << ",\"output\":";
        write_data_value(out, converter->convert(&input, nullptr));
        out << '}';
    }
    out << "]}\n";
}

bool is_arg(const char* arg, const char* target, const char* alt = nullptr)
{
    return arg != nullptr &&
           (std::strcmp(arg, target) == 0 ||
            (alt != nullptr && std::strcmp(arg, alt) == 0));
}
} // namespace

int main(int argc, const char* argv[])
{
    const char* filename = nullptr;
    ProbeOptions options;
    bool converterSamples = false;
    bool numberToListSamples = false;

    for (int i = 1; i < argc; ++i)
    {
        if (is_arg(argv[i], "--converter-samples"))
        {
            converterSamples = true;
            continue;
        }

        if (is_arg(argv[i], "--number-to-list-samples"))
        {
            numberToListSamples = true;
            continue;
        }

        if (is_arg(argv[i], "--file", "-f"))
        {
            if (i + 1 >= argc)
            {
                std::cerr << "--file requires a path\n";
                return 2;
            }
            filename = argv[++i];
            continue;
        }

        if (is_arg(argv[i], "--property-values"))
        {
            options.propertyValues = true;
            options.artboardPropertyValues = true;
            continue;
        }

        if (is_arg(argv[i], "--file-property-values"))
        {
            options.propertyValues = true;
            continue;
        }

        if (is_arg(argv[i], "--no-advance"))
        {
            options.advanceArtboards = false;
            continue;
        }

        if (is_arg(argv[i], "--complete-view-model-properties"))
        {
            options.completeViewModelProperties = true;
            continue;
        }

        if (is_arg(argv[i], "--data-context-lookups"))
        {
            options.dataContextLookups = true;
            continue;
        }

        if (filename == nullptr)
        {
            filename = argv[i];
            continue;
        }

        std::cerr << "unrecognized argument " << argv[i] << "\n";
        return 2;
    }

    if (converterSamples)
    {
        write_converter_samples(std::cout);
        return 0;
    }

    if (filename == nullptr)
    {
        std::cerr << "usage: rive_cpp_probe [--converter-samples] [--number-to-list-samples] [--property-values] [--file-property-values] [--no-advance] [--complete-view-model-properties] [--data-context-lookups] --file "
                     "path/to/file.riv\n";
        return 2;
    }

    try
    {
        rive::ImportResult result = rive::ImportResult::success;
        auto file = open_file(filename, &result);
        if (file == nullptr || result != rive::ImportResult::success)
        {
            std::cerr << "failed to import " << filename
                      << " result=" << import_result_name(result) << "\n";
            return 1;
        }

        if (options.completeViewModelProperties || options.dataContextLookups)
        {
            complete_view_model_properties(file.get());
        }

        if (numberToListSamples)
        {
            write_number_to_list_samples(std::cout, file.get());
            return 0;
        }

        write_file(std::cout, filename, file.get(), options);
    }
    catch (const std::exception& error)
    {
        std::cerr << error.what() << "\n";
        return 1;
    }

    return 0;
}
