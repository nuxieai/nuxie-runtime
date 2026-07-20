#include "nux_runtime.h"

#include <stddef.h>
#include <stdint.h>

_Static_assert(NUX_RUNTIME_ABI_MAJOR == 1, "unexpected runtime ABI major");
_Static_assert(NUX_RUNTIME_ABI_MINOR == 3, "unexpected runtime ABI minor");
_Static_assert(NUX_FLOW_SESSION_ABI_MINOR == 3,
               "unexpected flow-session ABI minor");
_Static_assert(NUX_SCRIPT_AUTHORIZATION_VISUAL_ONLY == 1,
               "script authorization values are part of the ABI");
_Static_assert(NUX_SCRIPT_AUTHORIZATION_AUTHENTICATED == 2,
               "script authorization values are part of the ABI");
_Static_assert(NUX_FLOW_EXTERNAL_ASSET_KIND_IMAGE == 1,
               "external asset kinds are part of the ABI");
_Static_assert(NUX_DIAGNOSTIC_SEVERITY_FATAL == 2,
               "diagnostic severities are part of the ABI");
_Static_assert(NUX_SURFACE_DISPOSITION_PRESENTED == 1,
               "surface disposition values are part of the ABI");
_Static_assert(NUX_SURFACE_DISPOSITION_FATAL == 9,
               "surface disposition values are part of the ABI");
_Static_assert(NUX_FLOW_QUERY_KIND_PLAYER_INPUTS == 4,
               "player-input query kind is part of the ABI");
_Static_assert(NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_BOOL == 9,
               "player-input mutation kinds are part of the ABI");
_Static_assert(NUX_FLOW_STATE_MUTATION_KIND_FIRE_INPUT_TRIGGER == 11,
               "player-input mutation kinds are part of the ABI");
_Static_assert(NUX_FLOW_STATE_MUTATION_KIND_SET_VIEW_MODEL == 12,
               "view-model replacement kind is part of the ABI");
_Static_assert(NUX_FLOW_VALUE_KIND_LIST_INDEX == 10,
               "list-index value kind is part of the ABI");
_Static_assert(NUX_FLOW_SCHEMA_PROPERTY_KIND_LIST_INDEX == 12,
               "list-index schema kind is part of the ABI");
_Static_assert(NUX_FLOW_PLAYER_SELECTION_EXPLICIT_STATE_MACHINE == 1,
               "player-selection branches are part of the ABI");
_Static_assert(NUX_FLOW_PLAYER_SELECTION_STATIC == 5,
               "player-selection branches are part of the ABI");
_Static_assert(sizeof(NuxStatus) == sizeof(uint32_t),
               "NuxStatus must remain a 32-bit ABI value");
_Static_assert(sizeof(NuxSurfaceDisposition) == sizeof(uint32_t),
               "NuxSurfaceDisposition must remain a 32-bit ABI value");
_Static_assert(sizeof(struct NuxByteView) == 16,
               "unexpected NuxByteView layout");
_Static_assert(offsetof(struct NuxByteView, len) == 8,
               "unexpected NuxByteView.len offset");
_Static_assert(sizeof(struct NuxFlowAuthorizationKey) == 40,
               "unexpected NuxFlowAuthorizationKey layout");
_Static_assert(offsetof(struct NuxFlowAuthorizationKey, key_id) == 8,
               "unexpected NuxFlowAuthorizationKey.key_id offset");
_Static_assert(sizeof(struct NuxFlowExternalAsset) == 80,
               "unexpected NuxFlowExternalAsset layout");
_Static_assert(offsetof(struct NuxFlowExternalAsset, unique_name) == 16,
               "unexpected NuxFlowExternalAsset.unique_name offset");
_Static_assert(offsetof(struct NuxFlowExternalAsset, bytes) == 64,
               "unexpected NuxFlowExternalAsset.bytes offset");
_Static_assert(sizeof(struct NuxFlowImportRequest) == 112,
               "unexpected NuxFlowImportRequest layout");
_Static_assert(offsetof(struct NuxFlowImportRequest, artifact_bytes) == 8,
               "unexpected NuxFlowImportRequest.artifact_bytes offset");
_Static_assert(offsetof(struct NuxFlowImportRequest, selected_key) == 88,
               "unexpected NuxFlowImportRequest.selected_key offset");
