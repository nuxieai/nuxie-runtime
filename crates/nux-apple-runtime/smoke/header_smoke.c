#include "nux_runtime.h"

#include <stddef.h>
#include <stdint.h>

_Static_assert(NUX_RUNTIME_ABI_MAJOR == 1, "unexpected runtime ABI major");
_Static_assert(NUX_RUNTIME_ABI_MINOR == 1, "unexpected runtime ABI minor");
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
    (void)attach_surface;
    (void)reattach_surface;
    (void)copy_metal_device;
    (void)advance;
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
