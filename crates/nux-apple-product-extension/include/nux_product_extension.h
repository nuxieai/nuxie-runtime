#ifndef NUX_PRODUCT_EXTENSION_H
#define NUX_PRODUCT_EXTENSION_H

#include "nux_capi_apple.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Import caller-authenticated Nuxie scene bytes after enabling the authored-
 * data converter format used by published product experiences.
 *
 * Pointer, result, and ownership semantics are identical to
 * nux_file_import_configured.
 */
NuxStatus nux_product_file_import_configured(const uint8_t *bytes,
                                             size_t len,
                                             const NuxFileImportConfig *config,
                                             NuxFile **out_file,
                                             NuxCapiResult **out_result);

#ifdef __cplusplus
}
#endif

#endif /* NUX_PRODUCT_EXTENSION_H */
