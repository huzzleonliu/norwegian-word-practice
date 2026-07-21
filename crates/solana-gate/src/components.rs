use leptos::prelude::*;

use crate::action::request_nft_gated_path;
use crate::state::{
    CryptoGateMode, GateRuntime, GateWalletState, NftGateState, OwnershipStatus,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateLocale {
    Zh,
    En,
}

fn t(locale: GateLocale, zh: &'static str, en: &'static str) -> &'static str {
    match locale {
        GateLocale::Zh => zh,
        GateLocale::En => en,
    }
}

/// 页面级守卫：仅在通过门禁时渲染子页面。
#[component]
pub fn RequireNftPage(
    #[prop(into)] target_path: String,
    #[prop(optional, into)] locale: Option<Signal<GateLocale>>,
    children: ChildrenFn,
) -> impl IntoView {
    let gate = expect_context::<NftGateState>();
    let wallet = expect_context::<GateWalletState>();
    let runtime = expect_context::<GateRuntime>();
    let locale = locale.unwrap_or_else(|| Signal::derive(|| GateLocale::Zh));
    let (mint_opened, set_mint_opened) = signal(false);

    let target_path_for_effect = target_path.clone();
    let runtime_for_effect = runtime.clone();
    Effect::new(move |_| {
        if gate.mode.get() == CryptoGateMode::NoBlockchain {
            return;
        }

        let ownership = gate.ownership.get();
        let connected = wallet.address.get().is_some();
        if !connected {
            if gate.pending.get_untracked().is_none() {
                request_nft_gated_path(
                    gate,
                    wallet,
                    runtime_for_effect.clone(),
                    target_path_for_effect.clone(),
                );
            }
            return;
        }

        if ownership == OwnershipStatus::Missing && !mint_opened.get_untracked()
        {
            set_mint_opened.set(true);
            runtime_for_effect.open_mint_in_new_tab.run(());
        }
    });

    view! {
        {move || {
            let current_locale = locale.get();
            if gate.mode.get() == CryptoGateMode::NoBlockchain {
                return children().into_any();
            }

            match gate.ownership.get() {
                OwnershipStatus::Owns => children().into_any(),
                OwnershipStatus::Checking => view! {
                    <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
                        <p class="text-sm text-slate-400">
                            {t(current_locale, "正在校验 NFT 持仓…", "Checking NFT ownership…")}
                        </p>
                    </main>
                }
                .into_any(),
                OwnershipStatus::Disconnected => view! {
                    <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
                        <div class="max-w-md text-center space-y-3">
                            <p class="text-sm text-slate-300">
                                {t(
                                    current_locale,
                                    "本页面需要连接钱包并持有指定 NFT。",
                                    "This page requires a connected wallet holding the required NFT.",
                                )}
                            </p>
                            <p class="text-xs text-slate-500">
                                {t(current_locale, "正在请求连接钱包…", "Requesting wallet connection…")}
                            </p>
                        </div>
                    </main>
                }
                .into_any(),
                OwnershipStatus::Missing => {
                    let fallback_path = runtime.fallback_path.clone();
                    let runtime_for_open = runtime.clone();
                    let runtime_for_refresh = runtime.clone();
                    let runtime_for_back = runtime.clone();
                    view! {
                        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
                            <div class="max-w-md text-center space-y-4">
                                <p class="text-sm text-slate-300">
                                    {t(
                                        current_locale,
                                        "当前钱包未持有所需 NFT，已在新标签页打开铸造页。",
                                        "This wallet does not hold the required NFT. Mint page was opened in a new tab.",
                                    )}
                                </p>
                                <button
                                    type="button"
                                    class="text-sm text-emerald-400 underline underline-offset-4 hover:text-emerald-300"
                                    on:click=move |_| runtime_for_open.open_mint_in_new_tab.run(())
                                >
                                    {t(current_locale, "再次打开铸造页", "Open mint page again")}
                                </button>
                                <button
                                    type="button"
                                    class="block mx-auto text-sm text-sky-400 underline underline-offset-4 hover:text-sky-300"
                                    on:click=move |_| {
                                        let Some(pubkey) = wallet.address.get_untracked() else {
                                            return;
                                        };
                                        runtime_for_refresh.request_ownership_refresh.run(pubkey);
                                    }
                                >
                                    {t(current_locale, "我已铸造，重新校验", "I minted — recheck")}
                                </button>
                                <button
                                    type="button"
                                    class="block mx-auto text-sm text-slate-400 underline underline-offset-4 hover:text-slate-300"
                                    on:click=move |_| runtime_for_back.navigate_to_path.run(fallback_path.clone())
                                >
                                    {t(current_locale, "返回练习模式", "Back to practice modes")}
                                </button>
                            </div>
                        </main>
                    }
                    .into_any()
                }
            }
        }}
    }
}

/// 组件级守卫：点击后走统一门禁，通过则导航到 `target_path`。
#[component]
pub fn NftGatedNavigateButton(
    #[prop(into)] target_path: String,
    #[prop(into)] class: String,
    children: Children,
) -> impl IntoView {
    let gate = expect_context::<NftGateState>();
    let wallet = expect_context::<GateWalletState>();
    let runtime = expect_context::<GateRuntime>();

    let on_click = move |_| {
        request_nft_gated_path(gate, wallet, runtime.clone(), target_path.clone());
    };

    view! {
        <button type="button" class=class on:click=on_click>
            {children()}
        </button>
    }
}
