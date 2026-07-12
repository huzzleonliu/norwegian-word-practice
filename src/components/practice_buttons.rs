use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::pages::AppPage;
use crate::structures::pracresult::{
    AnswerStats, MAX_WRONG_ANSWERS_PER_FORM, PracticeResult, PracticedWordEntryResult,
};
use crate::utils::i18n::tr;
use crate::utils::shuffle::shuffle_strings;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RestartTempBehavior {
    Keep,
    Reinitialize,
}

#[component]
pub fn FinishPracticeButton(
    word_bank_state: WordBankState,
    set_current_page: WriteSignal<AppPage>,
    set_status: WriteSignal<String>,
    finish_target_page: AppPage,
    summary_return_page: AppPage,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let lang = word_bank_state.ui_language;
    let finish_click = move |_| {
        word_bank_state
            .set_summary_return_page
            .set(summary_return_page);
        handle_finish_click(
            word_bank_state,
            set_current_page,
            set_status,
            finish_target_page,
        );
    };
    let label = label.unwrap_or_else(|| tr(lang.get_untracked(), "完成练习", "Finish Practice").to_string());
    let class = class.unwrap_or_else(|| {
        "rounded-lg border border-indigo-600 bg-indigo-700 px-4 py-3 text-sm font-semibold text-white hover:bg-indigo-600"
            .to_string()
    });

    view! {
        <button type="button" on:click=finish_click class=class>
            {label}
        </button>
    }
}

#[component]
pub fn RestartPracticeButton(
    word_bank_state: WordBankState,
    set_status: WriteSignal<String>,
    on_restart_ui: Callback<()>,
    restart_message: String,
    restart_temp_behavior: RestartTempBehavior,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let lang = word_bank_state.ui_language;
    let restart_click = move |_| {
        handle_restart_click(
            word_bank_state,
            set_status,
            on_restart_ui,
            &restart_message,
            restart_temp_behavior,
        );
    };
    let label =
        label.unwrap_or_else(|| tr(lang.get_untracked(), "重新练习", "Restart").to_string());
    let class = class.unwrap_or_else(|| {
        "rounded-lg border border-amber-600 bg-amber-700 px-4 py-3 text-sm font-semibold text-white hover:bg-amber-600"
            .to_string()
    });

    view! {
        <button type="button" on:click=restart_click class=class>
            {label}
        </button>
    }
}

#[component]
pub fn AbortPracticeButton(
    word_bank_state: WordBankState,
    set_current_page: WriteSignal<AppPage>,
    abort_target_page: AppPage,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let lang = word_bank_state.ui_language;
    let abort_click = move |_| {
        handle_abort_click(word_bank_state, set_current_page, abort_target_page);
    };
    let label = label.unwrap_or_else(|| {
        tr(lang.get_untracked(), "放弃练习并返回", "Abort and Return").to_string()
    });
    let class = class.unwrap_or_else(|| {
        "rounded-lg border border-rose-600 bg-rose-700 px-4 py-3 text-sm font-semibold text-white hover:bg-rose-600"
            .to_string()
    });

    view! {
        <button type="button" on:click=abort_click class=class>
            {label}
        </button>
    }
}

pub fn create_temp_practice_result(
    base: &PracticeResult,
    selected_ids: &[String],
) -> PracticeResult {
    PracticeResult {
        username: base.username.clone(),
        encryption_key: base.encryption_key.clone(),
        selected_word_entry_ids: selected_ids.to_vec(),
        practiced_word_entries: Vec::new(),
    }
}

pub fn handle_finish_click(
    word_bank_state: WordBankState,
    set_current_page: WriteSignal<AppPage>,
    set_status: WriteSignal<String>,
    finish_target_page: AppPage,
) {
    let lang = word_bank_state.ui_language.get_untracked();
    let mut selected_ids = word_bank_state.selected_word_entry_ids.get_untracked();
    if selected_ids.is_empty() {
        set_status.set(
            tr(lang, "当前没有可练习词条。", "No entries available for practice.").to_string(),
        );
        return;
    }

    let temp_snapshot = word_bank_state.temp_practice_result.get_untracked();
    word_bank_state.set_practice_result.update(|global_result| {
        merge_practice_result(global_result, &temp_snapshot);
    });
    word_bank_state
        .set_last_completed_practice_result
        .set(temp_snapshot.clone());

    shuffle_strings(&mut selected_ids);
    word_bank_state
        .set_selected_word_entry_ids
        .set(selected_ids.clone());

    let latest_global = word_bank_state.practice_result.get_untracked();
    word_bank_state
        .set_temp_practice_result
        .set(create_temp_practice_result(&latest_global, &selected_ids));
    set_current_page.set(finish_target_page);
}

