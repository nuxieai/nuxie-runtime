//! Mechanical translation of src/scriptnet/net_provider_windows.cpp.
//!
//! WinHTTP owns one Arc per request handle until HANDLE_CLOSING. CallGuard
//! defers handle closure while any thread is inside an API call. In particular,
//! no WinHTTP call is made while either of our mutexes is held: callbacks may
//! run synchronously and re-enter the request on the calling thread.
use super::http::{
    HttpHeader, HttpHeaders, HttpRequest, HttpResponse, NetError, NetErrorCode,
    standard_reason_phrase,
};
use super::net::{self, Provider, RequestId};
use super::net_policy::{NetLimits, NetPolicy};
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::{size_of, take};
use std::ptr::{null, null_mut};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, GetLastError, LocalFree};
use windows_sys::Win32::Globalization::{
    CP_ACP, CP_UTF8, MultiByteToWideChar, WideCharToMultiByte,
};
use windows_sys::Win32::Networking::WinHttp::*;
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6, InetPtonW};
use windows_sys::Win32::System::Diagnostics::Debug::{
    FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_HMODULE, FORMAT_MESSAGE_FROM_SYSTEM,
    FORMAT_MESSAGE_IGNORE_INSERTS, FormatMessageW,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

const READ_CHUNK_BYTES: usize = 64 * 1024;
// SDK expression macros omitted from windows-sys 0.52's generated bindings.
const WINHTTP_DECOMPRESSION_FLAG_ALL: u32 =
    WINHTTP_DECOMPRESSION_FLAG_GZIP | WINHTTP_DECOMPRESSION_FLAG_DEFLATE;
const WINHTTP_CALLBACK_FLAG_HANDLES: u32 =
    WINHTTP_CALLBACK_STATUS_HANDLE_CREATED | WINHTTP_CALLBACK_STATUS_HANDLE_CLOSING;
const WINHTTP_CALLBACK_FLAG_ALL_COMPLETIONS: u32 = WINHTTP_CALLBACK_STATUS_SENDREQUEST_COMPLETE
    | WINHTTP_CALLBACK_STATUS_HEADERS_AVAILABLE
    | WINHTTP_CALLBACK_STATUS_DATA_AVAILABLE
    | WINHTTP_CALLBACK_STATUS_READ_COMPLETE
    | WINHTTP_CALLBACK_STATUS_WRITE_COMPLETE
    | WINHTTP_CALLBACK_STATUS_REQUEST_ERROR
    | WINHTTP_CALLBACK_STATUS_GETPROXYFORURL_COMPLETE
    | WINHTTP_CALLBACK_STATUS_GETPROXYSETTINGS_COMPLETE;
// WinHTTP handles may cross threads; ownership and closure are governed by
// Registry and CallGuard, not by a Rust reference to their opaque allocation.
type Handle = usize;
fn handle(value: Handle) -> *mut c_void {
    value as *mut c_void
}
fn close(value: Handle) {
    if value != 0 {
        unsafe {
            WinHttpCloseHandle(handle(value));
        }
    }
}
fn error(code: NetErrorCode, message: impl Into<String>) -> NetError {
    NetError {
        code,
        message: message.into(),
    }
}
fn network_error() -> NetError {
    error(
        NetErrorCode::Network,
        error_message(unsafe { GetLastError() }),
    )
}
fn wide(bytes: &[u8]) -> Vec<u16> {
    if bytes.is_empty() {
        return Vec::new();
    }
    unsafe {
        let len = MultiByteToWideChar(
            CP_UTF8,
            0,
            bytes.as_ptr(),
            bytes.len() as i32,
            null_mut(),
            0,
        );
        if len <= 0 {
            return Vec::new();
        }
        let mut result = vec![0; len as usize];
        MultiByteToWideChar(
            CP_UTF8,
            0,
            bytes.as_ptr(),
            bytes.len() as i32,
            result.as_mut_ptr(),
            len,
        );
        result
    }
}
fn terminated(mut value: Vec<u16>) -> Vec<u16> {
    value.push(0);
    value
}
fn narrow(text: &[u16], codepage: u32) -> Vec<u8> {
    if text.is_empty() {
        return Vec::new();
    }
    unsafe {
        let len = WideCharToMultiByte(
            codepage,
            0,
            text.as_ptr(),
            text.len() as i32,
            null_mut(),
            0,
            null(),
            null_mut(),
        );
        if len <= 0 {
            return Vec::new();
        }
        let mut result = vec![0; len as usize];
        WideCharToMultiByte(
            codepage,
            0,
            text.as_ptr(),
            text.len() as i32,
            result.as_mut_ptr(),
            len,
            null(),
            null_mut(),
        );
        result
    }
}
fn error_message(code: u32) -> String {
    unsafe {
        let mut text: *mut u16 = null_mut();
        let module = terminated(wide(b"winhttp.dll"));
        let len = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER
                | FORMAT_MESSAGE_FROM_SYSTEM
                | FORMAT_MESSAGE_FROM_HMODULE
                | FORMAT_MESSAGE_IGNORE_INSERTS,
            GetModuleHandleW(module.as_ptr()) as *const c_void,
            code,
            0,
            (&mut text as *mut *mut u16).cast(),
            0,
            null(),
        );
        let mut message = if text.is_null() {
            String::new()
        } else {
            let result = String::from_utf8(narrow(
                std::slice::from_raw_parts(text, len as usize),
                CP_UTF8,
            ))
            .expect("Windows UTF-8 conversion");
            LocalFree(text.cast());
            result
        };
        while message.ends_with(['\r', '\n', ' ']) {
            message.pop();
        }
        if message.is_empty() {
            format!("request failed (error {code})")
        } else {
            message
        }
    }
}
fn query_header(request: Handle, info: u32) -> Option<Vec<u16>> {
    unsafe {
        let mut size = 0;
        WinHttpQueryHeaders(
            handle(request),
            info,
            null(),
            null_mut(),
            &mut size,
            null_mut(),
        );
        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            return None;
        }
        let mut buffer = vec![0; size as usize / size_of::<u16>()];
        if WinHttpQueryHeaders(
            handle(request),
            info,
            null(),
            buffer.as_mut_ptr().cast(),
            &mut size,
            null_mut(),
        ) == 0
        {
            return None;
        }
        buffer.truncate(buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len()));
        Some(buffer)
    }
}
fn query_number(request: Handle, info: u32) -> Option<u32> {
    let mut value = 0u32;
    let mut size = size_of::<u32>() as u32;
    (unsafe {
        WinHttpQueryHeaders(
            handle(request),
            info | WINHTTP_QUERY_FLAG_NUMBER,
            null(),
            (&mut value as *mut u32).cast(),
            &mut size,
            null_mut(),
        )
    } != 0)
        .then_some(value)
}
fn parse_headers(raw: &[u16]) -> HttpHeaders {
    let mut result = Vec::new();
    let mut start = 0;
    let mut status_line = true;
    while start < raw.len() {
        let end = raw[start..]
            .windows(2)
            .position(|s| s == [13, 10])
            .map_or(raw.len(), |at| start + at);
        let line = &raw[start..end];
        start = end + 2;
        if status_line {
            status_line = false;
            continue;
        }
        let Some(colon) = line.iter().position(|&c| c == b':' as u16) else {
            continue;
        };
        let mut first = colon + 1;
        let mut last = line.len();
        while first < last && [32, 9].contains(&line[first]) {
            first += 1;
        }
        while last > first && [32, 9].contains(&line[last - 1]) {
            last -= 1;
        }
        // WinHTTP decoded header octets using the system ANSI codepage.
        // Undo exactly that conversion, including raw UTF-8 Location bytes.
        result.push(HttpHeader {
            name: narrow(&line[..colon], CP_ACP),
            value: narrow(&line[first..last], CP_ACP),
        });
    }
    result
}
fn find_header<'a>(headers: &'a HttpHeaders, name: &[u8]) -> Option<&'a [u8]> {
    headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| h.value.as_slice())
}
fn expects_body(head: bool, status: u16) -> bool {
    !head && !matches!(status, 101 | 103 | 204 | 205 | 304)
}
// strtoull's decimal prefix, whitespace, sign, and saturation behavior.
fn content_length(bytes: &[u8]) -> u64 {
    let mut at = 0;
    while at < bytes.len() && matches!(bytes[at], b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c) {
        at += 1;
    }
    let negative = bytes.get(at) == Some(&b'-');
    if matches!(bytes.get(at), Some(b'+') | Some(b'-')) {
        at += 1;
    }
    let mut value = 0u64;
    let mut overflow = false;
    while let Some(&c @ b'0'..=b'9') = bytes.get(at) {
        match value
            .checked_mul(10)
            .and_then(|v| v.checked_add((c - b'0') as u64))
        {
            Some(v) => value = v,
            None => {
                overflow = true;
                value = u64::MAX;
            }
        }
        at += 1;
    }
    if overflow {
        u64::MAX
    } else if negative {
        value.wrapping_neg()
    } else {
        value
    }
}
unsafe fn wide_c_string<'a>(text: *const u16) -> &'a [u16] {
    let mut len = 0;
    unsafe {
        while *text.add(len) != 0 {
            len += 1;
        }
        std::slice::from_raw_parts(text, len)
    }
}
fn parse_address(text: &[u16]) -> Option<Vec<u8>> {
    let start = usize::from(text.first() == Some(&(b'[' as u16)));
    let tail = &text[start..];
    let end = tail
        .iter()
        .position(|&c| c == b']' as u16 || c == b'%' as u16)
        .unwrap_or(tail.len());
    let address = &tail[..end];
    let ipv6 = address.contains(&(b':' as u16));
    let address = terminated(address.to_vec());
    let mut bytes = [0u8; 16];
    if unsafe {
        InetPtonW(
            if ipv6 {
                AF_INET6 as i32
            } else {
                AF_INET as i32
            },
            address.as_ptr(),
            bytes.as_mut_ptr().cast(),
        )
    } != 1
    {
        return None;
    }
    Some(bytes[..if ipv6 { 16 } else { 4 }].to_vec())
}
// Windows' CRT towlower has the same wchar_t/wint_t representation used by
// the upstream C++ implementation, including its locale-sensitive comparison.
unsafe extern "C" {
    fn towlower(value: u16) -> u16;
}
fn same_host(a: &[u16], b: &[u16]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(&x, &y)| unsafe { towlower(x) == towlower(y) })
}
#[derive(Default)]
struct RegistryState {
    session: Handle,
    requests: HashMap<RequestId, Arc<Request>>,
    stopping: bool,
}
impl RegistryState {
    fn take_session_if_done(&mut self) -> Handle {
        if self.stopping && self.requests.is_empty() {
            take(&mut self.session)
        } else {
            0
        }
    }
}
#[derive(Default)]
struct Registry {
    state: Mutex<RegistryState>,
    wake: Condvar,
}
struct RequestState {
    session: Handle,
    hop: Arc<HttpRequest>,
    hops: u32,
    connection: Handle,
    hop_host: Vec<u16>,
    proxied: bool,
    chunk: Vec<u8>,
    body: Vec<u8>,
    request: Handle,
    calls: usize,
    closing: bool,
    hopping: bool,
    close_sent: bool,
    failed: bool,
    error: NetError,
    completed: bool,
    response: HttpResponse,
}
impl RequestState {
    fn take_close(&mut self) -> Handle {
        if !(self.closing || self.hopping)
            || self.close_sent
            || self.request == 0
            || self.calls != 0
        {
            return 0;
        }
        self.close_sent = true;
        self.request
    }
}
struct Request {
    registry: Arc<Registry>,
    id: RequestId,
    deadline: Instant,
    max_body_bytes: u32,
    allow_local_addresses: bool,
    state: Mutex<RequestState>,
}
struct CallGuard<'a> {
    request: &'a Request,
    held: bool,
}
impl<'a> CallGuard<'a> {
    fn new(request: &'a Request) -> Self {
        let mut state = request.state.lock().unwrap();
        let held = !state.closing && !state.hopping;
        if held {
            state.calls += 1;
        }
        Self { request, held }
    }
}
impl Drop for CallGuard<'_> {
    fn drop(&mut self) {
        if !self.held {
            return;
        }
        let to_close = {
            let mut state = self.request.state.lock().unwrap();
            state.calls -= 1;
            state.take_close()
        };
        close(to_close);
    }
}
impl Request {
    fn new(
        registry: Arc<Registry>,
        id: RequestId,
        request: &HttpRequest,
        limits: &NetLimits,
    ) -> Self {
        Self {
            registry,
            id,
            deadline: Instant::now() + Duration::from_millis(request.timeout_ms as u64),
            max_body_bytes: limits.max_response_body_bytes,
            allow_local_addresses: limits.allow_local_addresses,
            state: Mutex::new(RequestState {
                session: 0,
                hop: Arc::new(request.clone()),
                hops: 0,
                connection: 0,
                hop_host: Vec::new(),
                proxied: false,
                chunk: Vec::new(),
                body: Vec::new(),
                request: 0,
                calls: 0,
                closing: false,
                hopping: false,
                close_sent: false,
                failed: false,
                error: NetError::default(),
                completed: false,
                response: HttpResponse::default(),
            }),
        }
    }
    fn closing(&self) -> bool {
        self.state.lock().unwrap().closing
    }
    fn start(self: &Arc<Self>, session: Handle) {
        self.state.lock().unwrap().session = session;
        self.send_hop();
    }
    fn stop(&self, error: NetError) {
        self.settle(false, Some(error));
    }
    fn abort(&self) {
        self.settle(false, None);
    }
    fn settle(&self, completed: bool, error: Option<NetError>) {
        let to_close = {
            let mut state = self.state.lock().unwrap();
            if state.closing {
                return;
            }
            state.closing = true;
            if let Some(error) = error {
                state.failed = true;
                state.error = error;
            } else {
                state.completed = completed;
            }
            state.take_close()
        };
        close(to_close);
    }
    unsafe extern "system" fn on_status(
        raw_handle: *mut c_void,
        context: usize,
        status: u32,
        info: *mut c_void,
        length: u32,
    ) {
        if context == 0 {
            return;
        }
        // The handle's context owns a strong reference through its final callback.
        let pointer = context as *const Self;
        let request = unsafe {
            Arc::increment_strong_count(pointer);
            Arc::from_raw(pointer)
        };
        if status == WINHTTP_CALLBACK_STATUS_HANDLE_CLOSING {
            request.on_closing();
            // Release precisely the reference installed for this closed hop.
            unsafe {
                drop(Arc::from_raw(pointer));
            }
            return;
        }
        let guard = CallGuard::new(&request);
        if !guard.held {
            return;
        }
        let h = raw_handle as Handle;
        match status {
            WINHTTP_CALLBACK_STATUS_RESOLVING_NAME => {
                request.on_resolving(unsafe { wide_c_string(info.cast()) })
            }
            WINHTTP_CALLBACK_STATUS_CONNECTING_TO_SERVER => {
                request.on_connecting(unsafe { wide_c_string(info.cast()) })
            }
            WINHTTP_CALLBACK_STATUS_SENDREQUEST_COMPLETE => {
                if unsafe { WinHttpReceiveResponse(raw_handle, null_mut()) } == 0 {
                    request.stop(network_error());
                }
            }
            WINHTTP_CALLBACK_STATUS_HEADERS_AVAILABLE => request.on_headers(h),
            WINHTTP_CALLBACK_STATUS_READ_COMPLETE => request.on_read(h, length),
            WINHTTP_CALLBACK_STATUS_REQUEST_ERROR => {
                request.on_error(unsafe { (*(info as *const WINHTTP_ASYNC_RESULT)).dwError })
            }
            _ => {}
        }
    }
    fn send_hop(self: &Arc<Self>) {
        let guard = CallGuard::new(self);
        if !guard.held {
            self.end_without_handle(None);
            return;
        }
        let (hop, session) = {
            let state = self.state.lock().unwrap();
            (state.hop.clone(), state.session)
        };
        let mut url = wide(&hop.url);
        let url_length = url.len() as u32;
        // A zero length makes WinHTTP read a NUL-terminated string.
        url.push(0);
        let mut parts: URL_COMPONENTS = unsafe { std::mem::zeroed() };
        parts.dwStructSize = size_of::<URL_COMPONENTS>() as u32;
        parts.dwSchemeLength = u32::MAX;
        parts.dwHostNameLength = u32::MAX;
        parts.dwUrlPathLength = u32::MAX;
        parts.dwExtraInfoLength = u32::MAX;
        if unsafe { WinHttpCrackUrl(url.as_ptr(), url_length, 0, &mut parts) } == 0
            || ![WINHTTP_INTERNET_SCHEME_HTTP, WINHTTP_INTERNET_SCHEME_HTTPS]
                .contains(&parts.nScheme)
            || parts.dwHostNameLength == 0
        {
            self.fail_before_sending(error(NetErrorCode::Policy, "the URL could not be parsed"));
            return;
        }
        let component = |p: *const u16, n: u32| {
            if n == 0 {
                Vec::new()
            } else {
                unsafe { std::slice::from_raw_parts(p, n as usize).to_vec() }
            }
        };
        let host = component(parts.lpszHostName, parts.dwHostNameLength);
        {
            let mut state = self.state.lock().unwrap();
            state.hop_host = host.clone();
            state.proxied = false;
        }
        let mut path = component(parts.lpszUrlPath, parts.dwUrlPathLength);
        path.extend(component(parts.lpszExtraInfo, parts.dwExtraInfoLength));
        if path.is_empty() {
            path.push(b'/' as u16);
        }
        let host = terminated(host);
        let connection =
            unsafe { WinHttpConnect(handle(session), host.as_ptr(), parts.nPort, 0) } as Handle;
        let connection_failure = (connection == 0).then(network_error);
        self.state.lock().unwrap().connection = connection;
        if let Some(failure) = connection_failure {
            self.fail_before_sending(failure);
            return;
        }
        let method = terminated(wide(&hop.method));
        let path = terminated(path);
        let request = unsafe {
            WinHttpOpenRequest(
                handle(connection),
                method.as_ptr(),
                path.as_ptr(),
                null(),
                null(),
                null(),
                if parts.nScheme == WINHTTP_INTERNET_SCHEME_HTTPS {
                    WINHTTP_FLAG_SECURE
                } else {
                    0
                },
            )
        } as Handle;
        if request == 0 {
            self.fail_before_sending(network_error());
            return;
        }
        let context = Arc::into_raw(self.clone()) as usize;
        if unsafe {
            WinHttpSetOption(
                handle(request),
                WINHTTP_OPTION_CONTEXT_VALUE,
                (&context as *const usize).cast(),
                size_of::<usize>() as u32,
            )
        } == 0
        {
            let failure = network_error();
            unsafe {
                drop(Arc::from_raw(context as *const Self));
            }
            close(request);
            self.fail_before_sending(failure);
            return;
        }
        {
            let mut state = self.state.lock().unwrap();
            state.request = request;
            if state.closing {
                return;
            }
        }
        for (option, value) in [
            (
                WINHTTP_OPTION_DISABLE_FEATURE,
                WINHTTP_DISABLE_COOKIES | WINHTTP_DISABLE_AUTHENTICATION,
            ),
            (
                WINHTTP_OPTION_AUTOLOGON_POLICY,
                WINHTTP_AUTOLOGON_SECURITY_LEVEL_HIGH,
            ),
            (
                WINHTTP_OPTION_REDIRECT_POLICY,
                WINHTTP_OPTION_REDIRECT_POLICY_NEVER,
            ),
        ] {
            if !set_option(request, option, value) {
                self.stop(network_error());
                return;
            }
        }
        let timeout = hop.timeout_ms as i32;
        unsafe {
            WinHttpSetTimeouts(handle(request), timeout, timeout, timeout, timeout);
        }
        for header in &hop.headers {
            let mut line = wide(&header.name);
            line.extend([b':' as u16, b' ' as u16]);
            line.extend(wide(&header.value));
            let line = terminated(line);
            if unsafe {
                WinHttpAddRequestHeaders(
                    handle(request),
                    line.as_ptr(),
                    u32::MAX,
                    WINHTTP_ADDREQ_FLAG_ADD | WINHTTP_ADDREQ_FLAG_COALESCE_WITH_COMMA,
                )
            } == 0
            {
                self.stop(error(
                    NetErrorCode::Network,
                    format!(
                        "a header could not be set: {}",
                        error_message(unsafe { GetLastError() })
                    ),
                ));
                return;
            }
        }
        let body_size = hop.body.len() as u32;
        // The hop Arc keeps the send buffer alive through reentrant completions;
        // state.hop also retains it after this call until the response arrives.
        if unsafe {
            WinHttpSendRequest(
                handle(request),
                null(),
                0,
                if body_size == 0 {
                    null()
                } else {
                    hop.body.as_ptr().cast()
                },
                body_size,
                body_size,
                context,
            )
        } == 0
        {
            self.stop(network_error());
        }
    }
    fn fail_before_sending(&self, error: NetError) {
        let connection = { take(&mut self.state.lock().unwrap().connection) };
        close(connection);
        self.end_without_handle(Some(error));
    }
    fn end_without_handle(&self, failure: Option<NetError>) {
        let recorded = {
            let mut state = self.state.lock().unwrap();
            if !state.closing {
                state.closing = true;
                if let Some(error) = failure {
                    state.failed = true;
                    state.error = error;
                }
            }
            state.failed.then(|| state.error.clone())
        };
        self.forget();
        net::fail(
            self.id,
            recorded.unwrap_or_else(|| error(NetErrorCode::Aborted, "the request was cancelled")),
        );
    }
    fn forget(&self) {
        let session = {
            let mut registry = self.registry.state.lock().unwrap();
            registry.requests.remove(&self.id);
            registry.take_session_if_done()
        };
        close(session);
    }
    fn on_resolving(&self, name: &[u16]) {
        let mut state = self.state.lock().unwrap();
        state.proxied = !same_host(name, &state.hop_host);
    }
    fn on_connecting(&self, address: &[u16]) {
        let (proxied, hops) = {
            let state = self.state.lock().unwrap();
            (state.proxied, state.hops)
        };
        if proxied || self.allow_local_addresses {
            return;
        }
        let mut reason = String::new();
        if let Some(bytes) = parse_address(address) {
            if NetPolicy::check_address(&bytes, &mut reason) {
                return;
            }
        } else {
            reason = "the host resolves to an address that could not be checked".into();
        }
        self.stop(error(
            NetErrorCode::Policy,
            if hops == 0 {
                reason
            } else {
                format!("a redirect was refused: {reason}")
            },
        ));
    }
    fn on_headers(&self, request: Handle) {
        let Some(status) = query_number(request, WINHTTP_QUERY_STATUS_CODE).filter(|&s| s <= 999)
        else {
            self.stop(error(NetErrorCode::Network, "no HTTP response arrived"));
            return;
        };
        let mut response = HttpResponse {
            status: status as u16,
            ..Default::default()
        };
        if let Some(text) = query_header(request, WINHTTP_QUERY_STATUS_TEXT) {
            response.status_text = narrow(&text, CP_ACP);
        }
        if response.status_text.is_empty() {
            response.status_text = standard_reason_phrase(response.status).as_bytes().to_vec();
        }
        if let Some(raw) = query_header(request, WINHTTP_QUERY_RAW_HEADERS_CRLF) {
            response.headers = parse_headers(&raw);
        }
        let (hop, hops) = {
            let state = self.state.lock().unwrap();
            (state.hop.clone(), state.hops)
        };
        response.url = hop.url.clone();
        if matches!(response.status, 301 | 302 | 303 | 307 | 308) {
            if let Some(location) = find_header(&response.headers, b"location") {
                let mut next = (*hop).clone();
                let mut failure = NetError::default();
                if !NetPolicy::redirect(
                    &mut next,
                    response.status,
                    location,
                    hops,
                    &net::limits(),
                    &mut failure,
                ) {
                    self.stop(error(
                        failure.code,
                        format!("a redirect was refused: {}", failure.message),
                    ));
                    return;
                }
                let mut state = self.state.lock().unwrap();
                state.hop = Arc::new(next);
                state.hops += 1;
                if !state.closing {
                    state.hopping = true;
                }
                return;
            }
        }
        if expects_body(hop.method == b"HEAD", response.status) {
            let expected =
                find_header(&response.headers, b"content-length").map_or(0, content_length);
            if expected > self.max_body_bytes as u64 {
                self.too_large();
                return;
            }
            self.state.lock().unwrap().body.reserve(expected as usize);
        }
        {
            let mut state = self.state.lock().unwrap();
            state.response = response;
            state.chunk.resize(READ_CHUNK_BYTES, 0);
        }
        self.read(request);
    }
    fn too_large(&self) {
        self.stop(error(
            NetErrorCode::TooLarge,
            format!(
                "the response body is larger than {} bytes",
                self.max_body_bytes
            ),
        ));
    }
    fn read(&self, request: Handle) {
        let pointer = self.state.lock().unwrap().chunk.as_mut_ptr();
        // The allocation stays fixed until HANDLE_CLOSING, and WinHTTP invokes
        // READ_COMPLETE before we access its bytes or submit another read.
        if unsafe {
            WinHttpReadData(
                handle(request),
                pointer.cast(),
                READ_CHUNK_BYTES as u32,
                null_mut(),
            )
        } == 0
        {
            self.stop(network_error());
        }
    }
    fn on_read(&self, request: Handle, length: u32) {
        if length == 0 {
            self.settle(true, None);
            return;
        }
        {
            let mut state = self.state.lock().unwrap();
            if state.body.len() + length as usize > self.max_body_bytes as usize {
                drop(state);
                self.too_large();
                return;
            }
            let RequestState { body, chunk, .. } = &mut *state;
            body.extend_from_slice(&chunk[..length as usize]);
        }
        self.read(request);
    }
    fn on_error(&self, code: u32) {
        self.stop(match code {
            ERROR_WINHTTP_OPERATION_CANCELLED => {
                error(NetErrorCode::Aborted, "the request was cancelled")
            }
            ERROR_WINHTTP_TIMEOUT => error(NetErrorCode::Timeout, "no response before the timeout"),
            _ => error(NetErrorCode::Network, error_message(code)),
        });
    }
    fn on_closing(self: &Arc<Self>) {
        let connection = { take(&mut self.state.lock().unwrap().connection) };
        close(connection);
        let hop = {
            let mut state = self.state.lock().unwrap();
            state.request = 0;
            state.close_sent = false;
            let hop = state.hopping && !state.closing;
            state.hopping = false;
            hop
        };
        if hop {
            self.send_hop();
            return;
        }
        self.forget();
        let (failed, completed, failure, mut response, body) = {
            let mut state = self.state.lock().unwrap();
            (
                state.failed,
                state.completed,
                take(&mut state.error),
                take(&mut state.response),
                take(&mut state.body),
            )
        };
        if failed {
            net::fail(self.id, failure);
        } else if completed {
            response.body = body;
            net::complete(self.id, response);
        } else {
            net::fail(
                self.id,
                error(NetErrorCode::Aborted, "the request was cancelled"),
            );
        }
    }
}
fn set_option(request: Handle, option: u32, value: u32) -> bool {
    unsafe {
        WinHttpSetOption(
            handle(request),
            option,
            (&value as *const u32).cast(),
            size_of::<u32>() as u32,
        ) != 0
    }
}
pub struct WindowsProvider {
    registry: Arc<Registry>,
    watchdog: Option<JoinHandle<()>>,
}
impl WindowsProvider {
    pub fn new() -> Self {
        let agent = terminated(wide(b"Rive"));
        let mut session = unsafe {
            WinHttpOpen(
                agent.as_ptr(),
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                null(),
                null(),
                WINHTTP_FLAG_ASYNC,
            )
        } as Handle;
        if session == 0 {
            session = unsafe {
                WinHttpOpen(
                    agent.as_ptr(),
                    WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                    null(),
                    null(),
                    WINHTTP_FLAG_ASYNC,
                )
            } as Handle;
        }
        if session != 0 {
            set_option(
                session,
                WINHTTP_OPTION_DECOMPRESSION,
                WINHTTP_DECOMPRESSION_FLAG_ALL,
            );
            set_option(
                session,
                WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL,
                WINHTTP_PROTOCOL_FLAG_HTTP2,
            );
            let previous = unsafe {
                WinHttpSetStatusCallback(
                    handle(session),
                    Some(Request::on_status),
                    WINHTTP_CALLBACK_FLAG_ALL_COMPLETIONS
                        | WINHTTP_CALLBACK_FLAG_HANDLES
                        | WINHTTP_CALLBACK_STATUS_RESOLVING_NAME
                        | WINHTTP_CALLBACK_STATUS_CONNECTING_TO_SERVER,
                    0,
                )
            };
            // WINHTTP_INVALID_STATUS_CALLBACK is the function-pointer value -1.
            if previous.is_some_and(|callback| callback as usize == usize::MAX) {
                close(session);
                session = 0;
            }
        }
        let registry = Arc::new(Registry::default());
        registry.state.lock().unwrap().session = session;
        let watched = registry.clone();
        let watchdog = thread::spawn(move || Self::watch(watched));
        Self {
            registry,
            watchdog: Some(watchdog),
        }
    }
    fn watch(registry: Arc<Registry>) {
        let mut state = registry.state.lock().unwrap();
        while !state.stopping {
            let now = Instant::now();
            let mut next: Option<Instant> = None;
            let mut expired = Vec::new();
            for request in state.requests.values() {
                if request.closing() {
                    continue;
                }
                if request.deadline <= now {
                    expired.push(request.clone());
                } else {
                    next = Some(next.map_or(request.deadline, |n| n.min(request.deadline)));
                }
            }
            if !expired.is_empty() {
                drop(state);
                for request in expired {
                    request.stop(error(
                        NetErrorCode::Timeout,
                        "no response before the timeout",
                    ));
                }
                state = registry.state.lock().unwrap();
                continue;
            }
            state = if let Some(next) = next {
                registry
                    .wake
                    .wait_timeout(state, next.saturating_duration_since(Instant::now()))
                    .unwrap()
                    .0
            } else {
                registry.wake.wait(state).unwrap()
            };
        }
    }
}
impl Default for WindowsProvider {
    fn default() -> Self {
        Self::new()
    }
}
impl Provider for WindowsProvider {
    fn start(&self, id: RequestId, request: &HttpRequest) {
        let session = self.registry.state.lock().unwrap().session;
        if session == 0 {
            net::fail(id, error(NetErrorCode::Network, "WinHTTP is unavailable"));
            return;
        }
        let entry = Arc::new(Request::new(
            self.registry.clone(),
            id,
            request,
            &net::limits(),
        ));
        self.registry
            .state
            .lock()
            .unwrap()
            .requests
            .insert(id, entry.clone());
        self.registry.wake.notify_all();
        entry.start(session);
    }
    fn abort(&self, id: RequestId) {
        let entry = self
            .registry
            .state
            .lock()
            .unwrap()
            .requests
            .get(&id)
            .cloned();
        if let Some(entry) = entry {
            entry.abort();
        }
    }
}
impl Drop for WindowsProvider {
    fn drop(&mut self) {
        let (requests, session) = {
            let mut state = self.registry.state.lock().unwrap();
            state.stopping = true;
            let requests: Vec<_> = state.requests.values().cloned().collect();
            (requests, state.take_session_if_done())
        };
        close(session);
        self.registry.wake.notify_all();
        if let Some(watchdog) = self.watchdog.take() {
            watchdog.join().expect("WinHTTP watchdog panicked");
        }
        for request in requests {
            request.abort();
        }
    }
}
