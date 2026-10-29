//! Upstream net_provider_platform_test.cpp, 116807e5: real native transport and local fixture server.
use super::{
    http::*,
    net::{self, FetchListener, Provider, RequestId},
    net_policy::NetLimits,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
static TEST_LOCK: Mutex<()> = Mutex::new(());
const OWNER: u64 = 0xA11E0001;

struct LoopbackServer {
    // Upstream retains the listening socket even when no responses are queued.
    _listener: TcpListener,
    port: u16,
    stopping: Arc<AtomicBool>,
    requests: Arc<Mutex<Vec<String>>>,
    thread: Option<JoinHandle<()>>,
}
impl LoopbackServer {
    fn new(responses: Vec<(Vec<u8>, u64)>) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();
        let stopping = Arc::new(AtomicBool::new(false));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = stopping.clone();
        let recorded = requests.clone();
        let worker_listener = listener.try_clone().unwrap();
        let worker = thread::spawn(move || {
            for (response, delay) in responses {
                let deadline = Instant::now() + Duration::from_secs(5);
                let mut connection = loop {
                    if stop.load(Ordering::Relaxed) || Instant::now() >= deadline {
                        return;
                    }
                    match worker_listener.accept() {
                        Ok((connection, _)) => break connection,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(50))
                        }
                        Err(error) => panic!("accept: {error}"),
                    }
                };
                connection
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                recorded.lock().unwrap().push(read_request(&mut connection));
                let mut waited = 0;
                while waited < delay && !stop.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(10));
                    waited += 10;
                }
                // Client timeouts may close the stream before this send.
                let _ = connection.write_all(&response);
            }
        });
        Self {
            _listener: listener,
            port,
            stopping,
            requests,
            thread: Some(worker),
        }
    }
    fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}
impl Drop for LoopbackServer {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Relaxed);
        self.thread.take().unwrap().join().unwrap();
    }
}
fn read_request(stream: &mut TcpStream) -> String {
    let mut request = Vec::new();
    let mut expected = None;
    loop {
        if expected.is_some_and(|size| request.len() >= size) {
            break;
        }
        let mut chunk = [0u8; 4096];
        let count = match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        request.extend_from_slice(&chunk[..count]);
        if expected.is_none() {
            if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                let lower = String::from_utf8_lossy(&request[..end]).to_ascii_lowercase();
                let length = lower
                    .find("content-length:")
                    .map(|at| {
                        lower[at + 15..]
                            .split_whitespace()
                            .next()
                            .unwrap()
                            .parse::<usize>()
                            .unwrap()
                    })
                    .unwrap_or(0);
                expected = Some(end + 4 + length);
            }
        }
    }
    String::from_utf8(request).unwrap()
}
struct LoopbackProvider {
    inner: Arc<dyn Provider>,
    port: u16,
}
impl Provider for LoopbackProvider {
    fn start(&self, id: RequestId, request: &HttpRequest) {
        let mut local = request.clone();
        let path = local.url[b"https://".len()..]
            .iter()
            .position(|&b| b == b'/')
            .map(|i| i + b"https://".len());
        local.url = format!("http://127.0.0.1:{}", self.port).into_bytes();
        local
            .url
            .extend_from_slice(path.map(|i| &request.url[i..]).unwrap_or(b"/"));
        self.inner.start(id, &local);
    }
    fn abort(&self, id: RequestId) {
        self.inner.abort(id);
    }
}
#[derive(Default)]
struct RecordingListener {
    outcome: Mutex<Option<Result<HttpResponse, NetError>>>,
}
impl FetchListener for RecordingListener {
    fn on_response(&self, response: HttpResponse) {
        *self.outcome.lock().unwrap() = Some(Ok(response));
    }
    fn on_error(&self, error: &NetError) {
        *self.outcome.lock().unwrap() = Some(Err(error.clone()));
    }
}
struct LoopbackNetwork {
    _guard: MutexGuard<'static, ()>,
}
impl LoopbackNetwork {
    fn new(port: u16) -> Self {
        let guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        net::set_provider(Some(Arc::new(LoopbackProvider {
            inner: net::make_platform_provider().unwrap(),
            port,
        })));
        net::set_limits(NetLimits {
            allow_local_addresses: true,
            ..Default::default()
        });
        Self { _guard: guard }
    }
}
impl Drop for LoopbackNetwork {
    fn drop(&mut self) {
        net::cancel_all_for_owner(OWNER);
        net::set_provider(None);
        net::set_limits(NetLimits::default());
    }
}
fn get(path: &str) -> HttpRequest {
    HttpRequest {
        url: format!("https://fixture.example.com{path}").into_bytes(),
        ..Default::default()
    }
}
fn fetch_and_wait(request: HttpRequest) -> Result<HttpResponse, NetError> {
    let listener = Arc::new(RecordingListener::default());
    net::fetch(OWNER, request, listener.clone());
    let deadline = Instant::now() + Duration::from_secs(30);
    while listener.outcome.lock().unwrap().is_none() && Instant::now() < deadline {
        net::poll(16);
        thread::sleep(Duration::from_millis(5));
    }
    let result = listener
        .outcome
        .lock()
        .unwrap()
        .take()
        .expect("request did not settle within upstream deadline");
    result
}

