use gloo_net::http::Request;
use leptos::ev::Event;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app_state::WordBankState;
use crate::components::import_csv::ImportCsvButton;
use crate::components::lexicon_browser::parse_word_bank_csv;
use crate::pages::AppPage;
use crate::structures::pracresult::PracticeResult;

#[component]
pub fn HomePage() -> impl IntoView {
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

    let import_pracresult_change = move |ev: Event| {
        import_pracresult_from_file(ev, set_status, word_bank_state.set_practice_result);
    };

    let direct_start_practice_click = move |_| {
        let mut practice_result = word_bank_state.practice_result.get_untracked();
        if practice_result.selected_word_entry_ids.is_empty() {
            practice_result.selected_word_entry_ids =
                word_bank_state.selected_word_entry_ids.get_untracked();
        }
        word_bank_state.set_practice_result.set(practice_result);
        word_bank_state
            .set_temp_practice_result
            .set(PracticeResult::default());
        set_current_page.set(AppPage::PracticeModeSelect);
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
            <section class="w-full max-w-2xl p-8 rounded-2xl border border-slate-800 bg-slate-900 shadow-xl">
                <h1 class="text-3xl font-bold tracking-tight">"Norwegian Word Practice"</h1>
                <p class="mt-4 text-slate-300">
                    "导入你的练习结果（.pracresult）后开始练习。"
                </p>
                <p class="mt-2 text-sm text-slate-400">
                    {move || {
                        let count = word_bank_state.entries.get().len();
                        let source = word_bank_state.source_name.get();
                        if count == 0 {
                            format!("当前词库：{source}。")
                        } else {
                            format!("当前词库：{source}（共 {count} 条）。")
                        }
                    }}
                </p>
                <p class="mt-1 min-h-5 text-sm text-slate-300">{move || status.get()}</p>

                <input
                    id="pracresult-input"
                    type="file"
                    accept=".pracresult"
                    class="hidden"
                    on:change=import_pracresult_change
                />

                <div class="mt-6 grid grid-cols-1 gap-3 md:grid-cols-[1fr_auto_auto]">
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
                        input_id="home-import-csv-input".to_string()
                        label="导入词库 CSV".to_string()
                        class="inline-flex cursor-pointer items-center justify-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700".to_string()
                        set_entries=word_bank_state.set_entries
                        set_status=set_status
                        set_data_version=word_bank_state.set_data_version
                        set_source_name=word_bank_state.set_source_name
                    />
                </div>

                <div class="mt-3 flex flex-wrap items-center gap-3">
                    <label
                        for="pracresult-input"
                        class="inline-flex cursor-pointer items-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                    >
                        "导入练习结果"
                    </label>

                    <button
                        type="button"
                        on:click=direct_start_practice_click
                        class="inline-flex items-center rounded-lg border border-slate-700 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600"
                    >
                        "直接开始练习"
                    </button>
                </div>
            </section>
        </main>
    }
}

fn import_pracresult_from_file(
    ev: Event,
    set_status: WriteSignal<String>,
    set_practice_result: WriteSignal<PracticeResult>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let Some(target) = ev.target() else {
            set_status.set("导入失败：无法获取文件输入目标。".to_string());
            return;
        };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
            set_status.set("导入失败：文件输入类型不正确。".to_string());
            return;
        };
        let Some(files) = input.files() else {
            set_status.set("导入失败：未找到文件列表。".to_string());
            return;
        };
        let Some(file) = files.get(0) else {
            set_status.set("导入已取消。".to_string());
            return;
        };

        let file: web_sys::File = file;
        let file_name = file.name();
        input.set_value("");

        spawn_local(async move {
            let content = match wasm_bindgen_futures::JsFuture::from(file.text()).await {
                Ok(js_value) => match js_value.as_string() {
                    Some(content) => content,
                    None => {
                        set_status.set("导入失败：文件内容不是文本。".to_string());
                        return;
                    }
                },
                Err(err) => {
                    set_status.set(format!("导入失败：读取文件失败，{err:?}"));
                    return;
                }
            };

            match serde_json::from_str::<PracticeResult>(&content) {
                Ok(result) => {
                    set_practice_result.set(result);
                    set_status.set(format!("练习结果导入成功：{file_name}"));
                }
                Err(err) => {
                    set_status.set(format!("导入失败：练习结果格式不合法，{err}"));
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ev;
        let _ = set_practice_result;
        set_status.set("导入仅在浏览器环境可用。".to_string());
    }
}
