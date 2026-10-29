//! Optional script networking. No transport is installed until a host opts in.
pub mod http;
pub mod net;
pub mod net_policy;
#[cfg(target_vendor = "apple")]
pub mod net_provider_apple;
#[cfg(windows)]
pub mod net_provider_windows;
#[cfg(all(test, any(target_vendor = "apple", windows)))]
mod net_provider_platform_test;
pub use http::*;
pub use net::*;
pub use net_policy::*;
