//! 练习引擎目录入口：按职责拆分练习相关模块，降低跨文件耦合。
//! - `settings`：练习设置 UI（题量、提示字段、回答字段）
//! - `entry`：题目与输入 UI（输入键生成、可答性判断）
//! - `buttons`：流程按钮与结果合并（完成/重开/放弃）
//! - `utils`：纯函数逻辑（判题、补题、清理局部状态）

pub mod buttons;
pub mod entry;
pub mod settings;
pub mod utils;

pub use buttons::{
    AbortPracticeButton, FinishPracticeButton, RestartPracticeButton, RestartTempBehavior,
    create_temp_practice_result, normalize_for_compare, record_field_check_result,
};
pub use entry::{
    CheckPracticeButton, PracticeEntry, answer_input_key, entry_field_value,
    is_answer_field_available,
};
pub use settings::{AnswerFieldsSettings, PracticeSettings, default_answer_fields};
pub use utils::{
    CheckAnswersError, apply_check_results, clear_practice_round_local_state,
    evaluate_check_answers, merge_solved_question_ids, prepare_post_check_input_state,
    refill_active_question_ids, retain_unsolved_revealed_keys,
};
