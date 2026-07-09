use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app_state::WordBankState;
use crate::components::lexicon_browser::parse_word_bank_csv;
use crate::components::mini_console::MiniConsole;
use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;

#[component]
pub fn SeriseSelectPage() -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let word_bank_state = expect_context::<WordBankState>();
    let (status, set_status) = signal("请选择一个单词系列开始练习。".to_string());

    let cardinal_click = move |_| {
        load_series_word_bank(
            "number-bank.csv",
            "数词",
            AppPage::SeriseNumberPractice,
            word_bank_state,
            set_status,
            set_current_page,
        );
    };
    let month_click = move |_| {
        load_series_word_bank(
            "month-bank.csv",
            "月份",
            AppPage::SeriseMonthPractice,
            word_bank_state,
            set_status,
            set_current_page,
        );
    };
    let pronoun_click = move |_| {
        load_series_word_bank(
            "pronoun-bank.csv",
            "代词",
            AppPage::SerisePronounPractice,
            word_bank_state,
            set_status,
            set_current_page,
        );
    };
    let interrogative_click = move |_| {
        load_series_word_bank(
            "interrogative-bank.csv",
            "疑问词",
            AppPage::SeriseInterrogativePractice,
            word_bank_state,
            set_status,
            set_current_page,
        );
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-xl">
                <ReturnButton target_page=AppPage::PracticeModeSelect/>
                <h1 class="text-3xl font-bold tracking-tight">"单词系列练习"</h1>
                <MiniConsole
                    message=Signal::derive(move || {
                        let current = status.get();
                        format!("系列模式状态\n{current}")
                    })
                />

                <div class="mt-6 grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4">
                    <button
                        type="button"
                        on:click=cardinal_click
                        class="rounded-xl border border-slate-700 bg-slate-800 px-4 py-6 text-center text-base font-semibold text-slate-100 hover:bg-slate-700"
                    >
                        "数词"
                    </button>
                    <button
                        type="button"
                        on:click=month_click
                        class="rounded-xl border border-slate-700 bg-slate-800 px-4 py-6 text-center text-base font-semibold text-slate-100 hover:bg-slate-700"
                    >
                        "月份"
                    </button>
                    <button
                        type="button"
                        on:click=pronoun_click
                        class="rounded-xl border border-slate-700 bg-slate-800 px-4 py-6 text-center text-base font-semibold text-slate-100 hover:bg-slate-700"
                    >
                        "代词"
                    </button>
                    <button
                        type="button"
                        on:click=interrogative_click
                        class="rounded-xl border border-slate-700 bg-slate-800 px-4 py-6 text-center text-base font-semibold text-slate-100 hover:bg-slate-700"
                    >
                        "疑问词"
                    </button>
                </div>
            </section>
        </main>
    }
}

fn load_series_word_bank(
    selected_file: &'static str,
    series_name: &'static str,
    target_page: AppPage,
    word_bank_state: WordBankState,
    set_status: WriteSignal<String>,
    set_current_page: WriteSignal<AppPage>,
) {
    set_status.set(format!("正在加载{series_name}词库：{selected_file} ..."));

    spawn_local(async move {
        let result = async {
            let response = Request::get(&format!("/data/series-word-bank/{selected_file}"))
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
                let selected_ids = word_list
                    .iter()
                    .map(|entry| entry.id.clone())
                    .collect::<Vec<_>>();
                word_bank_state.set_entries.set(word_list);
                word_bank_state
                    .set_selected_word_entry_ids
                    .set(selected_ids);
                word_bank_state.set_data_version.update(|ver| *ver += 1);
                word_bank_state
                    .set_source_name
                    .set(format!("系列词库：{series_name}（{selected_file}）"));
                set_status.set(format!(
                    "已加载{series_name}词库，共 {count} 条。正在进入练习页面。"
                ));
                set_current_page.set(target_page);
            }
            Err(err) => {
                set_status.set(format!("{series_name}词库加载失败：{err}"));
            }
        }
    });
}
