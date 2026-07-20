//! 统一的门禁导航动作：连钱包 / 开 Mint / 进入受保护页。

use leptos::prelude::*;

use crate::app_state::{NavigateToPage, WalletState};
use crate::gates::state::{GateIntent, NftGateState, OwnershipStatus};
use crate::pages::AppPage;

/// 异步刷新当前钱包的持仓状态。
pub fn refresh_wallet_ownership(gate: NftGateState, pubkey: String) {
    gate.set_ownership.set(OwnershipStatus::Checking);
    leptos::task::spawn_local(async move {
        match crate::utils::nft_ownership::wallet_owns_collection_nft(&pubkey).await {
            Ok(true) => gate.set_ownership.set(OwnershipStatus::Owns),
            Ok(false) | Err(_) => {
                // 查询失败时保守视为未持有，避免误放行。
                gate.set_ownership.set(OwnershipStatus::Missing);
            }
        }
    });
}

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

/// 请求进入需持有 NFT 的页面。
///
/// - 未连接 → 触发连接，连接成功后继续本流程  
/// - 查询中 → 记入 pending，等结果  
/// - 已持有 → 同标签进入 `page`  
/// - 未持有 → 新标签打开 Mint
pub fn request_nft_gated_page(
    gate: NftGateState,
    wallet: WalletState,
    navigate: NavigateToPage,
    page: AppPage,
) {
    if wallet.address.get_untracked().is_none() {
        gate.set_pending.set(Some(GateIntent::EnterPage(page)));
        wallet.request_connect();
        return;
    }

    match gate.ownership.get_untracked() {
        OwnershipStatus::Checking => {
            gate.set_pending.set(Some(GateIntent::EnterPage(page)));
        }
        OwnershipStatus::Owns => {
            gate.clear_pending();
            navigate.set(page);
        }
        OwnershipStatus::Missing | OwnershipStatus::Disconnected => {
            gate.clear_pending();
            open_mint_in_new_tab();
        }
    }
}

/// 请求打开 Mint 页（同标签；不要求已持有 NFT）。
pub fn request_mint_page(gate: NftGateState, wallet: WalletState, navigate: NavigateToPage) {
    if wallet.address.get_untracked().is_none() {
        gate.set_pending.set(Some(GateIntent::OpenMint));
        wallet.request_connect();
        return;
    }
    gate.clear_pending();
    navigate.set(AppPage::MintNft);
}

/// 在 AppChrome 中挂载：地址变化时刷新持仓；pending 就绪时继续导航。
pub fn install_gate_effects(
    gate: NftGateState,
    wallet: WalletState,
    navigate: NavigateToPage,
) {
    // 钱包地址变化 → 刷新持仓。
    Effect::new(move |_| {
        let address = wallet.address.get();
        match address {
            None => {
                gate.set_ownership.set(OwnershipStatus::Disconnected);
            }
            Some(pubkey) => {
                refresh_wallet_ownership(gate, pubkey);
            }
        }
    });

    // 连接 / 持仓就绪后消化 pending 意图。
    Effect::new(move |_| {
        let Some(intent) = gate.pending.get() else {
            return;
        };
        if wallet.address.get().is_none() {
            return;
        }
        match intent {
            GateIntent::OpenMint => {
                gate.clear_pending();
                navigate.set(AppPage::MintNft);
            }
            GateIntent::EnterPage(page) => match gate.ownership.get() {
                OwnershipStatus::Checking => {}
                OwnershipStatus::Owns => {
                    gate.clear_pending();
                    navigate.set(page);
                }
                OwnershipStatus::Missing | OwnershipStatus::Disconnected => {
                    gate.clear_pending();
                    open_mint_in_new_tab();
                }
            },
        }
    });
}
