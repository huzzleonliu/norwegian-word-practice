//! Solana 门禁库：
//! - 门禁状态/动作
//! - 页面守卫与组件守卫

mod action;
mod components;
mod state;

pub use action::{
    build_gate_runtime, install_gate_effects, request_mint_page,
    request_nft_gated_path,
};
pub use components::{GateLocale, NftGatedNavigateButton, RequireNftPage};
pub use state::{
    CryptoGateMode, GateIntent, GateRuntime, GateWalletState, NftGateState,
    OwnershipStatus,
};
