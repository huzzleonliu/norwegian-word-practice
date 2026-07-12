use leptos::ev::Event;
use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use leptos::task::spawn_local;

#[cfg(target_arch = "wasm32")]
use crate::components::lexicon_browser::parse_word_bank_csv;
use crate::structures::word_bank_entry::WordBankEntry;

#[component]
pub fn ImportCsvButton(
    input_id: String,
    label: String,
    class: String,
    set_entries: WriteSignal<Vec<WordBankEntry>>,
    set_status: WriteSignal<String>,
    set_data_version: WriteSignal<u64>,
    #[prop(optional)] set_source_name: Option<WriteSignal<String>>,
) -> impl IntoView {
    let import_csv_click = move |ev: Event| {
        import_csv_from_file(
            ev,
            set_entries,
            set_status,
            set_data_version,
            set_source_name,
        );
    };

    view! {
        <input
            id=input_id.clone()
            type="file"
            accept=".csv,text/csv"
            class="hidden"
            on:change=import_csv_click
        />
        <label for=input_id class=class>
            {label}
        </label>
    }
}

fn import_csv_from_file(
    ev: Event,
    set_entries: WriteSignal<Vec<WordBankEntry>>,
    set_status: WriteSignal<String>,
    set_data_version: WriteSignal<u64>,
    set_source_name: Option<WriteSignal<String>>,
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

            match parse_word_bank_csv(&content) {
                Ok(word_list) => {
                    let count = word_list.len();
                    set_entries.set(word_list);
                    set_data_version.update(|ver| *ver += 1);
                    if let Some(set_source_name) = set_source_name {
                        set_source_name.set(format!("本地导入：{file_name}"));
                    }
                    set_status.set(format!("词库导入成功（已覆盖），共 {} 条。", count));
                }
                Err(err) => {
                    set_status.set(format!("导入失败：{err}"));
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ev;
        let _ = set_entries;
        let _ = set_data_version;
        let _ = set_source_name;
        set_status.set("导入仅在浏览器环境可用。".to_string());
    }
}
