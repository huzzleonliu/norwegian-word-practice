//! 页面模块与页面枚举定义：通过 `AppPage` 实现无 URL 的页面切换。

pub mod home;
pub mod instructions;
pub mod lexicon_practice;
pub mod lexicon_select;
pub mod lexicon_summary;
pub mod local_lexicon_editor;
pub mod practice_mode;
pub mod serise_practice;
pub mod serise_select;

#[derive(Clone, Copy, PartialEq, Eq)]
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
