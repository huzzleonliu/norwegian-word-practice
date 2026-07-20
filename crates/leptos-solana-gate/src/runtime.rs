//! Runtime context: wallet handle + navigation callbacks + checker.

use leptos::prelude::*;

use crate::checker::OwnershipChecker;
use crate::state::NftGateState;

/// Minimal wallet surface the gate needs (address + “please connect”).
#[derive(Clone, Copy)]
pub struct WalletGateHandle {
    /// Connected pubkey, if any.
    pub address: ReadSignal<Option<String>>,
    /// Trigger the host app’s connect flow (e.g. bump a nonce watched by a wallet button).
    pub request_connect: Callback<()>,
}

/// Bundled gate dependencies for components and effects.
#[derive(Clone)]
pub struct GateRuntime<R>
where
    R: Clone + Copy + PartialEq + Eq + Send + Sync + 'static,
{
    /// Gate signals.
    pub gate: NftGateState<R>,
    /// Wallet handle.
    pub wallet: WalletGateHandle,
    /// Navigate to a route value `R` (same tab).
    pub navigate: Callback<R>,
    /// Called when the user needs access but does not own (e.g. open mint in a new tab).
    pub on_access_denied: Callback<()>,
    /// Called for [`crate::GateIntent::Fallback`] after connect (e.g. open mint same tab).
    pub on_fallback: Callback<()>,
    /// Ownership predicate.
    pub checker: OwnershipChecker,
}

/// Provide [`GateRuntime`] (and nested [`NftGateState`] / [`WalletGateHandle`] for convenience).
pub fn provide_gate_runtime<R>(runtime: GateRuntime<R>)
where
    R: Clone + Copy + PartialEq + Eq + Send + Sync + 'static,
{
    provide_context(runtime.gate);
    provide_context(runtime.wallet);
    provide_context(runtime.checker.clone());
    provide_context(runtime);
}