pub fn handle_restart_click(
    word_bank_state: WordBankState,
    set_status: WriteSignal<String>,
    on_restart_ui: Callback<()>,
    restart_message: &str,
    restart_temp_behavior: RestartTempBehavior,
) {
    if restart_temp_behavior == RestartTempBehavior::Reinitialize {
        let selected_ids = word_bank_state.selected_word_entry_ids.get_untracked();
        let base_result = word_bank_state.practice_result.get_untracked();
        word_bank_state
            .set_temp_practice_result
            .set(create_temp_practice_result(&base_result, &selected_ids));
    }

    on_restart_ui.run(());
    set_status.set(restart_message.to_string());
}

pub fn handle_abort_click(
    word_bank_state: WordBankState,
    set_current_page: WriteSignal<AppPage>,
    abort_target_page: AppPage,
) {
    word_bank_state
        .set_temp_practice_result
        .set(PracticeResult::default());
    set_current_page.set(abort_target_page);
}

pub fn normalize_for_compare(value: &str) -> String {
    value
        .split('|')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("|")
        .to_lowercase()
        .replace('æ', "ae")
        .replace('ø', "oe")
        .replace('å', "aa")
}

pub fn record_field_check_result(
    temp_result: &mut PracticeResult,
    entry_id: &str,
    field: &str,
    is_correct: bool,
    user_input: &str,
) {
    let practiced_entry = get_or_insert_practiced_entry(temp_result, entry_id);
    let Some(stats) = answer_stats_mut(practiced_entry, field) else {
        return;
    };

    if is_correct {
        stats.correct_count += 1;
        return;
    }

    stats.wrong_count += 1;
    let wrong_input = user_input.trim();
    if wrong_input.is_empty() {
        return;
    }
    push_or_promote_wrong_answer(&mut stats.wrong_answers, wrong_input);
}

pub fn merge_practice_result(global_result: &mut PracticeResult, temp_result: &PracticeResult) {
    if global_result.username.trim().is_empty() && !temp_result.username.trim().is_empty() {
        global_result.username = temp_result.username.clone();
    }
    if global_result.encryption_key.trim().is_empty()
        && !temp_result.encryption_key.trim().is_empty()
    {
        global_result.encryption_key = temp_result.encryption_key.clone();
    }

    for id in &temp_result.selected_word_entry_ids {
        if !global_result
            .selected_word_entry_ids
            .iter()
            .any(|existing| existing == id)
        {
            global_result.selected_word_entry_ids.push(id.clone());
        }
    }

    for temp_entry in &temp_result.practiced_word_entries {
        if let Some(existing_entry) = global_result
            .practiced_word_entries
            .iter_mut()
            .find(|entry| entry.id == temp_entry.id)
        {
            merge_practiced_entry(existing_entry, temp_entry);
        } else {
            global_result
                .practiced_word_entries
                .push(temp_entry.clone());
        }
    }
}

