//! 词库文件加解密面板：
//! - CSV -> `.nwpdict`
//! - `.nwpdict` -> CSV

use leptos::ev::Event;
use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use leptos::task::spawn_local;

use crate::app_state::UiState;
use crate::structures::word_bank_entry::UiLanguage;
#[cfg(target_arch = "wasm32")]
use crate::utils::csv_schema::{parse_word_bank_csv, serialize_word_bank_csv};
#[cfg(target_arch = "wasm32")]
use crate::utils::dictionary_crypto::{
    parse_lexicon_import_content, serialize_encrypted_lexicon_export,
};
use crate::utils::i18n::tr;

#[component]
pub fn LexiconFileCryptoPanel(set_status: WriteSignal<String>) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    let (crypto_status, set_crypto_status) = signal(String::new());

    let encrypt_csv_click = move |ev: Event| {
        encrypt_csv_file(ev, lang.get_untracked(), set_status, set_crypto_status);
    };
    let decrypt_file_click = move |ev: Event| {
        decrypt_lexicon_file(ev, lang.get_untracked(), set_status, set_crypto_status);
    };

    view! {
        <section class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
            <h2 class="mb-3 text-lg font-semibold">
                {move || tr(lang.get(), "词库文件加解密", "Lexicon File Encrypt / Decrypt")}
            </h2>
            <p class="mb-3 text-sm text-slate-400">
                {move || {
                    tr(
                        lang.get(),
                        "将 CSV 加密为 .nwpdict，或将 .nwpdict 解密回 CSV，便于离线调整后再导入。",
                        "Encrypt CSV to .nwpdict, or decrypt .nwpdict back to CSV for offline adjustments.",
                    )
                }}
            </p>
            <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
                <article class="rounded-lg border border-slate-800 bg-slate-900/40 p-3">
                    <p class="mb-2 text-xs font-semibold text-slate-400">
                        {move || tr(lang.get(), "CSV -> 加密词库文件", "CSV -> Encrypted Lexicon File")}
                    </p>
                    <input
                        id="lexicon-editor-encrypt-csv-input"
                        type="file"
                        accept=".csv,text/csv,text/plain"
                        class="hidden"
                        on:change=encrypt_csv_click
                    />
                    <label
                        for="lexicon-editor-encrypt-csv-input"
                        class="inline-flex w-full cursor-pointer items-center justify-center rounded border border-slate-700 bg-emerald-700 px-3 py-2 text-sm font-medium text-white hover:bg-emerald-600"
                    >
                        {move || tr(lang.get(), "选择 CSV 并加密下载", "Choose CSV and Encrypt")}
                    </label>
                </article>
                <article class="rounded-lg border border-slate-800 bg-slate-900/40 p-3">
                    <p class="mb-2 text-xs font-semibold text-slate-400">
                        {move || tr(lang.get(), "加密词库文件 -> CSV", "Encrypted Lexicon File -> CSV")}
                    </p>
                    <input
                        id="lexicon-editor-decrypt-nwpdict-input"
                        type="file"
                        accept=".nwpdict,application/json,text/plain"
                        class="hidden"
                        on:change=decrypt_file_click
                    />
                    <label
                        for="lexicon-editor-decrypt-nwpdict-input"
                        class="inline-flex w-full cursor-pointer items-center justify-center rounded border border-slate-700 bg-slate-800 px-3 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                    >
                        {move || tr(lang.get(), "选择 .nwpdict 并解密下载", "Choose .nwpdict and Decrypt")}
                    </label>
                </article>
            </div>
            <p class="mt-3 text-sm text-slate-300">
                {move || {
                    let message = crypto_status.get();
                    if message.trim().is_empty() {
                        tr(
                            lang.get(),
                            "等待文件转换操作...",
                            "Waiting for file conversion action...",
                        )
                        .to_string()
                    } else {
                        message
                    }
                }}
            </p>
        </section>
    }
}

fn encrypt_csv_file(
    ev: Event,
    lang: UiLanguage,
    set_status: WriteSignal<String>,
    set_crypto_status: WriteSignal<String>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let Some(target) = ev.target() else {
            set_dual_status(
                lang,
                set_status,
                set_crypto_status,
                "加密失败：无法获取文件输入目标。",
                "Encryption failed: cannot get file input target.",
            );
            return;
        };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
            set_dual_status(
                lang,
                set_status,
                set_crypto_status,
                "加密失败：文件输入类型不正确。",
                "Encryption failed: invalid file input type.",
            );
            return;
        };
        let Some(files) = input.files() else {
            set_dual_status(
                lang,
                set_status,
                set_crypto_status,
                "加密失败：未找到文件列表。",
                "Encryption failed: file list not found.",
            );
            return;
        };
        let Some(file) = files.get(0) else {
            set_dual_status(
                lang,
                set_status,
                set_crypto_status,
                "加密已取消。",
                "Encryption cancelled.",
            );
            return;
        };

        let file: web_sys::File = file;
        let source_name = file.name();
        input.set_value("");

        spawn_local(async move {
            let content = match read_file_text(file).await {
                Ok(content) => content,
                Err(err) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!("{} {err}", tr(lang, "加密失败：", "Encryption failed:")),
                    );
                    return;
                }
            };

            let entries = match parse_word_bank_csv(&content) {
                Ok(entries) => entries,
                Err(err) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!(
                            "{} {err}",
                            tr(
                                lang,
                                "加密失败：CSV 内容不合法。",
                                "Encryption failed: invalid CSV content.",
                            )
                        ),
                    );
                    return;
                }
            };

            let normalized_csv = match serialize_word_bank_csv(&entries) {
                Ok(csv) => csv,
                Err(err) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!("{} {err}", tr(lang, "加密失败：", "Encryption failed:")),
                    );
                    return;
                }
            };

            let encrypted_payload = match serialize_encrypted_lexicon_export(&normalized_csv) {
                Ok(payload) => payload,
                Err(err) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!("{} {err}", tr(lang, "加密失败：", "Encryption failed:")),
                    );
                    return;
                }
            };

            let output_name = replace_file_extension(&source_name, "nwpdict");
            match download_text_file(&output_name, &encrypted_payload) {
                Ok(()) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!(
                            "{} {}（{} {} {}）",
                            tr(lang, "加密完成并已下载：", "Encryption finished and downloaded:"),
                            output_name,
                            tr(lang, "共", "total"),
                            entries.len(),
                            tr(lang, "条。", "entries."),
                        ),
                    );
                }
                Err(err) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!("{} {err}", tr(lang, "加密失败：", "Encryption failed:")),
                    );
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ev;
        set_dual_status(
            lang,
            set_status,
            set_crypto_status,
            "加解密转换仅在浏览器环境可用。",
            "Encrypt/decrypt conversion is only available in browser environment.",
        );
    }
}

