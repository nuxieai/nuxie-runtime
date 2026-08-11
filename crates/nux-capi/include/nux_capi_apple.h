#ifndef NUX_CAPI_APPLE_H
#define NUX_CAPI_APPLE_H

#if !defined(__APPLE__)
#error "nux_capi_apple.h is available only on Apple platforms"
#endif

/* Selects the Apple Metal extension declarations. The linked nux-capi archive
 * must be built with Cargo feature `apple-metal`; the portable archive omits
 * these symbols while retaining the same base ABI-v3 inventory. */
#define NUX_CAPI_APPLE_METAL 1
#include "nux_capi.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Product extension supplied by the composed Nuxie Apple distribution. The
 * product-neutral nux-capi archive does not export this symbol. */
NuxStatus nux_product_file_import_configured(const uint8_t *bytes,
                                              size_t len,
                                              const struct NuxFileImportConfig *config,
                                              struct NuxFile **out_file,
                                              struct NuxCapiResult **out_result);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* NUX_CAPI_APPLE_H */
