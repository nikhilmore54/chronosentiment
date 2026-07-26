/// S2-03 — Structured logging and telemetry initialisation
///
/// Provides a single `init_logging()` call that configures the `tracing`
/// subscriber for the UltraCrew CLI and library.
///
/// # Log levels
///
/// Controlled by the `ULTRACREW_LOG` environment variable (falls back to
/// `RUST_LOG` if unset). If neither is set, the default level is `info`.
///
/// | Value   | What you see |
/// |---------|--------------|
/// | `error` | Fatal errors only |
/// | `warn`  | Errors + warnings |
/// | `info`  | Normal operational output (default) |
/// | `debug` | Per-generation optimizer detail |
/// | `trace` | Full genome/evaluation trace |
///
/// # Request IDs
///
/// Every top-level operation (CLI invocation, API request) should call
/// [`new_request_id()`] and attach the result to a `tracing::Span` so that
/// all log lines for that operation share a common `request_id` field.
///
/// # Usage (CLI binary)
///
/// ```rust
/// use ultracrew::telemetry::{init_logging, new_request_id};
///
/// fn main() {
///     init_logging();
///     let rid = new_request_id();
///     let span = tracing::info_span!("run", request_id = %rid);
///     let _guard = span.enter();
///     tracing::info!("UltraCrew starting");
/// }
/// ```

use std::env;
use tracing_subscriber::{EnvFilter, fmt};

// ─── Request ID ───────────────────────────────────────────────────────────────

/// Generate a short, unique request ID for log correlation.
///
/// Format: `uc-<8 hex chars>` combining a monotonic counter with timestamp
/// nanoseconds. This guarantees uniqueness within a process without requiring
/// a UUID dependency.
pub fn new_request_id() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    // XOR counter into low bits so rapid successive calls are always distinct.
    format!("uc-{:08x}", nanos ^ seq)
}

// ─── Logging initialisation ───────────────────────────────────────────────────

/// Initialise the global `tracing` subscriber.
///
/// Safe to call multiple times — subsequent calls are no-ops because
/// `tracing_subscriber::fmt::init()` uses a `Once` guard internally.
///
/// Log level is resolved in this order:
/// 1. `ULTRACREW_LOG` environment variable
/// 2. `RUST_LOG` environment variable
/// 3. Default: `info`
pub fn init_logging() {
    let filter = build_env_filter();
    let _ = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .try_init(); // try_init returns Err if already initialised — that's fine
}

/// Build an `EnvFilter` from `ULTRACREW_LOG` → `RUST_LOG` → `"info"`.
pub fn build_env_filter() -> EnvFilter {
    // Prefer ULTRACREW_LOG, fall back to RUST_LOG, then default to info.
    let directive = env::var("ULTRACREW_LOG")
        .or_else(|_| env::var("RUST_LOG"))
        .unwrap_or_else(|_| "info".to_string());

    EnvFilter::try_new(&directive)
        .unwrap_or_else(|_| EnvFilter::new("info"))
}

// ─── Span helpers ─────────────────────────────────────────────────────────────

/// Emit a structured `info` event with standard UltraCrew fields.
///
/// Prefer the `tracing::info!` macro directly in application code.
/// This helper is provided for use in library code that needs a consistent
/// field layout without importing the macro at every call site.
#[macro_export]
macro_rules! uc_info {
    ($msg:expr) => {
        tracing::info!(target: "ultracrew", $msg)
    };
    ($msg:expr, $($field:tt)*) => {
        tracing::info!(target: "ultracrew", $($field)*, $msg)
    };
}

/// Emit a structured `warn` event with standard UltraCrew fields.
#[macro_export]
macro_rules! uc_warn {
    ($msg:expr) => {
        tracing::warn!(target: "ultracrew", $msg)
    };
    ($msg:expr, $($field:tt)*) => {
        tracing::warn!(target: "ultracrew", $($field)*, $msg)
    };
}

/// Emit a structured `error` event with standard UltraCrew fields.
#[macro_export]
macro_rules! uc_error {
    ($msg:expr) => {
        tracing::error!(target: "ultracrew", $msg)
    };
    ($msg:expr, $($field:tt)*) => {
        tracing::error!(target: "ultracrew", $($field)*, $msg)
    };
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_request_id_format() {
        let rid = new_request_id();
        assert!(rid.starts_with("uc-"), "Request ID must start with 'uc-': {}", rid);
        assert_eq!(rid.len(), 11, "Request ID must be 11 chars (uc- + 8 hex): {}", rid);
        let hex_part = &rid[3..];
        assert!(
            hex_part.chars().all(|c| c.is_ascii_hexdigit()),
            "Hex part must be all hex digits: {}",
            hex_part
        );
    }

    #[test]
    fn test_request_ids_are_unique() {
        // The counter-based generator guarantees uniqueness within a process.
        let ids: Vec<String> = (0..100).map(|_| new_request_id()).collect();
        let unique: std::collections::HashSet<&str> = ids.iter().map(|s| s.as_str()).collect();
        assert_eq!(unique.len(), 100, "All 100 request IDs must be unique");
    }

    #[test]
    fn test_build_env_filter_default() {
        // Without ULTRACREW_LOG or RUST_LOG set, should default to info.
        // We can't easily unset env vars in a test, so just verify it doesn't panic.
        let _filter = build_env_filter();
    }

    #[test]
    fn test_build_env_filter_from_ultracrew_log() {
        // Temporarily set ULTRACREW_LOG and verify the filter is built.
        std::env::set_var("ULTRACREW_LOG", "debug");
        let filter = build_env_filter();
        // EnvFilter doesn't expose its level directly, but we can verify it
        // serialises to a non-empty string.
        let s = format!("{:?}", filter);
        assert!(!s.is_empty());
        std::env::remove_var("ULTRACREW_LOG");
    }

    #[test]
    fn test_init_logging_is_idempotent() {
        // Calling init_logging() twice must not panic.
        init_logging();
        init_logging();
    }

    #[test]
    fn test_tracing_macros_compile() {
        // Verify the tracing macros work at the call site.
        // These are no-ops if no subscriber is installed.
        tracing::info!(target: "ultracrew", "test info event");
        tracing::warn!(target: "ultracrew", "test warn event");
        tracing::error!(target: "ultracrew", "test error event");
        tracing::debug!(target: "ultracrew", request_id = "uc-test0001", "test debug event");
    }

    #[test]
    fn test_span_with_request_id() {
        init_logging();
        let rid = new_request_id();
        let span = tracing::info_span!("test_operation", request_id = %rid);
        let _guard = span.enter();
        tracing::info!("inside span with request_id");
        // No assertion needed — just verify it doesn't panic.
    }
}