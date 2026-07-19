//! 右上角 Solana 钱包连接按钮。

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app_state::UiState;
use crate::utils::i18n::tr;
use crate::utils::solana_wallet::{connect_wallet, disconnect_wallet, shorten_address};

#[component]
pub fn WalletConnectButton() -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    let (address, set_address) = signal(Option::<String>::None);
    let (busy, set_busy) = signal(false);
    let (status, set_status) = signal(String::new());

    let on_click = move |_| {
        if busy.get_untracked() {
            return;
        }
        set_busy.set(true);
        set_status.set(String::new());
        let language = lang.get_untracked();
        spawn_local(async move {
            if address.get_untracked().is_some() {
                match disconnect_wallet().await {
                    Ok(()) => {
                        set_address.set(None);
                        set_status.set(String::new());
                    }
                    Err(err) => {
                        set_status.set(format!(
                            "{} {err}",
                            tr(language, "断开失败：", "Disconnect failed:")
                        ));
                    }
                }
            } else {
                match connect_wallet().await {
                    Ok(pubkey) => {
                        set_address.set(Some(pubkey));
                        set_status.set(String::new());
                    }
                    Err(err) => {
                        set_status.set(format!(
                            "{} {err}",
                            tr(language, "连接失败：", "Connect failed:")
                        ));
                    }
                }
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="relative">
            <button
                type="button"
                on:click=on_click
                disabled=move || busy.get()
                class="rounded-lg border border-slate-700 bg-slate-900/90 px-2.5 py-1.5 text-[11px] text-slate-100 hover:bg-slate-800 disabled:cursor-wait disabled:opacity-60 sm:px-3 sm:py-2 sm:text-xs"
                title=move || status.get()
            >
                {move || {
                    let language = lang.get();
                    if busy.get() {
                        return tr(language, "处理中…", "Working…").to_string();
                    }
                    match address.get() {
                        Some(pubkey) => format!(
                            "{} {}",
                            tr(language, "已连接", "Connected"),
                            shorten_address(&pubkey)
                        ),
                        None => tr(language, "连接钱包", "Connect Wallet").to_string(),
                    }
                }}
            </button>
            {move || {
                let message = status.get();
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
