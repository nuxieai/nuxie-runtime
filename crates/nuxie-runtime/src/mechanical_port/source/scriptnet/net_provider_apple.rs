//! Mechanical translation of scriptnet/net_provider_apple.mm through objc2.
//! Ephemeral NSURLSession, no ambient cookies/credentials, policy checked redirects.
use super::{
    http::*,
    net::{self, Provider, RequestId},
    net_policy::NetPolicy,
};
use block2::{DynBlock, StackBlock};
use dispatch2::{DispatchQoS, DispatchQueue, DispatchTime, GlobalQueueIdentifier};
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::{Bool, ProtocolObject};
use objc2::{AnyThread, DefinedClass, define_class, msg_send};
use objc2_foundation::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::{CStr, c_void},
    ptr::NonNull,
    sync::Mutex,
    time::Duration,
};

fn reason_phrase(status: isize) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        206 => "Partial Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        410 => "Gone",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        422 => "Unprocessable Content",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "",
    }
}
fn to_bytes(value: Option<&NSString>) -> Vec<u8> {
    let Some(value) = value else {
        return Vec::new();
    };
    let utf8 = value.UTF8String();
    if utf8.is_null() {
        Vec::new()
    } else {
        // SAFETY: NSString keeps its NUL-terminated UTF8String alive during this copy.
        unsafe { CStr::from_ptr(utf8) }.to_bytes().to_vec()
    }
}
// NSString stringWithUTF8String sees the C-string prefix and returns nil for
// malformed UTF-8. Ordinarily policy already proved ASCII, but Provider is a
// public host interface and may also be called directly.
fn from_bytes(value: &[u8]) -> Option<Retained<NSString>> {
    let end = value.iter().position(|&c| c == 0).unwrap_or(value.len());
    std::str::from_utf8(&value[..end])
        .ok()
        .map(NSString::from_str)
}
struct TaskState {
    id: RequestId,
    task: Retained<NSURLSessionTask>,
    max_body_bytes: u32,
    body: Vec<u8>,
    failed: bool,
    error: NetError,
}
#[derive(Default)]
struct Tasks {
    tasks: HashMap<usize, TaskState>,
    request_tasks: HashMap<RequestId, usize>,
}
#[derive(Default)]
struct DelegateIvars {
    tasks: Mutex<Tasks>,
}

