//! Literal scenarios from upstream scripting_fetch_test.cpp at 6cd5d108.
use super::ScriptVm;
use luaur_rt::{Table, Value};
use nuxie_runtime::source::scriptnet::{
    http::{HttpHeader, HttpRequest, HttpResponse, NetError, NetErrorCode},
    net::{self, FetchListener, Provider, RequestId},
    net_policy::NetLimits,
};
use nuxie_runtime::{rive_has_pending_async_work, rive_poll_async_work};
use std::sync::{
    Arc, Condvar, Mutex, MutexGuard,
    atomic::{AtomicBool, Ordering},
};

pub(super) static NET_TEST_LOCK: Mutex<()> = Mutex::new(());
#[derive(Default)]
struct MockProvider {
    starts: Mutex<Vec<(RequestId, HttpRequest)>>,
    aborts: Mutex<Vec<RequestId>>,
}
impl Provider for MockProvider {
    fn start(&self, id: RequestId, request: &HttpRequest) {
        self.starts.lock().unwrap().push((id, request.clone()));
    }
    fn abort(&self, id: RequestId) {
        self.aborts.lock().unwrap().push(id);
    }
}
struct MockNetwork {
    provider: Arc<MockProvider>,
    _guard: MutexGuard<'static, ()>,
}
impl MockNetwork {
    fn new() -> Self {
        let guard = NET_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let provider = Arc::new(MockProvider::default());
        net::set_provider(Some(provider.clone()));
        net::set_limits(NetLimits::default());
        Self {
            provider,
            _guard: guard,
        }
    }
    fn id(&self, index: usize) -> RequestId {
        self.provider.starts.lock().unwrap()[index].0
    }
    fn count(&self) -> usize {
        self.provider.starts.lock().unwrap().len()
    }
}
impl Drop for MockNetwork {
    fn drop(&mut self) {
        net::set_provider(None);
        net::set_limits(NetLimits::default());
    }
}
fn vm() -> ScriptVm {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    vm
}
fn state(vm: &ScriptVm, source: &str) -> Table {
    vm.lua().load(source).eval().unwrap()
}
fn string(state: &Table, key: &str) -> String {
    match state.get::<Value>(key).unwrap() {
        Value::Nil => "<nil>".into(),
        Value::String(s) => s.to_str().unwrap().to_owned(),
        value => panic!("expected string, got {value:?}"),
    }
}
fn number(state: &Table, key: &str) -> f64 {
    state.get(key).unwrap()
}
fn boolean(state: &Table, key: &str) -> bool {
    state.get(key).unwrap()
}
fn text_response(status: u16, body: &str) -> HttpResponse {
    HttpResponse {
        status,
        status_text: if status == 200 { "OK" } else { "Not Found" }.into(),
        url: "https://example.com/".into(),
        body: body.as_bytes().to_vec(),
        ..Default::default()
    }
}
fn poll() {
    rive_poll_async_work(32);
}

#[test]
fn fetch_returns_a_promise_that_settles_only_when_polled() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"
        local state = { settled = false }
        local promise = fetch("https://example.com/data")
        promise:andThen(function(response) state.settled = true; state.status = response.status end)
        state.before = promise:getStatus()
        return state
    "#,
    );
    assert_eq!(string(&s, "before"), "Pending");
    assert_eq!(network.count(), 1);
    assert_eq!(
        network.provider.starts.lock().unwrap()[0].1.url,
        b"https://example.com/data"
    );
    assert!(rive_has_pending_async_work());
    net::complete(network.id(0), text_response(200, "hi"));
    assert!(!boolean(&s, "settled"));
    poll();
    assert!(boolean(&s, "settled"));
    assert_eq!(number(&s, "status"), 200.0);
    assert!(!rive_has_pending_async_work());
}

#[test]
fn fetch_hands_the_provider_a_normalized_request() {
    let network = MockNetwork::new();
    let vm = vm();
    vm.lua().load(r##"fetch("https://Example.com/submit#fragment", {
        method = "post", headers = { ["Content-Type"] = "application/json", Authorization = "Bearer token" },
        body = '{"a":1}', timeout = 5000 })"##).exec().unwrap();
    assert_eq!(network.count(), 1);
    let request = network.provider.starts.lock().unwrap()[0].1.clone();
    assert_eq!(request.url, b"https://example.com/submit");
    assert_eq!(request.method, b"POST");
    assert_eq!(request.body, br#"{"a":1}"#);
    assert_eq!(request.timeout_ms, 5000);
    assert_eq!(request.headers.len(), 2);
    assert!(
        request
            .headers
            .iter()
            .any(|h| h.name == b"Content-Type" && h.value == b"application/json")
    );
    assert!(
        request
            .headers
            .iter()
            .any(|h| h.name == b"Authorization" && h.value == b"Bearer token")
    );
}

