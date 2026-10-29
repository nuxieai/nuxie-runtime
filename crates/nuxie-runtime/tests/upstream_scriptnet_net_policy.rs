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

fn address_refusal(address: &[u8]) -> String {
    let mut reason = String::new();
    if NetPolicy::check_address(address, &mut reason) {
        String::new()
    } else {
        reason
    }
}
fn v6_refusal(groups: &[u16]) -> String {
    let address: Vec<u8> = groups.iter().flat_map(|g| g.to_be_bytes()).collect();
    address_refusal(&address)
}
const LOOPBACK: &str = "the host resolves to a loopback address";
const PRIVATE: &str = "the host resolves to a private address";
const LINK_LOCAL: &str = "the host resolves to a link-local address";
const RESERVED: &str = "the host resolves to a reserved address";

#[test]
fn net_policy_lets_public_ipv4_addresses_through() {
    assert_eq!(address_refusal(&[93, 184, 216, 34]), "");
    assert_eq!(address_refusal(&[1, 1, 1, 1]), "");
    assert_eq!(address_refusal(&[8, 8, 8, 8]), "");
    assert_eq!(address_refusal(&[11, 0, 0, 1]), "");
    assert_eq!(address_refusal(&[172, 15, 255, 255]), "");
    assert_eq!(address_refusal(&[172, 32, 0, 1]), "");
    assert_eq!(address_refusal(&[192, 167, 255, 255]), "");
    assert_eq!(address_refusal(&[192, 169, 0, 1]), "");
    assert_eq!(address_refusal(&[100, 63, 255, 255]), "");
    assert_eq!(address_refusal(&[100, 128, 0, 1]), "");
    assert_eq!(address_refusal(&[169, 253, 0, 1]), "");
    assert_eq!(address_refusal(&[192, 0, 1, 1]), "");
    assert_eq!(address_refusal(&[192, 0, 3, 1]), "");
    assert_eq!(address_refusal(&[198, 17, 255, 255]), "");
    assert_eq!(address_refusal(&[198, 20, 0, 1]), "");
    assert_eq!(address_refusal(&[223, 255, 255, 255]), "");
}

#[test]
fn net_policy_refuses_local_private_reserved_ipv4_addresses() {
    assert_eq!(address_refusal(&[127, 0, 0, 1]), LOOPBACK);
    assert_eq!(address_refusal(&[127, 255, 255, 255]), LOOPBACK);
    assert_eq!(address_refusal(&[10, 0, 0, 1]), PRIVATE);
    assert_eq!(address_refusal(&[172, 16, 0, 1]), PRIVATE);
    assert_eq!(address_refusal(&[172, 31, 255, 255]), PRIVATE);
    assert_eq!(address_refusal(&[192, 168, 1, 1]), PRIVATE);
    assert_eq!(address_refusal(&[100, 64, 0, 1]), PRIVATE);
    assert_eq!(address_refusal(&[100, 127, 255, 255]), PRIVATE);
    assert_eq!(address_refusal(&[169, 254, 169, 254]), LINK_LOCAL);
    assert_eq!(address_refusal(&[0, 0, 0, 0]), RESERVED);
    assert_eq!(address_refusal(&[0, 1, 2, 3]), RESERVED);
    assert_eq!(address_refusal(&[192, 0, 0, 1]), RESERVED);
    assert_eq!(address_refusal(&[192, 0, 2, 1]), RESERVED);
    assert_eq!(address_refusal(&[198, 51, 100, 1]), RESERVED);
    assert_eq!(address_refusal(&[203, 0, 113, 1]), RESERVED);
    assert_eq!(address_refusal(&[198, 18, 0, 1]), RESERVED);
    assert_eq!(address_refusal(&[198, 19, 255, 255]), RESERVED);
    assert_eq!(address_refusal(&[224, 0, 0, 1]), RESERVED);
    assert_eq!(address_refusal(&[240, 0, 0, 1]), RESERVED);
    assert_eq!(address_refusal(&[255, 255, 255, 255]), RESERVED);
}

