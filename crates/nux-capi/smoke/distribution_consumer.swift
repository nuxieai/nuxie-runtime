import NuxieRuntimeC
import NuxieRuntimeFFI

@main
enum DistributionConsumer {
    static func main() {
        var file: OpaquePointer?
        var result: OpaquePointer?
        let status = nux_file_import_with_result(nil, 0, &file, &result)
        precondition(status != 0)
        precondition(file == nil)
        if let result {
            var diagnostic = NuxCapiDiagnosticView()
            diagnostic.struct_size = UInt32(MemoryLayout<NuxCapiDiagnosticView>.size)
            _ = nux_capi_result_diagnostic(result, &diagnostic)
            _ = nux_capi_result_free(result)
        }
        nux_experience_context_free(nil)
        nux_screen_session_free(nil)
        nux_screen_session_result_free(nil)
        precondition(!nux_screen_session_result_is_settled(nil))
        precondition(nux_capi_abi_version() == NUX_CAPI_ABI_VERSION)
    }
}