_Static_assert(offsetof(struct NuxFlowImportRequest, external_asset_count) == 104,
               "unexpected NuxFlowImportRequest.external_asset_count offset");
_Static_assert(sizeof(struct NuxDiagnosticView) == 40,
               "unexpected NuxDiagnosticView layout");
_Static_assert(sizeof(struct NuxFlowSessionDescriptor) == 40,
               "unexpected NuxFlowSessionDescriptor layout");
_Static_assert(offsetof(struct NuxFlowSessionDescriptor, artboard_name) == 8,
               "unexpected NuxFlowSessionDescriptor.artboard_name offset");
_Static_assert(offsetof(struct NuxFlowSessionDescriptor, state_machine_name) == 24,
               "unexpected NuxFlowSessionDescriptor.state_machine_name offset");
_Static_assert(sizeof(struct NuxAppleSurfaceDescriptor) == 12,
               "unexpected NuxAppleSurfaceDescriptor layout");
_Static_assert(offsetof(struct NuxAppleSurfaceDescriptor, pixel_width) == 4,
               "unexpected NuxAppleSurfaceDescriptor.pixel_width offset");
_Static_assert(sizeof(struct NuxFrameOperation) == 40,
               "unexpected NuxFrameOperation layout");
_Static_assert(offsetof(struct NuxFrameOperation, apple_drawable) == 16,
               "unexpected NuxFrameOperation.apple_drawable offset");
_Static_assert(offsetof(struct NuxFrameOperation, completion_context) == 24,
               "unexpected NuxFrameOperation.completion_context offset");
_Static_assert(offsetof(struct NuxFrameOperation, completion_callback) == 32,
               "unexpected NuxFrameOperation.completion_callback offset");

/* ABI 1.3 retains every ABI 1.2 caller-owned input and its ABI 1.1 prefix,
 * while extending result metadata through versioned output records. */
_Static_assert(sizeof(struct NuxFlowConfiguredSessionDescriptor) == 40,
               "unexpected NuxFlowConfiguredSessionDescriptor layout");
_Static_assert(offsetof(struct NuxFlowConfiguredSessionDescriptor,
                        artboard_name) == 8,
               "unexpected configured artboard-name offset");
_Static_assert(sizeof(struct NuxFlowValueNode) == 88,
               "unexpected NuxFlowValueNode layout");
_Static_assert(offsetof(struct NuxFlowValueNode, instance_id) == 40,
               "unexpected NuxFlowValueNode.instance_id offset");
_Static_assert(offsetof(struct NuxFlowValueNode, string_value) == 56,
               "unexpected NuxFlowValueNode.string_value offset");
_Static_assert(sizeof(struct NuxFlowValueEdge) == 24,
               "unexpected NuxFlowValueEdge layout");
_Static_assert(sizeof(struct NuxFlowValueArena) == 40,
               "unexpected NuxFlowValueArena layout");
_Static_assert(sizeof(struct NuxFlowNewInstance) == 40,
               "unexpected NuxFlowNewInstance layout");
_Static_assert(sizeof(struct NuxFlowInstanceReference) == 16,
               "unexpected NuxFlowInstanceReference layout");
_Static_assert(sizeof(struct NuxFlowStateMutation) == 88,
               "unexpected NuxFlowStateMutation layout");
_Static_assert(offsetof(struct NuxFlowStateMutation, path) == 40,
               "unexpected NuxFlowStateMutation.path offset");
_Static_assert(offsetof(struct NuxFlowStateMutation, input_name) == 56,
               "unexpected NuxFlowStateMutation.input_name offset");
_Static_assert(sizeof(struct NuxFlowStateBatch) == 56,
               "unexpected NuxFlowStateBatch layout");
_Static_assert(offsetof(struct NuxFlowStateBatch, value_arena) == 16,
               "unexpected NuxFlowStateBatch.value_arena offset");
_Static_assert(sizeof(struct NuxFlowPointerEvent) == 20,
               "unexpected NuxFlowPointerEvent layout");
_Static_assert(sizeof(struct NuxFlowPointerBatch) == 24,
               "unexpected NuxFlowPointerBatch layout");
_Static_assert(sizeof(struct NuxFlowAdvanceOperation) == 48,
               "unexpected NuxFlowAdvanceOperation layout");
