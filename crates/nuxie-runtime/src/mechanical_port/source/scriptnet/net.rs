//! Mechanical translation of rive/scriptnet/net.hpp and net.cpp.
use super::http::{HttpRequest, HttpResponse, NetError, NetErrorCode};
use super::net_policy::{NetLimits, NetPolicy};
use std::collections::{HashMap, VecDeque};
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicUsize, Ordering},
};
use std::thread::{self, ThreadId};
use std::time::Duration;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::Instant;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use web_time::Instant;

pub type RequestId = u32;
pub trait FetchListener: Send + Sync {
    /// Called only by poll on the thread that issued fetch.
    fn on_response(&self, response: HttpResponse);
    fn on_error(&self, error: &NetError);
    /// Cancellation may run while the VM is closing; never call Lua here.
    fn on_cancel(&self) {}
}
pub trait Provider: Send + Sync {
    fn start(&self, id: RequestId, request: &HttpRequest);
    fn abort(&self, _id: RequestId) {}
}
pub fn net_error_code_name(code: NetErrorCode) -> &'static str {
    match code {
        NetErrorCode::Disabled => "disabled",
        NetErrorCode::Policy => "policy",
        NetErrorCode::RateLimited => "rateLimited",
        NetErrorCode::Timeout => "timeout",
        NetErrorCode::Aborted => "aborted",
        NetErrorCode::TooLarge => "tooLarge",
        NetErrorCode::Network => "network",
    }
}
pub fn net_error_code_from_name(name: impl AsRef<[u8]>) -> NetErrorCode {
    [
        NetErrorCode::Disabled,
        NetErrorCode::Policy,
        NetErrorCode::RateLimited,
        NetErrorCode::Timeout,
        NetErrorCode::Aborted,
        NetErrorCode::TooLarge,
    ]
    .into_iter()
    .find(|&code| name.as_ref() == net_error_code_name(code).as_bytes())
    .unwrap_or(NetErrorCode::Network)
}
pub fn make_platform_provider() -> Option<Arc<dyn Provider>> {
    #[cfg(target_vendor = "apple")]
    {
        Some(Arc::new(super::net_provider_apple::AppleProvider::new()))
    }
    #[cfg(not(target_vendor = "apple"))]
    {
        None
    }
}

