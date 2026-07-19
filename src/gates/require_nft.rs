//! 页面守卫与受保护导航按钮。

use leptos::prelude::*;

use crate::app_state::{NavigateToPage, UiState, WalletState};
use crate::gates::action::{open_mint_in_new_tab, request_nft_gated_page};
use crate::gates::state::{NftGateState, OwnershipStatus};
use crate::pages::AppPage;
use crate::utils::i18n::tr;

/// 页面级守卫：仅在持有 NFT 时渲染子页面；否则引导连接 / 铸造。
#[component]
pub fn RequireNftPage(children: ChildrenFn) -> impl IntoView {
    let gate = expect_context::<NftGateState>();
    let wallet = expect_context::<WalletState>();
    let navigate = expect_context::<NavigateToPage>();
    let lang = expect_context::<UiState>().ui_language;
    let (mint_opened, set_mint_opened) = signal(false);

    // 直达受保护 URL 时：未连接则拉起连接；未持有则新标签打开 Mint（每会话一次）。
    Effect::new(move |_| {
        let ownership = gate.ownership.get();
        let connected = wallet.address.get().is_some();
        if !connected {
            if gate.pending.get_untracked().is_none() {
                request_nft_gated_page(
                    gate,
                    wallet,
                    navigate,
                    AppPage::LocalLexiconEditor,
                );
            }
            return;
        }
        if ownership == OwnershipStatus::Missing && !mint_opened.get_untracked() {
            set_mint_opened.set(true);
            open_mint_in_new_tab();
        }
    });

    view! {
        {move || {
            match gate.ownership.get() {
                OwnershipStatus::Owns => children().into_any(),
                OwnershipStatus::Checking => view! {
                    <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
                        <p class="text-sm text-slate-400">
                            {move || tr(lang.get(), "正在校验 NFT 持仓…", "Checking NFT ownership…")}
                        </p>
                    </main>
                }
                .into_any(),
                OwnershipStatus::Disconnected => view! {
                    <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
                        <div class="max-w-md text-center space-y-3">
                            <p class="text-sm text-slate-300">
                                {move || tr(
                                    lang.get(),
                                    "本地词库修改器需要连接钱包并持有本系列 NFT。",
                                    "The Local Lexicon Editor requires a connected wallet that holds this collection NFT.",
                                )}
                            </p>
                            <p class="text-xs text-slate-500">
                                {move || tr(
                                    lang.get(),
                                    "正在请求连接钱包…",
                                    "Requesting wallet connection…",
                                )}
                            </p>
                        </div>
                    </main>
                }
                .into_any(),
                OwnershipStatus::Missing => view! {
                    <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
                        <div class="max-w-md text-center space-y-4">
                            <p class="text-sm text-slate-300">
                                {move || tr(
                                    lang.get(),
                                    "当前钱包未持有本系列 NFT。已在新标签页打开铸造页。",
                                    "This wallet does not hold an NFT from this collection. Mint opened in a new tab.",
                                )}
                            </p>
                            <button
                                type="button"
                                class="text-sm text-emerald-400 underline underline-offset-4 hover:text-emerald-300"
                                on:click=move |_| open_mint_in_new_tab()
                            >
                                {move || tr(lang.get(), "再次打开铸造页", "Open mint page again")}
                            </button>
                            <button
                                type="button"
                                class="block mx-auto text-sm text-sky-400 underline underline-offset-4 hover:text-sky-300"
                                on:click=move |_| {
                                    let Some(pubkey) = wallet.address.get_untracked() else {
                                        return;
                                    };
                                    gate.set_ownership.set(OwnershipStatus::Checking);
                                    leptos::task::spawn_local(async move {
                                        match crate::utils::nft_ownership::wallet_owns_collection_nft(
                                            &pubkey,
                                        )
                                        .await
                                        {
                                            Ok(true) => {
                                                gate.set_ownership.set(OwnershipStatus::Owns)
                                            }
                                            Ok(false) | Err(_) => {
                                                gate.set_ownership.set(OwnershipStatus::Missing)
                                            }
                                        }
                                    });
                                }
                            >
                                {move || tr(lang.get(), "我已铸造，重新校验", "I minted — recheck")}
                            </button>
                            <button
                                type="button"
                                class="block mx-auto text-sm text-slate-400 underline underline-offset-4 hover:text-slate-300"
                                on:click=move |_| navigate.set(AppPage::PracticeModeSelect)
                            >
                                {move || tr(lang.get(), "返回练习模式", "Back to practice modes")}
                            </button>
                        </div>
                    </main>
                }
                .into_any(),
            }
        }}
    }
}

/// 组件级守卫：点击后走统一 NFT 门禁，通过则导航到 `target`。
#[component]
pub fn NftGatedNavigateButton(
    target: AppPage,
    #[prop(into)] class: String,
    children: Children,
) -> impl IntoView {
    let gate = expect_context::<NftGateState>();
    let wallet = expect_context::<WalletState>();
    let navigate = expect_context::<NavigateToPage>();

    let on_click = move |_| {
        request_nft_gated_page(gate, wallet, navigate, target);
    };

    view! {
        <button type="button" class=class on:click=on_click>
            {children()}
        </button>
    }
}