#[test]
fn platform_provider_round_trips_request_and_response() {
    let server=LoopbackServer::new(vec![("HTTP/1.1 201 Created\r\nContent-Type: text/plain\r\nX-Test: yes\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello".into(),0)]);
    let _network = LoopbackNetwork::new(server.port);
    let mut request = get("/items?page=2");
    request.method = "POST".into();
    request.headers = vec![
        HttpHeader {
            name: "X-Script".into(),
            value: "1".into(),
        },
        HttpHeader {
            name: "Content-Type".into(),
            value: "text/plain".into(),
        },
    ];
    request.body = b"ping".to_vec();
    let response = fetch_and_wait(request).unwrap();
    assert_eq!(response.status, 201);
    assert_eq!(response.status_text, b"Created");
    assert_eq!(response.body, b"hello");
    assert!(
        response
            .headers
            .iter()
            .any(|h| h.name.eq_ignore_ascii_case(b"x-test") && h.value == b"yes")
    );
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].starts_with("POST /items?page=2 HTTP/1.1\r\n"));
    assert!(requests[0].to_ascii_lowercase().contains("x-script: 1\r\n"));
    assert!(requests[0].ends_with("ping"));
}
#[test]
fn platform_provider_never_stores_or_sends_cookies() {
    let server=LoopbackServer::new(vec![("HTTP/1.1 200 OK\r\nSet-Cookie: session=secret; Path=/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(),0),
        ("HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(),0)]);
    let _network = LoopbackNetwork::new(server.port);
    fetch_and_wait(get("/login")).unwrap();
    fetch_and_wait(get("/me")).unwrap();
    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    assert!(!requests[1].to_ascii_lowercase().contains("cookie:"));
    assert!(!requests[1].contains("secret"));
}
#[test]
fn platform_provider_rechecks_redirect_policy() {
    let server=LoopbackServer::new(vec![("HTTP/1.1 302 Found\r\nLocation: https://api.rive.app/api/me\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(),0)]);
    let _network = LoopbackNetwork::new(server.port);
    let error = fetch_and_wait(get("/redirect")).unwrap_err();
    assert_eq!(error.code, NetErrorCode::Policy);
    assert_eq!(
        error.message,
        "a redirect was refused: Rive's own hosts are not reachable from scripts"
    );
}
#[test]
fn platform_provider_stops_body_over_limit() {
    let server = LoopbackServer::new(vec![(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\n{}",
            "x".repeat(100)
        )
        .into_bytes(),
        0,
    )]);
    let _network = LoopbackNetwork::new(server.port);
    net::set_limits(NetLimits {
        max_response_body_bytes: 8,
        ..net::limits()
    });
    assert_eq!(
        fetch_and_wait(get("/large")).unwrap_err().code,
        NetErrorCode::TooLarge
    );
}
#[test]
fn platform_provider_answers_head_for_resource_over_size_limit() {
    // The Content-Length describes the resource, which never follows.
    let server = LoopbackServer::new(vec![(
        "HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\n".into(),
        0,
    )]);
    let _network = LoopbackNetwork::new(server.port);
    net::set_limits(NetLimits {
        max_response_body_bytes: 8,
        ..net::limits()
    });
    let mut request = get("/large");
    request.method = "HEAD".into();
    let response = fetch_and_wait(request).unwrap();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());
    assert!(response.headers.iter().any(|header| {
        header.name.eq_ignore_ascii_case(b"content-length") && header.value == b"100"
    }));
}
#[test]
fn platform_provider_answers_304_for_resource_over_size_limit() {
    // A 304 may name the Content-Length of the representation it stands for.
    let server = LoopbackServer::new(vec![(
        "HTTP/1.1 304 Not Modified\r\nContent-Length: 100\r\nConnection: close\r\n\r\n".into(),
        0,
    )]);
    let _network = LoopbackNetwork::new(server.port);
    net::set_limits(NetLimits {
        max_response_body_bytes: 8,
        ..net::limits()
    });
    let mut request = get("/cached");
    request.headers = vec![HttpHeader {
        name: "If-None-Match".into(),
        value: "\"v1\"".into(),
    }];
    let response = fetch_and_wait(request).unwrap();
    assert_eq!(response.status, 304);
    assert!(response.body.is_empty());
}
#[test]
fn platform_provider_enforces_total_timeout() {
    let server = LoopbackServer::new(vec![(
        "HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(),
        2000,
    )]);
    let _network = LoopbackNetwork::new(server.port);
    let mut request = get("/slow");
    request.timeout_ms = 200;
    let started = Instant::now();
    let error = fetch_and_wait(request).unwrap_err();
    assert_eq!(error.code, NetErrorCode::Timeout);
    assert!(started.elapsed() < Duration::from_millis(1500));
}

#[test]
fn platform_provider_refuses_http_redirect() {
    let server = LoopbackServer::new(vec![(b"HTTP/1.1 302 Found\r\nLocation: http://example.com/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(), 0)]);
    let _network = LoopbackNetwork::new(server.port);
    let error = fetch_and_wait(get("/downgrade")).unwrap_err();
    assert_eq!(error.code, NetErrorCode::Policy);
    assert_eq!(
        error.message,
        "a redirect was refused: http: URLs are not allowed; use https:"
    );
}
#[test]
fn platform_provider_settles_timeout_shorter_than_setup() {
    let server = LoopbackServer::new(vec![]);
    let _network = LoopbackNetwork::new(server.port);
    let mut request = get("/never");
    request.timeout_ms = 1;
    assert_eq!(
        fetch_and_wait(request).unwrap_err().code,
        NetErrorCode::Timeout
    );
}
#[test]
fn platform_provider_delivers_3xx_without_location() {
    let server = LoopbackServer::new(vec![(b"HTTP/1.1 304 Not Modified\r\nETag: \"v2\"\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(), 0)]);
    let _network = LoopbackNetwork::new(server.port);
    assert_eq!(fetch_and_wait(get("/cached")).unwrap().status, 304);
}
#[cfg(windows)]
#[test]
fn windows_provider_preserves_header_bytes() {
    let server = LoopbackServer::new(vec![(b"HTTP/1.1 200 OK\r\nX-Bytes: caf\xc3\xa9 \x80\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(), 0)]);
    let _network = LoopbackNetwork::new(server.port);
    let response = fetch_and_wait(get("/bytes")).unwrap();
    let header = response
        .headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(b"x-bytes"))
        .unwrap();
    assert_eq!(header.value, b"caf\xc3\xa9 \x80");
}
#[cfg(windows)]
#[test]
fn windows_provider_encodes_raw_utf8_location_before_policy() {
    let server = LoopbackServer::new(vec![(b"HTTP/1.1 302 Found\r\nLocation: https://api.rive.app/caf\xc3\xa9 x\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(), 0)]);
    let _network = LoopbackNetwork::new(server.port);
    let error = fetch_and_wait(get("/redirect")).unwrap_err();
    assert_eq!(error.code, NetErrorCode::Policy);
    assert_eq!(
        error.message,
        "a redirect was refused: Rive's own hosts are not reachable from scripts"
    );
}
#[cfg(windows)]
#[test]
fn windows_provider_refuses_local_address_before_sending() {
    let server = LoopbackServer::new(vec![(
        b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(),
        0,
    )]);
    let _network = LoopbackNetwork::new(server.port);
    net::set_limits(NetLimits::default());
    let error = fetch_and_wait(get("/intranet")).unwrap_err();
    assert_eq!(error.code, NetErrorCode::Policy);
    assert_eq!(error.message, "the host resolves to a loopback address");
    assert!(server.requests().iter().all(String::is_empty));
}