define_class!(
    // SAFETY: NSObject has no additional subclass invariants; all mutable state is locked.
    #[unsafe(super = NSObject)]
    #[name = "NuxieScriptNetDelegate"]
    #[ivars = DelegateIvars]
    struct ScriptNetDelegate;
    unsafe impl NSObjectProtocol for ScriptNetDelegate {}
    unsafe impl NSURLSessionDelegate for ScriptNetDelegate {
        #[unsafe(method(URLSession:didReceiveChallenge:completionHandler:))]
        unsafe fn session_challenge(
            &self,
            _session: &NSURLSession,
            challenge: &NSURLAuthenticationChallenge,
            completion: &DynBlock<
                dyn Fn(NSURLSessionAuthChallengeDisposition, *mut NSURLCredential),
            >,
        ) {
            answer_challenge(challenge, completion);
        }
    }
    unsafe impl NSURLSessionTaskDelegate for ScriptNetDelegate {
        #[unsafe(method(URLSession:task:didReceiveChallenge:completionHandler:))]
        unsafe fn task_challenge(
            &self,
            _session: &NSURLSession,
            _task: &NSURLSessionTask,
            challenge: &NSURLAuthenticationChallenge,
            completion: &DynBlock<
                dyn Fn(NSURLSessionAuthChallengeDisposition, *mut NSURLCredential),
            >,
        ) {
            answer_challenge(challenge, completion);
        }
        #[unsafe(method(URLSession:task:willPerformHTTPRedirection:newRequest:completionHandler:))]
        unsafe fn redirect(
            &self,
            _session: &NSURLSession,
            task: &NSURLSessionTask,
            _response: &NSHTTPURLResponse,
            request: &NSURLRequest,
            completion: &DynBlock<dyn Fn(*mut NSURLRequest)>,
        ) {
            let mut hop = HttpRequest {
                url: to_bytes(
                    request
                        .URL()
                        .and_then(|url| url.absoluteString())
                        .as_deref(),
                ),
                method: to_bytes(request.HTTPMethod().as_deref()),
                ..HttpRequest::default()
            };
            let mut error = NetError::default();
            if NetPolicy::check(&mut hop, &net::limits(), &mut error) {
                completion.call((request as *const NSURLRequest as *mut NSURLRequest,));
            } else {
                completion.call((std::ptr::null_mut(),));
                self.stop_task(
                    task.taskIdentifier(),
                    NetError {
                        code: NetErrorCode::Policy,
                        message: format!("a redirect was refused: {}", error.message),
                    },
                );
            }
        }
        #[unsafe(method(URLSession:task:didCompleteWithError:))]
        fn finished(
            &self,
            _session: &NSURLSession,
            task: &NSURLSessionTask,
            error: Option<&NSError>,
        ) {
            let state = {
                let mut tasks = self.ivars().tasks.lock().unwrap();
                let Some(state) = tasks.tasks.remove(&task.taskIdentifier()) else {
                    return;
                };
                tasks.request_tasks.remove(&state.id);
                state
            };
            if state.failed {
                net::fail(state.id, state.error);
                return;
            }
            if let Some(error) = error {
                let mut code = NetErrorCode::Network;
                // SAFETY: Foundation's exported domain constant is immutable.
                if error.domain().isEqualToString(unsafe { NSURLErrorDomain }) {
                    if error.code() == NSURLErrorTimedOut {
                        code = NetErrorCode::Timeout;
                    } else if error.code() == NSURLErrorCancelled {
                        code = NetErrorCode::Aborted;
                    }
                }
                let description = error.localizedDescription();
                let bytes = to_bytes(Some(&description));
                let message = if bytes.is_empty() {
                    "request failed".into()
                } else {
                    String::from_utf8(bytes).expect("NSString UTF8String")
                };
                net::fail(state.id, NetError { code, message });
                return;
            }
            let response = task
                .response()
                .and_then(|response| response.downcast::<NSHTTPURLResponse>().ok());
            let Some(response) = response else {
                net::fail(
                    state.id,
                    NetError {
                        code: NetErrorCode::Network,
                        message: "no HTTP response arrived".into(),
                    },
                );
                return;
            };
            let mut result = HttpResponse {
                status: response.statusCode() as u16,
                status_text: reason_phrase(response.statusCode()).into(),
                url: to_bytes(
                    response
                        .URL()
                        .and_then(|url| url.absoluteString())
                        .as_deref(),
                ),
                body: state.body,
                ..HttpResponse::default()
            };
            let fields = response.allHeaderFields();
            // SAFETY: The immutable response owns the dictionary throughout enumeration.
            let keys = unsafe { fields.keyEnumerator() };
            while let Some(key) = keys.nextObject() {
                let Some(value) = fields.objectForKey(&key) else {
                    continue;
                };
                if let (Ok(key), Ok(value)) =
                    (key.downcast::<NSString>(), value.downcast::<NSString>())
                {
                    result.headers.push(HttpHeader {
                        name: to_bytes(Some(&key)),
                        value: to_bytes(Some(&value)),
                    });
                }
            }
            net::complete(state.id, result);
        }
    }
    unsafe impl NSURLSessionDataDelegate for ScriptNetDelegate {
        #[unsafe(method(URLSession:dataTask:didReceiveResponse:completionHandler:))]
        unsafe fn response(
            &self,
            _session: &NSURLSession,
            task: &NSURLSessionDataTask,
            response: &NSURLResponse,
            completion: &DynBlock<dyn Fn(NSURLSessionResponseDisposition)>,
        ) {
            let expected = response.expectedContentLength();
            let mut max = 0;
            {
                let mut tasks = self.ivars().tasks.lock().unwrap();
                if let Some(state) = tasks.tasks.get_mut(&task.taskIdentifier()) {
                    max = state.max_body_bytes;
                    if expected > 0 && expected <= i64::from(max) {
                        state.body.reserve(expected as usize);
                    }
                }
            }
            // NSURLResponseUnknownLength is -1 in Foundation.
            if expected != -1 && expected > i64::from(max) {
                completion.call((NSURLSessionResponseDisposition::Cancel,));
                self.stop_task(task.taskIdentifier(), too_large(max));
            } else {
                completion.call((NSURLSessionResponseDisposition::Allow,));
            }
        }
        #[unsafe(method(URLSession:dataTask:didReceiveData:))]
        fn data(&self, _session: &NSURLSession, task: &NSURLSessionDataTask, data: &NSData) {
            let too_large_limit = {
                let mut tasks = self.ivars().tasks.lock().unwrap();
                let Some(state) = tasks.tasks.get_mut(&task.taskIdentifier()) else {
                    return;
                };
                if state.failed {
                    return;
                }
                if state.body.len() + data.length() > state.max_body_bytes as usize {
                    Some(state.max_body_bytes)
                } else {
                    let body = RefCell::new(&mut state.body);
                    let block = StackBlock::new(
                        |bytes: NonNull<c_void>, range: NSRange, _stop: NonNull<Bool>| {
                            // SAFETY: NSData supplies a live byte range for this synchronous invocation.
                            let bytes = unsafe {
                                std::slice::from_raw_parts(
                                    bytes.as_ptr().cast::<u8>(),
                                    range.length,
                                )
                            };
                            body.borrow_mut().extend_from_slice(bytes);
                        },
                    );
                    data.enumerateByteRangesUsingBlock(&block);
                    None
                }
            };
            if let Some(max) = too_large_limit {
                self.stop_task(task.taskIdentifier(), too_large(max));
            }
        }
    }
);

