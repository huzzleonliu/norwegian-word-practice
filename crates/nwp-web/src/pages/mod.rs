//! 页面模块与页面枚举：每个 `AppPage` 对应一条 URL（由 `leptos_router` 匹配）。

pub mod dictionary_editor;
pub mod help;
pub mod home;
pub mod lexicon_practice;
pub mod lexicon_select;
pub mod practice_mode_select;
pub mod practice_result;
pub mod serise_practice;
pub mod serise_select;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppPage {
    Home,
    PracticeModeSelect,
    LexiconMode,
    LexiconPractice,
    LexiconSummary,
    LocalLexiconEditor,
    SeriseSelect,
    SeriseNumberPractice,
    SeriseMonthPractice,
    SerisePronounPractice,
    SeriseInterrogativePractice,
}

impl AppPage {
    /// 该页面对应的绝对路径（以 `/` 开头）。
    pub fn path(self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::PracticeModeSelect => "/practice",
            Self::LexiconMode => "/lexicon",
            Self::LexiconPractice => "/lexicon/practice",
            Self::LexiconSummary => "/lexicon/summary",
            Self::LocalLexiconEditor => "/editor",
            Self::SeriseSelect => "/series",
            Self::SeriseNumberPractice => "/series/number",
            Self::SeriseMonthPractice => "/series/month",
            Self::SerisePronounPractice => "/series/pronoun",
            Self::SeriseInterrogativePractice => "/series/interrogative",
        }
    }

    /// 从 pathname 解析页面；无法识别时返回 `None`。
    pub fn from_path(path: &str) -> Option<Self> {
        let normalized = normalize_path(path);
        match normalized.as_str() {
            "/" => Some(Self::Home),
            "/practice" => Some(Self::PracticeModeSelect),
            "/lexicon" => Some(Self::LexiconMode),
            "/lexicon/practice" => Some(Self::LexiconPractice),
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

    #[test]
    fn path_roundtrip() {
        let pages = [
            AppPage::Home,
            AppPage::PracticeModeSelect,
            AppPage::LexiconMode,
            AppPage::LexiconPractice,
            AppPage::LexiconSummary,
            AppPage::LocalLexiconEditor,
            AppPage::SeriseSelect,
            AppPage::SeriseNumberPractice,
            AppPage::SeriseMonthPractice,
            AppPage::SerisePronounPractice,
            AppPage::SeriseInterrogativePractice,
        ];
        for page in pages {
            assert_eq!(AppPage::from_path(page.path()), Some(page));
        }
    }

    #[test]
    fn from_path_normalizes_trailing_slash() {
        assert_eq!(AppPage::from_path("/editor/"), Some(AppPage::LocalLexiconEditor));
        assert_eq!(AppPage::from_path("/series/number/"), Some(AppPage::SeriseNumberPractice));
    }
}
