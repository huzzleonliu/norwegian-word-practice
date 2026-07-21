use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;

use crate::candy_machine::fetch_items_redeemed;
use crate::locale::GateLocale;
use crate::nft_collection::{
    CANDY_MACHINE_ID, COLLECTION_IMAGE_URL, COLLECTION_METADATA_URL, NFT_MINT_PRICE_SOL,
    NFT_TOTAL_SUPPLY,
};
use crate::oaoa_mint::mint_nft_with_wallet;
use crate::state::WalletState;

#[derive(Clone, Deserialize)]
struct CollectionMeta {
    name: Option<String>,
    description: Option<String>,
}

fn tr_locale(locale: GateLocale, zh: &str, en: &str) -> String {
    match locale {
        GateLocale::Zh => zh.to_string(),
        GateLocale::En => en.to_string(),
    }
}

/// 可复用 Mint 页面组件（依赖 `WalletState` context）。
#[component]
pub fn MintNftPage(
    #[prop(optional, into)] locale: Option<Signal<GateLocale>>,
    #[prop(optional)] on_back: Option<Callback<()>>,
) -> impl IntoView {
    let locale = locale.unwrap_or_else(|| Signal::derive(|| GateLocale::Zh));
    let wallet = expect_context::<WalletState>();
    let (sold, set_sold) = signal(0_u64);
    let (collection_name, set_collection_name) = signal("Once and Once Again—Painting".to_string());
    let (collection_description, set_collection_description) = signal(String::new());
    let (loading, set_loading) = signal(true);
    let (minting, set_minting) = signal(false);
    let (status, set_status) = signal(String::new());

    let refresh = move || {
        set_loading.set(true);
        spawn_local(async move {
            match fetch_items_redeemed().await {
                Ok(count) => {
                    set_sold.set(count.min(NFT_TOTAL_SUPPLY));
                    set_status.set(String::new());
                }
                Err(err) => set_status.set(err),
            }
            if let Ok(response) = gloo_net::http::Request::get(COLLECTION_METADATA_URL)
                .send()
                .await
            {
                if response.ok() {
                    if let Ok(meta) = response.json::<CollectionMeta>().await {
                        if let Some(name) = meta.name.filter(|s| !s.trim().is_empty()) {
                            set_collection_name.set(name);
                        }
                        if let Some(description) =
                            meta.description.filter(|s| !s.trim().is_empty())
                        {
                            set_collection_description.set(description);
                        }
                    }
                }
            }
            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        refresh();
    });

    let sold_out = Signal::derive(move || sold.get() >= NFT_TOTAL_SUPPLY);

    let on_mint = move |_| {
        if sold_out.get_untracked() || minting.get_untracked() {
            return;
        }
        if !wallet.is_connected() {
            wallet.request_connect();
            set_status.set(tr_locale(
                locale.get_untracked(),
                "请先连接钱包后再铸造。",
                "Connect your wallet before minting.",
            ));
            return;
        }
        set_minting.set(true);
        set_status.set(tr_locale(
            locale.get_untracked(),
            "正在提交铸造交易，请在钱包中确认…",
            "Submitting mint transaction — confirm in your wallet…",
        ));
        let current_locale = locale.get_untracked();
        spawn_local(async move {
            match mint_nft_with_wallet().await {
                Ok(result) => {
                    set_status.set(format!(
                        "{} {} | mint={}",
                        tr_locale(current_locale, "铸造成功：", "Mint succeeded:"),
                        result.signature,
                        result.mint
                    ));
                    if let Ok(count) = fetch_items_redeemed().await {
                        set_sold.set(count.min(NFT_TOTAL_SUPPLY));
                    }
                }
                Err(err) => {
                    set_status.set(format!(
                        "{} {err}",
                        tr_locale(current_locale, "铸造失败：", "Mint failed:")
                    ));
                }
            }
            set_minting.set(false);
        });
    };

    let on_back_btn = on_back.clone();

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-start justify-center p-3 sm:p-6">
            <section class="relative w-full max-w-4xl rounded-2xl border border-slate-800 bg-slate-900 p-4 sm:p-8 shadow-xl">
                {move || {
                    if let Some(back_cb) = on_back_btn.clone() {
                        view! {
                            <button
                                type="button"
                                class="absolute right-4 top-4 text-sm text-slate-300 underline underline-offset-4 hover:text-slate-100"
                                on:click=move |_| back_cb.run(())
                            >
                                {move || tr_locale(locale.get(), "返回", "Back")}
                            </button>
                        }
                        .into_any()
                    } else {
                        view! { <></> }.into_any()
                    }
                }}
                <h1 class="text-2xl sm:text-3xl font-bold tracking-tight pr-0 sm:pr-36">
                    {move || tr_locale(locale.get(), "铸造 NFT", "Mint NFT")}
                </h1>
                <p class="mt-2 text-sm text-slate-400">{move || collection_name.get()}</p>

                <div class="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)]">
                    <div class="overflow-hidden rounded-xl border border-slate-800 bg-slate-950/60">
                        <img
                            src=COLLECTION_IMAGE_URL
                            alt=move || collection_name.get()
                            class="aspect-square w-full object-contain bg-white"
                        />
                        <div class="border-t border-slate-800 px-4 py-3">
                            <p class="text-xs text-slate-400">
                                {move || tr_locale(locale.get(), "系列封面", "Collection cover")}
                            </p>
                            <p class="mt-1 text-sm font-medium text-slate-100">
                                {move || collection_name.get()}
                            </p>
                        </div>
                    </div>

                    <div class="flex flex-col gap-4">
                        <div class="rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                            <h2 class="text-lg font-semibold">
                                {move || tr_locale(locale.get(), "系列说明", "About this series")}
                            </h2>
                            <p class="mt-2 text-sm leading-relaxed text-slate-300">
                                {move || {
                                    let custom = collection_description.get();
                                    if custom.trim().is_empty() {
                                        tr_locale(
                                            locale.get(),
                                            "本系列是艺术项目 Once and Once Again 下的 Painting v1.0 子系列，共 20 件限量作品，部署于 Solana Devnet。每件 NFT 对应一幅独立数字墨水画。",
                                            "This is Painting v1.0 under the Once and Once Again art project: a limited drop of 20 works on Solana Devnet. Each NFT maps to one digital ink painting.",
                                        )
                                    } else {
                                        custom
                                    }
                                }}
                            </p>
                        </div>

                        <div class="rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                            <h2 class="text-lg font-semibold">{move || tr_locale(locale.get(), "库存", "Supply")}</h2>
                            <dl class="mt-3 grid grid-cols-2 gap-3 text-sm">
                                <div>
                                    <dt class="text-slate-400">{move || tr_locale(locale.get(), "总量", "Total")}</dt>
                                    <dd class="mt-1 text-xl font-semibold">{NFT_TOTAL_SUPPLY}</dd>
                                </div>
                                <div>
                                    <dt class="text-slate-400">{move || tr_locale(locale.get(), "已售", "Sold")}</dt>
                                    <dd class="mt-1 text-xl font-semibold">
                                        {move || {
                                            if loading.get() {
                                                "…".to_string()
                                            } else {
                                                sold.get().to_string()
                                            }
                                        }}
                                    </dd>
                                </div>
                                <div>
                                    <dt class="text-slate-400">{move || tr_locale(locale.get(), "剩余", "Remaining")}</dt>
                                    <dd class="mt-1 text-xl font-semibold">
                                        {move || {
                                            if loading.get() {
                                                "…".to_string()
                                            } else {
                                                NFT_TOTAL_SUPPLY.saturating_sub(sold.get()).to_string()
                                            }
                                        }}
                                    </dd>
                                </div>
                                <div>
                                    <dt class="text-slate-400">{move || tr_locale(locale.get(), "价格", "Price")}</dt>
                                    <dd class="mt-1 text-xl font-semibold">{format!("{NFT_MINT_PRICE_SOL} SOL")}</dd>
                                </div>
                            </dl>
                            <button
                                type="button"
                                on:click=move |_| refresh()
                                class="mt-3 text-xs text-slate-400 underline underline-offset-4 hover:text-slate-200"
                            >
                                {move || tr_locale(locale.get(), "刷新库存", "Refresh supply")}
                            </button>
                        </div>

                        <div class="rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                            <h2 class="text-lg font-semibold">{move || tr_locale(locale.get(), "NFT 功能", "NFT utility")}</h2>
                            <p class="mt-2 text-sm leading-relaxed text-slate-300">
                                {move || {
                                    tr_locale(
                                        locale.get(),
                                        "持有本系列任意一件 NFT，可解锁「本地词库修改器」（Dictionary Editor）完整功能，用于私人定制词库。",
                                        "Holding any NFT from this series unlocks the full Local Lexicon Editor (Dictionary Editor) for private word-bank customization.",
                                    )
                                }}
                            </p>
                        </div>

                        <button
                            type="button"
                            on:click=on_mint
                            disabled=move || sold_out.get() || minting.get() || loading.get()
                            class=move || {
                                if sold_out.get() {
                                    "inline-flex w-full items-center justify-center rounded-lg border border-slate-700 bg-slate-800/50 px-4 py-3 text-sm font-semibold text-slate-500 cursor-not-allowed"
                                        .to_string()
                                } else {
                                    "inline-flex w-full items-center justify-center rounded-lg border border-emerald-700 bg-emerald-800/80 px-4 py-3 text-sm font-semibold text-emerald-50 hover:bg-emerald-700 disabled:cursor-wait disabled:opacity-60"
                                        .to_string()
                                }
                            }
                        >
                            {move || {
                                let current_locale = locale.get();
                                if sold_out.get() {
                                    tr_locale(current_locale, "已售罄", "Sold Out")
                                } else if minting.get() {
                                    tr_locale(current_locale, "铸造中…", "Minting…")
                                } else {
                                    format!(
                                        "{} ({NFT_MINT_PRICE_SOL} SOL)",
                                        tr_locale(current_locale, "Mint", "Mint")
                                    )
                                }
                            }}
                        </button>

                        <p class="text-[11px] leading-relaxed text-slate-500 break-all">
                            {format!("Candy Machine: {CANDY_MACHINE_ID}")}
                        </p>
                        {move || {
                            let message = status.get();
                            if message.trim().is_empty() {
                                return view! { <></> }.into_any();
                            }
                            view! {
                                <p class="rounded-lg border border-slate-800 bg-slate-950/80 px-3 py-2 text-xs text-slate-300 break-all">
                                    {message}
                                </p>
                            }
                            .into_any()
                        }}
                    </div>
                </div>
            </section>
        </main>
    }
}
