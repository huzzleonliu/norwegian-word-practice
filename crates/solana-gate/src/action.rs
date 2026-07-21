use leptos::prelude::*;

use crate::state::{
    CryptoGateMode, GateIntent, GateRuntime, GateWalletState, NftGateState,
    OwnershipStatus,
};

/// 组装门禁运行时（具体持仓刷新逻辑由应用层注入）。
pub fn build_gate_runtime(
    navigate_to_path: Callback<String>,
    open_mint_in_new_tab: Callback<()>,
    request_ownership_refresh: Callback<String>,
    mint_path: String,
    fallback_path: String,
) -> GateRuntime {
    GateRuntime {
        navigate_to_path,
        open_mint_in_new_tab,
        request_ownership_refresh,
        mint_path,
        fallback_path,
    }
}

/// 请求进入受保护页面（path 由业务层提供）。
pub fn request_nft_gated_path(
    gate: NftGateState,
    wallet: GateWalletState,
    runtime: GateRuntime,
    path: String,
) {
    if gate.mode.get_untracked() == CryptoGateMode::NoBlockchain {
        gate.clear_pending();
        runtime.navigate_to_path.run(path);
        return;
    }

    if wallet.address.get_untracked().is_none() {
        gate.set_pending.set(Some(GateIntent::EnterPath(path)));
        wallet.request_connect();
        return;
    }

    match gate.ownership.get_untracked() {
        OwnershipStatus::Checking => {
            gate.set_pending.set(Some(GateIntent::EnterPath(path)));
        }
        OwnershipStatus::Owns => {
            gate.clear_pending();
            runtime.navigate_to_path.run(path);
        }
        OwnershipStatus::Missing | OwnershipStatus::Disconnected => {
            gate.clear_pending();
            runtime.open_mint_in_new_tab.run(());
        }
    }
}

/// 请求打开 Mint 页（同标签）。
pub fn request_mint_page(
    gate: NftGateState,
    wallet: GateWalletState,
    runtime: GateRuntime,
) {
    if gate.mode.get_untracked() == CryptoGateMode::NoBlockchain {
        gate.clear_pending();
        runtime.navigate_to_path.run(runtime.mint_path.clone());
        return;
    }

    if wallet.address.get_untracked().is_none() {
        gate.set_pending.set(Some(GateIntent::OpenMint));
        wallet.request_connect();
        return;
    }
    gate.clear_pending();
    runtime.navigate_to_path.run(runtime.mint_path.clone());
}

/// 在宿主 App 中挂载：
/// - 地址变化时刷新持仓
/// - pending 就绪后继续导航
pub fn install_gate_effects(
    gate: NftGateState,
    wallet: GateWalletState,
    runtime: GateRuntime,
) {
    let runtime_for_address = runtime.clone();
    Effect::new(move |_| {
        if gate.mode.get() == CryptoGateMode::NoBlockchain {
            gate.set_ownership.set(OwnershipStatus::Owns);
            return;
        }

        match wallet.address.get() {
            None => gate.set_ownership.set(OwnershipStatus::Disconnected),
            Some(pubkey) => {
                gate.set_ownership.set(OwnershipStatus::Checking);
                runtime_for_address.request_ownership_refresh.run(pubkey);
            }
        }
    });

    let runtime_for_pending = runtime.clone();
    Effect::new(move |_| {
        let Some(intent) = gate.pending.get() else {
            return;
        };

        if gate.mode.get() == CryptoGateMode::NoBlockchain {
            gate.clear_pending();
            match intent {
                GateIntent::OpenMint => {
                    runtime_for_pending
                        .navigate_to_path
                        .run(runtime_for_pending.mint_path.clone());
                }
                GateIntent::EnterPath(path) => {
                    runtime_for_pending.navigate_to_path.run(path);
                }
            }
            return;
        }

        if wallet.address.get().is_none() {
            return;
        }

        match intent {
            GateIntent::OpenMint => {
                gate.clear_pending();
                runtime_for_pending
                    .navigate_to_path
                    .run(runtime_for_pending.mint_path.clone());
            }
            GateIntent::EnterPath(path) => match gate.ownership.get() {
                OwnershipStatus::Checking => {}
                OwnershipStatus::Owns => {
                    gate.clear_pending();
                    runtime_for_pending.navigate_to_path.run(path);
                }
                OwnershipStatus::Missing | OwnershipStatus::Disconnected => {
                    gate.clear_pending();
                    runtime_for_pending.open_mint_in_new_tab.run(());
                }
            },
        }
    });
}