_Static_assert(offsetof(struct NuxFlowAdvanceOperation, apple_drawable) == 24,
               "unexpected NuxFlowAdvanceOperation.apple_drawable offset");
_Static_assert(sizeof(struct NuxFlowQuery) == 8,
               "unexpected NuxFlowQuery layout");
_Static_assert(sizeof(struct NuxFlowQueryBatch) == 24,
               "unexpected NuxFlowQueryBatch layout");
_Static_assert(sizeof(struct NuxFlowSessionOperation) == 48,
               "unexpected NuxFlowSessionOperation layout");
_Static_assert(offsetof(struct NuxFlowSessionOperation, state_batch) == 16,
               "unexpected NuxFlowSessionOperation.state_batch offset");
_Static_assert(sizeof(struct NuxFlowPlayerMetadataView) == 64,
               "unexpected NuxFlowPlayerMetadataView layout");
_Static_assert(offsetof(struct NuxFlowPlayerMetadataView, selection) == 8,
               "unexpected NuxFlowPlayerMetadataView.selection offset");
_Static_assert(offsetof(struct NuxFlowPlayerMetadataView, artboard_name) == 16,
               "unexpected NuxFlowPlayerMetadataView.artboard_name offset");
_Static_assert(sizeof(struct NuxFlowPlayerInputView) == 32,
               "unexpected NuxFlowPlayerInputView layout");
_Static_assert(sizeof(struct NuxFlowSchemaView) == 48,
               "unexpected NuxFlowSchemaView layout");
_Static_assert(sizeof(struct NuxFlowSchemaPropertyView) == 80,
               "unexpected NuxFlowSchemaPropertyView layout");
_Static_assert(offsetof(struct NuxFlowSchemaPropertyView,
                        referenced_schema_id) == 56,
               "unexpected referenced-schema offset");
_Static_assert(offsetof(struct NuxFlowSchemaPropertyView,
                        first_enum_label) == 72,
               "unexpected enum-label span offset");
_Static_assert(sizeof(struct NuxFlowEnumLabelView) == 24,
               "unexpected NuxFlowEnumLabelView layout");
_Static_assert(sizeof(struct NuxFlowInstanceTemplateView) == 40,
               "unexpected NuxFlowInstanceTemplateView layout");
_Static_assert(sizeof(struct NuxFlowInstanceView) == 56,
               "unexpected NuxFlowInstanceView layout");
_Static_assert(sizeof(struct NuxFlowValueRootView) == 16,
               "unexpected NuxFlowValueRootView layout");
_Static_assert(sizeof(struct NuxFlowCreatedInstanceView) == 16,
               "unexpected NuxFlowCreatedInstanceView layout");
_Static_assert(sizeof(struct NuxFlowEventPropertyView) == 40,
               "unexpected NuxFlowEventPropertyView layout");
_Static_assert(sizeof(struct NuxFlowOutputView) == 160,
               "unexpected NuxFlowOutputView layout");
_Static_assert(offsetof(struct NuxFlowOutputView, sequence) == 24,
               "unexpected NuxFlowOutputView.sequence offset");
_Static_assert(offsetof(struct NuxFlowOutputView, name) == 72,
               "unexpected NuxFlowOutputView.name offset");
_Static_assert(offsetof(struct NuxFlowOutputView, has_open_url) == 120,
               "unexpected NuxFlowOutputView.has_open_url offset");
_Static_assert(offsetof(struct NuxFlowOutputView, open_url) == 128,
               "unexpected NuxFlowOutputView.open_url offset");
_Static_assert(offsetof(struct NuxFlowOutputView, open_url_target) == 144,
               "unexpected NuxFlowOutputView.open_url_target offset");

