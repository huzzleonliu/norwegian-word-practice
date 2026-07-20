//! Leptos guards for Solana NFT (or arbitrary ownership) gated UI.
//!
//! This crate owns the **state machine + components**. Ownership checks come from
//! an injected [`OwnershipChecker`] (typically built with [`browser_solana::owns_via_window_fn`]).
//!
//! Route types are generic (`R`) so apps can use path enums without this crate
//! depending on product routes or i18n.

#![deny(missing_docs)]

mod action;
mod checker;
mod components;
mod runtime;
mod state;

pub use action::{
    install_gate_effects, refresh_wallet_ownership, request_fallback, request_gated_route,
};
pub use checker::{ownership_checker_from_window_fn, OwnershipChecker};
pub use components::{recheck_ownership, GateViewFn, GatedNavigateButton, RequireNftPage};
pub use runtime::{provide_gate_runtime, GateRuntime, WalletGateHandle};
pub use state::{GateIntent, NftGateState, OwnershipStatus};