struct Outstanding {
    owner_id: u64,
    thread: ThreadId,
    listener: Option<Arc<dyn FetchListener>>,
    deadline: Option<Instant>,
    started: bool,
    settled: bool,
    provider: Option<Arc<dyn Provider>>,
    starting: bool,
    cancelled: bool,
}
#[derive(Default)]
struct Owner {
    in_flight: u32,
    recent_starts: VecDeque<Instant>,
}
struct Outcome {
    id: RequestId,
    sequence: u64,
    result: Result<HttpResponse, NetError>,
}
struct Net {
    provider: Option<Arc<dyn Provider>>,
    limits: NetLimits,
    next_id: RequestId,
    outstanding: HashMap<RequestId, Outstanding>,
    outcomes: VecDeque<Outcome>,
    next_sequence: u64,
    owners: HashMap<u64, Owner>,
}
static OUTSTANDING_COUNT: AtomicUsize = AtomicUsize::new(0);
fn instance() -> &'static Mutex<Net> {
    static NET: OnceLock<Mutex<Net>> = OnceLock::new();
    NET.get_or_init(|| {
        Mutex::new(Net {
            provider: None,
            limits: NetLimits::default(),
            next_id: 1,
            outstanding: HashMap::new(),
            outcomes: VecDeque::new(),
            next_sequence: 0,
            owners: HashMap::new(),
        })
    })
}
impl Net {
    fn erase(&mut self, id: RequestId) {
        if let Some(entry) = self.outstanding.remove(&id) {
            if entry.started {
                if let Some(owner) = self.owners.get_mut(&entry.owner_id) {
                    if owner.in_flight > 0 {
                        owner.in_flight -= 1;
                    }
                }
            }
        }
        OUTSTANDING_COUNT.store(self.outstanding.len(), Ordering::Relaxed);
    }
    fn allocate_id(&mut self) -> RequestId {
        loop {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1);
            if id != 0 && !self.outstanding.contains_key(&id) {
                return id;
            }
        }
    }
    fn queue(&mut self, id: RequestId, result: Result<HttpResponse, NetError>) {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        self.outcomes.push_back(Outcome {
            id,
            sequence,
            result,
        });
    }
    fn settle(&mut self, id: RequestId, error: NetError) {
        self.outstanding.get_mut(&id).unwrap().settled = true;
        self.queue(id, Err(error));
    }
    fn take(
        &mut self,
        thread: ThreadId,
        before: u64,
    ) -> Option<(Option<Arc<dyn FetchListener>>, Outcome)> {
        let mut i = 0;
        while i < self.outcomes.len() && self.outcomes[i].sequence < before {
            let id = self.outcomes[i].id;
            let Some(entry) = self.outstanding.get_mut(&id) else {
                self.outcomes.remove(i);
                continue;
            };
            if entry.cancelled {
                self.outcomes.remove(i);
                continue;
            }
            if entry.thread != thread {
                i += 1;
                continue;
            }
            let listener = entry.listener.take();
            let outcome = self.outcomes.remove(i).unwrap();
            self.erase(id);
            return Some((listener, outcome));
        }
        None
    }
    fn admit(&mut self, owner_id: u64, request: &mut HttpRequest) -> Result<(), NetError> {
        if self.provider.is_none() {
            return Err(NetError {
                code: NetErrorCode::Disabled,
                message: "network access is not enabled for scripts in this host".into(),
            });
        }
        let mut error = NetError::default();
        if !NetPolicy::check(request, &self.limits, &mut error) {
            return Err(error);
        }
        let owner = self.owners.entry(owner_id).or_default();
        if owner.in_flight >= self.limits.max_in_flight_per_owner {
            return Err(NetError {
                code: NetErrorCode::RateLimited,
                message: format!(
                    "more than {} requests in flight",
                    self.limits.max_in_flight_per_owner
                ),
            });
        }
        let now = Instant::now();
        while owner
            .recent_starts
            .front()
            .is_some_and(|&start| now.duration_since(start) >= Duration::from_secs(60))
        {
            owner.recent_starts.pop_front();
        }
        if owner.recent_starts.len() >= self.limits.max_requests_per_minute_per_owner as usize {
            return Err(NetError {
                code: NetErrorCode::RateLimited,
                message: format!(
                    "more than {} requests in the last minute",
                    self.limits.max_requests_per_minute_per_owner
                ),
            });
        }
        owner.recent_starts.push_back(now);
        owner.in_flight += 1;
        Ok(())
    }
}

pub fn set_provider(provider: Option<Arc<dyn Provider>>) {
    instance().lock().unwrap().provider = provider;
}
pub fn has_provider() -> bool {
    instance().lock().unwrap().provider.is_some()
}
pub fn set_limits(limits: NetLimits) {
    instance().lock().unwrap().limits = limits;
}
pub fn limits() -> NetLimits {
    instance().lock().unwrap().limits
}

