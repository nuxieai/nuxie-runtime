#ifndef RIVE_CAPI_H
#define RIVE_CAPI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum RiveStatus
{
    RIVE_STATUS_OK = 0,
    RIVE_STATUS_NULL_ARGUMENT = 1,
    RIVE_STATUS_IMPORT_ERROR = 2,
    RIVE_STATUS_NOT_FOUND = 3,
} RiveStatus;

typedef struct RiveFile RiveFile;

typedef struct RiveStringView
{
    const char* data;
    size_t len;
} RiveStringView;

RiveStatus rive_file_import(const uint8_t* bytes, size_t len, RiveFile** out_file);
void rive_file_free(RiveFile* file);

size_t rive_file_artboard_count(const RiveFile* file);
RiveStatus rive_file_artboard_name(
    const RiveFile* file,
    size_t index,
    RiveStringView* out_name);
RiveStatus rive_file_artboard_animation_count(
    const RiveFile* file,
    size_t index,
    size_t* out_count);
RiveStatus rive_file_artboard_state_machine_count(
    const RiveFile* file,
    size_t index,
    size_t* out_count);

#ifdef __cplusplus
}
#endif

#endif
