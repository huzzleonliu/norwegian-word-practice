//! 词库浏览器搜索面板：包含关键字输入、列可见性选择和搜索触发按钮。

use leptos::prelude::*;

use super::utils::{DATA_COLUMN_KEYS, header_name, search_column_groups};
use crate::structures::word_bank_entry::{UiLanguage, WordBankEntry};
use crate::utils::i18n::{field_label, tr};
use crate::utils::lexicon_file::{
    LEXICON_EXPORT_FILE_NAME, serialize_encrypted_lexicon_from_entries,
};

/// 导出当前已提交词库为加密 `.nwpdict`（草稿需先「确认修改」）。
#[component]
pub fn ExportLexiconFileButton(
    lang: ReadSignal<UiLanguage>,
    entries: ReadSignal<Vec<WordBankEntry>>,
    set_status: WriteSignal<String>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_else(|| {
        "w-full rounded border border-slate-700 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600 sm:w-auto"
            .to_string()
    });

    let on_click = move |_| {
        let language = lang.get_untracked();
        let encrypted_payload = match serialize_encrypted_lexicon_from_entries(&entries.get_untracked())
        {
            Ok(content) => content,
            Err(err) => {
                set_status.set(format!(
                    "{} {err}",
                    tr(language, "导出失败：", "Export failed:")
                ));
                return;
            }
        };

        match export_lexicon_download(LEXICON_EXPORT_FILE_NAME, &encrypted_payload) {
            Ok(()) => set_status.set(
                tr(
                    language,
                    "加密词库已导出（.nwpdict）。",
                    "Encrypted lexicon exported (.nwpdict).",
                )
                .to_string(),
            ),
            Err(err) => set_status.set(format!(
                "{} {err}",
                tr(language, "导出失败：", "Export failed:")
            )),
        }
    };

    view! {
        <button type="button" class=class on:click=on_click>
            {move || tr(lang.get(), "导出词库文件", "Export Lexicon File")}
        </button>
    }
}

/// 浏览器端下载词库文件（WASM 环境通过 Blob + ObjectURL 触发保存）。
fn export_lexicon_download(filename: &str, content: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::{JsCast, JsValue};

        let parts = js_sys::Array::new();
        parts.push(&JsValue::from_str(content));
        let blob = web_sys::Blob::new_with_str_sequence(&parts)
            .map_err(|_| "无法创建导出文件 Blob".to_string())?;
        let object_url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|_| "无法创建下载 URL".to_string())?;

        let window = web_sys::window().ok_or_else(|| "无法获取 window".to_string())?;
        let document = window
            .document()
            .ok_or_else(|| "无法获取 document".to_string())?;
        let anchor = document
            .create_element("a")
            .map_err(|_| "无法创建下载节点".to_string())?
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .map_err(|_| "无法转换下载节点".to_string())?;

        anchor.set_href(&object_url);
        anchor.set_download(filename);

        let body = document
            .body()
            .ok_or_else(|| "页面 body 不存在".to_string())?;
        body.append_child(&anchor)
            .map_err(|_| "无法挂载下载节点".to_string())?;
        anchor.click();
        anchor.remove();

        web_sys::Url::revoke_object_url(&object_url).map_err(|_| "无法释放下载 URL".to_string())?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = filename;
        let _ = content;
        Err("导出仅在浏览器环境可用。".to_string())
    }
}

