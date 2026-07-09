use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::lexicon_browser::parse_word_bank_csv;
use crate::pages::AppPage;

#[component]
pub fn PracticeModePage() -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let (download_status, set_download_status) = signal("正在下载词库...".to_string());

    Effect::new(move |_| {
        let set_download_status = set_download_status;
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
                Ok(entries) => {
                    set_download_status
                        .set(format!("词库下载完成，共 {} 条词条。", entries.len()));
                }
                Err(err) => {
                    set_download_status.set(format!("词库下载失败：{err}"));
                }
            }
        });
    });

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
            <section class="w-full max-w-2xl p-8 rounded-2xl border border-slate-800 bg-slate-900 shadow-xl">
                <h1 class="text-3xl font-bold tracking-tight">"请选择练习模式"</h1>
                <p class="mt-4 text-slate-300">{move || download_status.get()}</p>

                <div class="mt-6 flex flex-wrap items-center gap-3">
                    <button
                        type="button"
                        on:click=move |_| set_current_page.set(AppPage::LexiconMode)
                        class="inline-flex items-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                    >
                        "词库模式"
                    </button>
                    <button
                        type="button"
                        class="inline-flex items-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                    >
                        "单词系列模式"
                    </button>
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
