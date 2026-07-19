//! 当前词库持久化：写入 / 读取 localStorage，刷新后仍可用。

use serde::{Deserialize, Serialize};

use crate::structures::word_bank_entry::WordBankEntry;

const LEXICON_STORAGE_KEY: &str = "lexicon_store_v1";
pub const DEFAULT_SOURCE_NAME: &str = "尚未加载词库";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredLexicon {
    source_name: String,
    entries: Vec<WordBankEntry>,
}

/// 从 localStorage 恢复词库；无数据或解析失败时返回 `None`。
pub fn load_stored_lexicon() -> Option<(Vec<WordBankEntry>, String)> {
    let storage = local_storage()?;
    let raw = storage.get_item(LEXICON_STORAGE_KEY).ok().flatten()?;
    let stored: StoredLexicon = serde_json::from_str(&raw).ok()?;
    Some((stored.entries, stored.source_name))
}

/// 将当前词库写入 localStorage（选择/导入/编辑后由根 Effect 调用）。
pub fn save_stored_lexicon(entries: &[WordBankEntry], source_name: &str) {
    let Some(storage) = local_storage() else {
        return;
    };
    let payload = StoredLexicon {
        source_name: source_name.to_string(),
        entries: entries.to_vec(),
    };
    if let Ok(raw) = serde_json::to_string(&payload) {
        let _ = storage.set_item(LEXICON_STORAGE_KEY, &raw);
    }
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
}
