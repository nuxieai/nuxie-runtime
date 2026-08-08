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

#endif /* NUX_CAPI_APPLE_H */
