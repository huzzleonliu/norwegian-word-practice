//! AI/词典查询模块入口：对外暴露 `AiResearcher` 组件与共享转换工具。

pub mod dictionary_search;
pub mod gemini_search;
pub mod layout;
pub mod search_process;
mod shared;
mod types;

pub use layout::{AiResearcher, AiResearcherActions, SingleEntryFormState};
pub(crate) use shared::{
    build_bulk_csv_from_results, hint_to_part_of_speech, join_pipe, merge_or_insert_word_result,
    merge_word_result, normalize_part_of_speech, normalize_text_opt,
};
pub(crate) use types::{GeminiWordResult, WORD_FORM_HINT_OPTIONS, form_hint_label};
