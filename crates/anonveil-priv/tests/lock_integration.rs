//! Real-filesystem tests for `anonveil_priv::lock`. Requires write access
//! to `/run` (root), same as `nftables_integration.rs` — run via
//! `cargo test -p anonveil-priv --features integration -- --test-threads=1`.
//! `--test-threads=1` matters here for the same reason it does for the
//! nftables tests: these all contend for the same real, global lock file,
//! so running them in parallel would make one test's "someone else holds
//! it" assertion depend on unrelated tests' timing instead of its own.
#![cfg(feature = "integration")]

use anonveil_priv::lock::StateLock;
use anonveil_priv::PrivError;

#[test]
fn second_concurrent_acquire_fails_fast_instead_of_blocking() {
    let _first = StateLock::acquire().expect("first acquire should succeed");
    let second = StateLock::acquire();
    assert!(
        matches!(second, Err(PrivError::AnotherOperationInProgress)),
        "expected AnotherOperationInProgress while the first guard is still held"
    );
}

#[test]
fn lock_is_released_as_soon_as_the_guard_drops() {
    {
        let _guard = StateLock::acquire().expect("first acquire should succeed");
    } // guard dropped here — the flock must be released with it
    let _second =
        StateLock::acquire().expect("should be able to acquire again once the first guard drops");
}
