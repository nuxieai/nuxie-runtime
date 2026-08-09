#include "nux_capi_apple.h"

#include <stddef.h>

int main(void) {
  NuxFile *file = NULL;
  NuxCapiResult *result = NULL;
  NuxStatus status = nux_file_import_with_result(NULL, 0, &file, &result);
  if (status == NUX_STATUS_OK || file != NULL) {
    return 1;
  }
  if (result != NULL) {
    NuxCapiDiagnosticView diagnostic = {
        .struct_size = sizeof(NuxCapiDiagnosticView),
    };
    (void)nux_capi_result_diagnostic(result, &diagnostic);
    (void)nux_capi_result_free(result);
  }
  NuxPlayerSchedulingInfo scheduling = {
      .struct_size = sizeof(NuxPlayerSchedulingInfo),
  };
  if (scheduling.struct_size < NUX_PLAYER_SCHEDULING_INFO_V3_MIN_SIZE ||
      nux_player_step_result_scheduling(NULL, &scheduling) !=
          NUX_STATUS_NULL_ARGUMENT ||
      nux_player_acknowledge_presented(NULL, 1) != NUX_STATUS_NULL_ARGUMENT) {
    return 3;
  }
  return nux_capi_abi_version() == NUX_CAPI_ABI_VERSION ? 0 : 2;
}
