//! Mechanical translation of rive/scriptnet/http.hpp.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HttpHeader {
    pub name: Vec<u8>,
    pub value: Vec<u8>,
}
pub type HttpHeaders = Vec<HttpHeader>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpRequest {
    pub url: Vec<u8>,
    pub method: Vec<u8>,
    pub headers: HttpHeaders,
    pub body: Vec<u8>,
    pub timeout_ms: u32,
}
impl Default for HttpRequest {
    fn default() -> Self {
        Self {
            url: Vec::new(),
            method: b"GET".to_vec(),
            headers: Vec::new(),
            body: Vec::new(),
            timeout_ms: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: Vec<u8>,
    pub url: Vec<u8>,
    pub headers: HttpHeaders,
    pub body: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum NetErrorCode {
    Disabled,
    Policy,
    RateLimited,
    Timeout,
    Aborted,
    TooLarge,
    #[default]
    Network,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NetError {
    pub code: NetErrorCode,
    pub message: String,
}

pub fn lower_ascii(value: impl AsRef<[u8]>) -> Vec<u8> {
    value.as_ref().to_ascii_lowercase()
}
