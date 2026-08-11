#include "nux_product_extension.h"

#include <stddef.h>

int main(void) {
  NuxFile *file = NULL;
  NuxCapiResult *result = NULL;
  NuxStatus status =
      nux_product_file_import_configured(NULL, 0, NULL, &file, &result);
  if (status == NUX_STATUS_OK || file != NULL) {
    return 1;
  }
  if (result != NULL) {
    (void)nux_capi_result_free(result);
  }
  return 0;
}
