use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::structures::pracresult::PracticeResult;

pub mod interrogative;
pub mod month;
pub mod number;
pub mod pronoun;

pub fn initialize_temp_practice_result(word_bank_state: WordBankState) {
    let base_result = word_bank_state.practice_result.get_untracked();
    let selected_ids = word_bank_state
        .entries
        .get_untracked()
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<Vec<_>>();

    word_bank_state
        .set_selected_word_entry_ids
        .set(selected_ids.clone());
    word_bank_state
        .set_temp_practice_result
        .set(PracticeResult {
            username: base_result.username,
            encryption_key: base_result.encryption_key,
            selected_word_entry_ids: selected_ids,
            practiced_word_entries: Vec::new(),
        });
}
