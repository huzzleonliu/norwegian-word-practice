use leptos::ev::SubmitEvent;
use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::components::import_csv::ImportCsvButton;
use crate::components::lexicon_browser::{
    LexiconBrowser, LexiconBrowserMode, PART_OF_SPEECH_OPTIONS, parse_pipe_list,
    parse_word_bank_csv, serialize_word_bank_csv,
};
use crate::components::mini_console::MiniConsole;
use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;
use crate::utils::dictionary::{
    SingleEntryDraft, draft_from_word_entry, parse_part_of_speech,
    validate_and_prepare_single_entry,
};

#[component]
pub fn LocalLexiconEditorPage() -> impl IntoView {
    let word_bank_state = expect_context::<WordBankState>();
    let entries = word_bank_state.entries;
    let set_entries = word_bank_state.set_entries;
    let data_version = word_bank_state.data_version;
    let set_data_version = word_bank_state.set_data_version;
    let (status, set_status) = signal({
        let count = entries.get_untracked().len();
        let source = word_bank_state.source_name.get_untracked();
        if count == 0 {
            "当前词库为空，请先回到首页加载或导入词库。".to_string()
        } else {
            format!("当前词库：{source}（共 {count} 条）。")
        }
    });

    let (single_selected, set_single_selected) = signal(true);
    let (single_pos, set_single_pos) = signal("noun".to_string());
    let (single_norwegian, set_single_norwegian) = signal(String::new());
    let (single_chinese, set_single_chinese) = signal(String::new());
    let (single_english, set_single_english) = signal(String::new());
    let (single_tags, set_single_tags) = signal(String::new());
    let (single_past_tense, set_single_past_tense) = signal(String::new());
    let (single_imperative, set_single_imperative) = signal(String::new());
    let (single_plural, set_single_plural) = signal(String::new());
    let (single_singular_definite, set_single_singular_definite) = signal(String::new());
    let (single_plural_definite, set_single_plural_definite) = signal(String::new());
    let (single_neuter_form, set_single_neuter_form) = signal(String::new());
    let (single_plural_form, set_single_plural_form) = signal(String::new());
    let (single_adjective_comparative, set_single_adjective_comparative) = signal(String::new());
    let (single_adjective_superlative_indefinite, set_single_adjective_superlative_indefinite) =
        signal(String::new());
    let (single_adjective_superlative_definite, set_single_adjective_superlative_definite) =
        signal(String::new());
    let (single_adverb_comparative, set_single_adverb_comparative) = signal(String::new());
    let (single_adverb_superlative, set_single_adverb_superlative) = signal(String::new());

    let (bulk_input, set_bulk_input) = signal(String::new());
    let (bulk_errors, set_bulk_errors) = signal(Vec::<String>::new());
    let (bulk_success_message, set_bulk_success_message) = signal(String::new());
    let add_single_entry = move |ev: SubmitEvent| {
        ev.prevent_default();

        let draft: SingleEntryDraft = SingleEntryDraft {
            id: String::new(),
            selected: single_selected.get(),
            part_of_speech: match parse_part_of_speech(&single_pos.get()) {
                Ok(value) => value,
                Err(err) => {
                    set_status.set(format!("单条添加失败：{err}"));
                    return;
                }
            },
            tags: parse_csv_list(&single_tags.get()),
            english: parse_csv_list(&single_english.get()),
            chinese: parse_csv_list(&single_chinese.get()),
            base_form: single_norwegian.get(),
            past_tense: parse_optional_input(&single_past_tense.get()),
            imperative: parse_optional_input(&single_imperative.get()),
            plural: parse_optional_input(&single_plural.get()),
            singular_definite: parse_optional_input(&single_singular_definite.get()),
            plural_definite: parse_optional_input(&single_plural_definite.get()),
            neuter_form: parse_optional_input(&single_neuter_form.get()),
            plural_form: parse_optional_input(&single_plural_form.get()),
            adjective_comparative: parse_optional_input(&single_adjective_comparative.get()),
            adjective_superlative_indefinite: parse_optional_input(
                &single_adjective_superlative_indefinite.get(),
            ),
            adjective_superlative_definite: parse_optional_input(
                &single_adjective_superlative_definite.get(),
            ),
            adverb_comparative: parse_optional_input(&single_adverb_comparative.get()),
            adverb_superlative: parse_optional_input(&single_adverb_superlative.get()),
        };
        let existing_entries = entries.get_untracked();
        let new_entry = match validate_and_prepare_single_entry(draft, &existing_entries) {
            Ok(entry) => entry,
            Err(err) => {
                set_status.set(format!("单条添加失败：{err}"));
                return;
            }
        };

        let generated_id = new_entry.id.clone();
        let mut total_after_add = existing_entries.len();
        set_entries.update(|list| {
            list.push(new_entry);
            total_after_add = list.len();
        });
        set_data_version.update(|ver| *ver += 1);
        set_status.set(format!(
            "已添加 1 条到本地缓存词库（id: {generated_id}），当前共 {total_after_add} 条。"
        ));

        set_single_selected.set(true);
        set_single_pos.set("noun".to_string());
        set_single_norwegian.set(String::new());
        set_single_chinese.set(String::new());
        set_single_english.set(String::new());
        set_single_tags.set(String::new());
        set_single_past_tense.set(String::new());
        set_single_imperative.set(String::new());
        set_single_plural.set(String::new());
        set_single_singular_definite.set(String::new());
        set_single_plural_definite.set(String::new());
        set_single_neuter_form.set(String::new());
        set_single_plural_form.set(String::new());
        set_single_adjective_comparative.set(String::new());
        set_single_adjective_superlative_indefinite.set(String::new());
        set_single_adjective_superlative_definite.set(String::new());
        set_single_adverb_comparative.set(String::new());
        set_single_adverb_superlative.set(String::new());
    };

    let add_bulk_entries = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_bulk_errors.set(Vec::new());
        set_bulk_success_message.set(String::new());

        let content = bulk_input.get();
        if content.trim().is_empty() {
            set_status.set("多条添加失败：文本框不能为空。".to_string());
            set_bulk_errors.set(vec!["输入不能为空。请粘贴带表头的 CSV 文本。".to_string()]);
            return;
        }

        match parse_word_bank_csv(&content) {
            Ok(parsed) => {
                if parsed.is_empty() {
                    set_status.set("多条添加失败：未检测到可导入条目。".to_string());
                    set_bulk_errors.set(vec!["CSV 中没有可导入的数据行。".to_string()]);
                    return;
                }

                let mut simulated_entries = entries.get_untracked();
                let mut validated_entries = Vec::with_capacity(parsed.len());
                let mut errors = Vec::new();

                for (index, row) in parsed.into_iter().enumerate() {
                    let line_no = index + 2;
                    let base_form_label = if row.base_form.trim().is_empty() {
                        "（原型为空）".to_string()
                    } else {
                        format!("（原型：{}）", row.base_form.trim())
                    };

                    let draft = match draft_from_word_entry(&row) {
                        Ok(draft) => draft,
                        Err(err) => {
                            errors.push(format!("第 {line_no} 行 {base_form_label}：{err}"));
                            continue;
                        }
                    };

                    match validate_and_prepare_single_entry(draft, &simulated_entries) {
                        Ok(valid_entry) => {
                            simulated_entries.push(valid_entry.clone());
                            validated_entries.push(valid_entry);
                        }
                        Err(err) => {
                            errors.push(format!("第 {line_no} 行 {base_form_label}：{err}"));
                        }
                    }
                }

                if !errors.is_empty() {
                    set_bulk_errors.set(errors);
                    set_status.set("批量添加失败：存在不合法条目，请先修正红字错误。".to_string());
                    return;
                }

                let add_count = validated_entries.len();
                set_entries.update(|list| list.extend(validated_entries));
                set_data_version.update(|ver| *ver += 1);
                set_status.set(format!("批量添加成功，新增 {} 条。", add_count));
                set_bulk_success_message.set("成功导入".to_string());
            }
            Err(err) => {
                set_status.set(format!("多条添加失败：请输入带表头的 CSV 文本。{err}"));
                set_bulk_errors.set(vec![format!("CSV 解析失败：{err}")]);
            }
        }
    };
    let export_csv_click = move |_| {
        let csv_content = match serialize_word_bank_csv(&entries.get_untracked()) {
            Ok(content) => content,
            Err(err) => {
                set_status.set(format!("导出失败：{err}"));
                return;
            }
        };

        match export_csv_download("word-bank.csv", &csv_content) {
            Ok(()) => set_status.set("词库 CSV 已导出。".to_string()),
            Err(err) => set_status.set(format!("导出失败：{err}")),
        }
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-xl">
                <ReturnButton target_page=AppPage::PracticeModeSelect/>
                <header class="mb-6 flex items-center gap-4">
                    <h1 class="text-2xl font-bold tracking-tight">"本地词库修改器"</h1>
                </header>

                <MiniConsole
                    message=Signal::derive(move || {
                        let status_line = {
                            let s = status.get();
                            if s.trim().is_empty() {
                                "等待词库编辑操作...".to_string()
                            } else {
                                s
                            }
                        };
                        format!("{status_line}\n编辑后请点击“确认修改”再导出。")
                    })
                />

                <form
                    on:submit=add_single_entry
                    class="rounded-xl border border-slate-800 bg-slate-950/50 p-4"
                >
                    <h2 class="mb-3 text-lg font-semibold">"单条添加"</h2>
                    <p class="mb-3 text-xs text-slate-400">
                        "序号会自动使用原型值生成；如重复将自动追加后缀（如 -2）。"
                    </p>
                    <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
                        <label class="flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm">
                            <input
                                type="checkbox"
                                prop:checked=move || single_selected.get()
                                on:change=move |ev| set_single_selected.set(event_target_checked(&ev))
                            />
                            <span>"selected"</span>
                        </label>
                        <select
                            prop:value=move || single_pos.get()
                            on:change=move |ev| set_single_pos.set(event_target_value(&ev))
                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                        >
                            {PART_OF_SPEECH_OPTIONS
                                .iter()
                                .map(|option| view! { <option value=*option>{*option}</option> })
                                .collect_view()}
                        </select>
                        <input
                            type="text"
                            placeholder="base_form (norwegian_base)"
                            prop:value=move || single_norwegian.get()
                            on:input=move |ev| set_single_norwegian.set(event_target_value(&ev))
                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                        />
                        <input
                            type="text"
                            placeholder="中文（| 分隔）"
                            prop:value=move || single_chinese.get()
                            on:input=move |ev| set_single_chinese.set(event_target_value(&ev))
                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                        />
                        <input
                            type="text"
                            placeholder="英文（| 分隔）"
                            prop:value=move || single_english.get()
                            on:input=move |ev| set_single_english.set(event_target_value(&ev))
                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                        />
                        <input
                            type="text"
                            placeholder="tags（| 分隔）"
                            prop:value=move || single_tags.get()
                            on:input=move |ev| set_single_tags.set(event_target_value(&ev))
                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                        />
                        {move || {
                            if single_pos.get() == "verb" {
                                view! {
                                    <>
                                        <input
                                            type="text"
                                            placeholder="past_tense（必填）"
                                            prop:value=move || single_past_tense.get()
                                            on:input=move |ev| set_single_past_tense.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                        <input
                                            type="text"
                                            placeholder="imperative（必填）"
                                            prop:value=move || single_imperative.get()
                                            on:input=move |ev| set_single_imperative.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                    </>
                                }
                                    .into_any()
                            } else if single_pos.get() == "noun" {
                                view! {
                                    <>
                                        <input
                                            type="text"
                                            placeholder="plural（必填）"
                                            prop:value=move || single_plural.get()
                                            on:input=move |ev| set_single_plural.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                        <input
                                            type="text"
                                            placeholder="singular_definite（必填）"
                                            prop:value=move || single_singular_definite.get()
                                            on:input=move |ev| set_single_singular_definite.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                        <input
                                            type="text"
                                            placeholder="plural_definite（必填）"
                                            prop:value=move || single_plural_definite.get()
                                            on:input=move |ev| set_single_plural_definite.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                    </>
                                }
                                    .into_any()
                            } else if single_pos.get() == "adjective" {
                                view! {
                                    <>
                                        <input
                                            type="text"
                                            placeholder="neuter_form（必填）"
                                            prop:value=move || single_neuter_form.get()
                                            on:input=move |ev| set_single_neuter_form.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                        <input
                                            type="text"
                                            placeholder="plural_form（必填）"
                                            prop:value=move || single_plural_form.get()
                                            on:input=move |ev| set_single_plural_form.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                        <input
                                            type="text"
                                            placeholder="adjective_comparative（必填）"
                                            prop:value=move || single_adjective_comparative.get()
                                            on:input=move |ev| set_single_adjective_comparative.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                        <input
                                            type="text"
                                            placeholder="adjective_superlative_indefinite（必填）"
                                            prop:value=move || single_adjective_superlative_indefinite.get()
                                            on:input=move |ev| set_single_adjective_superlative_indefinite.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                        <input
                                            type="text"
                                            placeholder="adjective_superlative_definite（必填）"
                                            prop:value=move || single_adjective_superlative_definite.get()
                                            on:input=move |ev| set_single_adjective_superlative_definite.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                    </>
                                }
                                    .into_any()
                            } else if single_pos.get() == "adverb" {
                                view! {
                                    <>
                                        <input
                                            type="text"
                                            placeholder="adverb_comparative（必填）"
                                            prop:value=move || single_adverb_comparative.get()
                                            on:input=move |ev| set_single_adverb_comparative.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                        <input
                                            type="text"
                                            placeholder="adverb_superlative（必填）"
                                            prop:value=move || single_adverb_superlative.get()
                                            on:input=move |ev| set_single_adverb_superlative.set(event_target_value(&ev))
                                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                        />
                                    </>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <p class="rounded-lg border border-slate-800 bg-slate-900 px-3 py-2 text-xs text-slate-400 md:col-span-3">
                                        "当前词性只需要基础字段：selected、词性、中文、原型。"
                                    </p>
                                }
                                    .into_any()
                            }
                        }}
                    </div>
                    <button
                        type="submit"
                        class="mt-3 rounded-lg border border-slate-700 bg-emerald-700 px-4 py-2 text-sm font-medium hover:bg-emerald-600"
                    >
                        "添加单条"
                    </button>
                </form>

                <form
                    on:submit=add_bulk_entries
                    class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-4"
                >
                    <h2 class="mb-3 text-lg font-semibold">"多条添加（CSV，多行）"</h2>
                    <textarea
                        placeholder="粘贴 CSV 文本（含表头），数组字段用 | 分隔"
                        prop:value=move || bulk_input.get()
                        on:input=move |ev| {
                            set_bulk_input.set(event_target_value(&ev));
                            set_bulk_errors.set(Vec::new());
                            set_bulk_success_message.set(String::new());
                        }
                        class="min-h-40 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                    ></textarea>
                    {move || {
                        let errors = bulk_errors.get();
                        if errors.is_empty() {
                            view! { <></> }.into_any()
                        } else {
                            view! {
                                <ul class="mt-3 list-disc space-y-1 pl-5 text-sm text-red-400">
                                    {errors
                                        .into_iter()
                                        .map(|item| view! { <li>{item}</li> })
                                        .collect_view()}
                                </ul>
                            }
                                .into_any()
                        }
                    }}
                    {move || {
                        let message = bulk_success_message.get();
                        if message.is_empty() {
                            view! { <></> }.into_any()
                        } else {
                            view! { <p class="mt-3 text-sm font-medium text-emerald-400">{message}</p> }
                                .into_any()
                        }
                    }}
                    <button
                        type="submit"
                        class="mt-3 rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium hover:bg-slate-700"
                    >
                        "添加多条"
                    </button>
                </form>

                <section class="mt-4">
                    <h2 class="mb-3 text-lg font-semibold">"词库浏览器"</h2>
                    <p class="mb-3 text-sm text-slate-400">
                        "以表格形式查看、直接修改并删除词条（类似 Excel）。"
                    </p>
                    <p class="mb-3 text-xs text-slate-500">"拖拽表头右侧边界可调整列宽（更像 Excel）。"</p>
                </section>

                <LexiconBrowser
                    entries=entries
                    set_entries=set_entries
                    set_status=set_status
                    data_version=data_version
                    mode=LexiconBrowserMode::Edit
                />

                <div class="mt-4 flex flex-wrap items-center justify-end gap-3 border-t border-slate-800 pt-4">
                    <ImportCsvButton
                        input_id="lexicon-import-csv-input".to_string()
                        label="导入词库 CSV".to_string()
                        class="inline-flex cursor-pointer items-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium hover:bg-slate-700".to_string()
                        set_entries=set_entries
                        set_status=set_status
                        set_data_version=set_data_version
                        set_source_name=word_bank_state.set_source_name
                    />
                    <button
                        type="button"
                        on:click=export_csv_click
                        class="inline-flex items-center rounded-lg border border-slate-700 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600"
                    >
                        "导出词库 CSV"
                    </button>
                </div>
            </section>
        </main>
    }
}

fn parse_csv_list(raw: &str) -> Vec<String> {
    parse_pipe_list(raw)
}

fn parse_optional_input(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn export_csv_download(filename: &str, content: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::{JsCast, JsValue};

        let parts = js_sys::Array::new();
        parts.push(&JsValue::from_str(content));
        let blob = web_sys::Blob::new_with_str_sequence(&parts)
            .map_err(|_| "无法创建 CSV Blob".to_string())?;
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
