//! 系列练习公共模块：提供各系列页面共享的临时结果初始化逻辑。

use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::structures::pracresult::PracticeResult;

pub mod interrogative;
pub mod month;
pub mod number;
pub mod pronoun;

/// 系列练习进入时初始化临时结果，并确保 selected ids 与当前词库一致。
pub fn initialize_temp_practice_result(word_bank_state: WordBankState) {
    let base_result = word_bank_state.practice_result.get_untracked();
    let all_ids = word_bank_state
        .entries
        .get_untracked()
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<Vec<_>>();
    let selected_from_state = word_bank_state.selected_word_entry_ids.get_untracked();
    let selected_ids = if selected_from_state.is_empty() {
        all_ids
    } else {
        selected_from_state
            .into_iter()
            .filter(|id| all_ids.iter().any(|existing| existing == id))
            .collect::<Vec<_>>()
    };

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
