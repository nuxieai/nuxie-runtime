//! Private ABI used by js/index.mjs. Rust owns every buffer: the host never
//! passes an arbitrary pointer or allocation length back for Rust to free.
use crate::{CompileInput, Diagnostic, LANGUAGE_VERSION, compile};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

const MAX_REQUEST_BYTES: usize = 192 * 1024 * 1024;

#[derive(Default)]
struct Bridge {
    request: Vec<u8>,
    metadata: Vec<u8>,
    riv: Vec<u8>,
}

thread_local! { static BRIDGE: RefCell<Bridge> = RefCell::new(Bridge::default()); }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    language_version: String,
    input: CompileInput,
}

// Serialize the typed fields directly: an intermediate serde_json::Value
// widens f32 metrics to f64 and changes their JSON numeric representation.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SuccessMetadata<'a> {
    ok: bool,
    language_version: &'a str,
    source_map: &'a [crate::SourceNode],
    runtime_requirements: &'a crate::RuntimeRequirements,
}

#[unsafe(no_mangle)]
pub extern "C" fn html_compiler_abi_version() -> u32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn html_compiler_request_alloc(len: u32) -> u32 {
    BRIDGE.with_borrow_mut(|bridge| {
        *bridge = Bridge::default();
        if len == 0 || len as usize > MAX_REQUEST_BYTES {
            return 0;
        }
        if bridge.request.try_reserve_exact(len as usize).is_err() {
            return 0;
        }
        bridge.request.resize(len as usize, 0);
        bridge.request.as_mut_ptr() as u32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn html_compiler_compile() -> u32 {
    BRIDGE.with_borrow_mut(|bridge| {
        bridge.riv.clear();
        let result = serde_json::from_slice::<Request>(&bridge.request)
            .map_err(|e| vec![Diagnostic::new("invalid-request", "request", e.to_string())])
            .and_then(|request| {
                if request.language_version != LANGUAGE_VERSION {
                    Err(vec![Diagnostic::new(
                        "unsupported-language-version",
                        "languageVersion",
                        format!("Expected {LANGUAGE_VERSION}"),
                    )])
                } else {
                    compile(&request.input)
                }
            });
        let (status, metadata) = match result {
            Ok(output) => {
                bridge.riv = output.riv;
                (
                    0,
                    serde_json::to_vec(&SuccessMetadata {
                        ok: true,
                        language_version: LANGUAGE_VERSION,
                        source_map: &output.source_map,
                        runtime_requirements: &output.runtime_requirements,
                    })
                    .expect("serializable compiler metadata"),
                )
            }
            Err(diagnostics) => (
                1,
                serde_json::to_vec(&serde_json::json!({"ok":false,"diagnostics":diagnostics}))
                    .expect("serializable diagnostics"),
            ),
        };
        bridge.metadata = metadata;
        bridge.request.clear();
        status
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn html_compiler_metadata_ptr() -> u32 {
    BRIDGE.with_borrow(|b| b.metadata.as_ptr() as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn html_compiler_metadata_len() -> u32 {
    BRIDGE.with_borrow(|b| b.metadata.len() as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn html_compiler_riv_ptr() -> u32 {
    BRIDGE.with_borrow(|b| b.riv.as_ptr() as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn html_compiler_riv_len() -> u32 {
    BRIDGE.with_borrow(|b| b.riv.len() as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn html_compiler_reset() {
    BRIDGE.with_borrow_mut(|b| *b = Bridge::default());
}
