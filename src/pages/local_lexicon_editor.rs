use gloo_net::http::Request;
use leptos::ev::{Event, SubmitEvent};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::lexicon_browser::LexiconBrowser;
use crate::lexicon::{WordEntry, parse_pipe_list, parse_word_bank_csv, serialize_word_bank_csv};
use crate::pages::AppPage;

#[component]
pub fn LocalLexiconEditorPage() -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let (entries, set_entries) = signal(Vec::<WordEntry>::new());
    let (data_version, set_data_version) = signal(0_u64);
    let (status, set_status) = signal("正在下载词库...".to_string());

    let (single_id, set_single_id) = signal(String::new());
    let (single_pos, set_single_pos) = signal(String::new());
    let (single_norwegian, set_single_norwegian) = signal(String::new());
    let (single_chinese, set_single_chinese) = signal(String::new());
    let (single_english, set_single_english) = signal(String::new());
    let (single_tags, set_single_tags) = signal(String::new());

    let (bulk_input, set_bulk_input) = signal(String::new());
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
                    set_status.set(format!("词库加载完成，共 {} 条。", word_list.len()));
                    set_entries.set(word_list);
                    set_data_version.update(|ver| *ver += 1);
                }
                Err(err) => {
                    set_status.set(format!("词库加载失败：{err}"));
                }
            }
        });
    });

    let add_single_entry = move |ev: SubmitEvent| {
        ev.prevent_default();

        if single_id.get().trim().is_empty() || single_norwegian.get().trim().is_empty() {
            set_status.set("单条添加失败：id 和原型不能为空。".to_string());
            return;
        }

        let new_entry = WordEntry {
            id: single_id.get().trim().to_string(),
            part_of_speech: single_pos.get().trim().to_string(),
            norwegian_base: single_norwegian.get().trim().to_string(),
            chinese: parse_csv_list(&single_chinese.get()),
            english: parse_csv_list(&single_english.get()),
            tags: parse_csv_list(&single_tags.get()),
        };

        set_entries.update(|list| list.push(new_entry));
        set_data_version.update(|ver| *ver += 1);
        set_status.set(format!("已添加 1 条，当前共 {} 条。", entries.get_untracked().len()));

        set_single_id.set(String::new());
        set_single_pos.set(String::new());
        set_single_norwegian.set(String::new());
        set_single_chinese.set(String::new());
        set_single_english.set(String::new());
        set_single_tags.set(String::new());
    };

    let add_bulk_entries = move |ev: SubmitEvent| {
        ev.prevent_default();
        let content = bulk_input.get();
        if content.trim().is_empty() {
            set_status.set("多条添加失败：文本框不能为空。".to_string());
            return;
        }

        match parse_word_bank_csv(&content) {
            Ok(mut parsed) => {
                let add_count = parsed.len();
                set_entries.update(|list| list.append(&mut parsed));
                set_data_version.update(|ver| *ver += 1);
                set_status.set(format!("批量添加成功，新增 {} 条。", add_count));
                set_bulk_input.set(String::new());
            }
            Err(err) => {
                set_status.set(format!("多条添加失败：请输入带表头的 CSV 文本。{err}"));
            }
        }
    };
    let import_csv_click = move |ev: Event| {
        import_csv_from_file(ev, set_entries, set_status, set_data_version);
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
            <section class="mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-xl">
                <header class="mb-6 flex items-center justify-between gap-4">
                    <h1 class="text-2xl font-bold tracking-tight">"本地词库修改器"</h1>
                    <button
                        type="button"
                        on:click=move |_| set_current_page.set(AppPage::PracticeModeSelect)
                        class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-sm hover:bg-slate-700"
                    >
                        "返回练习模式页"
                    </button>
                </header>

                <p class="mb-4 text-sm text-slate-300">{move || status.get()}</p>

                <form
                    on:submit=add_single_entry
                    class="rounded-xl border border-slate-800 bg-slate-950/50 p-4"
                >
                    <h2 class="mb-3 text-lg font-semibold">"单条添加"</h2>
                    <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
                        <input
                            type="text"
                            placeholder="id"
                            prop:value=move || single_id.get()
                            on:input=move |ev| set_single_id.set(event_target_value(&ev))
                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                        />
                        <input
                            type="text"
                            placeholder="词性 part_of_speech"
                            prop:value=move || single_pos.get()
                            on:input=move |ev| set_single_pos.set(event_target_value(&ev))
                            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                        />
                        <input
                            type="text"
                            placeholder="原型 norwegian_base"
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
                        on:input=move |ev| set_bulk_input.set(event_target_value(&ev))
                        class="min-h-40 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                    ></textarea>
                    <button
                        type="submit"
                        class="mt-3 rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium hover:bg-slate-700"
                    >
                        "添加多条"
                    </button>
                </form>

                <LexiconBrowser
                    entries=entries
                    set_entries=set_entries
                    set_status=set_status
                    data_version=data_version
                />

                <div class="mt-4 flex flex-wrap items-center justify-end gap-3 border-t border-slate-800 pt-4">
                    <input
                        id="lexicon-import-csv-input"
                        type="file"
                        accept=".csv,text/csv"
                        class="hidden"
                        on:change=import_csv_click
                    />
                    <label
                        for="lexicon-import-csv-input"
                        class="inline-flex cursor-pointer items-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium hover:bg-slate-700"
                    >
                        "导入词库 CSV"
                    </label>
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

fn import_csv_from_file(
    ev: Event,
    set_entries: WriteSignal<Vec<WordEntry>>,
    set_status: WriteSignal<String>,
    set_data_version: WriteSignal<u64>,
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

            match parse_word_bank_csv(&content) {
                Ok(word_list) => {
                    let count = word_list.len();
                    set_entries.set(word_list);
                    set_data_version.update(|ver| *ver += 1);
                    set_status.set(format!("词库导入成功，共 {} 条。", count));
                }
                Err(err) => set_status.set(format!("导入失败：{err}")),
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ev;
        let _ = set_entries;
        let _ = set_data_version;
        set_status.set("导入仅在浏览器环境可用。".to_string());
    }
}

fn export_csv_download(filename: &str, content: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::{JsCast, JsValue};

        let parts = js_sys::Array::new();
        parts.push(&JsValue::from_str(content));
        let blob =
            web_sys::Blob::new_with_str_sequence(&parts).map_err(|_| "无法创建 CSV Blob".to_string())?;
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

        web_sys::Url::revoke_object_url(&object_url)
            .map_err(|_| "无法释放下载 URL".to_string())?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = filename;
        let _ = content;
        Err("导出仅在浏览器环境可用。".to_string())
    }
}