pub fn fetch(
    owner_id: u64,
    mut request: HttpRequest,
    listener: Arc<dyn FetchListener>,
) -> RequestId {
    let (id, provider) = {
        let mut net = instance().lock().unwrap();
        let id = net.allocate_id();
        net.outstanding.insert(
            id,
            Outstanding {
                owner_id,
                thread: thread::current().id(),
                listener: Some(listener),
                deadline: None,
                started: false,
                settled: false,
                provider: None,
                starting: false,
                cancelled: false,
            },
        );
        OUTSTANDING_COUNT.store(net.outstanding.len(), Ordering::Relaxed);
        if let Err(error) = net.admit(owner_id, &mut request) {
            net.settle(id, error);
            return id;
        }
        let provider = net.provider.clone().unwrap();
        let grace = net.limits.watchdog_grace_ms;
        let entry = net.outstanding.get_mut(&id).unwrap();
        entry.started = true;
        entry.starting = true;
        entry.provider = Some(provider.clone());
        entry.deadline = Some(
            Instant::now()
                + Duration::from_millis(u64::from(request.timeout_ms))
                + Duration::from_millis(u64::from(grace)),
        );
        (id, provider)
    };
    provider.start(id, &request);
    let mut abort_now = false;
    {
        let mut net = instance().lock().unwrap();
        if let Some(entry) = net.outstanding.get_mut(&id) {
            entry.starting = false;
            if entry.cancelled {
                abort_now = !entry.settled;
                net.erase(id);
            }
        }
    }
    if abort_now {
        provider.abort(id);
    }
    id
}
pub fn cancel(id: RequestId) -> bool {
    let (listener, provider) = {
        let mut net = instance().lock().unwrap();
        let Some(entry) = net.outstanding.get_mut(&id) else {
            return false;
        };
        if entry.cancelled {
            return false;
        }
        let listener = entry.listener.take();
        let mut provider = None;
        if entry.starting {
            entry.cancelled = true;
        } else {
            if entry.started && !entry.settled {
                provider = entry.provider.clone();
            }
            net.erase(id);
        }
        (listener, provider)
    };
    if let Some(provider) = provider {
        provider.abort(id);
    }
    if let Some(listener) = listener {
        listener.on_cancel();
    }
    true
}
pub fn cancel_all_for_owner(owner_id: u64) {
    let mut listeners = Vec::new();
    let mut to_abort = Vec::new();
    {
        let mut net = instance().lock().unwrap();
        let ids: Vec<_> = net
            .outstanding
            .iter()
            .filter_map(|(&id, entry)| (entry.owner_id == owner_id).then_some(id))
            .collect();
        for id in ids {
            let entry = net.outstanding.get_mut(&id).unwrap();
            listeners.push(entry.listener.take());
            if entry.starting {
                entry.cancelled = true;
                continue;
            }
            if entry.started && !entry.settled {
                to_abort.push((id, entry.provider.clone()));
            }
            net.erase(id);
        }
        net.owners.remove(&owner_id);
    }
    for (id, provider) in to_abort {
        if let Some(provider) = provider {
            provider.abort(id);
        }
    }
    for listener in listeners.into_iter().flatten() {
        listener.on_cancel();
    }
}
pub fn poll(max_deliveries: u32) -> u32 {
    if OUTSTANDING_COUNT.load(Ordering::Relaxed) == 0 {
        return 0;
    }
    let (overdue, before) = {
        let mut net = instance().lock().unwrap();
        let now = Instant::now();
        let overdue: Vec<_> = net
            .outstanding
            .iter()
            .filter_map(|(&id, e)| {
                (e.started && !e.settled && e.deadline.is_some_and(|deadline| now >= deadline))
                    .then(|| (id, e.provider.clone()))
            })
            .collect();
        for (id, _) in &overdue {
            net.settle(
                *id,
                NetError {
                    code: NetErrorCode::Timeout,
                    message: "no response before the timeout".into(),
                },
            );
        }
        (overdue, net.next_sequence)
    };
    for (id, provider) in overdue {
        if let Some(provider) = provider {
            provider.abort(id);
        }
    }
    let thread = thread::current().id();
    let mut delivered = 0;
    while delivered < max_deliveries {
        let next = instance().lock().unwrap().take(thread, before);
        let Some((listener, outcome)) = next else {
            break;
        };
        delivered += 1;
        if let Some(listener) = listener {
            match outcome.result {
                Ok(response) => listener.on_response(response),
                Err(error) => listener.on_error(&error),
            }
        }
    }
    delivered
}
pub fn has_pending_work() -> bool {
    if OUTSTANDING_COUNT.load(Ordering::Relaxed) == 0 {
        return false;
    }
    let thread = thread::current().id();
    instance()
        .lock()
        .unwrap()
        .outstanding
        .values()
        .any(|e| e.thread == thread)
}
/// Pending requests for this owner, regardless of which thread started them.
pub fn has_pending_work_for_owner(owner_id: u64) -> bool {
    let net = instance();
    if OUTSTANDING_COUNT.load(Ordering::Relaxed) == 0 {
        return false;
    }
    net.lock()
        .unwrap()
        .outstanding
        .values()
        .any(|entry| entry.owner_id == owner_id)
}

pub fn complete(id: RequestId, response: HttpResponse) {
    let mut net = instance().lock().unwrap();
    if net.outstanding.get(&id).is_none_or(|e| e.settled) {
        return;
    }
    if response.body.len() > net.limits.max_response_body_bytes as usize {
        let message = format!(
            "the response body is larger than {} bytes",
            net.limits.max_response_body_bytes
        );
        net.settle(
            id,
            NetError {
                code: NetErrorCode::TooLarge,
                message,
            },
        );
        return;
    }
    net.outstanding.get_mut(&id).unwrap().settled = true;
    net.queue(id, Ok(response));
}
pub fn fail(id: RequestId, error: NetError) {
    let mut net = instance().lock().unwrap();
    if net.outstanding.get(&id).is_none_or(|e| e.settled) {
        return;
    }
    net.settle(id, error);
}