#[test]
fn net_policy_judges_ipv6_addresses() {
    assert_eq!(v6_refusal(&[0x2606, 0x4700, 0, 0, 0, 0, 0, 1]), "");
    assert_eq!(
        v6_refusal(&[0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888]),
        ""
    );
    assert_eq!(v6_refusal(&[0x2001, 0x0db9, 0, 0, 0, 0, 0, 1]), "");
    assert_eq!(v6_refusal(&[0, 0, 0, 0, 0, 0, 0, 1]), LOOPBACK);
    assert_eq!(v6_refusal(&[0, 0, 0, 0, 0, 0, 0, 0]), RESERVED);
    assert_eq!(v6_refusal(&[0xfe80, 0, 0, 0, 0, 0, 0, 1]), LINK_LOCAL);
    assert_eq!(v6_refusal(&[0xfebf, 0, 0, 0, 0, 0, 0, 1]), LINK_LOCAL);
    assert_eq!(v6_refusal(&[0xfc00, 0, 0, 0, 0, 0, 0, 1]), PRIVATE);
    assert_eq!(v6_refusal(&[0xfd12, 0x3456, 0, 0, 0, 0, 0, 1]), PRIVATE);
    assert_eq!(v6_refusal(&[0xfec0, 0, 0, 0, 0, 0, 0, 1]), PRIVATE);
    assert_eq!(v6_refusal(&[0xff02, 0, 0, 0, 0, 0, 0, 1]), RESERVED);
    assert_eq!(v6_refusal(&[0x0100, 0, 0, 0, 0, 0, 0, 1]), RESERVED);
    assert_eq!(v6_refusal(&[0x2001, 0x0db8, 0, 0, 0, 0, 0, 1]), RESERVED);
}

#[test]
fn net_policy_judges_ipv6_carried_ipv4_addresses() {
    assert_eq!(
        v6_refusal(&[0, 0, 0, 0, 0, 0xffff, 0x7f00, 0x0001]),
        LOOPBACK
    );
    assert_eq!(
        v6_refusal(&[0, 0, 0, 0, 0, 0xffff, 0x0a00, 0x0001]),
        PRIVATE
    );
    assert_eq!(v6_refusal(&[0, 0, 0, 0, 0, 0xffff, 0x0101, 0x0101]), "");
    assert_eq!(v6_refusal(&[0, 0, 0, 0, 0, 0, 0xc0a8, 0x0001]), PRIVATE);
    assert_eq!(v6_refusal(&[0, 0, 0, 0, 0, 0, 0x0101, 0x0101]), "");
    assert_eq!(
        v6_refusal(&[0x0064, 0xff9b, 0, 0, 0, 0, 0x0a00, 0x0001]),
        PRIVATE
    );
    assert_eq!(
        v6_refusal(&[0x0064, 0xff9b, 0, 0, 0, 0, 0x0101, 0x0101]),
        ""
    );
    assert_eq!(
        v6_refusal(&[0x0064, 0xff9b, 1, 0, 0, 0, 0x0101, 0x0101]),
        PRIVATE
    );
    assert_eq!(
        v6_refusal(&[0x2002, 0x0a00, 0x0001, 0, 0, 0, 0, 1]),
        PRIVATE
    );
    assert_eq!(
        v6_refusal(&[0x2002, 0xa9fe, 0x0001, 0, 0, 0, 0, 1]),
        LINK_LOCAL
    );
    assert_eq!(v6_refusal(&[0x2002, 0x0101, 0x0101, 0, 0, 0, 0, 1]), "");
}

#[test]
fn net_policy_refuses_an_address_of_another_size() {
    assert_eq!(
        address_refusal(&[1, 2, 3]),
        "the host resolves to an address of an unknown kind"
    );
    assert_eq!(
        address_refusal(&[1, 2, 3, 4, 5]),
        "the host resolves to an address of an unknown kind"
    );
}

