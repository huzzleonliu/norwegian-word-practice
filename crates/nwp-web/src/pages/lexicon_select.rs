//! 词库选词页：通过查询模式浏览器勾选词条，并启动练习队列。

use leptos::prelude::*;

use crate::app_state::{LexiconState, NavigateToPage, PracticeState, UiState};
use crate::components::lexicon_browser::{LexiconBrowser, LexiconBrowserMode};
use crate::components::mini_console::MiniConsole;
use crate::layout::{PageHeading, PageShell, PageTitle, PageTopbar, ShellAttach};
use crate::pages::AppPage;
use crate::structures::word_bank_entry::WordBankEntry;
use crate::utils::i18n::tr;
use crate::utils::shuffle::shuffle_strings;

#[component]
pub fn LexiconSelectPage() -> impl IntoView {
    let set_current_page = expect_context::<NavigateToPage>();
    let lexicon_state = expect_context::<LexiconState>();
    let practice_state = expect_context::<PracticeState>();
    let entries = lexicon_state.entries;
    let set_entries = lexicon_state.set_entries;
    let set_selected_word_entry_ids = practice_state.set_selected_word_entry_ids;
    let data_version = lexicon_state.data_version;
    let lang = expect_context::<UiState>().ui_language;
    let (status, set_status) = signal(String::new());

    Effect::new(move |_| {
        let _ = data_version.get();
        let count = entries.get_untracked().len();
        let source = lexicon_state.source_name.get_untracked();
        let prepared_count = practice_state.selected_word_entry_ids.get_untracked().len();
        let language = lang.get_untracked();
        if count == 0 {
            set_status.set(
                tr(
                    language,
                    "当前词库为空，请先回到首页加载或导入字典。",
                    "Current lexicon is empty. Please load/import a dictionary first.",
                )
                .to_string(),
            );
        } else if prepared_count == 0 {
            set_status.set(format!(
                "{}（{}：{source}，{} {count} {}）",
                tr(language, "请选择想要练习的单词", "Select words to practice"),
                tr(language, "词库", "lexicon"),
                tr(language, "共", "total"),
                tr(language, "条", "entries")
            ));
        } else {
            set_status.set(format!(
                "{}（{}：{source}，{} {count} {}，{} {prepared_count} {}）",
                tr(language, "请选择想要练习的单词", "Select words to practice"),
                tr(language, "词库", "lexicon"),
                tr(language, "共", "total"),
                tr(language, "条", "entries"),
                tr(language, "已准备", "prepared"),
                tr(language, "条", "entries")
            ));
        }
    });

    let start_remember_click = move |_| {
        let language = lang.get_untracked();
        let mut selected_ids = collect_selected_entry_ids(&entries.get_untracked());
        if selected_ids.is_empty() {
            set_status.set(
                tr(
                    language,
                    "请先选择至少 1 条词条，再开始练习。",
                    "Please select at least one entry before starting.",
                )
                .to_string(),
            );
            return;
        }
        shuffle_strings(&mut selected_ids);
        set_selected_word_entry_ids.set(selected_ids);
        set_current_page.set(AppPage::LexiconRemember);
    };

    let start_practice_click = move |_| {
        let language = lang.get_untracked();
        let mut selected_ids = collect_selected_entry_ids(&entries.get_untracked());
        if selected_ids.is_empty() {
            set_status.set(
                tr(
                    language,
                    "请先选择至少 1 条词条，再开始练习。",
                    "Please select at least one entry before starting.",
                )
                .to_string(),
            );
            return;
        }
        shuffle_strings(&mut selected_ids);
        set_selected_word_entry_ids.set(selected_ids);
        set_current_page.set(AppPage::LexiconPractice);
    };

    view! {
        <PageShell attach=ShellAttach::Content>
            <PageTopbar>
                <PageHeading>
                    <PageTitle>
                        {move || tr(lang.get(), "请选择想要练习的单词", "Select words to practice")}
                    </PageTitle>
                </PageHeading>
            </PageTopbar>
                <MiniConsole
                    message=Signal::derive(move || {
                        let language = lang.get();
                        let count = entries.get().len();
                        let source = lexicon_state.source_name.get();
                        let source_line = if count == 0 {
                            format!(
                                "{}（{}：{source}）。",
                                tr(language, "当前词库为空", "Current lexicon is empty"),
                                tr(language, "来源", "source")
                            )
                        } else {
                            format!(
                                "{}：{source}（{} {count} {}）。",
                                tr(language, "当前词库", "Current lexicon"),
                                tr(language, "共", "total"),
                                tr(language, "条词条", "entries")
                            )
                        };
                        let status_line = {
                            let s = status.get();
                            if s.trim().is_empty() {
                                tr(language, "等待选词并开始练习...", "Waiting for selection...").to_string()
                            } else {
                                s
                            }
                        };
                        format!("{source_line}\n{status_line}")
                    })
                />

                <LexiconBrowser
                    entries=entries
                    set_entries=set_entries
                    set_status=set_status
                    data_version=data_version
                    mode=LexiconBrowserMode::Query
                />

                <div class="mt-6 flex flex-col justify-center gap-3 sm:flex-row">
                    <button
                        type="button"
                        on:click=start_remember_click
                        class="inline-flex w-full items-center justify-center rounded-lg border border-emerald-700 bg-emerald-700 px-6 py-2 text-sm font-medium text-white hover:bg-emerald-600 sm:w-auto"
                    >
                        {move || tr(lang.get(), "快速记忆练习", "Quick Memory Practice")}
                    </button>

                    <button
                        type="button"
                        on:click=start_practice_click
                        class="inline-flex w-full items-center justify-center rounded-lg border border-emerald-700 bg-emerald-700 px-6 py-2 text-sm font-medium text-white hover:bg-emerald-600 sm:w-auto"
                    >
                        {move || tr(lang.get(), "拼写练习", "Spelling Practice")}
                    </button>
                </div>
        </PageShell>
    }
}

fn collect_selected_entry_ids(entries: &[WordBankEntry]) -> Vec<String> {
    // 仅收集已提交词库中 `selected=true` 的条目 id。
    entries
        .iter()
        .filter(|entry| entry.selected)
        .map(|entry| entry.id.clone())
        .collect()
}
