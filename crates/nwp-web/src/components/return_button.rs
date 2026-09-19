//! 通用返回按钮：将当前页面切换到指定 `AppPage`。

use leptos::prelude::*;

use crate::app_state::{NavigateToPage, UiState};
use crate::pages::AppPage;
use crate::utils::i18n::tr;

#[component]
pub fn ReturnButton(
    target_page: AppPage,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let set_current_page = expect_context::<NavigateToPage>();
    let lang = expect_context::<UiState>().ui_language;
    let mut classes = "ui-btn-ghost shrink-0".to_string();
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