fn sent_request(url: &str, method: &str, headers: HttpHeaders, body: Vec<u8>) -> HttpRequest {
    let mut request = HttpRequest {
        url: url.into(),
        method: method.into(),
        headers,
        body,
        ..Default::default()
    };
    assert!(refusal(&mut request, NetLimits::default()).is_empty());
    request
}
fn hop_url(from: &str, status: u16, location: &str) -> String {
    let mut request = sent_request(from, "GET", vec![], vec![]);
    let mut error = NetError::default();
    if !NetPolicy::redirect(
        &mut request,
        status,
        location,
        0,
        &NetLimits::default(),
        &mut error,
    ) {
        return format!("refused: {}", error.message);
    }
    String::from_utf8(request.url).unwrap()
}
fn has_header(request: &HttpRequest, name: &str) -> bool {
    request
        .headers
        .iter()
        .any(|header| header.name.to_ascii_lowercase() == name.as_bytes())
}
#[test]
fn net_policy_redirect_resolves_location() {
    let from = "https://example.com/a/b?y=1";
    for (location, expected) in [
        ("https://other.example.com/x", "https://other.example.com/x"),
        ("//cdn.example.com/a", "https://cdn.example.com/a"),
        ("/root?q=1", "https://example.com/root?q=1"),
        ("c", "https://example.com/a/c"),
        ("?x=2", "https://example.com/a/b?x=2"),
        ("#section", "https://example.com/a/b?y=1"),
        (
            "/café menu\\%20x",
            "https://example.com/caf%C3%A9%20menu%5C%20x",
        ),
        ("  /spaced  ", "https://example.com/spaced"),
        ("/p#frag", "https://example.com/p"),
        ("", "refused: the Location header could not be resolved"),
    ] {
        assert_eq!(hop_url(from, 302, location), expected);
    }
    assert_eq!(
        hop_url("https://example.com/a/b/", 302, "../d"),
        "https://example.com/a/d"
    );
    assert_eq!(
        hop_url("https://example.com/a/b/", 302, "./"),
        "https://example.com/a/b/"
    );
    assert_eq!(
        hop_url("https://example.com", 302, "x"),
        "https://example.com/x"
    );
}
#[test]
fn net_policy_redirect_continues_as_get_where_browsers_do() {
    let headers: HttpHeaders = [("Content-Type", "text/plain"), ("X-Keep", "1")]
        .into_iter()
        .map(|(name, value)| HttpHeader {
            name: name.into(),
            value: value.into(),
        })
        .collect();
    let body = b"ping".to_vec();
    let mut error = NetError::default();
    let mut see_other = sent_request("https://example.com/", "PUT", headers.clone(), body.clone());
    assert!(NetPolicy::redirect(
        &mut see_other,
        303,
        "/next",
        0,
        &NetLimits::default(),
        &mut error
    ));
    assert_eq!(see_other.method, b"GET");
    assert!(see_other.body.is_empty());
    assert!(!has_header(&see_other, "content-type"));
    assert!(has_header(&see_other, "x-keep"));
    let mut found = sent_request(
        "https://example.com/",
        "POST",
        headers.clone(),
        body.clone(),
    );
    assert!(NetPolicy::redirect(
        &mut found,
        302,
        "/next",
        0,
        &NetLimits::default(),
        &mut error
    ));
    assert_eq!(found.method, b"GET");
    assert!(found.body.is_empty());
    let mut put = sent_request("https://example.com/", "PUT", headers.clone(), body.clone());
    assert!(NetPolicy::redirect(
        &mut put,
        302,
        "/next",
        0,
        &NetLimits::default(),
        &mut error
    ));
    assert_eq!(put.method, b"PUT");
    assert_eq!(put.body, body);
    assert!(has_header(&put, "content-type"));
    let mut temporary = sent_request("https://example.com/", "POST", headers, body.clone());
    assert!(NetPolicy::redirect(
        &mut temporary,
        307,
        "/next",
        0,
        &NetLimits::default(),
        &mut error
    ));
    assert_eq!(temporary.method, b"POST");
    assert_eq!(temporary.body, body);
}
#[test]
fn net_policy_redirect_drops_authorization_when_leaving_origin() {
    let headers: HttpHeaders = [("Authorization", "Bearer secret"), ("X-Keep", "1")]
        .into_iter()
        .map(|(name, value)| HttpHeader {
            name: name.into(),
            value: value.into(),
        })
        .collect();
    for (from, location, keep) in [
        ("https://example.com/a", "/b", true),
        ("https://example.com:443/a", "https://example.com/b", true),
        ("https://example.com:0443/a", "https://example.com/b", true),
        (
            "https://example.com:08443/a",
            "https://example.com:8443/b",
            true,
        ),
        (
            "https://example.com/a",
            "https://other.example.com/b",
            false,
        ),
        ("https://example.com/a", "https://example.com:8443/b", false),
    ] {
        let mut request = sent_request(from, "GET", headers.clone(), vec![]);
        let mut error = NetError::default();
        assert!(NetPolicy::redirect(
            &mut request,
            302,
            location,
            0,
            &NetLimits::default(),
            &mut error
        ));
        assert_eq!(has_header(&request, "authorization"), keep);
        if location == "https://other.example.com/b" {
            assert!(has_header(&request, "x-keep"));
        }
    }
}
#[test]
fn net_policy_redirect_refuses_policy_hops_and_hops_past_limit() {
    let from = "https://example.com/";
    assert_eq!(
        hop_url(from, 302, "http://example.com/"),
        "refused: http: URLs are not allowed; use https:"
    );
    assert_eq!(
        hop_url(from, 302, "https://api.rive.app/"),
        format!("refused: {RIVE_HOST}")
    );
    assert_eq!(
        hop_url(from, 302, "https://10.0.0.1/"),
        format!("refused: {IP_HOST}")
    );
    let mut request = sent_request(from, "GET", vec![], vec![]);
    let limits = NetLimits::default();
    let mut error = NetError::default();
    assert!(NetPolicy::redirect(
        &mut request,
        302,
        "/a",
        limits.max_redirects - 1,
        &limits,
        &mut error
    ));
    assert!(!NetPolicy::redirect(
        &mut request,
        302,
        "/b",
        limits.max_redirects,
        &limits,
        &mut error
    ));
    assert_eq!(error.code, NetErrorCode::Network);
    assert_eq!(error.message, "more than 20 redirects");
    assert_eq!(request.url, b"https://example.com/a");
}
