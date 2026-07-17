//! 首页：支持导入练习结果文件，或直接进入练习模式页。

use leptos::ev::Event;
use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use leptos::task::spawn_local;

use crate::app_state::{LexiconState, PracticeState, UiState};
use crate::components::mini_console::MiniConsole;
use crate::pages::AppPage;
use crate::structures::pracresult::PracticeResult;
use crate::structures::word_bank_entry::UiLanguage;
use crate::utils::i18n::tr;
#[cfg(target_arch = "wasm32")]
use crate::utils::pracresult_crypto::parse_practice_result_from_import;

#[component]
pub fn HomePage() -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let lexicon_state = expect_context::<LexiconState>();
    let practice_state = expect_context::<PracticeState>();
    let ui_state = expect_context::<UiState>();
    let lang = ui_state.ui_language;
    let (status, set_status) = signal(String::new());

    let import_pracresult_change = move |ev: Event| {
        import_pracresult_from_file(
            ev,
            set_status,
            set_current_page,
            practice_state.set_practice_result,
            practice_state.set_selected_word_entry_ids,
            lang,
        );
    };

    let direct_start_practice_click = move |_| {
        let mut practice_result = practice_state.practice_result.get_untracked();
        if practice_result.selected_word_entry_ids.is_empty() {
            practice_result.selected_word_entry_ids =
                practice_state.selected_word_entry_ids.get_untracked();
        }
        practice_state.set_practice_result.set(practice_result);
        practice_state
            .set_temp_practice_result
            .set(PracticeResult::default());
        set_current_page.set(AppPage::PracticeModeSelect);
    };
    let current_lexicon_line = Signal::derive(move || {
        let language = lang.get();
        let count = lexicon_state.entries.get().len();
        let source = lexicon_state.source_name.get();
        if count == 0 {
            tr(
                language,
                "尚未加载词库，请先进入练习模式页选择词库。",
                "No lexicon loaded yet. Choose one in Practice Mode first.",
            )
            .to_string()
        } else {
            format!(
                "{}：{source}（{} {count} {}）。",
                tr(language, "当前词库", "Current Lexicon"),
                tr(language, "共", "total"),
                tr(language, "条词条", "entries")
            )
        }
    });
    let console_status_line = Signal::derive(move || {
        let language = lang.get();
        let s = status.get();
        if s.trim().is_empty() {
            tr(language, "等待操作...", "Waiting for action...").to_string()
        } else {
            s
        }
    });

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-start justify-center p-3 sm:p-6">
            <section class="w-full max-w-2xl rounded-2xl border border-slate-800 bg-slate-900 p-4 sm:p-8 shadow-xl">
                <h1 class="text-2xl sm:text-3xl font-bold tracking-tight">"Norwegian Word Practice"</h1>
                <p class="mt-3 text-xs text-slate-400">{move || current_lexicon_line.get()}</p>
                <MiniConsole message=console_status_line max_entries=80/>
                <p class="mt-4 text-slate-300">
                    {move || {
                        tr(
                            lang.get(),
                            "导入你的练习结果（.pracresult）后开始练习。",
                            "Import your practice result (.pracresult) and start practicing.",
                        )
                    }}
                </p>

                <input
                    id="pracresult-input"
                    type="file"
                    accept=".pracresult"
                    class="hidden"
                    on:change=import_pracresult_change
                />

                <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
                    <button
                        type="button"
                        on:click=direct_start_practice_click
                        class="inline-flex w-full items-center justify-center rounded-lg border border-slate-700 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600 sm:w-auto"
                    >
                        {move || tr(lang.get(), "直接开始练习", "Start Practice")}
                    </button>

                    <label
                        for="pracresult-input"
                        class="inline-flex w-full cursor-pointer items-center justify-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700 sm:w-auto"
                    >
                        {move || tr(lang.get(), "导入练习结果", "Import Practice Result")}
                    </label>


                </div>
            </section>
        </main>
    }
}

fn import_pracresult_from_file(
    ev: Event,
    set_status: WriteSignal<String>,
    set_current_page: WriteSignal<AppPage>,
    set_practice_result: WriteSignal<PracticeResult>,
    set_selected_word_entry_ids: WriteSignal<Vec<String>>,
    ui_language: ReadSignal<UiLanguage>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        // 读取并解析 `.pracresult`，成功后恢复历史结果并进入模式页。
        use wasm_bindgen::JsCast;

        let lang = ui_language.get_untracked();
        let Some(target) = ev.target() else {
            set_status.set(
                tr(
                    lang,
                    "导入失败：无法获取文件输入目标。",
                    "Import failed: cannot get file input target.",
                )
                .to_string(),
            );
            return;
        };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
            set_status.set(
                tr(
                    lang,
                    "导入失败：文件输入类型不正确。",
                    "Import failed: invalid file input type.",
                )
                .to_string(),
            );
            return;
        };
        let Some(files) = input.files() else {
            set_status.set(
                tr(
                    lang,
                    "导入失败：未找到文件列表。",
                    "Import failed: file list not found.",
                )
                .to_string(),
            );
            return;
        };
        let Some(file) = files.get(0) else {
            set_status.set(tr(lang, "导入已取消。", "Import cancelled.").to_string());
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
                        set_status.set(
                            tr(
                                lang,
                                "导入失败：文件内容不是文本。",
                                "Import failed: file content is not text.",
                            )
                            .to_string(),
                        );
                        return;
                    }
                },
                Err(err) => {
                    set_status.set(format!(
                        "{} {err:?}",
                        tr(
                            lang,
                            "导入失败：读取文件失败，",
                            "Import failed: read file error,"
                        )
                    ));
                    return;
                }
            };

            match parse_practice_result_from_import(&content) {
                Ok(result) => {
                    set_selected_word_entry_ids.set(result.selected_word_entry_ids.clone());
                    set_practice_result.set(result);
                    set_status.set(format!(
                        "{}：{file_name}",
                        tr(
                            lang,
                            "练习结果导入成功",
                            "Practice result imported successfully"
                        )
                    ));
                    set_current_page.set(AppPage::PracticeModeSelect);
                }
                Err(err) => {
                    set_status.set(format!(
                        "{} {err}",
                        tr(
                            lang,
                            "导入失败：练习结果解密或解析失败，",
                            "Import failed: decrypt/parse error,"
                        )
                    ));
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ev;
        let _ = set_current_page;
        let _ = set_practice_result;
        let _ = set_selected_word_entry_ids;
        set_status.set(
            tr(
                ui_language.get_untracked(),
                "导入仅在浏览器环境可用。",
                "Import is only available in browser environment.",
            )
            .to_string(),
        );
    }
}
