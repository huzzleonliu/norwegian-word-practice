pub mod home;
pub mod lexicon_practice;
pub mod lexicon_summary;
pub mod lexicon_mode;
pub mod local_lexicon_editor;
pub mod practice_mode;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Home,
    PracticeModeSelect,
    LexiconMode,
    LexiconPractice,
    LexiconSummary,
    LocalLexiconEditor,
}