#[test]
fn fetch_sends_a_buffer_body_and_defaults_the_timeout() {
    let network = MockNetwork::new();
    let vm = vm();
    vm.lua()
        .load(
            r#"local body=buffer.create(3)
        buffer.writeu8(body,0,1); buffer.writeu8(body,1,2); buffer.writeu8(body,2,255)
        fetch("https://example.com/upload", {method="PUT", body=body})"#,
        )
        .exec()
        .unwrap();
    assert_eq!(network.count(), 1);
    let request = network.provider.starts.lock().unwrap()[0].1.clone();
    assert_eq!(request.method, b"PUT");
    assert_eq!(request.body, [1, 2, 255]);
    assert_eq!(request.timeout_ms, NetLimits::default().default_timeout_ms);
}

#[test]
fn fetch_resolves_with_a_response() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={}
        async(function()
            local _, response=await(fetch("https://example.com/"))
            state.status=response.status; state.statusText=response.statusText
            state.ok=response.ok; state.url=response.url
            local _, text=await(response:text()); state.text=text
            local _, bytes=await(response:arrayBuffer()); state.bytes=buffer.len(bytes)
            state.firstByte=buffer.readu8(bytes,0)
            state.contentType=response:header("CONTENT-TYPE")
            state.missing=response:header("x-missing")==nil
            state.joined=response:header("x-multi"); state.table=response.headers["x-multi"]
            state.lowered=response.headers["content-type"]
            return nil
        end)
        return state"#,
    );
    assert_eq!(network.count(), 1);
    let mut response = text_response(200, "hello");
    response.headers = [
        ("Content-Type", "text/plain"),
        ("X-Multi", "a"),
        ("x-multi", "b"),
    ]
    .into_iter()
    .map(|(n, v)| HttpHeader {
        name: n.into(),
        value: v.into(),
    })
    .collect();
    net::complete(network.id(0), response);
    poll();
    assert_eq!(number(&s, "status"), 200.0);
    assert_eq!(string(&s, "statusText"), "OK");
    assert!(boolean(&s, "ok"));
    assert_eq!(string(&s, "url"), "https://example.com/");
    assert_eq!(string(&s, "text"), "hello");
    assert_eq!(number(&s, "bytes"), 5.0);
    assert_eq!(number(&s, "firstByte"), f64::from(b'h'));
    assert_eq!(string(&s, "contentType"), "text/plain");
    assert!(boolean(&s, "missing"));
    assert_eq!(string(&s, "joined"), "a, b");
    assert_eq!(string(&s, "table"), "a, b");
    assert_eq!(string(&s, "lowered"), "text/plain");
}

#[test]
fn non_2xx_response_still_resolves() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={}
        fetch("https://example.com/missing"):andThen(function(response)
            state.status=response.status; state.ok=response.ok
        end,function(err) state.error=err end)
        return state"#,
    );
    assert_eq!(network.count(), 1);
    net::complete(network.id(0), text_response(404, "nope"));
    poll();
    assert_eq!(number(&s, "status"), 404.0);
    assert!(!boolean(&s, "ok"));
    assert_eq!(string(&s, "error"), "<nil>");
}

#[test]
fn no_transport_rejects_only_when_polled() {
    let _network = MockNetwork::new();
    net::set_provider(None);
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={}
        fetch("https://example.com/"):catch(function(err) state.error=err end)
        return state"#,
    );
    assert_eq!(string(&s, "error"), "<nil>");
    poll();
    assert_eq!(
        string(&s, "error"),
        "disabled: network access is not enabled for scripts in this host"
    );
}

#[test]
fn policy_refusals_do_not_reach_provider() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={}
        fetch("http://example.com/"):catch(function(err) state.scheme=err end)
        fetch("https://api.rive.app/api/me"):catch(function(err) state.rive=err end)
        fetch("https://example.com/",{headers={Cookie="a=b"}}):catch(function(err) state.cookie=err end)
        return state"#,
    );
    poll();
    assert_eq!(network.count(), 0);
    assert_eq!(
        string(&s, "scheme"),
        "policy: http: URLs are not allowed; use https:"
    );
    assert_eq!(
        string(&s, "rive"),
        "policy: Rive's own hosts are not reachable from scripts"
    );
    assert_eq!(
        string(&s, "cookie"),
        "policy: scripts cannot set the 'Cookie' header"
    );
}

