use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::locale::GateLocale;
use crate::state::WalletState;
use crate::{connect_wallet, disconnect_wallet, shorten_address};

fn tr_locale(locale: GateLocale, zh: &str, en: &str) -> String {
    match locale {
        GateLocale::Zh => zh.to_string(),
        GateLocale::En => en.to_string(),
    }
}

/// 可复用钱包连接按钮（依赖 `WalletState` context）。
#[component]
pub fn WalletConnectButton(
    #[prop(optional, into)] locale: Option<Signal<GateLocale>>,
) -> impl IntoView {
    let locale = locale.unwrap_or_else(|| Signal::derive(|| GateLocale::Zh));
    let wallet = expect_context::<WalletState>();

    let run_connect = move || {
        if wallet.busy.get_untracked() {
            return;
        }
        wallet.set_busy.set(true);
        wallet.set_status.set(String::new());
        let current_locale = locale.get_untracked();
        spawn_local(async move {
            match connect_wallet().await {
                Ok(pubkey) => {
                    wallet.set_address.set(Some(pubkey));
                    wallet.set_status.set(String::new());
                }
                Err(err) => {
                    wallet.set_status.set(format!(
                        "{} {err}",
                        tr_locale(current_locale, "连接失败：", "Connect failed:")
                    ));
                }
            }
            wallet.set_busy.set(false);
        });
    };

    // 外部（如 Mint 入口）递增 connect_nonce 时，触发与按钮相同的连接流程。
    Effect::new(move |_| {
        let nonce = wallet.connect_nonce.get();
        if nonce == 0 {
            return;
        }
        if wallet.address.get_untracked().is_some() {
            return;
        }
        run_connect();
    });

    let on_click = move |_| {
        if wallet.busy.get_untracked() {
            return;
        }
        if wallet.address.get_untracked().is_some() {
            wallet.set_busy.set(true);
            wallet.set_status.set(String::new());
            let current_locale = locale.get_untracked();
            spawn_local(async move {
                match disconnect_wallet().await {
                    Ok(()) => {
                        wallet.set_address.set(None);
                        wallet.set_status.set(String::new());
                    }
                    Err(err) => {
                        wallet.set_status.set(format!(
                            "{} {err}",
                            tr_locale(current_locale, "断开失败：", "Disconnect failed:")
                        ));
                    }
                }
                wallet.set_busy.set(false);
            });
        } else {
            run_connect();
        }
    };

    view! {
        <div class="relative">
            <button
                type="button"
                on:click=on_click
                disabled=move || wallet.busy.get()
                class="rounded-lg border border-slate-700 bg-slate-900/90 px-2.5 py-1.5 text-[11px] text-slate-100 hover:bg-slate-800 disabled:cursor-wait disabled:opacity-60 sm:px-3 sm:py-2 sm:text-xs"
                title=move || wallet.status.get()
            >
                {move || {
                    let current_locale = locale.get();
                    if wallet.busy.get() {
                        return tr_locale(current_locale, "处理中…", "Working…");
                    }
                    match wallet.address.get() {
                        Some(pubkey) => format!(
                            "{} {}",
                            tr_locale(current_locale, "已连接", "Connected"),
                            shorten_address(&pubkey)
                        ),
                        None => tr_locale(current_locale, "连接钱包", "Connect Wallet"),
                    }
                }}
            </button>
            {move || {
                let message = wallet.status.get();
                if message.trim().is_empty() {
                    return view! { <></> }.into_any();
                }
                view! {
                    <div class="absolute right-0 top-full z-[110] mt-1 max-w-[16rem] rounded-lg border border-slate-700 bg-slate-900 px-2 py-1.5 text-[10px] leading-snug text-rose-300 shadow-lg sm:max-w-xs sm:text-[11px]">
                        {message}
                    </div>
                }
                .into_any()
            }}
        </div>
    }
}