#[component]
pub fn LexiconSearchPanel(
    lang: ReadSignal<UiLanguage>,
    search_text: ReadSignal<String>,
    set_search_text: WriteSignal<String>,
    search_columns: ReadSignal<Vec<bool>>,
    set_search_columns: WriteSignal<Vec<bool>>,
    search_scope_expanded: ReadSignal<bool>,
    set_search_scope_expanded: WriteSignal<bool>,
    on_apply_search: Callback<()>,
) -> impl IntoView {
    const DATA_COLUMN_COUNT: usize = DATA_COLUMN_KEYS.len();

    view! {
        <div class="mb-4 rounded-lg border border-slate-800 bg-slate-900/40 p-3">
                <div class="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center">
                    <input
                        type="text"
                        placeholder=move || tr(lang.get(), "输入要查找的字符", "Type to search")
                        prop:value=move || search_text.get()
                        on:input=move |ev| set_search_text.set(event_target_value(&ev))
                        class="w-full min-w-0 rounded border border-slate-700 bg-slate-950 px-3 py-2 text-sm sm:min-w-60 sm:flex-1"
                    />
                    <button
                        type="button"
                        on:click=move |_| on_apply_search.run(())
                        class="w-full rounded border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium hover:bg-slate-700 sm:w-auto"
                    >
                        {move || tr(lang.get(), "查找", "Search")}
                    </button>
                    <button
                        type="button"
                        on:click=move |_| set_search_columns.set(vec![true; DATA_COLUMN_COUNT])
                        class="w-full rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium hover:bg-slate-700 sm:w-auto"
                    >
                        {move || tr(lang.get(), "全选列", "Select all columns")}
                    </button>
                    <button
                        type="button"
                        on:click=move |_| set_search_columns.set(vec![false; DATA_COLUMN_COUNT])
                        class="w-full rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium hover:bg-slate-700 sm:w-auto"
                    >
                        {move || tr(lang.get(), "全不选列", "Unselect all columns")}
                    </button>
                </div>
                <div class="mt-3">
                    <button
                        type="button"
                        on:click=move |_| {
                            set_search_scope_expanded.update(|expanded| *expanded = !*expanded)
                        }
                        class="inline-flex w-full items-center justify-center gap-2 rounded border border-slate-700 bg-slate-900 px-3 py-1 text-xs font-medium text-slate-200 hover:bg-slate-800 sm:w-auto sm:justify-start"
                    >
                        {move || {
                            if search_scope_expanded.get() {
                                tr(lang.get(), "隐藏搜索列范围", "Hide Search Columns")
                            } else {
                                tr(lang.get(), "展开搜索列范围", "Show Search Columns")
                            }
                        }}
                    </button>
                    {move || {
                        if search_scope_expanded.get() {
                            view! {
                                <div class="mt-3 space-y-3">
                                    {search_column_groups()
                                        .into_iter()
                                        .map(|(group_name, indices)| {
                                            view! {
                                                <section class="rounded border border-slate-800 bg-slate-950/40 p-2">
                                                    <p class="mb-2 text-xs font-semibold text-slate-400">
                                                        {move || match group_name {
                                                            "core" => tr(lang.get(), "基础组", "Core"),
                                                            "verb" => tr(lang.get(), "动词变体组", "Verb Forms"),
                                                            "noun" => tr(lang.get(), "名词变体组", "Noun Forms"),
                                                            "adjective" => tr(lang.get(), "形容词变体组", "Adjective Forms"),
                                                            "pronoun" => tr(lang.get(), "代词变体组", "Pronoun Forms"),
                                                            "determinative" => tr(lang.get(), "限定词变体组", "Determinative Forms"),
                                                            "adverb" => tr(lang.get(), "副词变体组", "Adverb Forms"),
                                                            _ => group_name,
                                                        }}
                                                    </p>
                                                    <div class="grid grid-cols-1 gap-2 sm:grid-cols-2 md:grid-cols-4 lg:grid-cols-5">
                                                        {indices
                                                            .into_iter()
                                                            .map(|idx| {
                                                                view! {
                                                                    <label class="inline-flex items-center gap-2 rounded border border-slate-800 bg-slate-950/60 px-2 py-1 text-xs text-slate-300">
                                                                        <input
                                                                            type="checkbox"
                                                                            prop:checked=move || {
                                                                                search_columns.get().get(idx).copied().unwrap_or(false)
                                                                            }
                                                                            on:change=move |ev| {
                                                                                let checked = event_target_checked(&ev);
                                                                                set_search_columns.update(|cols| {
                                                                                    if idx < cols.len() {
                                                                                        cols[idx] = checked;
                                                                                    }
                                                                                });
                                                                            }
                                                                        />
                                                                        <span>{move || field_label(lang.get(), header_name(idx))}</span>
                                                                    </label>
                                                                }
                                                            })
                                                            .collect_view()}
                                                    </div>
                                                </section>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                                .into_any()
                        } else {
                            view! { <></> }.into_any()
                        }
                    }}
                </div>
            </div>
    }
}
