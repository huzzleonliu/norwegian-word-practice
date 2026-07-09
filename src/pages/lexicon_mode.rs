use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::lexicon_browser::{LexiconBrowser, WordEntry, parse_word_bank_csv};

#[component]
pub fn LexiconModePage() -> impl IntoView {
    let (entries, set_entries) = signal(Vec::<WordEntry>::new());
    let (data_version, set_data_version) = signal(0_u64);
    let (status, set_status) = signal("正在下载词库...".to_string());

    Effect::new(move |_| {
        let set_status = set_status;
        let set_entries = set_entries;
        let set_data_version = set_data_version;
        spawn_local(async move {
            let result = async {
                let response = Request::get("/data/word-bank.csv")
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
                    set_entries.set(word_list.clone());
                    set_data_version.update(|ver| *ver += 1);
                    set_status.set(format!("请选择想要练习的单词（共 {} 条）", word_list.len()));
                }
                Err(err) => {
                    set_status.set(format!("词库加载失败：{err}"));
                }
            }
        });
    });

    let start_practice_click = move |_| {
        set_status.set("开始练习功能建设中。".to_string());
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-6">
            <section class="mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-xl">
                <h1 class="text-2xl font-bold tracking-tight">"请选择想要练习的单词"</h1>
                <p class="mt-3 text-sm text-slate-300">{move || status.get()}</p>

                <LexiconBrowser
                    entries=entries
                    set_entries=set_entries
                    set_status=set_status
                    data_version=data_version
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
