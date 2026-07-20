//! 通用词库导入按钮：支持明文 CSV 与加密 `.nwpdict`，读取后覆盖全局词库状态。

use leptos::ev::Event;
use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use leptos::task::spawn_local;

use crate::structures::word_bank_entry::{UiLanguage, WordBankEntry};
use crate::utils::i18n::tr;
use crate::utils::lexicon_file::LEXICON_FILE_INPUT_ACCEPT;
#[cfg(target_arch = "wasm32")]
use crate::utils::lexicon_file::parse_lexicon_entries_from_text;

#[component]
pub fn ImportCsvButton(
    input_id: String,
    #[prop(into)] label: Signal<String>,
    class: String,
    set_entries: WriteSignal<Vec<WordBankEntry>>,
    set_status: WriteSignal<String>,
    set_data_version: WriteSignal<u64>,
    #[prop(optional)] ui_language: Option<ReadSignal<UiLanguage>>,
    #[prop(optional)] set_source_name: Option<WriteSignal<String>>,
) -> impl IntoView {
    let import_csv_click = move |ev: Event| {
        import_csv_from_file(
            ev,
            set_entries,
            set_status,
            set_data_version,
            ui_language,
            set_source_name,
        );
    };

    view! {
        <input
            id=input_id.clone()
            type="file"
            accept=LEXICON_FILE_INPUT_ACCEPT
            class="hidden"
            on:change=import_csv_click
        />
        <label for=input_id class=class>
            {move || label.get()}
        </label>
    }
}

fn import_csv_from_file(
    ev: Event,
    set_entries: WriteSignal<Vec<WordBankEntry>>,
    set_status: WriteSignal<String>,
    set_data_version: WriteSignal<u64>,
    ui_language: Option<ReadSignal<UiLanguage>>,
    set_source_name: Option<WriteSignal<String>>,
) {
    let lang = ui_language
        .map(|signal| signal.get_untracked())
        .unwrap_or(UiLanguage::Zh);
    #[cfg(target_arch = "wasm32")]
    {
        // 浏览器端读取本地文件，解析后覆盖词库并触发版本变更。
        use wasm_bindgen::JsCast;

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

            match parse_lexicon_entries_from_text(&content) {
                Ok(word_list) => {
                    let count = word_list.len();
                    // 导入语义为“覆盖当前词库”，并 bump `data_version` 触发依赖组件重置草稿。
                    set_entries.set(word_list);
                    set_data_version.update(|ver| *ver += 1);
                    if let Some(set_source_name) = set_source_name {
                        set_source_name.set(format!(
                            "{}：{file_name}",
                            tr(lang, "本地导入", "Local Import")
                        ));
                    }
                    set_status.set(format!(
                        "{} {} {}",
                        tr(
                            lang,
                            "词库导入成功（已覆盖），共",
                            "Lexicon imported (overwritten),"
                        ),
                        count,
                        tr(lang, "条。", "entries.")
                    ));
                }
                Err(err) => {
                    set_status.set(format!(
                        "{} {err}",
                        tr(lang, "导入失败：", "Import failed:")
                    ));
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ev;
        let _ = set_entries;
        let _ = set_data_version;
        let _ = ui_language;
        let _ = set_source_name;
        set_status.set(
            tr(
                lang,
                "导入仅在浏览器环境可用。",
                "Import is only available in browser environment.",
            )
            .to_string(),
        );
    }
}
