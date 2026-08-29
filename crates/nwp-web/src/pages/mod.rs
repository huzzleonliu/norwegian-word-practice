//! 页面模块与页面枚举：每个 `AppPage` 对应一条逻辑路径；
//! 实际 URL 带语言前缀（`/ch/...` 或 `/en/...`）。

pub mod dictionary_editor;
pub mod help;
pub mod home;
pub mod lexicon_practice;
pub mod lexicon_remember;
pub mod lexicon_select;
pub mod practice_mode_select;
pub mod practice_result;
pub mod serise_practice;
pub mod serise_select;

use crate::structures::word_bank_entry::UiLanguage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppPage {
    Home,
    PracticeModeSelect,
    LexiconMode,
    LexiconPractice,
    LexiconRemember,
    LexiconSummary,
    LocalLexiconEditor,
    SeriseSelect,
    SeriseNumberPractice,
    SeriseMonthPractice,
    SerisePronounPractice,
    SeriseInterrogativePractice,
}

impl AppPage {
    /// 逻辑路径（不含语言前缀，以 `/` 开头；首页为 `/`）。
    pub fn path(self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::PracticeModeSelect => "/practice",
            Self::LexiconMode => "/lexicon",
            Self::LexiconPractice => "/lexicon/practice",
            Self::LexiconRemember => "/lexicon/remember",
            Self::LexiconSummary => "/lexicon/summary",
            Self::LocalLexiconEditor => "/editor",
            Self::SeriseSelect => "/series",
            Self::SeriseNumberPractice => "/series/number",
            Self::SeriseMonthPractice => "/series/month",
            Self::SerisePronounPractice => "/series/pronoun",
            Self::SeriseInterrogativePractice => "/series/interrogative",
        }
    }

    /// 带语言前缀的绝对路径，如 `/ch/practice`、`/en`。
    pub fn localized_path(self, lang: UiLanguage) -> String {
        let code = lang.route_code();
        match self {
            Self::Home => format!("/{code}"),
            _ => format!("/{code}{}", self.path()),
        }
    }

    /// 从完整 pathname 解析语言与页面；无法识别时返回 `None`。
    pub fn from_localized_path(path: &str) -> Option<(UiLanguage, Self)> {
        let normalized = normalize_path(path);
        let remainder = normalized.trim_start_matches('/');
        let (code, rest) = match remainder.split_once('/') {
            Some((code, rest)) => (code, rest),
            None => (remainder, ""),
        };
        let lang = UiLanguage::from_route_code(code)?;
        let logical = if rest.is_empty() {
            "/".to_string()
        } else {
            format!("/{rest}")
        };
        let page = Self::from_logical_path(&logical)?;
        Some((lang, page))
    }

    /// 兼容旧路径（无语言前缀）与带前缀路径：仅解析页面，语言默认中文。
    pub fn from_path(path: &str) -> Option<Self> {
        if let Some((_, page)) = Self::from_localized_path(path) {
            return Some(page);
        }
        Self::from_logical_path(&normalize_path(path))
    }

    fn from_logical_path(path: &str) -> Option<Self> {
        match path {
            "/" => Some(Self::Home),
            "/practice" => Some(Self::PracticeModeSelect),
            "/lexicon" => Some(Self::LexiconMode),
            "/lexicon/practice" => Some(Self::LexiconPractice),
            "/lexicon/remember" => Some(Self::LexiconRemember),
            "/lexicon/summary" => Some(Self::LexiconSummary),
            "/editor" => Some(Self::LocalLexiconEditor),
            "/series" => Some(Self::SeriseSelect),
            "/series/number" => Some(Self::SeriseNumberPractice),
            "/series/month" => Some(Self::SeriseMonthPractice),
            "/series/pronoun" => Some(Self::SerisePronounPractice),
            "/series/interrogative" => Some(Self::SeriseInterrogativePractice),
            _ => None,
        }
    }
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return "/".to_string();
    }
    let without_query = trimmed.split('?').next().unwrap_or(trimmed);
    let without_hash = without_query.split('#').next().unwrap_or(without_query);
    let mut normalized = without_hash.trim_end_matches('/').to_string();
    if normalized.is_empty() {
        normalized.push('/');
    } else if !normalized.starts_with('/') {
        normalized.insert(0, '/');
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::AppPage;
    use crate::structures::word_bank_entry::UiLanguage;

    #[test]
    fn path_roundtrip() {
        let pages = [
            AppPage::Home,
            AppPage::PracticeModeSelect,
            AppPage::LexiconMode,
            AppPage::LexiconPractice,
            AppPage::LexiconRemember,
            AppPage::LexiconSummary,
            AppPage::LocalLexiconEditor,
            AppPage::SeriseSelect,
            AppPage::SeriseNumberPractice,
            AppPage::SeriseMonthPractice,
            AppPage::SerisePronounPractice,
            AppPage::SeriseInterrogativePractice,
        ];
        for page in pages {
            assert_eq!(AppPage::from_logical_path(page.path()), Some(page));
            for lang in [UiLanguage::Zh, UiLanguage::En] {
                let localized = page.localized_path(lang);
                assert_eq!(
                    AppPage::from_localized_path(&localized),
                    Some((lang, page))
                );
            }
        }
    }

    #[test]
    fn from_path_normalizes_trailing_slash() {
        assert_eq!(
            AppPage::from_path("/ch/editor/"),
            Some(AppPage::LocalLexiconEditor)
        );
        assert_eq!(
            AppPage::from_path("/en/series/number/"),
            Some(AppPage::SeriseNumberPractice)
        );
        assert_eq!(
            AppPage::from_path("/editor/"),
            Some(AppPage::LocalLexiconEditor)
        );
    }

    #[test]
    fn localized_home_paths() {
        assert_eq!(AppPage::Home.localized_path(UiLanguage::Zh), "/ch");
        assert_eq!(AppPage::Home.localized_path(UiLanguage::En), "/en");
        assert_eq!(
            AppPage::from_localized_path("/ch"),
            Some((UiLanguage::Zh, AppPage::Home))
        );
    }
}
