use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app_state::WordBankState;
use crate::components::import_csv::ImportCsvButton;
use crate::components::lexicon_browser::parse_word_bank_csv;
use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;

#[component]
pub fn PracticeModePage() -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let word_bank_state = expect_context::<WordBankState>();
    let (selected_word_bank, set_selected_word_bank) = signal("word-bank.csv".to_string());
    let (status, set_status) = signal(String::new());

    let choose_word_bank_click = move |_| {
        let selected_file = selected_word_bank.get_untracked();
        let word_bank_state = word_bank_state;
        set_status.set(format!("正在加载内置词库：{selected_file} ..."));

        spawn_local(async move {
            let result = async {
                let response = Request::get(&format!("/data/{selected_file}"))
                    .send()
                    .await
                    .map_err(|err| format!("下载失败: {err}"))?;
                let csv_text = response
                    .text()
                    .await
                    .map_err(|err| format!("读取响应失败: {err}"))?;
                parse_word_bank_csv(&csv_text)
            }
            .await;

            match result {
                Ok(word_list) => {
                    let count = word_list.len();
                    word_bank_state.set_entries.set(word_list);
                    word_bank_state.set_data_version.update(|ver| *ver += 1);
                    word_bank_state
                        .set_source_name
                        .set(format!("内置词库：{selected_file}"));
                    set_status.set(format!("词库切换成功（已覆盖），共 {count} 条。"));
                }
                Err(err) => {
                    set_status.set(format!("词库切换失败：{err}"));
                }
            }
        });
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
            <section class="relative w-full max-w-5xl p-8 rounded-2xl border border-slate-800 bg-slate-900 shadow-xl">
                <ReturnButton target_page=AppPage::Home/>
                <h1 class="text-3xl font-bold tracking-tight">"请选择练习模式"</h1>
                <p class="mt-4 text-slate-300">
                    {move || {
                        let count = word_bank_state.entries.get().len();
                        let source = word_bank_state.source_name.get();
                        if count == 0 {
                            "尚未加载词库，请先在本页选择词库或导入词库 CSV。".to_string()
                        } else {
                            format!("当前词库：{source}（共 {count} 条词条）。")
                        }
                    }}
                </p>
                <p class="mt-1 min-h-5 text-sm text-slate-300">{move || status.get()}</p>

                <div class="mt-6 grid grid-cols-1 gap-4 lg:grid-cols-2">
                    <section class="rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                        <h2 class="text-lg font-semibold">"词库练习"</h2>
                        <p class="mt-2 text-sm text-slate-400">"先选择或导入词库，再进入词库模式。"</p>
                        <div class="mt-4 grid grid-cols-1 gap-3 md:grid-cols-[1fr_auto_auto]">
                            <select
                                prop:value=move || selected_word_bank.get()
                                on:change=move |ev| set_selected_word_bank.set(event_target_value(&ev))
                                class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-sm text-slate-100"
                            >
                                <option value="word-bank-mini.csv">"word-bank-mini.csv"</option>
                                <option value="word-bank.csv">"word-bank.csv"</option>
                                <option value="word-bank-full.csv">"word-bank-full.csv"</option>
                            </select>
                            <button
                                type="button"
                                on:click=choose_word_bank_click
                                class="inline-flex items-center justify-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                            >
                                "选择词库"
                            </button>
                            <ImportCsvButton
                                input_id="practice-mode-import-csv-input".to_string()
                                label="导入词库 CSV".to_string()
                                class="inline-flex cursor-pointer items-center justify-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700".to_string()
                                set_entries=word_bank_state.set_entries
                                set_status=set_status
                                set_data_version=word_bank_state.set_data_version
                                set_source_name=word_bank_state.set_source_name
                            />
                        </div>
                        <div class="mt-4">
                            <button
                                type="button"
                                on:click=move |_| set_current_page.set(AppPage::LexiconMode)
                                class="inline-flex items-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                            >
                                "词库模式"
                            </button>
                        </div>
                    </section>

                    <section class="rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                        <h2 class="text-lg font-semibold">"单词系列练习"</h2>
                        <p class="mt-2 text-sm text-slate-400">"数词、月份、代词、疑问词等固定系列练习。"</p>
                        <div class="mt-4">
                            <button
                                type="button"
                                class="inline-flex items-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                            >
                                "单词系列模式"
                            </button>
                        </div>
                    </section>
                </div>

                <div class="mt-10 text-right">
                    <button
                        type="button"
                        on:click=move |_| set_current_page.set(AppPage::LocalLexiconEditor)
                        class="text-sm text-slate-400 underline underline-offset-4 hover:text-slate-300"
                    >
                        "本地词库修改器"
                    </button>
                </div>
            </section>
        </main>
    }
}
