import Foundation
import NuxieRuntimeC

func check(_ condition: @autoclosure () -> Bool, _ message: String) {
    guard condition() else {
        FileHandle.standardError.write(Data("swift-capi-smoke FAILED: \(message)\n".utf8))
        exit(1)
    }
}

guard CommandLine.arguments.count == 2 else {
    FileHandle.standardError.write(Data("usage: capi_lifetime <smi_test.riv>\n".utf8))
    exit(2)
}

var bytes: Data? = try Data(contentsOf: URL(fileURLWithPath: CommandLine.arguments[1]))
var file: OpaquePointer?
var importResult: OpaquePointer?
let importStatus = bytes!.withUnsafeBytes { raw in
    nux_file_import_with_result(
        raw.bindMemory(to: UInt8.self).baseAddress,
        raw.count,
        &file,
        &importResult
    )
}
check(importStatus == NUX_STATUS_OK.rawValue, "import")
check(file != nil && importResult != nil, "import outputs")
check(nux_capi_result_free(importResult) == NUX_STATUS_OK.rawValue, "free import result")
bytes = nil // The runtime must own everything it needs after import returns.

var assetCount = 0
check(nux_file_asset_count(file, &assetCount) == NUX_STATUS_OK.rawValue, "asset catalog count")
var asset = NuxFileAssetDescriptorView()
asset.struct_size = UInt32(MemoryLayout<NuxFileAssetDescriptorView>.size)
if assetCount == 0 {
    check(nux_file_asset_descriptor(file, 0, &asset) == NUX_STATUS_NOT_FOUND.rawValue,
          "empty asset catalog")
} else {
    check(nux_file_asset_descriptor(file, 0, &asset) == NUX_STATUS_OK.rawValue,
          "first asset descriptor")
}

var catalog: OpaquePointer?
check(nux_file_view_model_catalog(file, &catalog) == NUX_STATUS_OK.rawValue, "catalog")
var catalogInfo = NuxViewModelCatalogInfo()
catalogInfo.struct_size = UInt32(MemoryLayout<NuxViewModelCatalogInfo>.size)
check(nux_view_model_catalog_info(catalog, &catalogInfo) == NUX_STATUS_OK.rawValue, "catalog info")

var artboard: OpaquePointer?
check(nux_artboard_instance_new(file, 1, &artboard) == NUX_STATUS_OK.rawValue, "artboard")
var player: OpaquePointer?
var playerResult: OpaquePointer?
check(
    nux_player_new_default_with_result(artboard, &player, &playerResult) == NUX_STATUS_OK.rawValue,
    "default player"
)
check(player != nil && playerResult != nil, "player outputs")
check(nux_capi_result_free(playerResult) == NUX_STATUS_OK.rawValue, "free player result")

// Deliberately release the public parents first. The player owns the native
// artboard occurrence and copied metadata, so this order must remain valid.
check(nux_file_free(file) == NUX_STATUS_OK.rawValue, "file-first release")
check(nux_artboard_instance_free(artboard) == NUX_STATUS_OK.rawValue, "artboard-second release")
check(nux_view_model_catalog_info(catalog, &catalogInfo) == NUX_STATUS_OK.rawValue,
      "owned catalog after file release")
check(nux_view_model_catalog_free(catalog) == NUX_STATUS_OK.rawValue, "catalog release")

var info = NuxPlayerInfo()
info.struct_size = UInt32(MemoryLayout<NuxPlayerInfo>.size)
check(nux_player_info(player, &info) == NUX_STATUS_OK.rawValue, "player metadata after parent release")
check(info.kind == NUX_PLAYER_KIND_STATE_MACHINE.rawValue, "default player kind")
let name = UnsafeRawBufferPointer(
    start: info.name.data,
    count: info.name.len
)
check(String(decoding: name, as: UTF8.self) == "State Machine 1", "owned player name")

var step = NuxPlayerStep()
step.struct_size = UInt32(MemoryLayout<NuxPlayerStep>.size)
step.elapsed_seconds = 0
var stepResult: OpaquePointer?
check(nux_player_step(player, &step, &stepResult) == NUX_STATUS_OK.rawValue,
      "player step")
var scheduling = NuxPlayerSchedulingInfo()
scheduling.struct_size = UInt32(MemoryLayout<NuxPlayerSchedulingInfo>.size)
check(nux_player_step_result_scheduling(stepResult, &scheduling) == NUX_STATUS_OK.rawValue,
      "scheduling snapshot")
check(scheduling.dirty, "initial settlement changed observable runtime state")
check(!scheduling.settled, "active state-machine work remains unsettled")
check(scheduling.render_required, "initial occurrence requires presentation")
check(scheduling.render_revision != 0, "nonzero render revision")
check(!scheduling.has_wake_deadline && scheduling.wake_deadline_monotonic_ns == 0,
      "runtime does not manufacture a wake deadline")
check(scheduling.wake_deadline_clock == NUX_MONOTONIC_CLOCK_DOMAIN_UNSPECIFIED.rawValue,
      "an absent deadline has no clock domain")
check(nux_player_acknowledge_presented(player, scheduling.render_revision) == NUX_STATUS_OK.rawValue,
      "acknowledge exact presented revision")
check(nux_player_step_result_free(stepResult) == NUX_STATUS_OK.rawValue,
      "free scheduling step")
check(nux_player_free(player) == NUX_STATUS_OK.rawValue, "player-last release")

print("swift-capi-lifetime ok")
