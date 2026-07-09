use leptos::prelude::*;

mod app_state;
mod components;
mod pages;
mod structures;
mod utils;

fn main() {
    leptos::mount::mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    let (current_page, set_current_page) = signal(pages::AppPage::Home);
    let (word_bank_entries, set_word_bank_entries) = signal(Vec::new());
    let (word_bank_data_version, set_word_bank_data_version) = signal(0_u64);
    let (word_bank_source_name, set_word_bank_source_name) = signal("尚未加载词库".to_string());

    provide_context(set_current_page);
    provide_context(app_state::WordBankState {
        entries: word_bank_entries,
        set_entries: set_word_bank_entries,
        data_version: word_bank_data_version,
        set_data_version: set_word_bank_data_version,
        source_name: word_bank_source_name,
        set_source_name: set_word_bank_source_name,
    });

    view! {
        {move || match current_page.get() {
            pages::AppPage::Home => view! { <pages::home::HomePage/> }.into_any(),
            pages::AppPage::PracticeModeSelect => {
                view! { <pages::practice_mode::PracticeModePage/> }.into_any()
            }
            pages::AppPage::LexiconMode => {
                view! { <pages::lexicon_mode::LexiconModePage/> }.into_any()
            }
            pages::AppPage::LocalLexiconEditor => {
                view! { <pages::local_lexicon_editor::LocalLexiconEditorPage/> }.into_any()
            }
        }}
    }
}
