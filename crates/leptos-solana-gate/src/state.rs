//! Gate status and pending navigation intents.

use leptos::prelude::*;

/// Wallet ownership relative to the configured checker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnershipStatus {
    /// No wallet connected.
    Disconnected,
    /// Ownership query in flight.
    Checking,
    /// Checker returned `true`.
    Owns,
    /// Connected but checker returned `false` (or failed conservatively).
    Missing,
}

/// Intent to resume after connect / ownership settles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateIntent<R>
where
    R: Clone + Copy + PartialEq + Eq + Send + Sync,
{
    /// Navigate to a gated route once ownership is confirmed.
    Enter(R),
    /// Run the app-defined fallback (e.g. open mint page in the same tab).
    Fallback,
}

/// Shared NFT / ownership gate state (provide via context, usually inside [`crate::GateRuntime`]).
#[derive(Clone, Copy)]
pub struct NftGateState<R>
where
    R: Clone + Copy + PartialEq + Eq + Send + Sync + 'static,
{
    /// Current ownership status.
    pub ownership: ReadSignal<OwnershipStatus>,
    /// Setter for ownership status.
    pub set_ownership: WriteSignal<OwnershipStatus>,
    /// Pending intent after connect.
    pub pending: ReadSignal<Option<GateIntent<R>>>,
    /// Setter for pending intent.
    pub set_pending: WriteSignal<Option<GateIntent<R>>>,
}

impl<R> NftGateState<R>
where
    R: Clone + Copy + PartialEq + Eq + Send + Sync + 'static,
{
    /// Clear any pending intent.
    pub fn clear_pending(self) {
        self.set_pending.set(None);
    }
}
