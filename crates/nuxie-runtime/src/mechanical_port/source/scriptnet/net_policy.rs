//! Mechanical translation of rive/scriptnet/net_policy.hpp and net_policy.cpp.
use super::http::{HttpRequest, NetError, NetErrorCode};

/// Host ceilings: requests beyond active/rate limits wait in their owner's queue.
/// Only a full queue, policy refusal or size limit rejects a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetLimits {
    /// Requests one VM may have with the transport at once.
    pub max_in_flight_per_owner: u32,
    /// Starts in a rolling 60-second window; zero means no limit.
    pub max_requests_per_minute_per_owner: u32,
    /// Waiting requests and their combined body bytes, per VM.
    pub max_queued_per_owner: u32,
    pub max_queued_body_bytes_per_owner: u32,
    /// Undelivered answers, reserving a count slot for each in-flight request.
    pub max_undelivered_per_owner: u32,
    /// Bodies already answered. In-flight bodies may exceed this cap by up to
    /// max_in_flight_per_owner times max_response_body_bytes when they arrive.
    pub max_undelivered_bytes_per_owner: u32,
    pub max_request_body_bytes: u32,
    pub max_response_body_bytes: u32,
    pub default_timeout_ms: u32,
    pub max_timeout_ms: u32,
    pub watchdog_grace_ms: u32,
    pub max_url_length: u32,
    pub max_header_count: u32,
    pub max_header_bytes: u32,
    /// Maximum hops for transports that follow redirects themselves.
    pub max_redirects: u32,
    /// Test-only opt-in for resolved local/private addresses.
    pub allow_local_addresses: bool,
}
impl Default for NetLimits {
    fn default() -> Self {
        Self {
            max_in_flight_per_owner: 8,
            max_requests_per_minute_per_owner: 0,
            max_queued_per_owner: 1024,
            max_queued_body_bytes_per_owner: 32 * 1024 * 1024,
            max_undelivered_per_owner: 64,
            max_undelivered_bytes_per_owner: 64 * 1024 * 1024,
            max_request_body_bytes: 8 * 1024 * 1024,
            max_response_body_bytes: 16 * 1024 * 1024,
            default_timeout_ms: 30_000,
            max_timeout_ms: 60_000,
            watchdog_grace_ms: 5_000,
            max_url_length: 8 * 1024,
            max_header_count: 64,
            max_header_bytes: 16 * 1024,
            max_redirects: 20,
            allow_local_addresses: false,
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

/// Shared request, redirect and address policy. Apple retains its known
/// DNS-to-private-address gap; Windows checks each connecting address.
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

fn ipv4_kind(a: &[u8]) -> Option<&'static str> {
    if a[0] == 127 {
        return Some("loopback");
    }
    if a[0] == 10
        || (a[0] == 172 && (a[1] & 0xf0) == 16)
        || (a[0] == 192 && a[1] == 168)
        || (a[0] == 100 && (a[1] & 0xc0) == 64)
    {
        return Some("private");
    }
    if a[0] == 169 && a[1] == 254 {
        return Some("link-local");
    }
    if a[0] == 0
        || (a[0] == 192 && a[1] == 0 && (a[2] == 0 || a[2] == 2))
        || (a[0] == 198 && a[1] == 51 && a[2] == 100)
        || (a[0] == 203 && a[1] == 0 && a[2] == 113)
        || (a[0] == 198 && (a[1] & 0xfe) == 18)
        || a[0] >= 224
    {
        return Some("reserved");
    }
    None
}
fn is_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|&b| b == 0)
}
fn ipv6_kind(a: &[u8]) -> Option<&'static str> {
    if is_zero(&a[..15]) {
        return match a[15] {
            0 => Some("reserved"),
            1 => Some("loopback"),
            _ => None,
        };
    }
    if is_zero(&a[..10]) && ((a[10] == 0xff && a[11] == 0xff) || (a[10] == 0 && a[11] == 0)) {
        return ipv4_kind(&a[12..]);
    }
    if a[..4] == [0, 0x64, 0xff, 0x9b] {
        if a[4] == 0 && a[5] == 1 {
            return Some("private");
        }
        return if is_zero(&a[4..12]) {
            ipv4_kind(&a[12..])
        } else {
            None
        };
    }
    if a[0] == 0x20 && a[1] == 0x02 {
        return ipv4_kind(&a[2..]);
    }
    if (a[0] & 0xfe) == 0xfc || (a[0] == 0xfe && (a[1] & 0xc0) == 0xc0) {
        return Some("private");
    }
    if a[0] == 0xfe && (a[1] & 0xc0) == 0x80 {
        return Some("link-local");
    }
    if a[0] == 0xff || (a[0] == 1 && is_zero(&a[1..8])) || a[..4] == [0x20, 0x01, 0x0d, 0xb8] {
        return Some("reserved");
    }
    None
}

