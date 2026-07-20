//! Gate navigation actions and reactive effects.

use leptos::prelude::*;

use crate::checker::OwnershipChecker;
use crate::runtime::{GateRuntime, WalletGateHandle};
use crate::state::{GateIntent, NftGateState, OwnershipStatus};

/// Refresh ownership for `pubkey` using `checker`.
pub fn refresh_wallet_ownership<R>(
    gate: NftGateState<R>,
    checker: OwnershipChecker,
    pubkey: String,
) where
    R: Clone + Copy + PartialEq + Eq + Send + Sync + 'static,
{
    gate.set_ownership.set(OwnershipStatus::Checking);
    leptos::task::spawn_local(async move {
        match checker.check(pubkey).await {
            Ok(true) => gate.set_ownership.set(OwnershipStatus::Owns),
            Ok(false) | Err(_) => {
                // Fail closed: treat errors as missing to avoid accidental unlocks.
                gate.set_ownership.set(OwnershipStatus::Missing);
            }
        }
    });
}

/// Request navigation to a gated route.
///
/// - Not connected → request connect, stash [`GateIntent::Enter`]
/// - Checking → stash pending
/// - Owns → `navigate`
/// - Missing / disconnected → `on_access_denied`
pub fn request_gated_route<R>(
    gate: NftGateState<R>,
    wallet: WalletGateHandle,
    navigate: Callback<R>,
    on_access_denied: Callback<()>,
    route: R,
) where
    R: Clone + Copy + PartialEq + Eq + Send + Sync + 'static,
{
    if wallet.address.get_untracked().is_none() {
        gate.set_pending.set(Some(GateIntent::Enter(route)));
        wallet.request_connect.run(());
        return;
    }

    match gate.ownership.get_untracked() {
        OwnershipStatus::Checking => {
            gate.set_pending.set(Some(GateIntent::Enter(route)));
        }
        OwnershipStatus::Owns => {
            gate.clear_pending();
            navigate.run(route);
        }
        OwnershipStatus::Missing | OwnershipStatus::Disconnected => {
            gate.clear_pending();
            on_access_denied.run(());
        }
    }
}

/// Request the app fallback flow (e.g. mint page), connecting first if needed.
pub fn request_fallback<R>(
    gate: NftGateState<R>,
    wallet: WalletGateHandle,
    on_fallback: Callback<()>,
) where
    R: Clone + Copy + PartialEq + Eq + Send + Sync + 'static,
{
    if wallet.address.get_untracked().is_none() {
        gate.set_pending.set(Some(GateIntent::Fallback));
        wallet.request_connect.run(());
        return;
    }
    gate.clear_pending();
    on_fallback.run(());
}

/// Install effects: address → ownership refresh; pending → navigate / deny / fallback.
pub fn install_gate_effects<R>(runtime: GateRuntime<R>)
where
    R: Clone + Copy + PartialEq + Eq + Send + Sync + 'static,
{
    let gate = runtime.gate;
    let wallet = runtime.wallet;
    let navigate = runtime.navigate;
    let on_access_denied = runtime.on_access_denied;
    let on_fallback = runtime.on_fallback;
    let checker = runtime.checker;

    Effect::new(move |_| {
        let address = wallet.address.get();
        match address {
            None => {
                gate.set_ownership.set(OwnershipStatus::Disconnected);
            }
            Some(pubkey) => {
                refresh_wallet_ownership(gate, checker.clone(), pubkey);
            }
        }
    });

    Effect::new(move |_| {
        let Some(intent) = gate.pending.get() else {
            return;
        };
        if wallet.address.get().is_none() {
            return;
        }
        match intent {
            GateIntent::Fallback => {
                gate.clear_pending();
                on_fallback.run(());
            }
            GateIntent::Enter(route) => match gate.ownership.get() {
                OwnershipStatus::Checking => {}
                OwnershipStatus::Owns => {
                    gate.clear_pending();
                    navigate.run(route);
                }
                OwnershipStatus::Missing | OwnershipStatus::Disconnected => {
                    gate.clear_pending();
                    on_access_denied.run(());
                }
            },
        }
    });
}
