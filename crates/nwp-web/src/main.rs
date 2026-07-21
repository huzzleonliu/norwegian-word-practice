//! 应用入口：全局 Context + 官方 `leptos_router` 路由表。

use leptos::prelude::*;
use leptos_router::components::{Redirect, Route, Router, Routes};
use leptos_router::hooks::{use_location, use_navigate};
use leptos_router::path;

mod app_state;
mod components;
mod pages;
mod structures;
#[cfg(test)]
mod tests;
mod utils;

use app_state::NavigateToPage;
use pages::AppPage;
use structures::word_bank_entry::{UiLanguage, UiTheme};
use utils::i18n::tr;
use utils::lexicon_storage::{load_stored_lexicon, save_stored_lexicon, DEFAULT_SOURCE_NAME};
use utils::theme::{apply_theme, load_stored_theme};

fn main() {
    leptos::mount::mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    let (initial_entries, initial_source_name) =
        load_stored_lexicon().unwrap_or_else(|| (Vec::new(), DEFAULT_SOURCE_NAME.to_string()));
    let (word_bank_entries, set_word_bank_entries) = signal(initial_entries);
    let (word_bank_data_version, set_word_bank_data_version) = signal(0_u64);
    let (word_bank_source_name, set_word_bank_source_name) = signal(initial_source_name);
    let (ui_language, set_ui_language) = signal(UiLanguage::Zh);
    let (ui_theme, set_ui_theme) = signal(load_stored_theme());
    let (selected_word_entry_ids, set_selected_word_entry_ids) = signal(Vec::<String>::new());
    let (practice_result, set_practice_result) =
        signal(structures::pracresult::PracticeResult::default());
    let (temp_practice_result, set_temp_practice_result) =
        signal(structures::pracresult::PracticeResult::default());
    let (last_completed_practice_result, set_last_completed_practice_result) =
        signal(structures::pracresult::PracticeResult::default());
    let (summary_return_page, set_summary_return_page) = signal(AppPage::LexiconPractice);

    let (help_open, set_help_open) = signal(false);

    Effect::new(move |_| {
        apply_theme(ui_theme.get());
    });

    // 词库变更（选择 / 导入 / 编辑）后写入 localStorage，刷新可恢复。
    Effect::new(move |_| {
        let entries = word_bank_entries.get();
        let source_name = word_bank_source_name.get();
        save_stored_lexicon(&entries, &source_name);
    });

    provide_context(app_state::LexiconState {
        entries: word_bank_entries,
        set_entries: set_word_bank_entries,
        data_version: word_bank_data_version,
        set_data_version: set_word_bank_data_version,
        source_name: word_bank_source_name,
        set_source_name: set_word_bank_source_name,
    });
    provide_context(app_state::PracticeState {
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
    provide_context(app_state::UiState {
        ui_language,
        ui_theme,
    });

    view! {
        <Router>
            <AppChrome
                ui_language=ui_language
                set_ui_language=set_ui_language
                ui_theme=ui_theme
                set_ui_theme=set_ui_theme
                help_open=help_open
                set_help_open=set_help_open
            />
        </Router>
    }
}
#[component]
fn AppChrome(
    ui_language: ReadSignal<UiLanguage>,
    set_ui_language: WriteSignal<UiLanguage>,
    ui_theme: ReadSignal<UiTheme>,
    set_ui_theme: WriteSignal<UiTheme>,
    help_open: ReadSignal<bool>,
    set_help_open: WriteSignal<bool>,
) -> impl IntoView {
    let navigate = use_navigate();
    let navigate_to_page = NavigateToPage(Callback::new({
        let navigate = navigate.clone();
        move |page: AppPage| {
            navigate(page.path(), Default::default());
        }
    }));
    provide_context(navigate_to_page);

    let location = use_location();
    let current_page = Signal::derive(move || {
        AppPage::from_path(&location.pathname.get()).unwrap_or(AppPage::Home)
    });

    view! {
        <div>
            <div class="fixed right-2 top-2 z-[100] flex items-center gap-2 sm:right-4 sm:top-4">
                <button
                    type="button"
                    on:click=move |_| set_ui_theme.update(|theme| *theme = theme.toggle())
                    class="rounded-lg border border-slate-700 bg-slate-900/90 px-2.5 py-1.5 text-[11px] text-slate-100 hover:bg-slate-800 sm:px-3 sm:py-2 sm:text-xs"
                >
                    {move || {
                        let lang = ui_language.get();
                        match ui_theme.get() {
                            UiTheme::Dark => tr(lang, "黑夜模式", "Dark Mode"),
                            UiTheme::Light => tr(lang, "白天模式", "Light Mode"),
                        }
                    }}
                </button>
                <button
                    type="button"
                    on:click=move |_| set_ui_language.update(|lang| *lang = lang.toggle())
                    class="rounded-lg border border-slate-700 bg-slate-900/90 px-2.5 py-1.5 text-[11px] text-slate-100 hover:bg-slate-800 sm:px-3 sm:py-2 sm:text-xs"
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
            </div>
            <div class="fixed bottom-2 right-2 z-[100] flex items-center gap-2 sm:bottom-4 sm:right-4">
                <button
                    type="button"
                    on:click=move |_| set_help_open.set(true)
                    class="rounded-lg border border-slate-700 bg-slate-900/90 px-2.5 py-1.5 text-[11px] text-slate-100 hover:bg-slate-800 sm:px-3 sm:py-2 sm:text-xs"
                >
                    {move || tr(ui_language.get(), "使用说明", "Instructions")}
                </button>
                <a
                    href="https://discord.gg/U2z3FeUrmA"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="rounded-lg border border-slate-700 bg-slate-900/90 px-2.5 py-1.5 text-[11px] text-slate-100 hover:bg-slate-800 sm:px-3 sm:py-2 sm:text-xs"
                >
                    {move || tr(ui_language.get(), "问题反馈", "Feedback")}
                </a>
            </div>
            <pages::help::HelpOverlay
                open=help_open
                set_open=set_help_open
                page=current_page
            />
            <Routes fallback=|| view! { <Redirect path="/" /> }>
                <Route path=path!("/") view=pages::home::HomePage/>
                <Route path=path!("/practice") view=pages::practice_mode_select::PracticeModePage/>
                <Route path=path!("/lexicon") view=pages::lexicon_select::LexiconSelectPage/>
                <Route path=path!("/lexicon/practice") view=pages::lexicon_practice::LexiconPracticePage/>
                <Route path=path!("/lexicon/remember") view=pages::lexicon_remember::LexiconRememberPage/>
                <Route path=path!("/lexicon/summary") view=pages::practice_result::LexiconSummaryPage/>
                <Route path=path!("/editor") view=pages::dictionary_editor::LocalLexiconEditorPage/>
                <Route path=path!("/series") view=pages::serise_select::SeriseSelectPage/>
                <Route path=path!("/series/number") view=pages::serise_practice::number::NumberSerisePracticePage/>
                <Route path=path!("/series/month") view=pages::serise_practice::month::MonthSerisePracticePage/>
                <Route path=path!("/series/pronoun") view=pages::serise_practice::pronoun::PronounSerisePracticePage/>
                <Route
                    path=path!("/series/interrogative")
                    view=pages::serise_practice::interrogative::InterrogativeSerisePracticePage
                />
            </Routes>
        </div>
    }
}
