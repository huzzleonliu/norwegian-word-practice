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
    let (selected_word_entry_ids, set_selected_word_entry_ids) = signal(Vec::<String>::new());
    let (practice_result, set_practice_result) =
        signal(structures::pracresult::PracticeResult::default());
    let (temp_practice_result, set_temp_practice_result) =
        signal(structures::pracresult::PracticeResult::default());
    let (last_completed_practice_result, set_last_completed_practice_result) =
        signal(structures::pracresult::PracticeResult::default());

    provide_context(set_current_page);
    provide_context(app_state::WordBankState {
        entries: word_bank_entries,
        set_entries: set_word_bank_entries,
        data_version: word_bank_data_version,
        set_data_version: set_word_bank_data_version,
        source_name: word_bank_source_name,
        set_source_name: set_word_bank_source_name,
        selected_word_entry_ids,
        set_selected_word_entry_ids,
        practice_result,
        set_practice_result,
        temp_practice_result,
        set_temp_practice_result,
        last_completed_practice_result,
        set_last_completed_practice_result,
    });

    view! {
        {move || match current_page.get() {
            pages::AppPage::Home => view! { <pages::home::HomePage/> }.into_any(),
            pages::AppPage::PracticeModeSelect => {
                view! { <pages::practice_mode::PracticeModePage/> }.into_any()
            }
            pages::AppPage::LexiconMode => {
                view! { <pages::lexicon_select::LexiconSelectPage/> }.into_any()
            }
            pages::AppPage::LexiconPractice => {
                view! { <pages::lexicon_practice::LexiconPracticePage/> }.into_any()
            }
            pages::AppPage::LexiconSummary => {
                view! { <pages::lexicon_summary::LexiconSummaryPage/> }.into_any()
            }
            pages::AppPage::LocalLexiconEditor => {
                view! { <pages::local_lexicon_editor::LocalLexiconEditorPage/> }.into_any()
            }
            pages::AppPage::SeriseSelect => {
                view! { <pages::serise_select::SeriseSelectPage/> }.into_any()
            }
            pages::AppPage::SeriseNumberPractice => {
                view! { <pages::serise_practice::number::NumberSerisePracticePage/> }.into_any()
            }
            pages::AppPage::SeriseMonthPractice => {
                view! { <pages::serise_practice::month::MonthSerisePracticePage/> }.into_any()
            }
            pages::AppPage::SerisePronounPractice => {
                view! { <pages::serise_practice::pronoun::PronounSerisePracticePage/> }.into_any()
            }
            pages::AppPage::SeriseInterrogativePractice => {
                view! { <pages::serise_practice::interrogative::InterrogativeSerisePracticePage/> }
                    .into_any()
            }
        }}
    }
}
