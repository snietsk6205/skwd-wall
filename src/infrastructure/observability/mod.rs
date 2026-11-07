#[cfg(feature = "obs-heap")]
pub use skwd_log::alloc as allocation;
#[cfg(feature = "obs-heap")]
pub use skwd_log::proc as memory;
pub mod logging;
mod macros;
pub mod metrics;
pub mod perf;
mod startup;
pub mod timing;
#[cfg(feature = "obs-trace")]
mod trace_setup;

pub use startup::{elapsed_ms, init_startup_clock, log_startup_checkpoint, rss_label};
