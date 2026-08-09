#include "nux_runtime.h"

#include <stddef.h>

int main(void) {
  /* The v0.4.0 migration release intentionally retains the allowlisted
   * product-shaped lane. Prove its null-safe lifecycle roots still link and
   * execute without coupling a C translation unit to both header families. */
  nux_experience_context_free(NULL);
  nux_screen_session_free(NULL);
  nux_screen_session_result_free(NULL);
  return nux_screen_session_result_is_settled(NULL) ? 1 : 0;
}
