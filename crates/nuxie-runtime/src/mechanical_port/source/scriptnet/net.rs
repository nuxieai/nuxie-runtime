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
    queued: bool,
    request: HttpRequest,
    started: bool,
    counted: bool,
    settled: bool,
    provider: Option<Arc<dyn Provider>>,
    starting: bool,
    cancelled: bool,
}
#[derive(Default)]
struct Owner {
    in_flight: u32,
    recent_starts: VecDeque<Instant>,
    waiting: VecDeque<RequestId>,
    waiting_body_bytes: usize,
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
    fn release(&mut self, id: RequestId) {
        let entry = self.outstanding.get_mut(&id).unwrap();
        if !entry.counted {
            return;
        }
        entry.counted = false;
        if let Some(owner) = self.owners.get_mut(&entry.owner_id) {
            if owner.in_flight > 0 {
                owner.in_flight -= 1;
            }
        }
    }
    fn erase(&mut self, id: RequestId) {
        if self.outstanding.contains_key(&id) {
            self.release(id);
        }
        if let Some(entry) = self.outstanding.remove(&id) {
            if entry.queued {
                if let Some(owner) = self.owners.get_mut(&entry.owner_id) {
                    if let Some(at) = owner.waiting.iter().position(|&waiting| waiting == id) {
                        owner.waiting.remove(at);
                    }
                    owner.waiting_body_bytes -= entry.request.body.len();
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
        self.release(id);
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
    fn allow(&self, request: &mut HttpRequest) -> Result<(), NetError> {
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
        Ok(())
    }
    fn can_start(limits: &NetLimits, owner: &mut Owner, now: Instant) -> bool {
        if owner.in_flight >= limits.max_in_flight_per_owner {
            return false;
        }
        if limits.max_requests_per_minute_per_owner == 0 {
            return true;
        }
        while owner
            .recent_starts
            .front()
            .is_some_and(|&start| now.duration_since(start) >= Duration::from_secs(60))
        {
            owner.recent_starts.pop_front();
        }
        owner.recent_starts.len() < limits.max_requests_per_minute_per_owner as usize
    }
    fn begin(&mut self, id: RequestId, timeout_ms: u32) {
        let now = Instant::now();
        let entry = self.outstanding.get_mut(&id).unwrap();
        let owner = self.owners.get_mut(&entry.owner_id).unwrap();
        if self.limits.max_requests_per_minute_per_owner != 0 {
            owner.recent_starts.push_back(now);
        }
        owner.in_flight += 1;
        entry.counted = true;
        entry.started = true;
        entry.starting = true;
        entry.provider = self.provider.clone();
        entry.deadline = Some(
            now + Duration::from_millis(u64::from(timeout_ms))
                + Duration::from_millis(u64::from(self.limits.watchdog_grace_ms)),
        );
    }
    fn enqueue(&mut self, id: RequestId, request: HttpRequest) -> Result<(), NetError> {
        let entry = self.outstanding.get_mut(&id).unwrap();
        let owner = self.owners.get_mut(&entry.owner_id).unwrap();
        let bytes = request.body.len();
        if owner.waiting.len() >= self.limits.max_queued_per_owner as usize {
            return Err(NetError {
                code: NetErrorCode::RateLimited,
                message: format!(
                    "more than {} requests waiting",
                    self.limits.max_queued_per_owner
                ),
            });
        }
        if owner.waiting_body_bytes.wrapping_add(bytes)
            > self.limits.max_queued_body_bytes_per_owner as usize
        {
            return Err(NetError {
                code: NetErrorCode::RateLimited,
                message: format!(
                    "more than {} bytes of request bodies waiting",
                    self.limits.max_queued_body_bytes_per_owner
                ),
            });
        }
        entry.queued = true;
        entry.request = request;
        owner.waiting.push_back(id);
        owner.waiting_body_bytes = owner.waiting_body_bytes.wrapping_add(bytes);
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
                queued: false,
                request: HttpRequest::default(),
                started: false,
                counted: false,
                settled: false,
                provider: None,
                starting: false,
                cancelled: false,
            },
        );
        OUTSTANDING_COUNT.store(net.outstanding.len(), Ordering::Relaxed);
        if let Err(error) = net.allow(&mut request) {
            net.settle(id, error);
            return id;
        }
        let limits = net.limits;
        let owner = net.owners.entry(owner_id).or_default();
        if !owner.waiting.is_empty() || !Net::can_start(&limits, owner, Instant::now()) {
            if let Err(error) = net.enqueue(id, request) {
                net.settle(id, error);
            }
            return id;
        }
        net.begin(id, request.timeout_ms);
        (id, net.provider.clone().unwrap())
    };
    start_request(id, &request, provider);
    id
}
// Provider calls run without the Net mutex; cancellation during start is
// reconciled afterwards against the transport that actually started it.
fn start_request(id: RequestId, request: &HttpRequest, provider: Arc<dyn Provider>) {
    provider.start(id, request);
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
    let thread = thread::current().id();
    let (overdue, ready, provider, before) = {
        let mut net = instance().lock().unwrap();
        let now = Instant::now();
        let provider = net.provider.clone();
        let mut ready = Vec::new();
        let owner_ids: Vec<_> = net.owners.keys().copied().collect();
        for owner_id in owner_ids {
            loop {
                let Some(id) = net.owners[&owner_id].waiting.front().copied() else {
                    break;
                };
                let Some(entry) = net.outstanding.get(&id) else {
                    net.owners.get_mut(&owner_id).unwrap().waiting.pop_front();
                    continue;
                };
                if entry.thread != thread {
                    break;
                }
                let limits = net.limits;
                if !Net::can_start(&limits, net.owners.get_mut(&owner_id).unwrap(), now) {
                    break;
                }
                let bytes = net.outstanding[&id].request.body.len();
                let owner = net.owners.get_mut(&owner_id).unwrap();
                owner.waiting.pop_front();
                owner.waiting_body_bytes -= bytes;
                net.outstanding.get_mut(&id).unwrap().queued = false;
                if provider.is_none() {
                    net.settle(
                        id,
                        NetError {
                            code: NetErrorCode::Disabled,
                            message: "network access is not enabled for scripts in this host"
                                .into(),
                        },
                    );
                    continue;
                }
                let timeout_ms = net.outstanding[&id].request.timeout_ms;
                net.begin(id, timeout_ms);
                let request = std::mem::take(&mut net.outstanding.get_mut(&id).unwrap().request);
                ready.push((id, request));
            }
        }

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
        (overdue, ready, provider, net.next_sequence)
    };
    for (id, provider) in overdue {
        if let Some(provider) = provider {
            provider.abort(id);
        }
    }
    for (id, request) in ready {
        start_request(id, &request, provider.as_ref().unwrap().clone());
    }
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
/// Any request is outstanding, for any owner on any thread.
pub fn has_outstanding_requests() -> bool {
    OUTSTANDING_COUNT.load(Ordering::Relaxed) != 0
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
    net.release(id);
    net.queue(id, Ok(response));
}
pub fn fail(id: RequestId, error: NetError) {
    let mut net = instance().lock().unwrap();
    if net.outstanding.get(&id).is_none_or(|e| e.settled) {
        return;
    }
    net.settle(id, error);
}
