pub mod home;
pub mod lexicon_mode;
pub mod local_lexicon_editor;
pub mod practice_mode;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Home,
    PracticeModeSelect,
    LexiconMode,
    LocalLexiconEditor,
}
