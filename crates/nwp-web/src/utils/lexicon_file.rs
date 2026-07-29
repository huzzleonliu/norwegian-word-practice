//! 词库文件工具：
//! - 统一导入解析（`.nwpdict`；内置加载仍兼容明文 CSV）
//! - 统一导出封装（`WordBankEntry[]` -> 加密 `.nwpdict`）
//! - 统一内置词库加载（URL -> 词条列表）

use crate::structures::word_bank_entry::WordBankEntry;
use crate::utils::csv_schema::{parse_word_bank_csv, serialize_word_bank_csv};
use crate::utils::dictionary_crypto::{
    parse_lexicon_import_content, serialize_encrypted_lexicon_export,
};

/// 用户导入字典时文件输入框可接受的类型。
pub const DICTIONARY_IMPORT_FILE_ACCEPT: &str = ".nwpdict";

/// 内置词库默认文件（练习模式页）。
pub const DEFAULT_BUILTIN_WORD_BANK_FILE: &str = "word-bank.nwpdict";
/// 系列模式默认词库文件。
pub const DEFAULT_SERIES_WORD_BANK_FILE: &str = "series-word-bank.nwpdict";
/// 导出文件名（加密格式）。
pub const LEXICON_EXPORT_FILE_NAME: &str = "word-bank.nwpdict";

/// 从词库文本解析词条：
/// - 若是 `.nwpdict` 会先解密为 CSV 文本；
/// - 若是明文 CSV 则直接解析；
/// - 最终统一输出 `Vec<WordBankEntry>`。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn parse_lexicon_entries_from_text(raw: &str) -> Result<Vec<WordBankEntry>, String> {
    let csv_text = parse_lexicon_import_content(raw)?;
    parse_word_bank_csv(&csv_text)
}

/// 把当前词条列表导出为加密 `.nwpdict` 文本。
pub fn serialize_encrypted_lexicon_from_entries(
    entries: &[WordBankEntry],
) -> Result<String, String> {
    let csv_content = serialize_word_bank_csv(entries)?;
    serialize_encrypted_lexicon_export(&csv_content)
}

/// 从内置词库 URL 拉取并解析词条（自动支持 `.csv` / `.nwpdict`）。
#[cfg(target_arch = "wasm32")]
pub async fn load_lexicon_entries_from_url(path: &str) -> Result<Vec<WordBankEntry>, String> {
    use gloo_net::http::Request;

    let request_url = format!("{path}?v={}", js_sys::Date::now());
    let response = Request::get(&request_url)
        .send()
        .await
        .map_err(|err| format!("下载失败: {err}"))?;
    let raw_text = response
        .text()
        .await
        .map_err(|err| format!("读取响应失败: {err}"))?;
    parse_lexicon_entries_from_text(&raw_text)
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub async fn load_lexicon_entries_from_url(_path: &str) -> Result<Vec<WordBankEntry>, String> {
    Err("内置词库加载仅在浏览器环境可用。".to_string())
}
