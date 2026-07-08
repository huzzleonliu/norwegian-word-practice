pub mod home;
pub mod local_lexicon_editor;
pub mod practice_mode;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Home,
    PracticeModeSelect,
    LocalLexiconEditor,
}
