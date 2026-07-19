//! 页面模块与页面枚举定义：通过 `AppPage` 实现无 URL 的页面切换。

pub mod dictionary_editor;
pub mod help;
pub mod home;
pub mod lexicon_practice;
pub mod lexicon_select;
pub mod mint_nft;
pub mod practice_mode_select;
pub mod practice_result;
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
    MintNft,
    SeriseSelect,
    SeriseNumberPractice,
    SeriseMonthPractice,
    SerisePronounPractice,
    SeriseInterrogativePractice,
}
