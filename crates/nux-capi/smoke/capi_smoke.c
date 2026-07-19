/* C smoke test for the nux-capi embed loop:
 * import -> artboard instance -> default state machine -> inputs ->
 * advance -> draw through a render-callback vtable.
 *
 * Usage: capi_smoke <path-to-smi_test.riv>
 * Exits 0 and prints "capi-smoke ok" on success.
 */

#include "nux_capi.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Artboard index of "artboard to nest" in fixtures/animation/smi_test.riv,
 * whose default state machine has inputs "bool", "num", and "trig". */
#define SMOKE_ARTBOARD_INDEX 1

#define CHECK(condition)                                                      \
    do                                                                        \
    {                                                                         \
        if (!(condition))                                                     \
        {                                                                     \
            fprintf(stderr,                                                   \
                    "capi-smoke FAILED at %s:%d: %s\n",                       \
                    __FILE__,                                                 \
                    __LINE__,                                                 \
                    #condition);                                              \
            exit(1);                                                          \
        }                                                                     \
    } while (0)

typedef struct SmokeCounters
{
    uint64_t next_handle;
    size_t made;
    size_t released;
    size_t draw_paths;
    size_t saves;
    size_t restores;
} SmokeCounters;

static uint64_t smoke_make_render_path(void* user_data,
                                       const NuxRawPathView* path,
                                       uint8_t fill_rule)
{
    SmokeCounters* counters = (SmokeCounters*)user_data;
    (void)fill_rule;
    CHECK(path != NULL);
    CHECK(path->verb_count == 0 || path->verbs != NULL);
    CHECK(path->point_count == 0 || path->points != NULL);
    counters->made += 1;
    return ++counters->next_handle;
}

static uint64_t smoke_make_handle(void* user_data)
{
    SmokeCounters* counters = (SmokeCounters*)user_data;
    counters->made += 1;
    return ++counters->next_handle;
}

static void smoke_release(void* user_data, uint64_t handle)
{
    SmokeCounters* counters = (SmokeCounters*)user_data;
    CHECK(handle != 0);
    counters->released += 1;
}

static void smoke_draw_path(void* user_data, uint64_t path, uint64_t paint)
{
    SmokeCounters* counters = (SmokeCounters*)user_data;
    CHECK(path != 0);
    CHECK(paint != 0);
    counters->draw_paths += 1;
}

static void smoke_save(void* user_data)
{
    ((SmokeCounters*)user_data)->saves += 1;
}

static void smoke_restore(void* user_data)
{
    ((SmokeCounters*)user_data)->restores += 1;
}

static uint8_t* read_file(const char* path, size_t* out_len)
{
    FILE* file = fopen(path, "rb");
    if (file == NULL)
    {
        fprintf(stderr, "capi-smoke FAILED: cannot open %s\n", path);
        exit(1);
    }
    CHECK(fseek(file, 0, SEEK_END) == 0);
    long size = ftell(file);
    CHECK(size > 0);
    CHECK(fseek(file, 0, SEEK_SET) == 0);
    uint8_t* bytes = (uint8_t*)malloc((size_t)size);
    CHECK(bytes != NULL);
    CHECK(fread(bytes, 1, (size_t)size, file) == (size_t)size);
    fclose(file);
    *out_len = (size_t)size;
    return bytes;
}

int main(int argc, char** argv)
{
    if (argc != 2)
    {
        fprintf(stderr, "usage: capi_smoke <path-to-smi_test.riv>\n");
        return 1;
    }

    CHECK(nux_capi_abi_version() == NUX_CAPI_ABI_VERSION);
    CHECK(nux_capi_require_abi(NUX_CAPI_ABI_VERSION) == NUX_STATUS_OK);
    CHECK(nux_capi_require_abi(NUX_CAPI_ABI_VERSION + 1) ==
          NUX_STATUS_ABI_MISMATCH);
    NuxRuntimeInfo runtime_info = {0};
    CHECK(nux_capi_runtime_info(&runtime_info) == NUX_STATUS_OK);
    CHECK(runtime_info.abi_version == NUX_CAPI_ABI_VERSION);
    CHECK(runtime_info.runtime_version.data != NULL);
    CHECK(runtime_info.runtime_version.len > 0);
    CHECK(runtime_info.source_revision.data != NULL);
    CHECK(runtime_info.source_revision.len > 0);

    size_t len = 0;
    uint8_t* bytes = read_file(argv[1], &len);

    NuxFile* file = NULL;
    CHECK(nux_file_import(bytes, len, &file) == NUX_STATUS_OK);
    CHECK(file != NULL);
    free(bytes);

    CHECK(nux_file_artboard_count(file) > SMOKE_ARTBOARD_INDEX);

    size_t state_machine_count = 0;
    CHECK(nux_file_artboard_state_machine_count(
              file, SMOKE_ARTBOARD_INDEX, &state_machine_count) == NUX_STATUS_OK);
    CHECK(state_machine_count >= 1);

    NuxStringView state_machine_name = {NULL, 0};
    CHECK(nux_file_artboard_state_machine_name(
              file, SMOKE_ARTBOARD_INDEX, 0, &state_machine_name) == NUX_STATUS_OK);
    CHECK(state_machine_name.len == strlen("State Machine 1"));
    CHECK(memcmp(state_machine_name.data,
                 "State Machine 1",
                 state_machine_name.len) == 0);

    NuxArtboardInstance* instance = NULL;
    CHECK(nux_artboard_instance_new(file, SMOKE_ARTBOARD_INDEX, &instance) ==
          NUX_STATUS_OK);
    CHECK(instance != NULL);

    NuxStateMachineInstance* state_machine = NULL;
    CHECK(nux_state_machine_instance_new_default(instance, &state_machine) ==
          NUX_STATUS_OK);
    CHECK(state_machine != NULL);

    CHECK(nux_state_machine_instance_set_bool(state_machine, "bool", true) ==
          NUX_STATUS_OK);
    CHECK(nux_state_machine_instance_set_number(state_machine, "num", 42.0f) ==
          NUX_STATUS_OK);
    CHECK(nux_state_machine_instance_fire_trigger(state_machine, "trig") ==
          NUX_STATUS_OK);
    CHECK(nux_state_machine_instance_set_bool(state_machine, "missing", true) ==
          NUX_STATUS_NOT_FOUND);
    CHECK(nux_state_machine_instance_set_number(state_machine, "bool", 1.0f) ==
          NUX_STATUS_INVALID_ARGUMENT);

    bool changed = false;
    CHECK(nux_state_machine_instance_advance(
              instance, state_machine, 0.016f, &changed) == NUX_STATUS_OK);
    CHECK(nux_state_machine_instance_advance(
              instance, state_machine, 0.016f, NULL) == NUX_STATUS_OK);

    /* Pointer events: down/move/up must succeed (with and without out_hit)
     * and the state machine must still advance cleanly afterwards. */
    bool hit = true;
    CHECK(nux_state_machine_instance_pointer_down(
              instance, state_machine, 10.0f, 10.0f, &hit) == NUX_STATUS_OK);
    CHECK(nux_state_machine_instance_pointer_move(
              instance, state_machine, 12.0f, 12.0f, NULL) == NUX_STATUS_OK);
    CHECK(nux_state_machine_instance_pointer_up(
              instance, state_machine, 12.0f, 12.0f, &hit) == NUX_STATUS_OK);
    CHECK(nux_state_machine_instance_pointer_down(
              NULL, state_machine, 0.0f, 0.0f, NULL) ==
          NUX_STATUS_NULL_ARGUMENT);
    CHECK(nux_state_machine_instance_advance(
              instance, state_machine, 0.016f, NULL) == NUX_STATUS_OK);

    /* View-model surface. This repo-local fixture's artboard declares no view
     * model, so the default constructor must report NOT_FOUND; this still
     * exercises the C linkage of the view-model ABI and its NULL handling.
     * (A functional set/bind is covered by the Rust tests against a databind
     * fixture, since no repo-local fixture ships a settable view model.) */
    NuxViewModelInstance* view_model = NULL;
    CHECK(nux_view_model_instance_new_default(instance, &view_model) ==
          NUX_STATUS_NOT_FOUND);
    CHECK(view_model == NULL);
    CHECK(nux_view_model_instance_set_number(NULL, "num", 1.0f) ==
          NUX_STATUS_NULL_ARGUMENT);
    CHECK(nux_artboard_instance_bind_view_model(instance, NULL) ==
          NUX_STATUS_NULL_ARGUMENT);
    nux_view_model_instance_free(view_model); /* NULL-safe */

    SmokeCounters counters;
    memset(&counters, 0, sizeof(counters));

    NuxRenderCallbacks callbacks;
    memset(&callbacks, 0, sizeof(callbacks));
    callbacks.user_data = &counters;
    callbacks.make_render_path = smoke_make_render_path;
    callbacks.make_empty_render_path = smoke_make_handle;
    callbacks.make_render_paint = smoke_make_handle;
    callbacks.release_render_path = smoke_release;
    callbacks.release_render_paint = smoke_release;
    callbacks.release_render_shader = smoke_release;
    callbacks.draw_path = smoke_draw_path;
    callbacks.save = smoke_save;
    callbacks.restore = smoke_restore;

    NuxRenderCache* render_cache = NULL;
    CHECK(nux_render_cache_new(instance, &callbacks, &render_cache) ==
          NUX_STATUS_OK);
    CHECK(render_cache != NULL);
    CHECK(nux_artboard_instance_draw_cached(instance, render_cache) ==
          NUX_STATUS_OK);
    size_t made_after_first_draw = counters.made;
    size_t released_after_first_draw = counters.released;
    CHECK(nux_artboard_instance_draw_cached(instance, render_cache) ==
          NUX_STATUS_OK);
    CHECK(counters.made == made_after_first_draw);
    CHECK(counters.released == released_after_first_draw);
    CHECK(counters.draw_paths > 0);
    CHECK(counters.saves == counters.restores);
    CHECK(counters.made > 0);
    nux_render_cache_free(render_cache);
    CHECK(counters.made == counters.released);

    /* A fully NULL vtable must also draw cleanly (null renderer). */
    NuxRenderCallbacks null_callbacks;
    memset(&null_callbacks, 0, sizeof(null_callbacks));
    CHECK(nux_artboard_instance_draw(instance, &null_callbacks) ==
          NUX_STATUS_OK);

    nux_state_machine_instance_free(state_machine);
    nux_artboard_instance_free(instance);
    nux_file_free(file);

    printf("capi-smoke ok (draw_paths=%zu objects=%zu)\n",
           counters.draw_paths,
           counters.made);
    return 0;
}
