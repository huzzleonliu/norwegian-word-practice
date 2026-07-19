//! NFT 访问守卫：全局资格状态、页面守卫与组件守卫。

mod action;
mod require_nft;
mod state;

pub use action::{install_gate_effects, request_mint_page};
pub use require_nft::{NftGatedNavigateButton, RequireNftPage};
pub use state::{GateIntent, NftGateState, OwnershipStatus};
