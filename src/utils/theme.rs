//! UI 主题：读写本地偏好，并同步到 `document.documentElement` 的 class。

use crate::structures::word_bank_entry::UiTheme;

const THEME_STORAGE_KEY: &str = "ui_theme";

/// 从 localStorage 读取主题；缺失或非法时回退黑夜模式。
pub fn load_stored_theme() -> UiTheme {
    let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    else {
        return UiTheme::Dark;
    };
    match storage
        .get_item(THEME_STORAGE_KEY)
        .ok()
        .flatten()
        .as_deref()
    {
        Some("light") => UiTheme::Light,
        _ => UiTheme::Dark,
    }
}

/// 将主题应用到根节点，并持久化到 localStorage。
pub fn apply_theme(theme: UiTheme) {
    if let Some(document) = web_sys::window().and_then(|window| window.document()) {
        if let Some(root) = document.document_element() {
            let class_list = root.class_list();
            let _ = class_list.remove_1("theme-dark");
            let _ = class_list.remove_1("theme-light");
            let _ = match theme {
                UiTheme::Dark => class_list.add_1("theme-dark"),
                UiTheme::Light => class_list.add_1("theme-light"),
            };
        }
    }

    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    {
        let value = match theme {
            UiTheme::Dark => "dark",
            UiTheme::Light => "light",
        };
        let _ = storage.set_item(THEME_STORAGE_KEY, value);
    }
}
