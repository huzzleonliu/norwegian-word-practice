use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::components::lexicon_browser::{LexiconBrowser, LexiconBrowserMode};
use crate::components::mini_console::MiniConsole;
use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;
use crate::structures::word_bank_entry::WordBankEntry;
use crate::utils::i18n::tr;
use crate::utils::shuffle::shuffle_strings;

#[component]
pub fn LexiconSelectPage() -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let word_bank_state = expect_context::<WordBankState>();
    let entries = word_bank_state.entries;
    let set_entries = word_bank_state.set_entries;
    let set_selected_word_entry_ids = word_bank_state.set_selected_word_entry_ids;
    let data_version = word_bank_state.data_version;
    let lang = word_bank_state.ui_language;
    let (status, set_status) = signal(String::new());

    Effect::new(move |_| {
        let _ = data_version.get();
        let count = entries.get_untracked().len();
        let source = word_bank_state.source_name.get_untracked();
        let prepared_count = word_bank_state
            .selected_word_entry_ids
            .get_untracked()
            .len();
        let language = lang.get_untracked();
        if count == 0 {
            set_status.set(
                tr(
                    language,
                    "当前词库为空，请先回到首页加载或导入词库。",
                    "Current lexicon is empty. Please load/import lexicon first.",
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
        <main class="min-h-screen bg-slate-950 text-slate-100 p-3 sm:p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-4 sm:p-6 shadow-xl">
                <ReturnButton target_page=AppPage::PracticeModeSelect/>
                <header class="flex items-center gap-4">
                    <h1 class="text-xl sm:text-2xl font-bold tracking-tight">
                        {move || tr(lang.get(), "请选择想要练习的单词", "Select words to practice")}
                    </h1>
                </header>
                <MiniConsole
                    message=Signal::derive(move || {
                        let language = lang.get();
                        let count = entries.get().len();
                        let source = word_bank_state.source_name.get();
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

                <div class="mt-6 flex justify-center">
                    <button
                        type="button"
                        on:click=start_practice_click
                        class="inline-flex w-full items-center justify-center rounded-lg border border-emerald-700 bg-emerald-700 px-6 py-2 text-sm font-medium text-white hover:bg-emerald-600 sm:w-auto"
                    >
                        {move || tr(lang.get(), "开始练习", "Start Practice")}
                    </button>
                </div>
            </section>
        </main>
    }
}

fn collect_selected_entry_ids(entries: &[WordBankEntry]) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| entry.selected)
        .map(|entry| entry.id.clone())
        .collect()
}