#[test]
fn in_flight_limit_rejects_ninth_request() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={rejected=0}
        for i=1,9 do fetch("https://example.com/"..i):catch(function(err)
            state.rejected+=1; state.error=err end) end
        return state"#,
    );
    assert_eq!(network.count(), 8);
    poll();
    assert_eq!(number(&s, "rejected"), 1.0);
    assert_eq!(
        string(&s, "error"),
        "rateLimited: more than 8 requests in flight"
    );
}

#[test]
fn per_minute_limit_rejects_third_request() {
    let network = MockNetwork::new();
    net::set_limits(NetLimits {
        max_requests_per_minute_per_owner: 2,
        ..Default::default()
    });
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={}
        for i=1,3 do fetch("https://example.com/"..i):catch(function(err) state.error=err end) end
        return state"#,
    );
    assert_eq!(network.count(), 2);
    poll();
    assert_eq!(
        string(&s, "error"),
        "rateLimited: more than 2 requests in the last minute"
    );
}

#[test]
fn oversized_response_is_rejected() {
    let network = MockNetwork::new();
    net::set_limits(NetLimits {
        max_response_body_bytes: 4,
        ..Default::default()
    });
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={}
        fetch("https://example.com/"):catch(function(err) state.error=err end)
        return state"#,
    );
    assert_eq!(network.count(), 1);
    net::complete(network.id(0), text_response(200, "hello"));
    poll();
    assert_eq!(
        string(&s, "error"),
        "tooLarge: the response body is larger than 4 bytes"
    );
}

#[test]
fn transport_failure_keeps_code_and_ignores_second_outcome() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={}
        fetch("https://example.com/"):catch(function(err) state.error=err end)
        return state"#,
    );
    assert_eq!(network.count(), 1);
    net::fail(
        network.id(0),
        NetError {
            code: NetErrorCode::Network,
            message: "connection reset".into(),
        },
    );
    net::complete(network.id(0), text_response(200, "late"));
    poll();
    assert_eq!(string(&s, "error"), "network: connection reset");
}

#[test]
fn cancel_aborts_and_never_settles() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={settled=false}
        local promise=fetch("https://example.com/")
        promise:andThen(function() state.settled=true end,function() state.settled=true end)
        promise:cancel(); state.status=promise:getStatus()
        return state"#,
    );
    assert_eq!(network.count(), 1);
    let id = network.id(0);
    assert_eq!(*network.provider.aborts.lock().unwrap(), [id]);
    assert_eq!(string(&s, "status"), "Cancelled");
    assert!(!net::has_pending_work());
    net::complete(id, text_response(200, "late"));
    poll();
    assert!(!boolean(&s, "settled"));
}

#[test]
fn handler_can_cancel_another_outcome_in_same_poll() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={secondSettled=false}; local second
        fetch("https://example.com/a"):andThen(function() second:cancel() end)
        second=fetch("https://example.com/b")
        second:andThen(function() state.secondSettled=true end,function() state.secondSettled=true end)
        return state"#,
    );
    assert_eq!(network.count(), 2);
    net::complete(network.id(0), text_response(200, "a"));
    net::complete(network.id(1), text_response(200, "b"));
    poll();
    assert!(!boolean(&s, "secondSettled"));
    assert!(!net::has_pending_work());
    let first = vm.lua().create_registry_value(1).unwrap();
    let next = vm.lua().create_registry_value(2).unwrap();
    assert_eq!(vm.lua().registry_value::<i32>(&first).unwrap(), 1);
    assert_eq!(vm.lua().registry_value::<i32>(&next).unwrap(), 2);
    vm.lua().remove_registry_value(first).unwrap();
    vm.lua().remove_registry_value(next).unwrap();
}

#[test]
fn closing_vm_aborts_outstanding_requests() {
    let network = MockNetwork::new();
    let id;
    {
        let vm = vm();
        vm.lua()
            .load(r#"fetch("https://example.com/")"#)
            .exec()
            .unwrap();
        assert_eq!(network.count(), 1);
        id = network.id(0);
        assert!(net::has_pending_work());
    }
    assert_eq!(*network.provider.aborts.lock().unwrap(), [id]);
    assert!(!net::has_pending_work());
    net::complete(id, text_response(200, "late"));
    assert_eq!(net::poll(16), 0);
}

#[test]
fn watchdog_times_out_provider_that_never_answers() {
    let network = MockNetwork::new();
    net::set_limits(NetLimits {
        watchdog_grace_ms: 0,
        ..Default::default()
    });
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={}
        fetch("https://example.com/",{timeout=1}):catch(function(err) state.error=err end)
        return state"#,
    );
    assert_eq!(network.count(), 1);
    std::thread::sleep(std::time::Duration::from_millis(10));
    poll();
    assert_eq!(
        string(&s, "error"),
        "timeout: no response before the timeout"
    );
    assert_eq!(*network.provider.aborts.lock().unwrap(), [network.id(0)]);
}

