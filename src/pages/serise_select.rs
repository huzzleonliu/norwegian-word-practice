//! 系列练习选择页：按标签加载系列词库并跳转到对应系列练习页面。

use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app_state::{LexiconState, NavigateToPage, PracticeState, UiState};
use crate::components::lexicon_browser::parse_word_bank_csv;
use crate::components::mini_console::MiniConsole;
use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;
use crate::structures::word_bank_entry::UiLanguage;
use crate::utils::i18n::tr;

#[component]
pub fn SeriseSelectPage() -> impl IntoView {
    let set_current_page = expect_context::<NavigateToPage>();
    let lexicon_state = expect_context::<LexiconState>();
    let practice_state = expect_context::<PracticeState>();
    let lang = expect_context::<UiState>().ui_language;
    let (status, set_status) = signal(
        tr(
            lang.get_untracked(),
            "请选择一个单词系列开始练习。",
            "Please choose a series to start practice.",
        )
        .to_string(),
    );

    let cardinal_click = move |_| {
        load_series_word_bank(
            &["cardinal_number", "ordinal_number"],
            "数词",
            "Number",
            AppPage::SeriseNumberPractice,
            lexicon_state,
            practice_state,
            set_status,
            set_current_page,
            lang,
        );
    };
    let month_click = move |_| {
        load_series_word_bank(
            &["month"],
            "月份",
            "Month",
            AppPage::SeriseMonthPractice,
            lexicon_state,
            practice_state,
            set_status,
            set_current_page,
            lang,
        );
    };
    let pronoun_click = move |_| {
        load_series_word_bank(
            &["pronoun"],
            "代词",
            "Pronoun",
            AppPage::SerisePronounPractice,
            lexicon_state,
            practice_state,
            set_status,
            set_current_page,
            lang,
        );
    };
    let interrogative_click = move |_| {
        load_series_word_bank(
            &["interrogative"],
            "疑问词",
            "Interrogative",
            AppPage::SeriseInterrogativePractice,
            lexicon_state,
            practice_state,
            set_status,
            set_current_page,
            lang,
        );
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-3 sm:p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-4 sm:p-6 shadow-xl">
                <ReturnButton target_page=AppPage::PracticeModeSelect/>
                <h1 class="text-2xl sm:text-3xl font-bold tracking-tight">
                    {move || tr(lang.get(), "单词系列练习", "Series Practice")}
                </h1>
                <MiniConsole
                    message=Signal::derive(move || {
                        let current = status.get();
                        format!("{}\n{current}", tr(lang.get(), "系列模式状态", "Series mode status"))
                    })
                />

                <div class="mt-6 grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4">
                    <button
                        type="button"
                        on:click=cardinal_click
                        class="w-full rounded-xl border border-slate-700 bg-slate-800 px-4 py-6 text-center text-base font-semibold text-slate-100 hover:bg-slate-700"
                    >
                        {move || tr(lang.get(), "数词", "Number")}
                    </button>
                    <button
                        type="button"
                        on:click=month_click
                        class="w-full rounded-xl border border-slate-700 bg-slate-800 px-4 py-6 text-center text-base font-semibold text-slate-100 hover:bg-slate-700"
                    >
                        {move || tr(lang.get(), "月份", "Month")}
                    </button>
                    <button
                        type="button"
                        on:click=pronoun_click
                        class="w-full rounded-xl border border-slate-700 bg-slate-800 px-4 py-6 text-center text-base font-semibold text-slate-100 hover:bg-slate-700"
                    >
                        {move || tr(lang.get(), "代词", "Pronoun")}
                    </button>
                    <button
                        type="button"
                        on:click=interrogative_click
                        class="w-full rounded-xl border border-slate-700 bg-slate-800 px-4 py-6 text-center text-base font-semibold text-slate-100 hover:bg-slate-700"
                    >
                        {move || tr(lang.get(), "疑问词", "Interrogative")}
                    </button>
                </div>
            </section>
        </main>
    }
}

fn load_series_word_bank(
    selected_tags: &'static [&'static str],
    series_name_zh: &'static str,
    series_name_en: &'static str,
    target_page: AppPage,
    lexicon_state: LexiconState,
    practice_state: PracticeState,
    set_status: WriteSignal<String>,
    set_current_page: NavigateToPage,
    lang: ReadSignal<UiLanguage>,
) {
    // 系列模式统一加载 `series-word-bank.csv`，再按 tag 过滤选题范围。
    let selected_file = "series-word-bank.csv";
    let language = lang.get_untracked();
    let series_name = tr(language, series_name_zh, series_name_en);
    set_status.set(format!(
        "{} {series_name} {}：{selected_file} ...",
        tr(language, "正在加载", "Loading"),
        tr(language, "词库", "lexicon")
    ));

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
                    .filter(|entry| {
                        entry
                            .tags
                            .iter()
                            .any(|tag| selected_tags.iter().any(|expected| tag == expected))
                    })
                    .map(|entry| entry.id.clone())
                    .collect::<Vec<_>>();
                if selected_ids.is_empty() {
                    set_status.set(format!(
                        "{} {series_name} {}",
                        tr(language, "加载失败：", "Load failed:"),
                        tr(
                            language,
                            "未找到与该系列标签匹配的词条。",
                            "No entries matched the selected series tags.",
                        )
                    ));
                    return;
                }
                lexicon_state.set_entries.set(word_list);
                practice_state.set_selected_word_entry_ids.set(selected_ids);
                lexicon_state.set_data_version.update(|ver| *ver += 1);
                lexicon_state.set_source_name.set(format!(
                    "{}：{series_name}（{selected_file}; tags={}）",
                    tr(language, "系列词库", "Series Lexicon"),
                    selected_tags.join("|")
                ));
                set_status.set(format!(
                    "{} {series_name} {} {count} {}",
                    tr(language, "已加载", "Loaded"),
                    tr(language, "词库，共", "lexicon,"),
                    tr(
                        language,
                        "条。正在进入练习页面。",
                        "entries. Entering practice page."
                    )
                ));
                set_current_page.set(target_page);
            }
            Err(err) => {
                set_status.set(format!(
                    "{series_name} {} {err}",
                    tr(language, "词库加载失败：", "lexicon loading failed:")
                ));
            }
        }
    });
}
