//! Run 29 (Phase 1 1a) — the core's one long-lived async runtime.
//! Implements the Run 27 `runtime-decision` ruling: the core OWNS a single
//! multi-thread tokio runtime, constructed lazily on first use (OnceLock),
//! default builder settings, no worker tuning, no shutdown path. Blocking FFI
//! functions borrow it via `rt().block_on(...)` instead of constructing a
//! runtime per call (the Run 7 shape: "adequate for one probe, not for sync").
//! Explicit-at-startup initialisation is the shell's `init_core` entry point
//! (Run 30), which touches `rt()`; lazy construction here is the backstop.
use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// The core's shared runtime. First call constructs it; every later call
/// returns the same instance for the life of the process. No shutdown path
/// by ruling — the runtime lives as long as the core does.
pub(crate) fn rt() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("lfs_core: failed to construct the shared tokio runtime")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run 27 done-when: reuse proven, not construct-per-call. Two consecutive
    /// borrows in one process return the same runtime instance.
    #[test]
    fn rt_is_shared_across_consecutive_calls() {
        let a = rt() as *const Runtime;
        let b = rt() as *const Runtime;
        assert_eq!(a, b, "rt() must return the same runtime instance");
        // and it actually runs work
        assert_eq!(rt().block_on(async { 41 + 1 }), 42);
        assert_eq!(rt().block_on(async { 2 + 2 }), 4);
    }
}
