#ifndef NUXIE_FREESTANDING_STDLIB_H
#define NUXIE_FREESTANDING_STDLIB_H
#include <stddef.h>
/* Declaration only; no allocator, libc, or replacement implementation. The
 * verifier does not reach the signing-only abort call. */
_Noreturn void abort(void);
#endif
