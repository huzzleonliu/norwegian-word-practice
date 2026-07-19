//! OAOA Painting NFT 系列配置。

/// 系列总量。
pub const NFT_TOTAL_SUPPLY: u64 = 20;

/// 一级铸造价格（SOL）。
pub const NFT_MINT_PRICE_SOL: f64 = 0.1;

/// Devnet Candy Machine 地址。
pub const CANDY_MACHINE_ID: &str = "C2sf5YPcnrKabkXqs1615iF6Y2udrimMHis6rYnUQVN7";

/// Devnet Candy Guard 地址。
#[allow(dead_code)]
pub const CANDY_GUARD_ID: &str = "EoDwnkGMgZcoikFS6dLuaDB552m7yx3u35eJvXoSPX6P";

/// Collection mint 地址。
#[allow(dead_code)]
pub const COLLECTION_MINT_ID: &str = "9qcraUcqpzRuXKSuPBuGvc3xn5wb184MYzWadDUWfgay";

/// 收款钱包（solPayment destination）。
#[allow(dead_code)]
pub const TREASURY_WALLET: &str = "YGR2KWyPtqGTcWZ2QiSMfdq5ProDGVvhwFe6KqrYZJx";

/// Devnet RPC。
pub const SOLANA_RPC_URL: &str = "https://api.devnet.solana.com";

/// Candy Machine 账户中 `items_redeemed` 字段偏移（mpl-candy-machine-core）。
pub const ITEMS_REDEEMED_OFFSET: usize = 112;

/// 展示页使用的 Collection 封面图（Trunk `copy-file` → `/nft/collection.png`）。
pub const COLLECTION_IMAGE_URL: &str = "/nft/collection.png";

/// Collection 元数据（可选，用于标题）。
pub const COLLECTION_METADATA_URL: &str = "/nft/collection.json";
