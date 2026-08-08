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
    NuxRuntimeInfo runtime_info = {.struct_size = sizeof(NuxRuntimeInfo)};
    CHECK(nux_capi_runtime_info(&runtime_info) == NUX_STATUS_OK);
    CHECK(runtime_info.abi_version == NUX_CAPI_ABI_VERSION);
    CHECK(runtime_info.runtime_version.data != NULL);
    CHECK(runtime_info.runtime_version.len > 0);
    CHECK(runtime_info.source_revision.data != NULL);
    CHECK(runtime_info.source_revision.len > 0);

    NuxViewModelMutationBatch empty_batch = {
        .struct_size = sizeof(NuxViewModelMutationBatch),
        .mutations = NULL,
        .mutation_count = 0,
    };
    NuxViewModelMutationResult* empty_result = NULL;
    CHECK(nux_view_model_mutate(&empty_batch, &empty_result) == NUX_STATUS_OK);
    CHECK(empty_result != NULL);
    NuxViewModelMutationResultInfo empty_result_info = {
        .struct_size = sizeof(NuxViewModelMutationResultInfo)};
    CHECK(nux_view_model_mutation_result_info(empty_result, &empty_result_info) ==
          NUX_STATUS_OK);
    CHECK(empty_result_info.status == NUX_STATUS_OK);
    CHECK(empty_result_info.applied_count == 0);
    CHECK(nux_view_model_mutation_result_free(empty_result) == NUX_STATUS_OK);

    size_t len = 0;
    uint8_t* bytes = read_file(argv[1], &len);

    NuxFile* file = NULL;
    NuxCapiResult* import_result = NULL;
    CHECK(nux_file_import_with_result(bytes, len, &file, &import_result) ==
          NUX_STATUS_OK);
    CHECK(file != NULL);
    CHECK(import_result != NULL);
    NuxStatus import_result_status = NUX_STATUS_RUNTIME_ERROR;
    CHECK(nux_capi_result_status(import_result, &import_result_status) ==
          NUX_STATUS_OK);
    CHECK(import_result_status == NUX_STATUS_OK);
    NuxCapiDiagnosticView import_diagnostic = {
        .struct_size = sizeof(NuxCapiDiagnosticView)};
    CHECK(nux_capi_result_diagnostic(import_result, &import_diagnostic) ==
          NUX_STATUS_OK);
    CHECK(import_diagnostic.status == NUX_STATUS_OK);
    CHECK(nux_capi_result_free(import_result) == NUX_STATUS_OK);
    free(bytes);

    NuxViewModelCatalog* catalog = NULL;
    CHECK(nux_file_view_model_catalog(file, &catalog) == NUX_STATUS_OK);
    CHECK(catalog != NULL);
    NuxViewModelCatalogInfo catalog_info = {
        .struct_size = sizeof(NuxViewModelCatalogInfo)};
    CHECK(nux_view_model_catalog_info(catalog, &catalog_info) == NUX_STATUS_OK);
    CHECK(nux_view_model_catalog_schema(catalog,
                                        catalog_info.schema_count,
                                        &(NuxViewModelSchemaView){
                                            .struct_size = sizeof(NuxViewModelSchemaView)}) ==
          NUX_STATUS_NOT_FOUND);

    size_t artboard_count = 0;
    CHECK(nux_file_artboard_count(file, &artboard_count) == NUX_STATUS_OK);
    CHECK(artboard_count > SMOKE_ARTBOARD_INDEX);

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

    size_t animation_count = 0;
    CHECK(nux_file_artboard_animation_count(
              file, SMOKE_ARTBOARD_INDEX, &animation_count) == NUX_STATUS_OK);
    CHECK(animation_count >= 1);
    NuxStringView animation_name = {NULL, 0};
    CHECK(nux_file_artboard_animation_name(
              file, SMOKE_ARTBOARD_INDEX, 0, &animation_name) == NUX_STATUS_OK);
    CHECK(animation_name.len == strlen("Timeline 1"));

    NuxArtboardInstance* instance = NULL;
    CHECK(nux_artboard_instance_new(file, SMOKE_ARTBOARD_INDEX, &instance) ==
          NUX_STATUS_OK);
    CHECK(instance != NULL);

    NuxStateMachineInstance* state_machine = NULL;
    CHECK(nux_state_machine_instance_new_default(instance, &state_machine) ==
          NUX_STATUS_OK);
    CHECK(state_machine != NULL);

    NuxPlayer* animation_player = NULL;
    NuxCapiResult* animation_result = NULL;
    CHECK(nux_player_new_linear_animation_named_with_result(
              instance, animation_name, &animation_player, &animation_result) ==
          NUX_STATUS_OK);
    CHECK(animation_player != NULL);
    CHECK(animation_result != NULL);
    CHECK(nux_capi_result_free(animation_result) == NUX_STATUS_OK);
    NuxPlayerInfo animation_info = {.struct_size = sizeof(NuxPlayerInfo)};
    CHECK(nux_player_info(animation_player, &animation_info) == NUX_STATUS_OK);
    CHECK(animation_info.kind == NUX_PLAYER_KIND_LINEAR_ANIMATION);
    CHECK(nux_player_free(animation_player) == NUX_STATUS_OK);

    NuxPlayer* player = NULL;
    NuxCapiResult* player_result = NULL;
    CHECK(nux_player_new_default_with_result(instance, &player, &player_result) ==
          NUX_STATUS_OK);
    CHECK(player != NULL);
    CHECK(player_result != NULL);
    CHECK(nux_capi_result_free(player_result) == NUX_STATUS_OK);
    NuxPlayerInfo player_info = {.struct_size = sizeof(NuxPlayerInfo)};
    CHECK(nux_player_info(player, &player_info) == NUX_STATUS_OK);
    CHECK(player_info.kind == NUX_PLAYER_KIND_STATE_MACHINE);
    CHECK(player_info.name.len == strlen("State Machine 1"));

    NuxPlayerInputChange step_inputs[] = {
        {.kind = NUX_PLAYER_INPUT_KIND_BOOL,
         .name = {.data = "bool", .len = 4},
         .bool_value = 1},
        {.kind = NUX_PLAYER_INPUT_KIND_NUMBER,
         .name = {.data = "num", .len = 3},
         .number_value = 42.0f},
        {.kind = NUX_PLAYER_INPUT_KIND_TRIGGER,
         .name = {.data = "trig", .len = 4}},
    };
    NuxPlayerStep player_step = {
        .struct_size = sizeof(NuxPlayerStep),
        .inputs = step_inputs,
        .input_count = sizeof(step_inputs) / sizeof(step_inputs[0]),
        .elapsed_seconds = 0.016f,
    };
    NuxPlayerStepResult* step_result = NULL;
    CHECK(nux_player_step(player, &player_step, &step_result) == NUX_STATUS_OK);
    CHECK(step_result != NULL);
    NuxPlayerStepInfo step_info = {.struct_size = sizeof(NuxPlayerStepInfo)};
    CHECK(nux_player_step_result_info(step_result, &step_info) == NUX_STATUS_OK);
    CHECK(step_info.pointer_result_count == 0);
    if (step_info.state_change_count != 0)
    {
        NuxPlayerStateChangeView state = {
            .struct_size = sizeof(NuxPlayerStateChangeView)};
        CHECK(nux_player_step_result_state_change(step_result, 0, &state) ==
              NUX_STATUS_OK);
        CHECK(state.state_core_type != 0);
    }
    CHECK(nux_player_step_result_free(step_result) == NUX_STATUS_OK);

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
    callbacks.struct_size = sizeof(callbacks);
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

    CHECK(nux_artboard_instance_draw(instance, &callbacks) == NUX_STATUS_OK);
    size_t made_after_first_draw = counters.made;
    size_t released_after_first_draw = counters.released;
    CHECK(nux_artboard_instance_draw(instance, &callbacks) == NUX_STATUS_OK);
    CHECK(counters.made == made_after_first_draw);
    CHECK(counters.released == released_after_first_draw);
    CHECK(counters.draw_paths > 0);
    CHECK(counters.saves == counters.restores);
    CHECK(counters.made > 0);
    /* Deliberately release public parents first. The player retains the native
     * artboard occurrence and renderer binding through its own last release. */
    CHECK(nux_file_free(file) == NUX_STATUS_OK);
    CHECK(nux_state_machine_instance_free(state_machine) == NUX_STATUS_OK);
    CHECK(nux_artboard_instance_free(instance) == NUX_STATUS_OK);
    CHECK(nux_player_info(player, &player_info) == NUX_STATUS_OK);
    CHECK(counters.made != counters.released);
    CHECK(nux_player_free(player) == NUX_STATUS_OK);
    CHECK(nux_view_model_catalog_info(catalog, &catalog_info) == NUX_STATUS_OK);
    CHECK(nux_view_model_catalog_free(catalog) == NUX_STATUS_OK);
    CHECK(counters.made == counters.released);

    printf("capi-smoke ok (draw_paths=%zu objects=%zu)\n",
           counters.draw_paths,
           counters.made);
    return 0;
}
