pub mod home;
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
