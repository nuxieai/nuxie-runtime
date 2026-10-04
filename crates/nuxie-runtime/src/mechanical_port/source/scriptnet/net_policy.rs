//! Mechanical translation of rive/scriptnet/net_policy.hpp and net_policy.cpp.
use super::http::{HttpRequest, NetError, NetErrorCode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetLimits {
    pub max_in_flight_per_owner: u32,
    pub max_requests_per_minute_per_owner: u32,
    pub max_request_body_bytes: u32,
    pub max_response_body_bytes: u32,
    pub default_timeout_ms: u32,
    pub max_timeout_ms: u32,
    pub watchdog_grace_ms: u32,
    pub max_url_length: u32,
    pub max_header_count: u32,
    pub max_header_bytes: u32,
}
impl Default for NetLimits {
    fn default() -> Self {
        Self {
            max_in_flight_per_owner: 8,
            max_requests_per_minute_per_owner: 120,
            max_request_body_bytes: 8 * 1024 * 1024,
            max_response_body_bytes: 16 * 1024 * 1024,
            default_timeout_ms: 30_000,
            max_timeout_ms: 60_000,
            watchdog_grace_ms: 5_000,
            max_url_length: 8 * 1024,
            max_header_count: 64,
            max_header_bytes: 16 * 1024,
        }
    }
}

fn token(value: &[u8]) -> bool {
    !value.is_empty()
        && value
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(c))
}
fn under(host: &str, domain: &str) -> bool {
    host == domain
        || host
            .strip_suffix(domain)
            .is_some_and(|prefix| prefix.ends_with('.'))
}

/// Host-name policy only: the upstream known DNS-to-private-address gap is retained.
pub struct NetPolicy;
impl NetPolicy {
    pub fn check_host(host: impl AsRef<[u8]>, reason: &mut String) -> bool {
        let host = host.as_ref();
        let refuse = |reason: &mut String, message: &str| {
            *reason = message.into();
            false
        };
        if host.is_empty() {
            return refuse(reason, "the URL has no host");
        }
        if host.len() > 253 {
            return refuse(reason, "the host name is too long");
        }
        for c in host {
            if b"[]:".contains(c) {
                return refuse(reason, "IP address hosts are not allowed");
            }
            if !(c.is_ascii_lowercase() || c.is_ascii_digit() || b"-._".contains(c)) {
                return refuse(
                    reason,
                    "the host must be an ASCII domain name (punycode for international names)",
                );
            }
        }
        let host = std::str::from_utf8(host).expect("ASCII host");
        let labels: Vec<_> = host.split('.').collect();
        for label in &labels {
            if label.is_empty() {
                return refuse(reason, "the host has an empty label");
            }
            if label.len() > 63 {
                return refuse(reason, "a host label is longer than 63 characters");
            }
        }
        let last = labels.last().unwrap();
        if last.bytes().all(|c| c.is_ascii_digit())
            || last
                .strip_prefix("0x")
                .is_some_and(|v| v.bytes().all(|c| c.is_ascii_hexdigit()))
        {
            return refuse(reason, "IP address hosts are not allowed");
        }
        if labels.len() < 2 {
            return refuse(reason, "the host must be a fully qualified domain name");
        }
        if [
            "localhost",
            "local",
            "internal",
            "home.arpa",
            "localdomain",
            "lan",
        ]
        .iter()
        .any(|v| under(host, v))
        {
            return refuse(reason, "local network hosts are not allowed");
        }
        if ["rive.app", "2dimensions.com", "riveusercontent.com"]
            .iter()
            .any(|v| under(host, v))
        {
            return refuse(reason, "Rive's own hosts are not reachable from scripts");
        }
        true
    }

    pub fn is_forbidden_header(name: impl AsRef<[u8]>) -> bool {
        let name = name.as_ref().to_ascii_lowercase();
        name.starts_with(b"proxy-")
            || name.starts_with(b"sec-")
            || [
                "accept-charset",
                "accept-encoding",
                "access-control-request-headers",
                "access-control-request-method",
                "access-control-request-private-network",
                "connection",
                "content-length",
                "cookie",
                "cookie2",
                "date",
                "dnt",
                "expect",
                "host",
                "keep-alive",
                "origin",
                "referer",
                "set-cookie",
                "te",
                "trailer",
                "transfer-encoding",
                "upgrade",
                "via",
                "x-http-method",
                "x-http-method-override",
                "x-method-override",
            ]
            .iter()
            .any(|candidate| candidate.as_bytes() == name)
    }