// RFC 3986 section 5.2.4, on a path that starts with '/'.
fn remove_dot_segments(path: &[u8]) -> Vec<u8> {
    let mut kept = Vec::new();
    let mut directory = false;
    let mut start = 1;
    while start <= path.len() {
        let slash = path[start..]
            .iter()
            .position(|&c| c == b'/')
            .map(|i| start + i);
        let last = slash.is_none();
        let end = slash.unwrap_or(path.len());
        let segment = &path[start..end];
        if segment == b"." {
            directory = last;
        } else if segment == b".." {
            kept.pop();
            directory = last;
        } else {
            kept.push(segment);
            directory = false;
        }
        if last {
            break;
        }
        start = end + 1;
    }
    let mut result = Vec::new();
    for segment in kept {
        result.push(b'/');
        result.extend_from_slice(segment);
    }
    if directory {
        result.push(b'/');
    }
    if result.is_empty() {
        result.push(b'/');
    }
    result
}
fn resolve_location(base: &[u8], location: &[u8]) -> Option<Vec<u8>> {
    let first = location.iter().position(|c| !b" \t".contains(c))?;
    let end = location.iter().rposition(|c| !b" \t".contains(c)).unwrap() + 1;
    let mut location_encoded = Vec::new();
    for &byte in &location[first..end] {
        if byte <= 0x20 || byte >= 0x7f || byte == b'\\' {
            const HEX: &[u8] = b"0123456789ABCDEF";
            location_encoded.extend_from_slice(&[
                b'%',
                HEX[(byte >> 4) as usize],
                HEX[(byte & 0xf) as usize],
            ]);
        } else {
            location_encoded.push(byte);
        }
    }
    let location = location_encoded.as_slice();
    if location[0].is_ascii_alphabetic() {
        let mut at = 1;
        while at < location.len()
            && (location[at].is_ascii_alphanumeric() || b"+-.".contains(&location[at]))
        {
            at += 1;
        }
        if at < location.len() && location[at] == b':' {
            return Some(location.to_vec());
        }
    }
    let authority_start = base.windows(3).position(|s| s == b"://")? + 3;
    if location.starts_with(b"//") {
        return Some([&base[..authority_start - 2], location].concat());
    }
    let authority_end = base[authority_start..]
        .iter()
        .position(|c| b"/?#".contains(c))
        .map_or(base.len(), |i| authority_start + i);
    let origin = &base[..authority_end];
    let path_end = base[authority_end..]
        .iter()
        .position(|c| b"?#".contains(c))
        .map_or(base.len(), |i| authority_end + i);
    let mut path = &base[authority_end..path_end];
    if path.is_empty() {
        path = b"/";
    }
    if location[0] == b'#' {
        return Some(base.to_vec());
    }
    let merged = if location[0] == b'/' {
        location.to_vec()
    } else if location[0] == b'?' {
        [path, location].concat()
    } else {
        [
            &path[..path.iter().rposition(|&c| c == b'/').map_or(0, |i| i + 1)],
            location,
        ]
        .concat()
    };
    let merged_path_end = merged
        .iter()
        .position(|c| b"?#".contains(c))
        .unwrap_or(merged.len());
    Some(
        [
            origin,
            &remove_dot_segments(&merged[..merged_path_end]),
            &merged[merged_path_end..],
        ]
        .concat(),
    )
}
fn origin_of(url: &[u8]) -> Vec<u8> {
    let authority_start = url
        .windows(3)
        .position(|s| s == b"://")
        .map_or(0, |i| i + 3);
    let end = url[authority_start..]
        .iter()
        .position(|c| b"/?#".contains(c))
        .map_or(url.len(), |i| authority_start + i);
    let mut origin = url[..end].to_vec();
    let Some(colon) = origin[authority_start..]
        .iter()
        .position(|&c| c == b':')
        .map(|i| authority_start + i)
    else {
        return origin;
    };
    let mut port = 0u32;
    for &c in &origin[colon + 1..] {
        port = port
            .wrapping_mul(10)
            .wrapping_add(u32::from(c.wrapping_sub(b'0')));
    }
    origin.truncate(colon);
    if port != 443 {
        origin.push(b':');
        origin.extend_from_slice(port.to_string().as_bytes());
    }
    origin
}
fn drop_headers(headers: &mut super::http::HttpHeaders, names: &[&[u8]]) {
    headers.retain(|header| !names.contains(&header.name.to_ascii_lowercase().as_slice()));
}
impl NetPolicy {
    /// Checks the resolved address for each connection hop. Apple does not yet
    /// apply this check; Windows does so before sending, in its connecting callback.
    pub fn check_address(address: &[u8], reason: &mut String) -> bool {
        let kind = match address.len() {
            4 => ipv4_kind(address),
            16 => ipv6_kind(address),
            _ => {
                *reason = "the host resolves to an address of an unknown kind".into();
                return false;
            }
        };
        if let Some(kind) = kind {
            *reason = format!("the host resolves to a {kind} address");
            false
        } else {
            true
        }
    }
    /// Carries the last sent request to a checked redirect hop. Failure leaves
    /// the request unchanged; redirects preserve bytes, including Location UTF-8.
    pub fn redirect(
        request: &mut HttpRequest,
        status: u16,
        location: impl AsRef<[u8]>,
        hops: u32,
        limits: &NetLimits,
        error: &mut NetError,
    ) -> bool {
        if hops >= limits.max_redirects {
            *error = NetError {
                code: NetErrorCode::Network,
                message: format!("more than {} redirects", limits.max_redirects),
            };
            return false;
        }
        let mut hop = request.clone();
        let Some(url) = resolve_location(&request.url, location.as_ref()) else {
            *error = NetError {
                code: NetErrorCode::Policy,
                message: "the Location header could not be resolved".into(),
            };
            return false;
        };
        hop.url = url;
        if (status == 303 && hop.method != b"GET" && hop.method != b"HEAD")
            || ((status == 301 || status == 302) && hop.method == b"POST")
        {
            hop.method = b"GET".to_vec();
            hop.body.clear();
            drop_headers(
                &mut hop.headers,
                &[
                    b"content-encoding",
                    b"content-language",
                    b"content-location",
                    b"content-type",
                ],
            );
        }
        if !Self::check(&mut hop, limits, error) {
            return false;
        }
        if origin_of(&hop.url) != origin_of(&request.url) {
            drop_headers(&mut hop.headers, &[b"authorization"]);
        }
        *request = hop;
        true
    }
}
