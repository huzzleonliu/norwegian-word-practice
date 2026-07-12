use leptos::prelude::*;

use crate::pages::AppPage;
use crate::structures::pracresult::PracticeResult;
use crate::structures::word_bank_entry::{UiLanguage, WordBankEntry};

#[derive(Clone, Copy)]
pub struct WordBankState {
    pub entries: ReadSignal<Vec<WordBankEntry>>,
    pub set_entries: WriteSignal<Vec<WordBankEntry>>,
    pub data_version: ReadSignal<u64>,
    pub set_data_version: WriteSignal<u64>,
    pub source_name: ReadSignal<String>,
    pub set_source_name: WriteSignal<String>,
    pub ui_language: ReadSignal<UiLanguage>,
    pub selected_word_entry_ids: ReadSignal<Vec<String>>,
    pub set_selected_word_entry_ids: WriteSignal<Vec<String>>,
    pub practice_result: ReadSignal<PracticeResult>,
    pub set_practice_result: WriteSignal<PracticeResult>,
    pub temp_practice_result: ReadSignal<PracticeResult>,
    pub set_temp_practice_result: WriteSignal<PracticeResult>,
    pub last_completed_practice_result: ReadSignal<PracticeResult>,
    pub set_last_completed_practice_result: WriteSignal<PracticeResult>,
    pub summary_return_page: ReadSignal<AppPage>,
    pub set_summary_return_page: WriteSignal<AppPage>,
}
