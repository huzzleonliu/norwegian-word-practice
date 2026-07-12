use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::pages::AppPage;
use crate::utils::i18n::tr;

#[component]
pub fn ReturnButton(
    target_page: AppPage,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let word_bank_state = expect_context::<WordBankState>();
    let lang = word_bank_state.ui_language;
    let mut classes = "absolute right-6 top-6 rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-sm text-slate-100 hover:bg-slate-700".to_string();
    if let Some(extra) = class {
        let extra = extra.trim();
        if !extra.is_empty() {
            classes.push(' ');
            classes.push_str(extra);
        }
    }
    let fixed_label = label.filter(|value| !value.trim().is_empty());

    view! {
        <button type="button" on:click=move |_| set_current_page.set(target_page) class=classes>
            {move || {
                fixed_label
                    .clone()
                    .unwrap_or_else(|| tr(lang.get(), "返回上一级页面", "Back").to_string())
            }}
        </button>
    }
}