fn decrypt_lexicon_file(
    ev: Event,
    lang: UiLanguage,
    set_status: WriteSignal<String>,
    set_crypto_status: WriteSignal<String>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let Some(target) = ev.target() else {
            set_dual_status(
                lang,
                set_status,
                set_crypto_status,
                "解密失败：无法获取文件输入目标。",
                "Decryption failed: cannot get file input target.",
            );
            return;
        };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
            set_dual_status(
                lang,
                set_status,
                set_crypto_status,
                "解密失败：文件输入类型不正确。",
                "Decryption failed: invalid file input type.",
            );
            return;
        };
        let Some(files) = input.files() else {
            set_dual_status(
                lang,
                set_status,
                set_crypto_status,
                "解密失败：未找到文件列表。",
                "Decryption failed: file list not found.",
            );
            return;
        };
        let Some(file) = files.get(0) else {
            set_dual_status(
                lang,
                set_status,
                set_crypto_status,
                "解密已取消。",
                "Decryption cancelled.",
            );
            return;
        };

        let file: web_sys::File = file;
        let source_name = file.name();
        input.set_value("");

        spawn_local(async move {
            let content = match read_file_text(file).await {
                Ok(content) => content,
                Err(err) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!("{} {err}", tr(lang, "解密失败：", "Decryption failed:")),
                    );
                    return;
                }
            };

            let csv_text = match parse_lexicon_import_content(&content) {
                Ok(csv_text) => csv_text,
                Err(err) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!("{} {err}", tr(lang, "解密失败：", "Decryption failed:")),
                    );
                    return;
                }
            };

            let entries = match parse_word_bank_csv(&csv_text) {
                Ok(entries) => entries,
                Err(err) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!(
                            "{} {err}",
                            tr(
                                lang,
                                "解密失败：词库内容不合法。",
                                "Decryption failed: invalid lexicon content.",
                            )
                        ),
                    );
                    return;
                }
            };

            let normalized_csv = match serialize_word_bank_csv(&entries) {
                Ok(csv) => csv,
                Err(err) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!("{} {err}", tr(lang, "解密失败：", "Decryption failed:")),
                    );
                    return;
                }
            };

            let output_name = replace_file_extension(&source_name, "csv");
            match download_text_file(&output_name, &normalized_csv) {
                Ok(()) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!(
                            "{} {}（{} {} {}）",
                            tr(lang, "解密完成并已下载：", "Decryption finished and downloaded:"),
                            output_name,
                            tr(lang, "共", "total"),
                            entries.len(),
                            tr(lang, "条。", "entries."),
                        ),
                    );
                }
                Err(err) => {
                    set_dual_status_message(
                        set_status,
                        set_crypto_status,
                        format!("{} {err}", tr(lang, "解密失败：", "Decryption failed:")),
                    );
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ev;
        set_dual_status(
            lang,
            set_status,
            set_crypto_status,
            "加解密转换仅在浏览器环境可用。",
            "Encrypt/decrypt conversion is only available in browser environment.",
        );
    }
}

fn set_dual_status(
    lang: UiLanguage,
    set_status: WriteSignal<String>,
    set_crypto_status: WriteSignal<String>,
    zh: &'static str,
    en: &'static str,
) {
    set_dual_status_message(set_status, set_crypto_status, tr(lang, zh, en).to_string());
}

fn set_dual_status_message(
    set_status: WriteSignal<String>,
    set_crypto_status: WriteSignal<String>,
    message: String,
) {
    set_crypto_status.set(message.clone());
    set_status.set(message);
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn replace_file_extension(file_name: &str, ext: &str) -> String {
    let stem = file_name
        .rsplit_once('.')
        .map(|(name, _)| name)
        .unwrap_or(file_name)
        .trim();
    let safe_stem = if stem.is_empty() { "word-bank" } else { stem };
    format!("{safe_stem}.{ext}")
}

#[cfg(target_arch = "wasm32")]
async fn read_file_text(file: web_sys::File) -> Result<String, String> {
    let js_value = wasm_bindgen_futures::JsFuture::from(file.text())
        .await
        .map_err(|err| format!("{err:?}"))?;
    js_value
        .as_string()
        .ok_or_else(|| "文件内容不是文本。".to_string())
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn download_text_file(filename: &str, content: &str) -> Result<(), String> {
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
