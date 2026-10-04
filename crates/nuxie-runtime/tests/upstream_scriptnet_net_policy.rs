#![cfg(feature = "scriptnet")]
//! Literal cases from tests/unit_tests/runtime/scriptnet/net_policy_test.cpp.
use nuxie_runtime::mechanical_port::source::scriptnet::*;
fn refusal(request: &mut HttpRequest, limits: NetLimits) -> String {
    let mut error = NetError::default();
    if NetPolicy::check(request, &limits, &mut error) {
        return String::new();
    }
    assert_eq!(error.code, NetErrorCode::Policy);
    error.message
}
fn url_refusal(url: &str) -> String {
    refusal(
        &mut HttpRequest {
            url: url.into(),
            ..Default::default()
        },
        NetLimits::default(),
    )
}
fn normalized_url(url: &str) -> Vec<u8> {
    let mut request = HttpRequest {
        url: url.into(),
        ..Default::default()
    };
    assert!(refusal(&mut request, NetLimits::default()).is_empty());
    request.url
}
const IP_HOST: &str = "IP address hosts are not allowed";
const LOCAL_HOST: &str = "local network hosts are not allowed";
const RIVE_HOST: &str = "Rive's own hosts are not reachable from scripts";

#[test]
fn mixed_invalid_host_bytes_preserve_first_refusal() {
    // net_policy.cpp checks each byte in order, not whole-host categories.
    assert_eq!(
        url_refusal("https://%:x:443/"),
        "the host must be an ASCII domain name (punycode for international names)"
    );
}

