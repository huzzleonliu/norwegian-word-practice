//! 全局浮动控件：主题、语言、使用说明与反馈入口。

use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};

use crate::app_state::UiState;
use crate::pages::AppPage;
use crate::structures::word_bank_entry::{UiLanguage, UiTheme};
use crate::utils::i18n::tr;

use super::help::HelpOverlay;

const FEEDBACK_URL: &str = "https://discord.gg/U2z3FeUrmA";

#[component]
pub fn MasterChrome() -> impl IntoView {
    let ui = expect_context::<UiState>();
    let (help_open, set_help_open) = signal(false);
    let location = use_location();
    let current_page = Signal::derive(move || {
        AppPage::from_path(&location.pathname.get()).unwrap_or(AppPage::Home)
    });

    view! {
        <div class="fixed right-2 top-2 z-[100] flex items-center gap-2 sm:right-4 sm:top-4">
            <ThemeToggle/>
            <LanguageToggle/>
        </div>
        <div class="fixed bottom-2 right-2 z-[100] flex items-center gap-2 sm:bottom-4 sm:right-4">
            <button
                type="button"
                on:click=move |_| set_help_open.set(true)
                class="ui-chrome"
            >
                {move || tr(ui.ui_language.get(), "使用说明", "Instructions")}
            </button>
            <a
                href=FEEDBACK_URL
                target="_blank"
                rel="noopener noreferrer"
                class="ui-chrome"
            >
                {move || tr(ui.ui_language.get(), "问题反馈", "Feedback")}
            </a>
        </div>
        <HelpOverlay open=help_open set_open=set_help_open page=current_page/>
    }
}

#[component]
fn ThemeToggle() -> impl IntoView {
    let ui = expect_context::<UiState>();
    view! {
        <button
            type="button"
            on:click=move |_| ui.set_ui_theme.update(|theme| *theme = theme.toggle())
            class="ui-chrome"
        >
            {move || {
                let lang = ui.ui_language.get();
                match ui.ui_theme.get() {
                    UiTheme::Dark => tr(lang, "黑夜模式", "Dark Mode"),
                    UiTheme::Light => tr(lang, "白天模式", "Light Mode"),
                }
            }}
        </button>
    }
}

#[component]
fn LanguageToggle() -> impl IntoView {
    let ui = expect_context::<UiState>();
    let navigate = use_navigate();
    let location = use_location();
    let toggle_language = move |_| {
        let pathname = location.pathname.get_untracked();
        let page = AppPage::from_path(&pathname).unwrap_or(AppPage::Home);
        let next = ui.ui_language.get_untracked().toggle();
        ui.set_ui_language.set(next);
        navigate(&page.localized_path(next), Default::default());
    };

    view! {
        <button
            type="button"
            on:click=toggle_language
            title=move || {
                match ui.ui_language.get() {
                    UiLanguage::Zh => "Switch to English",
                    UiLanguage::En => "切换到中文",
                }
            }
            aria-label=move || {
                match ui.ui_language.get() {
                    UiLanguage::Zh => "Language: Chinese. Switch to English",
                    UiLanguage::En => "Language: English. Switch to Chinese",
                }
            }
            class="ui-chrome"
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="h-4 w-4"
                aria-hidden="true"
            >
                <circle cx="12" cy="12" r="9"></circle>
                <path d="M3 12h18"></path>
                <path d="M12 3a15 15 0 0 1 0 18"></path>
                <path d="M12 3a15 15 0 0 0 0 18"></path>
            </svg>
            <span class="text-xs tracking-wide">
                {move || match ui.ui_language.get() {
                    UiLanguage::Zh => "中",
                    UiLanguage::En => "EN",
                }}
            </span>
        </button>
    }
}
