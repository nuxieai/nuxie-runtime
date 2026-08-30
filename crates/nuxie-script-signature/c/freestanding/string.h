#ifndef NUXIE_FREESTANDING_STRING_H
#define NUXIE_FREESTANDING_STRING_H
#include <stddef.h>
/* Rust's compiler-builtins supplies any non-inlined memory operation. */
void *memcpy(void *restrict destination, const void *restrict source, size_t length);
#endif
