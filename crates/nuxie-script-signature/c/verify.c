/* The included cryptographic source files are byte-identical to the pinned
 * libhydrogen-sys package. Only public symbol names are isolated so native
 * differential tests can link this verifier beside the original library. */
#define hydro_hash_init nuxie_script_hydro_hash_init
#define hydro_hash_update nuxie_script_hydro_hash_update
#define hydro_hash_final nuxie_script_hydro_hash_final
#define hydro_hash_hash nuxie_script_hydro_hash_hash
#define hydro_hash_keygen nuxie_script_hydro_hash_keygen
#define hydro_sign_keygen nuxie_script_hydro_sign_keygen
#define hydro_sign_keygen_deterministic nuxie_script_hydro_sign_keygen_deterministic
#define hydro_sign_init nuxie_script_hydro_sign_init
#define hydro_sign_update nuxie_script_hydro_sign_update
#define hydro_sign_final_create nuxie_script_hydro_sign_final_create
#define hydro_sign_final_verify nuxie_script_hydro_sign_final_verify
#define hydro_sign_create nuxie_script_hydro_sign_create
#define hydro_sign_verify nuxie_script_hydro_sign_verify

#include "upstream/hydrogen.h"
#include "upstream/impl/common.h"
#include "upstream/impl/hydrogen_p.h"
#include "upstream/impl/gimli-core.h"
#include "upstream/impl/hash.h"
#include "upstream/impl/x25519.h"
#include "upstream/impl/sign.h"

/* No random provider is defined. The unused signing/keygen functions in the
 * original headers are discarded by the linker. Retaining one must fail the
 * import-free WASM qualification, never obtain a stub RNG. */