#[test]
fn await_fetch_delivers_newly_queued_refusal_next_poll() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={done=false}
        async(function()
            local ok,response=await(fetch("https://example.com/"))
            state.firstOk=ok; state.status=response.status
            local refusedOk,err=await(fetch("http://example.com/"))
            state.secondOk=refusedOk; state.error=err; state.done=true
        end)
        return state"#,
    );
    assert_eq!(network.count(), 1);
    net::complete(network.id(0), text_response(200, "hi"));
    poll();
    assert!(boolean(&s, "firstOk"));
    assert_eq!(number(&s, "status"), 200.0);
    assert!(!boolean(&s, "done"));
    poll();
    assert!(boolean(&s, "done"));
    assert!(!boolean(&s, "secondOk"));
    assert_eq!(
        string(&s, "error"),
        "policy: http: URLs are not allowed; use https:"
    );
}

#[test]
fn fetch_delivers_only_on_originating_thread() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={settled=false}
        fetch("https://example.com/"):andThen(function() state.settled=true end)
        return state"#,
    );
    assert_eq!(network.count(), 1);
    net::complete(network.id(0), text_response(200, "hi"));
    let (delivered, pending) = std::thread::spawn(|| (net::poll(16), net::has_pending_work()))
        .join()
        .unwrap();
    assert_eq!(delivered, 0);
    assert!(!pending);
    assert!(!boolean(&s, "settled"));
    poll();
    assert!(boolean(&s, "settled"));
}

#[test]
fn malformed_arguments_raise_without_starting_request() {
    let network = MockNetwork::new();
    for (source, message) in [
        (
            r#"fetch("https://example.com/",{headers={A=1}})"#,
            "fetch: options.headers must map header names to string values",
        ),
        (
            r#"fetch("https://example.com/",{method=1})"#,
            "fetch: options.method must be a string",
        ),
        (
            r#"fetch("https://example.com/",{body={}})"#,
            "fetch: options.body must be a string or a buffer",
        ),
        (
            r#"fetch("https://example.com/",{timeout=-1})"#,
            "fetch: options.timeout must be a non-negative number",
        ),
        (r#"fetch("https://example.com/","GET")"#, "table expected"),
        ("fetch()", "string expected"),
    ] {
        let vm = vm();
        let error = vm.lua().load(source).exec().unwrap_err().to_string();
        assert!(error.contains(message), "{error}");
    }
    assert_eq!(network.count(), 0);
}

#[derive(Default)]
struct RecordingListener {
    cancelled: AtomicBool,
}

// Supplemental source-review regressions for lua_scriptnet.cpp's two reads
// and its strict userdata/C-string diagnostic boundaries.
#[test]
fn fetch_rereads_options_after_validation() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"
        local state = { order = "" }
        local counts = {}
        local options = setmetatable({}, { __index = function(_, key)
            state.order ..= key .. ","
            counts[key] = (counts[key] or 0) + 1
            local second = counts[key] == 2
            if key == "method" then return if second then "POST" else "GET" end
            if key == "headers" then return { Test = if second then "second" else "first" } end
            if key == "body" then return if second then "second" else "first" end
            if key == "timeout" then return if second then 2000 else 1000 end
        end })
        state.promise = fetch("https://example.com/", options)
        return state
    "#,
    );
    assert_eq!(
        string(&s, "order"),
        "method,headers,body,timeout,method,headers,body,timeout,"
    );
    let request = network.provider.starts.lock().unwrap()[0].1.clone();
    assert_eq!(request.method, b"POST");
    assert_eq!(request.body, b"second");
    assert_eq!(request.timeout_ms, 2000);
    assert_eq!(
        request.headers,
        [HttpHeader {
            name: b"Test".to_vec(),
            value: b"second".to_vec()
        }]
    );
}

#[test]
fn transport_error_uses_c_string_message_prefix() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"local s={}; fetch("https://example.com/"):catch(function(e) s.error=e end); return s"#,
    );
    net::fail(
        network.id(0),
        NetError {
            code: NetErrorCode::Network,
            message: "first\0hidden".into(),
        },
    );
    poll();
    assert_eq!(string(&s, "error"), "network: first");
}

