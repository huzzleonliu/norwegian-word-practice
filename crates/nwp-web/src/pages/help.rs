//! 使用说明浮层：按当前 `AppPage` 与界面语言加载对应 Markdown 文档。

use leptos::prelude::*;
use markdown_view_leptos::markdown_view;

use crate::app_state::UiState;
use crate::pages::AppPage;
use crate::structures::word_bank_entry::UiLanguage;
use crate::utils::i18n::tr;

#[component]
pub fn HelpOverlay(
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
    page: Signal<AppPage>,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;

    view! {
        {move || {
            if !open.get() {
                return view! { <></> }.into_any();
            }

            view! {
                <div
                    class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60 p-3 sm:p-6"
                    on:click=move |_| set_open.set(false)
                >
                    <div
                        class="relative flex max-h-[85vh] w-full max-w-3xl flex-col overflow-hidden rounded-2xl border border-slate-700 bg-slate-900 shadow-2xl"
                        on:click=move |ev| ev.stop_propagation()
                    >
                        <div class="flex items-center justify-between border-b border-slate-800 px-4 py-3">
                            <h2 class="text-base font-semibold text-slate-100 sm:text-lg">
                                {move || tr(lang.get(), "使用说明", "Instructions")}
                            </h2>
                            <button
                                type="button"
                                on:click=move |_| set_open.set(false)
                                class="inline-flex h-8 w-8 items-center justify-center rounded-lg border border-slate-700 bg-slate-800 text-lg leading-none text-slate-100 hover:bg-slate-700"
                                aria-label=move || tr(lang.get(), "关闭", "Close")
                            >
                                "×"
                            </button>
                        </div>
                        <div class="markdown-view instructions-markdown flex-1 overflow-y-auto px-4 py-4 text-sm text-slate-200 sm:px-6 sm:py-5">
                            {move || help_markdown_view(page.get(), lang.get())}
                        </div>
                    </div>
                </div>
            }
            .into_any()
        }}
    }
}

fn help_markdown_view(page: AppPage, language: UiLanguage) -> AnyView {
    match (page, language) {
        (AppPage::Home, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/home.zh.md")} }.into_any()
        }
        (AppPage::Home, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/home.en.md")} }.into_any()
        }
        (AppPage::PracticeModeSelect, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/practice_mode_select.zh.md")} }.into_any()
        }
        (AppPage::PracticeModeSelect, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/practice_mode_select.en.md")} }.into_any()
        }
        (AppPage::LexiconMode, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/lexicon_select.zh.md")} }.into_any()
        }
        (AppPage::LexiconMode, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/lexicon_select.en.md")} }.into_any()
        }
        (AppPage::LexiconPractice, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/lexicon_practice.zh.md")} }.into_any()
        }
        (AppPage::LexiconPractice, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/lexicon_practice.en.md")} }.into_any()
        }
        (AppPage::LexiconRemember, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/lexicon_practice.zh.md")} }.into_any()
        }
        (AppPage::LexiconRemember, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/lexicon_practice.en.md")} }.into_any()
        }
        (AppPage::LexiconSummary, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/practice_result.zh.md")} }.into_any()
        }
        (AppPage::LexiconSummary, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/practice_result.en.md")} }.into_any()
        }
        (AppPage::LocalLexiconEditor, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/dictionary_editor.zh.md")} }.into_any()
        }
        (AppPage::LocalLexiconEditor, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/dictionary_editor.en.md")} }.into_any()
        }
        (AppPage::SeriseSelect, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/serise_select.zh.md")} }.into_any()
        }
        (AppPage::SeriseSelect, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/serise_select.en.md")} }.into_any()
        }
        (AppPage::SeriseNumberPractice, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/serise_practice_number.zh.md")} }.into_any()
        }
        (AppPage::SeriseNumberPractice, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/serise_practice_number.en.md")} }.into_any()
        }
        (AppPage::SeriseMonthPractice, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/serise_practice_month.zh.md")} }.into_any()
        }
        (AppPage::SeriseMonthPractice, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/serise_practice_month.en.md")} }.into_any()
        }
        (AppPage::SerisePronounPractice, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/serise_practice_pronoun.zh.md")} }.into_any()
        }
        (AppPage::SerisePronounPractice, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/serise_practice_pronoun.en.md")} }.into_any()
        }
        (AppPage::SeriseInterrogativePractice, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/serise_practice_interrogative.zh.md")} }
                .into_any()
        }
        (AppPage::SeriseInterrogativePractice, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/serise_practice_interrogative.en.md")} }
                .into_any()
        }
        (AppPage::SeriseCountryPractice, UiLanguage::Zh) => {
            view! { {markdown_view!(file = "docs/help/serise_practice_country.zh.md")} }.into_any()
        }
        (AppPage::SeriseCountryPractice, UiLanguage::En) => {
            view! { {markdown_view!(file = "docs/help/serise_practice_country.en.md")} }.into_any()
        }
    }
}