fn answer_challenge(
    challenge: &NSURLAuthenticationChallenge,
    completion: &DynBlock<dyn Fn(NSURLSessionAuthChallengeDisposition, *mut NSURLCredential)>,
) {
    // SAFETY: Foundation's authentication-method constant is immutable.
    let server_trust = challenge
        .protectionSpace()
        .authenticationMethod()
        .isEqualToString(unsafe { NSURLAuthenticationMethodServerTrust });
    completion.call((
        if server_trust {
            NSURLSessionAuthChallengeDisposition::PerformDefaultHandling
        } else {
            NSURLSessionAuthChallengeDisposition::RejectProtectionSpace
        },
        std::ptr::null_mut(),
    ));
}
fn too_large(max: u32) -> NetError {
    NetError {
        code: NetErrorCode::TooLarge,
        message: format!("the response body is larger than {max} bytes"),
    }
}
impl ScriptNetDelegate {
    fn new() -> Retained<Self> {
        let this = Self::alloc().set_ivars(DelegateIvars::default());
        // SAFETY: NSObject's initializer requires no additional arguments.
        unsafe { msg_send![super(this), init] }
    }
    fn track(&self, task: &NSURLSessionTask, id: RequestId, max_body_bytes: u32) {
        let mut tasks = self.ivars().tasks.lock().unwrap();
        tasks.request_tasks.insert(id, task.taskIdentifier());
        // SAFETY: The borrowed task is live and non-null; retaining extends its lifetime.
        let retained =
            unsafe { Retained::retain(task as *const NSURLSessionTask as *mut NSURLSessionTask) }
                .unwrap();
        tasks.tasks.insert(
            task.taskIdentifier(),
            TaskState {
                id,
                task: retained,
                max_body_bytes,
                body: Vec::new(),
                failed: false,
                error: NetError::default(),
            },
        );
    }
    fn cancel_request(&self, id: RequestId) {
        let task = {
            let tasks = self.ivars().tasks.lock().unwrap();
            tasks
                .request_tasks
                .get(&id)
                .and_then(|task_id| tasks.tasks.get(task_id))
                .map(|state| state.task.clone())
        };
        if let Some(task) = task {
            task.cancel();
        }
    }
    fn stop_task(&self, task_id: usize, error: NetError) {
        let task = {
            let mut tasks = self.ivars().tasks.lock().unwrap();
            let Some(state) = tasks.tasks.get_mut(&task_id) else {
                return;
            };
            if state.failed {
                return;
            }
            state.failed = true;
            state.error = error;
            state.task.clone()
        };
        task.cancel();
    }
}