    pub fn check(request: &mut HttpRequest, limits: &NetLimits, error: &mut NetError) -> bool {
        match Self::check_inner(request, limits) {
            Ok(()) => true,
            Err(message) => {
                *error = NetError {
                    code: NetErrorCode::Policy,
                    message,
                };
                false
            }
        }
    }
    fn check_inner(request: &mut HttpRequest, limits: &NetLimits) -> Result<(), String> {
        let url = &request.url;
        if url.is_empty() {
            return Err("the URL is empty".into());
        }
        if url.len() > limits.max_url_length as usize {
            return Err(format!(
                "the URL is longer than {} characters",
                limits.max_url_length
            ));
        }
        if url.iter().any(|&c| c <= 0x20 || c >= 0x7f || c == b'\\') {
            return Err(
                "the URL must be ASCII with no spaces or backslashes; percent-encode anything else"
                    .into(),
            );
        }
        // The byte checks above prove ASCII. Do not decode before them: Lua strings
        // and upstream std::string are byte strings, including malformed UTF-8.
        let url = std::str::from_utf8(url).expect("ASCII URL");
        let colon = url.find(':');
        let scheme = colon.map_or_else(String::new, |colon| url[..colon].to_ascii_lowercase());
        if scheme == "http" {
            return Err("http: URLs are not allowed; use https:".into());
        }
        if scheme != "https" || colon.is_none() {
            return Err("only https: URLs are allowed".into());
        }
        let after = colon.unwrap() + 1;
        if !url[after..].starts_with("//") {
            return Err("the URL has no host".into());
        }
        let begin = after + 2;
        let end = url[begin..]
            .find(['/', '?', '#'])
            .map_or(url.len(), |n| begin + n);
        let authority = &url[begin..end];
        if authority.contains('@') {
            return Err("URLs may not carry credentials".into());
        }
        if authority.starts_with('[') {
            return Err("IP address hosts are not allowed".into());
        }
        let (host, port) = authority.rsplit_once(':').unwrap_or((authority, ""));
        if !port.is_empty() {
            let mut value = 0u32;
            for c in port.bytes() {
                if !c.is_ascii_digit() || value > 65535 {
                    value = 65536;
                    break;
                }
                value = value * 10 + u32::from(c - b'0');
            }
            if value == 0 || value > 65535 {
                return Err("the URL has an invalid port".into());
            }
        }
        let mut host = host.to_ascii_lowercase();
        if host.ends_with('.') {
            host.pop();
        }
        let mut reason = String::new();
        if !Self::check_host(&host, &mut reason) {
            return Err(reason);
        }
        let fragment = url[end..].find('#').map_or(url.len(), |n| end + n);
        let rest = &url[end..fragment];
        request.url = if port.is_empty() {
            format!("https://{host}{rest}")
        } else {
            format!("https://{host}:{port}{rest}")
        }
        .into_bytes();
        let method = if request.method.is_empty() {
            b"GET".to_vec()
        } else {
            request.method.to_ascii_uppercase()
        };
        if ![
            b"GET".as_slice(),
            b"HEAD",
            b"POST",
            b"PUT",
            b"PATCH",
            b"DELETE",
            b"OPTIONS",
        ]
        .contains(&method.as_slice())
        {
            return Err(if token(&method) {
                format!(
                    "the {} method is not allowed",
                    std::str::from_utf8(&method).expect("ASCII token")
                )
            } else {
                "the method is not a valid HTTP token".into()
            });
        }
        request.method = method;
        if (request.method == b"GET" || request.method == b"HEAD") && !request.body.is_empty() {
            return Err("GET and HEAD requests cannot have a body".into());
        }
        if request.headers.len() > limits.max_header_count as usize {
            return Err(format!("more than {} headers", limits.max_header_count));
        }
        let mut bytes = 0usize;
        for header in &request.headers {
            if !token(&header.name) {
                return Err("a header name is not a valid HTTP token".into());
            }
            let name = std::str::from_utf8(&header.name).expect("ASCII token");
            if header
                .value
                .iter()
                .any(|&c| (c < 0x20 && c != b'\t') || c >= 0x7f)
            {
                return Err(format!(
                    "the value of the '{name}' header must be printable ASCII"
                ));
            }
            if Self::is_forbidden_header(name) {
                return Err(format!("scripts cannot set the '{name}' header"));
            }
            bytes += header.name.len() + header.value.len();
        }
        if bytes > limits.max_header_bytes as usize {
            return Err(format!(
                "the headers are larger than {} bytes",
                limits.max_header_bytes
            ));
        }
        if request.body.len() > limits.max_request_body_bytes as usize {
            return Err(format!(
                "the request body is larger than {} bytes",
                limits.max_request_body_bytes
            ));
        }
        if request.timeout_ms == 0 {
            request.timeout_ms = limits.default_timeout_ms;
        }
        request.timeout_ms = request.timeout_ms.min(limits.max_timeout_ms);
        Ok(())
    }
}
