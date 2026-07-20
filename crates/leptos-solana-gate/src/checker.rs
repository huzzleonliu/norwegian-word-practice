//! Pluggable ownership checker used by gate effects.

use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use browser_solana::owns_via_window_fn;
use send_wrapper::SendWrapper;

type CheckFuture = Pin<Box<dyn Future<Output = Result<bool, String>>>>;
type CheckFn = Rc<dyn Fn(String) -> CheckFuture>;

/// Async ownership predicate: `owner_pubkey -> owns?`.
///
/// Wrapped for Leptos context (`Send + Sync`) while remaining single-threaded on WASM.
#[derive(Clone)]
pub struct OwnershipChecker {
    inner: SendWrapper<CheckFn>,
}

impl OwnershipChecker {
    /// Wrap any async ownership function.
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(String) -> Fut + 'static,
        Fut: Future<Output = Result<bool, String>> + 'static,
    {
        let boxed: CheckFn = Rc::new(move |owner| Box::pin(f(owner)));
        Self {
            inner: SendWrapper::new(boxed),
        }
    }

    /// Run the checker for `owner`.
    pub async fn check(&self, owner: String) -> Result<bool, String> {
        (self.inner)(owner).await
    }
}

/// Build a checker that calls `window[global_fn](owner) -> bool`.
///
/// `global_fn` must remain valid for the lifetime of the checker (typically a `'static` str).
pub fn ownership_checker_from_window_fn(global_fn: &'static str) -> OwnershipChecker {
    OwnershipChecker::new(move |owner| async move {
        owns_via_window_fn(global_fn, &owner)
            .await
            .map_err(|err| err.to_string())
    })
}
