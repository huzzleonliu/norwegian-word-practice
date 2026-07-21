//! Solana Leptos Components：
//! - 钱包连接与 Mint 页面组件
//! - NFT 持仓查询与链上辅助函数

mod candy_machine;
mod components;
mod locale;
mod mint_page;
mod nft_collection;
mod oaoa_mint;
mod ownership;
mod state;
mod wallet;
mod wallet_button;

pub use candy_machine::fetch_items_redeemed;
pub use components::{MintNftPage, WalletConnectButton};
pub use locale::GateLocale;
pub use nft_collection::{
    CANDY_GUARD_ID, CANDY_MACHINE_ID, COLLECTION_IMAGE_URL, COLLECTION_METADATA_URL,
    COLLECTION_MINT_ID, ITEMS_REDEEMED_OFFSET, NFT_MINT_PRICE_SOL, NFT_TOTAL_SUPPLY,
    SOLANA_RPC_URL, TREASURY_WALLET,
};
pub use oaoa_mint::{mint_nft_with_wallet, MintResult};
pub use ownership::wallet_owns_collection_nft;
pub use state::WalletState;
pub use wallet::{connect_wallet, disconnect_wallet, shorten_address};
