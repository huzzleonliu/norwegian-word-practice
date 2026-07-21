use leptos::prelude::*;

use crate::crypto_module::constants::{
    GATE_FALLBACK_ROUTE_PATH, MINT_ROUTE_PATH,
};

/// 构建应用使用的门禁运行时（持仓查询由组件库提供）。
pub fn build_app_gate_runtime(
    gate: solana_gate::NftGateState,
    navigate_to_path: Callback<String>,
    open_mint_in_new_tab: Callback<()>,
) -> solana_gate::GateRuntime {
    let request_ownership_refresh = Callback::new(move |pubkey: String| {
        gate.set_ownership.set(solana_gate::OwnershipStatus::Checking);
        let gate_for_task = gate;
        leptos::task::spawn_local(async move {
            match solana_leptos_component::wallet_owns_collection_nft(&pubkey).await
            {
                Ok(true) => {
                    gate_for_task.set_ownership.set(solana_gate::OwnershipStatus::Owns);
                }
                Ok(false) | Err(_) => {
                    // 查询失败时保守视为未持有，避免误放行。
                    gate_for_task
                        .set_ownership
                        .set(solana_gate::OwnershipStatus::Missing);
                }
            }
        });
    });

    solana_gate::build_gate_runtime(
        navigate_to_path,
        open_mint_in_new_tab,
        request_ownership_refresh,
        MINT_ROUTE_PATH.to_string(),
        GATE_FALLBACK_ROUTE_PATH.to_string(),
    )
}

pub fn open_path_in_new_tab(path: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let url = window
        .location()
        .origin()
        .ok()
        .map(|origin| format!("{origin}{path}"))
        .unwrap_or_else(|| path.to_string());
    let _ = window.open_with_url_and_target(&url, "_blank");
}
