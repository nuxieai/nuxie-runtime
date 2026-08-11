import NuxieRuntimeC

@main
enum ProductExtensionConsumer {
    static func main() {
        var file: OpaquePointer?
        var result: OpaquePointer?
        let status = nux_product_file_import_configured(
            nil,
            0,
            nil,
            &file,
            &result
        )
        precondition(status != NUX_STATUS_OK.rawValue)
        precondition(file == nil)
        if let result {
            _ = nux_capi_result_free(result)
        }
    }
}
