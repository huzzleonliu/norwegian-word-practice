//! 首页：支持导入练习结果文件，或直接进入练习模式页。

use leptos::ev::Event;
use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use leptos::task::spawn_local;

use crate::app_state::{LexiconState, NavigateToPage, PracticeState, UiState};
use crate::layout::PageShell;
use crate::components::mini_console::MiniConsole;
use crate::pages::AppPage;
use crate::structures::pracresult::PracticeResult;
use crate::structures::word_bank_entry::UiLanguage;
use crate::utils::i18n::tr;
#[cfg(target_arch = "wasm32")]
use crate::utils::pracresult_crypto::parse_practice_result_from_import;

#[component]
pub fn HomePage() -> impl IntoView {
    let set_current_page = expect_context::<NavigateToPage>();
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
        <PageShell narrow=true>
            <p class="ui-eyebrow">
                {move || tr(lang.get(), "拼写练习", "spelling + inflection")}
            </p>
            <h1 class="ui-brand">
                <span class="brand-read">"Norwegian"</span>
                " "
                <span class="brand-er">"Word"</span>
                " "
                <span class="brand-err">"Practice"</span>
            </h1>
            <p class="ui-muted">{move || current_lexicon_line.get()}</p>
            <MiniConsole message=console_status_line max_entries=80/>
            <p class="ui-copy">
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

            <div class="ui-card-actions mt-6">
                <button
                    type="button"
                    on:click=direct_start_practice_click
                    class="ui-btn-primary"
                >
                    {move || tr(lang.get(), "直接开始练习", "Start Practice")}
                </button>

                <label for="pracresult-input" class="ui-btn-ghost cursor-pointer">
                    {move || tr(lang.get(), "导入练习结果", "Import Practice Result")}
                </label>
            </div>
        </PageShell>
    }
}

fn import_pracresult_from_file(
    ev: Event,
    set_status: WriteSignal<String>,
    set_current_page: NavigateToPage,
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
