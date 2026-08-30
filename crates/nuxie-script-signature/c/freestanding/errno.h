#ifndef NUXIE_FREESTANDING_ERRNO_H
#define NUXIE_FREESTANDING_ERRNO_H
/* common.h includes errno.h, but the selected verifier never uses errno.
 * No errno definition is supplied: a new use must fail compilation. */
#endif