#[test]
fn response_indices_are_strict_and_methods_are_namecall_only() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"
        local s={}
        fetch("https://example.com/"):andThen(function(response)
            s.unknown, s.error = pcall(function() return response["bad\0hidden"] end)
            s.methodField = pcall(function() return response.text end)
            s.unknownMethod = pcall(function() return response:missing() end)
            response:text():andThen(function(text) s.text = text end)
        end)
        return s
    "#,
    );
    net::complete(network.id(0), text_response(200, "body"));
    poll();
    assert!(!boolean(&s, "unknown"));
    assert!(!boolean(&s, "methodField"));
    assert!(!boolean(&s, "unknownMethod"));
    let error = string(&s, "error");
    assert!(
        error.contains("'bad' is not a valid index of Response"),
        "{error}"
    );
    assert!(!error.contains("hidden"), "{error}");
    assert_eq!(string(&s, "text"), "body");
}

// Supplemental coverage for the byte-oriented request fields in NetPolicy.
#[test]
fn invalid_utf8_text_is_an_async_policy_rejection_not_a_lua_type_error() {
    let network = MockNetwork::new();
    let vm = vm();
    let s = state(
        &vm,
        r#"local state={}
        local bad=string.char(255)
        fetch("https://example.com/"..bad):catch(function(e) state.url=e end)
        fetch("https://example.com/",{method=bad}):catch(function(e) state.method=e end)
        fetch("https://example.com/",{headers={[bad]="ok"}}):catch(function(e) state.name=e end)
        fetch("https://example.com/",{headers={Test=bad}}):catch(function(e) state.value=e end)
        return state"#,
    );
    for key in ["url", "method", "name", "value"] {
        assert_eq!(string(&s, key), "<nil>");
    }
    poll();
    assert_eq!(network.count(), 0);
    assert_eq!(
        string(&s, "url"),
        "policy: the URL must be ASCII with no spaces or backslashes; percent-encode anything else"
    );
    assert_eq!(
        string(&s, "method"),
        "policy: the method is not a valid HTTP token"
    );
    assert_eq!(
        string(&s, "name"),
        "policy: a header name is not a valid HTTP token"
    );
    assert_eq!(
        string(&s, "value"),
        "policy: the value of the 'Test' header must be printable ASCII"
    );
}

impl FetchListener for RecordingListener {
    fn on_response(&self, _: HttpResponse) {}
    fn on_error(&self, _: &NetError) {}
    fn on_cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}
fn get_request() -> HttpRequest {
    HttpRequest {
        url: "https://example.com/".into(),
        ..Default::default()
    }
}

#[test]
fn request_aborts_through_original_transport() {
    let network = MockNetwork::new();
    let listener = Arc::new(RecordingListener::default());
    let id = net::fetch(1, get_request(), listener.clone());
    assert_eq!(network.count(), 1);
    let replacement = Arc::new(MockProvider::default());
    net::set_provider(Some(replacement.clone()));
    net::cancel(id);
    assert_eq!(*network.provider.aborts.lock().unwrap(), [id]);
    assert!(replacement.aborts.lock().unwrap().is_empty());
    assert!(listener.cancelled.load(Ordering::SeqCst));
}

#[derive(Default)]
struct BlockingState {
    entered: RequestId,
    released: bool,
    events: Vec<&'static str>,
}
#[derive(Default)]
struct BlockingProvider {
    state: Mutex<BlockingState>,
    changed: Condvar,
}
impl Provider for BlockingProvider {
    fn start(&self, id: RequestId, _: &HttpRequest) {
        let mut state = self.state.lock().unwrap();
        state.entered = id;
        self.changed.notify_all();
        while !state.released {
            state = self.changed.wait(state).unwrap();
        }
        state.events.push("start");
    }
    fn abort(&self, _: RequestId) {
        self.state.lock().unwrap().events.push("abort");
    }
}
#[test]
fn cancellation_during_start_aborts_after_transport_takes_request() {
    let _network = MockNetwork::new();
    let provider = Arc::new(BlockingProvider::default());
    net::set_provider(Some(provider.clone()));
    let listener = Arc::new(RecordingListener::default());
    let other = listener.clone();
    let fetching = std::thread::spawn(move || net::fetch(1, get_request(), other));
    let id = {
        let mut state = provider.state.lock().unwrap();
        while state.entered == 0 {
            state = provider.changed.wait(state).unwrap();
        }
        state.entered
    };
    net::cancel(id);
    assert!(listener.cancelled.load(Ordering::SeqCst));
    {
        let mut state = provider.state.lock().unwrap();
        assert!(state.events.is_empty());
        state.released = true;
    }
    provider.changed.notify_all();
    fetching.join().unwrap();
    assert_eq!(provider.state.lock().unwrap().events, ["start", "abort"]);
    net::set_provider(None);
}