static void typecheck_product_api(void)
{
    uint16_t (*abi_major)(void) = nux_runtime_abi_major;
    NuxStatus (*require_abi)(uint16_t, uint16_t) =
        nux_runtime_require_abi;
    NuxStatus (*create_context)(const struct NuxFlowImportRequest*,
                                struct NuxFlowRuntimeContext**,
                                struct NuxOperationResult**) =
        nux_flow_runtime_context_create;
    NuxStatus (*create_session)(const struct NuxFlowRuntimeContext*,
                                const struct NuxFlowSessionDescriptor*,
                                struct NuxFlowRenderSession**,
                                struct NuxOperationResult**) =
        nux_flow_render_session_create;
    NuxStatus (*create_configured_session)(
        const struct NuxFlowRuntimeContext*,
        const struct NuxFlowConfiguredSessionDescriptor*,
        struct NuxFlowRenderSession**,
        struct NuxFlowSessionResult**) =
        nux_flow_render_session_create_configured;
    NuxStatus (*attach_surface)(const struct NuxFlowRenderSession*,
                                const struct NuxAppleSurfaceDescriptor*,
                                struct NuxAppleSurface**,
                                struct NuxOperationResult**) =
        nux_flow_render_session_attach_apple_surface;
    NuxStatus (*reattach_surface)(const struct NuxAppleSurface*,
                                  const struct NuxAppleSurfaceDescriptor*,
                                  struct NuxOperationResult**) =
        nux_apple_surface_reattach;
    NuxStatus (*copy_metal_device)(const struct NuxAppleSurface*,
                                   void**,
                                   struct NuxOperationResult**) =
        nux_apple_surface_copy_metal_device;
    NuxStatus (*advance)(const struct NuxFlowRenderSession*,
                         const struct NuxFrameOperation*,
                         struct NuxOperationResult**) =
        nux_flow_render_session_advance;
    NuxStatus (*perform)(const struct NuxFlowRenderSession*,
                         const struct NuxFlowSessionOperation*,
                         struct NuxFlowSessionResult**) =
        nux_flow_render_session_perform;
    NuxStatus (*session_result_status)(const struct NuxFlowSessionResult*) =
        nux_flow_session_result_status;
    NuxStatus (*player_metadata)(const struct NuxFlowSessionResult*,
                                 struct NuxFlowPlayerMetadataView*) =
        nux_flow_session_result_player_metadata;
    uint64_t (*player_input_count)(const struct NuxFlowSessionResult*) =
        nux_flow_session_result_player_input_count;
    NuxStatus (*player_input_at)(const struct NuxFlowSessionResult*,
                                 uint64_t,
                                 struct NuxFlowPlayerInputView*) =
        nux_flow_session_result_player_input_at;
    uint64_t (*value_node_count)(const struct NuxFlowSessionResult*) =
        nux_flow_session_result_value_node_count;
    NuxStatus (*value_node_at)(const struct NuxFlowSessionResult*,
                               uint64_t,
                               struct NuxFlowValueNode*) =
        nux_flow_session_result_value_node_at;
    uint64_t (*output_count)(const struct NuxFlowSessionResult*) =
        nux_flow_session_result_output_count;
    NuxStatus (*output_at)(const struct NuxFlowSessionResult*,
                           uint64_t,
                           struct NuxFlowOutputView*) =
        nux_flow_session_result_output_at;
    NuxStatus (*wake_after_seconds)(const struct NuxFlowSessionResult*,
                                    double*) =
        nux_flow_session_result_wake_after_seconds;
    void (*free_session_result)(struct NuxFlowSessionResult*) =
        nux_flow_session_result_free;
    NuxScriptAuthorization (*script_authorization)(
        const struct NuxOperationResult*) =
        nux_operation_result_script_authorization;
    NuxStatus (*authenticated_key_id)(const struct NuxOperationResult*,
                                      struct NuxByteView*) =
        nux_operation_result_authenticated_key_id;
    uint64_t (*diagnostic_count)(const struct NuxOperationResult*) =
        nux_operation_result_diagnostic_count;
    NuxStatus (*diagnostic_at)(const struct NuxOperationResult*,
                               uint64_t,
                               struct NuxDiagnosticView*) =
        nux_operation_result_diagnostic_at;

    (void)abi_major;
    (void)require_abi;
    (void)create_context;
    (void)create_session;
    (void)create_configured_session;
    (void)attach_surface;
    (void)reattach_surface;
    (void)copy_metal_device;
    (void)advance;
    (void)perform;
    (void)session_result_status;
    (void)player_metadata;
    (void)player_input_count;
    (void)player_input_at;
    (void)value_node_count;
    (void)value_node_at;
    (void)output_count;
    (void)output_at;
    (void)wake_after_seconds;
    (void)free_session_result;
    (void)script_authorization;
    (void)authenticated_key_id;
    (void)diagnostic_count;
    (void)diagnostic_at;
}

int main(void)
{
    typecheck_product_api();
    return 0;
}