#[test]
fn net_policy_lets_https_requests_to_public_hosts_through() {
    for url in [
        "https://example.com/",
        "https://api.example.com:8443/v1/items?limit=5",
        "https://xn--bcher-kva.example/",
        "https://example.com./",
        "https://my_service.example.com/",
        "https://notrive.app/",
        "https://rive.app.example.com/",
        "https://localhost.example.com/",
    ] {
        assert!(url_refusal(url).is_empty(), "{url}");
    }
}
#[test]
fn net_policy_normalizes_the_url_it_lets_through() {
    for (input, output) in [
        (
            "HTTPS://Example.COM/Path?Q=1",
            "https://example.com/Path?Q=1",
        ),
        (
            "https://example.com/page#section",
            "https://example.com/page",
        ),
        ("https://example.com.:443/", "https://example.com:443/"),
        ("https://example.com:/", "https://example.com/"),
        ("https://example.com?x", "https://example.com?x"),
    ] {
        assert_eq!(normalized_url(input), output.as_bytes());
    }
}
#[test]
fn net_policy_refuses_every_scheme_but_https() {
    for url in ["http://example.com/", "HTTP://example.com/"] {
        assert_eq!(url_refusal(url), "http: URLs are not allowed; use https:");
    }
    for url in [
        "ftp://example.com/",
        "wss://example.com/",
        "file:///etc/passwd",
        "example.com",
    ] {
        assert_eq!(url_refusal(url), "only https: URLs are allowed");
    }
    for url in ["https:example.com", "https:///path"] {
        assert_eq!(url_refusal(url), "the URL has no host");
    }
    assert_eq!(url_refusal(""), "the URL is empty");
}
#[test]
fn net_policy_refuses_urls_it_cannot_parse_unambiguously() {
    let ascii = "the URL must be ASCII with no spaces or backslashes; percent-encode anything else";
    for url in [
        "https://example.com/a b",
        "https://example.com\\@evil.example/",
        "https://exämple.com/",
        "https://example.com/\n",
    ] {
        assert_eq!(url_refusal(url), ascii);
    }
    for url in [
        "https://user:secret@example.com/",
        "https://user@example.com/",
    ] {
        assert_eq!(url_refusal(url), "URLs may not carry credentials");
    }
    for url in [
        "https://example.com:0/",
        "https://example.com:65536/",
        "https://example.com:44a/",
    ] {
        assert_eq!(url_refusal(url), "the URL has an invalid port");
    }
    for url in ["https://example..com/", "https://.example.com/"] {
        assert_eq!(url_refusal(url), "the host has an empty label");
    }
    assert_eq!(
        url_refusal("https://exa%6Dple.com/"),
        "the host must be an ASCII domain name (punycode for international names)"
    );
}
#[test]
fn net_policy_refuses_ip_literals_in_every_form_the_url_standard_reads() {
    for url in [
        "https://127.0.0.1/",
        "https://1.2.3.4/",
        "https://127.1/",
        "https://0x7f.1/",
        "https://0x7f000001/",
        "https://2130706433/",
        "https://192.168.0.1./",
        "https://[::1]/",
        "https://[2001:db8::1]:443/",
    ] {
        assert_eq!(url_refusal(url), IP_HOST);
    }
}
#[test]
fn net_policy_refuses_local_network_and_single_label_hosts() {
    for url in ["https://localhost/", "https://intranet/"] {
        assert_eq!(
            url_refusal(url),
            "the host must be a fully qualified domain name"
        );
    }
    for url in [
        "https://app.localhost/",
        "https://printer.local/",
        "https://db.internal/",
        "https://router.home.arpa/",
        "https://nas.lan/",
    ] {
        assert_eq!(url_refusal(url), LOCAL_HOST);
    }
}
#[test]
fn net_policy_refuses_rives_own_hosts() {
    for url in [
        "https://rive.app/",
        "https://API.Rive.App/api/me",
        "https://editor.rive.app./file",
        "https://2dimensions.com/",
        "https://net.riveusercontent.com/v1/",
    ] {
        assert_eq!(url_refusal(url), RIVE_HOST);
    }
}
#[test]
fn net_policy_upper_cases_and_limits_methods() {
    let mut request = HttpRequest {
        url: "https://example.com/".into(),
        method: "patch".into(),
        ..Default::default()
    };
    assert!(refusal(&mut request, NetLimits::default()).is_empty());
    assert_eq!(request.method, b"PATCH");
    request.method.clear();
    assert!(refusal(&mut request, NetLimits::default()).is_empty());
    assert_eq!(request.method, b"GET");
    for (method, message) in [
        ("CONNECT", "the CONNECT method is not allowed"),
        ("trace", "the TRACE method is not allowed"),
        ("G ET", "the method is not a valid HTTP token"),
    ] {
        request.method = method.into();
        assert_eq!(refusal(&mut request, NetLimits::default()), message);
    }
    request.body = vec![1, 2, 3];
    for method in ["GET", "HEAD"] {
        request.method = method.into();
        assert_eq!(
            refusal(&mut request, NetLimits::default()),
            "GET and HEAD requests cannot have a body"
        );
    }
    request.method = "POST".into();
    assert!(refusal(&mut request, NetLimits::default()).is_empty());
}
#[test]
fn net_policy_keeps_scripts_off_forbidden_headers() {
    let header_refusal = |name: &str, value: &str| {
        refusal(
            &mut HttpRequest {
                url: "https://example.com/".into(),
                headers: vec![HttpHeader {
                    name: name.into(),
                    value: value.into(),
                }],
                ..Default::default()
            },
            NetLimits::default(),
        )
    };
    for (name, value) in [
        ("Authorization", "Bearer token"),
        ("Content-Type", "application/json"),
        ("X-Custom", "a\tb"),
    ] {
        assert!(header_refusal(name, value).is_empty());
    }
    for (name, value) in [
        ("Cookie", "a=b"),
        ("cookie2", "a=b"),
        ("Host", "rive.app"),
        ("Origin", "https://editor.rive.app"),
        ("Referer", "x"),
        ("Content-Length", "1"),
        ("Sec-Fetch-Site", "same-origin"),
        ("Proxy-Authorization", "x"),
        ("X-HTTP-Method-Override", "DELETE"),
    ] {
        assert_eq!(
            header_refusal(name, value),
            format!("scripts cannot set the '{name}' header")
        );
    }
    for name in ["Bad Name", ""] {
        assert_eq!(
            header_refusal(name, "x"),
            "a header name is not a valid HTTP token"
        );
    }
    assert_eq!(
        header_refusal("X-Split", "a\r\nInjected: 1"),
        "the value of the 'X-Split' header must be printable ASCII"
    );
    assert_eq!(
        header_refusal("X-Accent", "café"),
        "the value of the 'X-Accent' header must be printable ASCII"
    );
    assert!(NetPolicy::is_forbidden_header("SEC-WEBSOCKET-KEY"));
    assert!(!NetPolicy::is_forbidden_header("Accept"));
}
#[test]
fn net_policy_enforces_size_limits() {
    let limits = NetLimits {
        max_url_length: 32,
        max_request_body_bytes: 4,
        max_header_count: 2,
        max_header_bytes: 16,
        ..Default::default()
    };
    let mut request = HttpRequest {
        url: "https://example.com/a-path-that-is-too-long".into(),
        ..Default::default()
    };
    assert_eq!(
        refusal(&mut request, limits),
        "the URL is longer than 32 characters"
    );
    request.url = "https://example.com/".into();
    request.method = "POST".into();
    request.body = vec![1, 2, 3, 4, 5];
    assert_eq!(
        refusal(&mut request, limits),
        "the request body is larger than 4 bytes"
    );
    request.body = vec![1, 2, 3, 4];
    assert!(refusal(&mut request, limits).is_empty());
    request.headers = [("A", "1"), ("B", "2"), ("C", "3")]
        .map(|(name, value)| HttpHeader {
            name: name.into(),
            value: value.into(),
        })
        .into();
    assert_eq!(refusal(&mut request, limits), "more than 2 headers");
    request.headers = vec![HttpHeader {
        name: "X-Long".into(),
        value: "0123456789abcdef".into(),
    }];
    assert_eq!(
        refusal(&mut request, limits),
        "the headers are larger than 16 bytes"
    );
}
#[test]
fn net_policy_defaults_and_clamps_the_timeout() {
    let limits = NetLimits::default();
    let mut request = HttpRequest {
        url: "https://example.com/".into(),
        ..Default::default()
    };
    assert!(refusal(&mut request, limits).is_empty());
    assert_eq!(request.timeout_ms, limits.default_timeout_ms);
    request.timeout_ms = 5000;
    assert!(refusal(&mut request, limits).is_empty());
    assert_eq!(request.timeout_ms, 5000);
    request.timeout_ms = limits.max_timeout_ms * 10;
    assert!(refusal(&mut request, limits).is_empty());
    assert_eq!(request.timeout_ms, limits.max_timeout_ms);
}