pub struct AppleProvider {
    session: Retained<NSURLSession>,
    delegate: Retained<ScriptNetDelegate>,
}
impl Default for AppleProvider {
    fn default() -> Self {
        Self::new()
    }
}
impl AppleProvider {
    pub fn new() -> Self {
        autoreleasepool(|_| {
            let configuration = NSURLSessionConfiguration::ephemeralSessionConfiguration();
            configuration.setHTTPCookieAcceptPolicy(NSHTTPCookieAcceptPolicy::Never);
            configuration.setHTTPShouldSetCookies(false);
            configuration.setHTTPCookieStorage(None);
            configuration.setURLCredentialStorage(None);
            configuration.setURLCache(None);
            configuration
                .setRequestCachePolicy(NSURLRequestCachePolicy::ReloadIgnoringLocalCacheData);
            let delegate = ScriptNetDelegate::new();
            let queue = NSOperationQueue::new();
            queue.setMaxConcurrentOperationCount(1);
            queue.setName(Some(&NSString::from_str("app.rive.scriptnet")));
            // SAFETY: Delegate is thread-safe, retained both here and by the session until invalidation.
            let session = unsafe {
                NSURLSession::sessionWithConfiguration_delegate_delegateQueue(
                    &configuration,
                    Some(ProtocolObject::from_ref(&*delegate)),
                    Some(&queue),
                )
            };
            Self { session, delegate }
        })
    }
}
impl Drop for AppleProvider {
    fn drop(&mut self) {
        self.session.invalidateAndCancel();
    }
}
impl Provider for AppleProvider {
    fn start(&self, id: RequestId, request: &HttpRequest) {
        autoreleasepool(|_| {
            let Some(url) = from_bytes(&request.url).and_then(|text| NSURL::URLWithString(&text))
            else {
                net::fail(
                    id,
                    NetError {
                        code: NetErrorCode::Policy,
                        message: "the URL could not be parsed".into(),
                    },
                );
                return;
            };
            let native = NSMutableURLRequest::requestWithURL(&url);
            let method = from_bytes(&request.method);
            // SAFETY: This is the Foundation setter's ABI. Preserve upstream's
            // nil result from stringWithUTF8String for direct provider callers.
            unsafe {
                let _: () = msg_send![&*native, setHTTPMethod: method.as_deref()];
            }
            native.setHTTPShouldHandleCookies(false);
            native.setTimeoutInterval(f64::from(request.timeout_ms) / 1000.0);
            for header in &request.headers {
                if let (Some(name), Some(value)) =
                    (from_bytes(&header.name), from_bytes(&header.value))
                {
                    native.addValue_forHTTPHeaderField(&value, &name);
                }
            }
            if !request.body.is_empty() {
                native.setHTTPBody(Some(&NSData::with_bytes(&request.body)));
            }
            let task = self.session.dataTaskWithRequest(&native);
            self.delegate
                .track(&task, id, net::limits().max_response_body_bytes);
            let task_id = task.taskIdentifier();
            let delegate = self.delegate.clone();
            let when = DispatchTime::try_from(Duration::from_millis(u64::from(request.timeout_ms)))
                .expect("u32 milliseconds fit dispatch time");
            let _ = DispatchQueue::global_queue(GlobalQueueIdentifier::QualityOfService(
                DispatchQoS::Utility,
            ))
            .after(when, move || {
                delegate.stop_task(
                    task_id,
                    NetError {
                        code: NetErrorCode::Timeout,
                        message: "no response before the timeout".into(),
                    },
                );
            });
            task.resume();
        });
    }
    fn abort(&self, id: RequestId) {
        self.delegate.cancel_request(id);
    }
}
