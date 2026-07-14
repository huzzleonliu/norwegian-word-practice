//! 应用入口：初始化全局信号、注入 Context，并按 `AppPage` 分发页面组件。

use leptos::prelude::*;

mod app_state;
mod components;
mod pages;
mod structures;
#[cfg(test)]
mod tests;
mod utils;

use structures::word_bank_entry::UiLanguage;
use utils::i18n::tr;

fn main() {
    leptos::mount::mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    let (current_page, set_current_page) = signal(pages::AppPage::Home);
    let (word_bank_entries, set_word_bank_entries) = signal(Vec::new());
    let (word_bank_data_version, set_word_bank_data_version) = signal(0_u64);
    let (word_bank_source_name, set_word_bank_source_name) = signal("尚未加载词库".to_string());
    let (ui_language, set_ui_language) = signal(UiLanguage::Zh);
    let (selected_word_entry_ids, set_selected_word_entry_ids) = signal(Vec::<String>::new());
    let (practice_result, set_practice_result) =
        signal(structures::pracresult::PracticeResult::default());
    let (temp_practice_result, set_temp_practice_result) =
        signal(structures::pracresult::PracticeResult::default());
    let (last_completed_practice_result, set_last_completed_practice_result) =
        signal(structures::pracresult::PracticeResult::default());
    let (summary_return_page, set_summary_return_page) = signal(pages::AppPage::LexiconPractice);

    // 统一在根组件注入页面路由与全局状态，子页面通过 `expect_context` 获取。
    provide_context(set_current_page);
    provide_context(app_state::WordBankState {
        entries: word_bank_entries,
        set_entries: set_word_bank_entries,
        data_version: word_bank_data_version,
        set_data_version: set_word_bank_data_version,
        source_name: word_bank_source_name,
        set_source_name: set_word_bank_source_name,
        ui_language,
        selected_word_entry_ids,
        set_selected_word_entry_ids,
        practice_result,
        set_practice_result,
        temp_practice_result,
        set_temp_practice_result,
        last_completed_practice_result,
        set_last_completed_practice_result,
        summary_return_page,
        set_summary_return_page,
    });

    view! {
        <div>
            <button
                type="button"
                on:click=move |_| set_ui_language.update(|lang| *lang = lang.toggle())
                class="fixed right-2 top-2 z-[100] rounded-lg border border-slate-700 bg-slate-900/90 px-2.5 py-1.5 text-[11px] text-slate-100 hover:bg-slate-800 sm:right-4 sm:top-4 sm:px-3 sm:py-2 sm:text-xs"
            >
                {move || {
                    let lang = ui_language.get();
                    format!(
                        "{}: {}",
                        tr(lang, "语言", "Language"),
                        tr(lang, "中文", "English")
                    )
                }}
            </button>
            // 本项目不使用 URL 路由，页面切换通过 `AppPage` 枚举进行。
            {move || match current_page.get() {
                pages::AppPage::Home => view! { <pages::home::HomePage/> }.into_any(),
                pages::AppPage::PracticeModeSelect => {
                    view! { <pages::practice_mode::PracticeModePage/> }.into_any()
                }
                pages::AppPage::LexiconMode => {
                    view! { <pages::lexicon_select::LexiconSelectPage/> }.into_any()
                }
                pages::AppPage::LexiconPractice => {
                    view! { <pages::lexicon_practice::LexiconPracticePage/> }.into_any()
                }
                pages::AppPage::LexiconSummary => {
                    view! { <pages::lexicon_summary::LexiconSummaryPage/> }.into_any()
                }
                pages::AppPage::LocalLexiconEditor => {
                    view! { <pages::local_lexicon_editor::LocalLexiconEditorPage/> }.into_any()
                }
                pages::AppPage::SeriseSelect => {
                    view! { <pages::serise_select::SeriseSelectPage/> }.into_any()
                }
                pages::AppPage::SeriseNumberPractice => {
                    view! { <pages::serise_practice::number::NumberSerisePracticePage/> }.into_any()
                }
                pages::AppPage::SeriseMonthPractice => {
                    view! { <pages::serise_practice::month::MonthSerisePracticePage/> }.into_any()
                }
                pages::AppPage::SerisePronounPractice => {
                    view! { <pages::serise_practice::pronoun::PronounSerisePracticePage/> }.into_any()
                }
                pages::AppPage::SeriseInterrogativePractice => {
                    view! { <pages::serise_practice::interrogative::InterrogativeSerisePracticePage/> }
                        .into_any()
                }
            }}
        </div>
    }
}
