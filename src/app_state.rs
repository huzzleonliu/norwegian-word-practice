use leptos::prelude::*;

use crate::components::lexicon_browser::WordEntry;

#[derive(Clone, Copy)]
pub struct WordBankState {
    pub entries: ReadSignal<Vec<WordEntry>>,
    pub set_entries: WriteSignal<Vec<WordEntry>>,
    pub data_version: ReadSignal<u64>,
    pub set_data_version: WriteSignal<u64>,
    pub source_name: ReadSignal<String>,
    pub set_source_name: WriteSignal<String>,
}