fn merge_practiced_entry(
    existing_entry: &mut PracticedWordEntryResult,
    temp_entry: &PracticedWordEntryResult,
) {
    merge_answer_stats(&mut existing_entry.english, &temp_entry.english);
    merge_answer_stats(&mut existing_entry.chinese, &temp_entry.chinese);
    merge_answer_stats(&mut existing_entry.base_form, &temp_entry.base_form);
    merge_answer_stats(
        &mut existing_entry.verb_present_tense,
        &temp_entry.verb_present_tense,
    );
    merge_answer_stats(&mut existing_entry.verb_past_tense, &temp_entry.verb_past_tense);
    merge_answer_stats(&mut existing_entry.verb_imperative, &temp_entry.verb_imperative);
    merge_answer_stats(&mut existing_entry.noun_plural, &temp_entry.noun_plural);
    merge_answer_stats(
        &mut existing_entry.noun_singular_definite,
        &temp_entry.noun_singular_definite,
    );
    merge_answer_stats(
        &mut existing_entry.noun_plural_definite,
        &temp_entry.noun_plural_definite,
    );
    merge_answer_stats(&mut existing_entry.adjective_neuter_form, &temp_entry.adjective_neuter_form);
    merge_answer_stats(&mut existing_entry.adjective_plural_form, &temp_entry.adjective_plural_form);
    merge_answer_stats(
        &mut existing_entry.adjective_comparative,
        &temp_entry.adjective_comparative,
    );
    merge_answer_stats(
        &mut existing_entry.adjective_superlative_indefinite,
        &temp_entry.adjective_superlative_indefinite,
    );
    merge_answer_stats(
        &mut existing_entry.adjective_superlative_definite,
        &temp_entry.adjective_superlative_definite,
    );
    merge_answer_stats(
        &mut existing_entry.adverb_comparative,
        &temp_entry.adverb_comparative,
    );
    merge_answer_stats(
        &mut existing_entry.adverb_superlative,
        &temp_entry.adverb_superlative,
    );
}

fn merge_answer_stats(existing_stats: &mut AnswerStats, temp_stats: &AnswerStats) {
    existing_stats.correct_count += temp_stats.correct_count;
    existing_stats.wrong_count += temp_stats.wrong_count;

    for wrong in temp_stats.wrong_answers.iter().rev() {
        push_or_promote_wrong_answer(&mut existing_stats.wrong_answers, wrong);
    }
}

fn push_or_promote_wrong_answer(list: &mut Vec<String>, wrong_answer: &str) {
    let normalized = normalize_for_compare(wrong_answer);
    if normalized.is_empty() {
        return;
    }

    if let Some(existing_pos) = list
        .iter()
        .position(|current| normalize_for_compare(current) == normalized)
    {
        list.remove(existing_pos);
    }

    list.insert(0, wrong_answer.trim().to_string());
    if list.len() > MAX_WRONG_ANSWERS_PER_FORM {
        list.truncate(MAX_WRONG_ANSWERS_PER_FORM);
    }
}

fn get_or_insert_practiced_entry<'a>(
    temp_result: &'a mut PracticeResult,
    entry_id: &str,
) -> &'a mut PracticedWordEntryResult {
    if let Some(index) = temp_result
        .practiced_word_entries
        .iter()
        .position(|entry| entry.id == entry_id)
    {
        return &mut temp_result.practiced_word_entries[index];
    }

    let entry = PracticedWordEntryResult {
        id: entry_id.to_string(),
        ..PracticedWordEntryResult::default()
    };
    temp_result.practiced_word_entries.push(entry);
    let last_index = temp_result.practiced_word_entries.len() - 1;
    &mut temp_result.practiced_word_entries[last_index]
}

fn answer_stats_mut<'a>(
    practiced_entry: &'a mut PracticedWordEntryResult,
    field: &str,
) -> Option<&'a mut AnswerStats> {
    match field {
        "english" => Some(&mut practiced_entry.english),
        "chinese" => Some(&mut practiced_entry.chinese),
        "base_form" => Some(&mut practiced_entry.base_form),
        "verb_present_tense" => Some(&mut practiced_entry.verb_present_tense),
        "verb_past_tense" => Some(&mut practiced_entry.verb_past_tense),
        "verb_imperative" => Some(&mut practiced_entry.verb_imperative),
        "noun_plural" => Some(&mut practiced_entry.noun_plural),
        "noun_singular_definite" => Some(&mut practiced_entry.noun_singular_definite),
        "noun_plural_definite" => Some(&mut practiced_entry.noun_plural_definite),
        "adjective_neuter_form" => Some(&mut practiced_entry.adjective_neuter_form),
        "adjective_plural_form" => Some(&mut practiced_entry.adjective_plural_form),
        "adjective_comparative" => Some(&mut practiced_entry.adjective_comparative),
        "adjective_superlative_indefinite" => {
            Some(&mut practiced_entry.adjective_superlative_indefinite)
        }
        "adjective_superlative_definite" => {
            Some(&mut practiced_entry.adjective_superlative_definite)
        }
        "adverb_comparative" => Some(&mut practiced_entry.adverb_comparative),
        "adverb_superlative" => Some(&mut practiced_entry.adverb_superlative),
        _ => None,
    }
}
