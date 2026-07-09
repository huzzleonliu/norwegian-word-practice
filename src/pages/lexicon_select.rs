use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::components::lexicon_browser::{LexiconBrowser, LexiconBrowserMode, WordEntry};
use crate::components::mini_console::MiniConsole;
use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;
use crate::utils::shuffle::shuffle_strings;

#[component]
pub fn LexiconSelectPage() -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let word_bank_state = expect_context::<WordBankState>();
    let entries = word_bank_state.entries;
    let set_entries = word_bank_state.set_entries;
    let set_selected_word_entry_ids = word_bank_state.set_selected_word_entry_ids;
    let data_version = word_bank_state.data_version;
    let (status, set_status) = signal(String::new());

    Effect::new(move |_| {
        let _ = data_version.get();
        let count = entries.get_untracked().len();
        let source = word_bank_state.source_name.get_untracked();
        let prepared_count = word_bank_state
            .selected_word_entry_ids
            .get_untracked()
            .len();
        if count == 0 {
            set_status.set("当前词库为空，请先回到首页加载或导入词库。".to_string());
        } else if prepared_count == 0 {
            set_status.set(format!(
                "请选择想要练习的单词（词库：{source}，共 {count} 条）"
            ));
        } else {
            set_status.set(format!(
                "请选择想要练习的单词（词库：{source}，共 {count} 条，已准备 {prepared_count} 条）"
            ));
        }
    });

    let start_practice_click = move |_| {
        let mut selected_ids = collect_selected_entry_ids(&entries.get_untracked());
        if selected_ids.is_empty() {
            set_status.set("请先选择至少 1 条词条，再开始练习。".to_string());
            return;
        }

        shuffle_strings(&mut selected_ids);
        set_selected_word_entry_ids.set(selected_ids);
        set_current_page.set(AppPage::LexiconPractice);
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-xl">
                <ReturnButton target_page=AppPage::PracticeModeSelect/>
                <header class="flex items-center gap-4">
                    <h1 class="text-2xl font-bold tracking-tight">"请选择想要练习的单词"</h1>
                </header>
                <MiniConsole
                    message=Signal::derive(move || {
                        let count = entries.get().len();
                        let source = word_bank_state.source_name.get();
                        let source_line = if count == 0 {
                            format!("当前词库为空（来源：{source}）。")
                        } else {
                            format!("当前词库：{source}（共 {count} 条词条）。")
                        };
                        let status_line = {
                            let s = status.get();
                            if s.trim().is_empty() {
                                "等待选词并开始练习...".to_string()
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
                        class="inline-flex items-center rounded-lg border border-emerald-700 bg-emerald-700 px-6 py-2 text-sm font-medium text-white hover:bg-emerald-600"
                    >
                        "开始练习"
                    </button>
                </div>
            </section>
        </main>
    }
}

fn collect_selected_entry_ids(entries: &[WordEntry]) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| entry.selected)
        .map(|entry| entry.id.clone())
        .collect()
}
