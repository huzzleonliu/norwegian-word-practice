//! Browser-side Solana helpers for `wasm32` frontends.
//!
//! This crate is UI-framework agnostic. It covers:
//! - `window.solana` wallet connect / disconnect
//! - calling page-injected JS helpers (ownership / mint bridges)
//! - reading Candy Machine `items_redeemed` over JSON-RPC
//!
//! It does **not** hard-code collection IDs, mint scripts, or product routes.
//! Pass those in from your app (or from a thin adapter module).

#![deny(missing_docs)]

pub mod candy_machine;
pub mod error;
pub mod js_bridge;
pub mod mint;
pub mod ownership;
pub mod wallet;

pub use candy_machine::{fetch_items_redeemed, CandyMachineQuery};
pub use error::BrowserSolanaError;
pub use js_bridge::{call_window_async, js_error_message};
pub use mint::{mint_via_window_fn, MintResult};
pub use ownership::owns_via_window_fn;
pub use wallet::{connect_wallet, disconnect_wallet, shorten_address};
