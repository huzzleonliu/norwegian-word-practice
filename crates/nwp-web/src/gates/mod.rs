//! NFT 访问守卫：对本应用路由 / 文案的薄封装，核心逻辑在 `leptos-solana-gate`。

mod require_nft;

pub use leptos_solana_gate::{
    install_gate_effects, ownership_checker_from_window_fn, provide_gate_runtime, recheck_ownership,
    request_fallback, GateIntent, GateRuntime, NftGateState, OwnershipStatus, WalletGateHandle,
};
pub use require_nft::{NftGatedNavigateButton, RequireNftPage};

use crate::pages::AppPage;

/// 本应用的门禁状态别名。
pub type AppNftGateState = NftGateState<AppPage>;
/// 本应用的门禁意图别名。
pub type AppGateIntent = GateIntent<AppPage>;
/// 本应用的 gate runtime 别名。
pub type AppGateRuntime = GateRuntime<AppPage>;

/// 在新标签页打开 Mint 页。
pub fn open_mint_in_new_tab() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let path = AppPage::MintNft.path();
    let url = window
        .location()
        .origin()
        .ok()
        .map(|origin| format!("{origin}{path}"))
        .unwrap_or_else(|| path.to_string());
    let _ = window.open_with_url_and_target(&url, "_blank");
}

/// 请求打开 Mint 页（同标签；需先连接钱包）。
pub fn request_mint_page(runtime: &AppGateRuntime) {
    request_fallback(runtime.gate, runtime.wallet, runtime.on_fallback);
}
