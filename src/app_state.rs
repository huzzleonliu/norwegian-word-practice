//! 全局状态定义：集中管理词库、练习结果、页面返回目标等跨页面共享数据。

use leptos::prelude::*;

use crate::pages::AppPage;
use crate::structures::pracresult::PracticeResult;
use crate::structures::word_bank_entry::{UiLanguage, UiTheme, WordBankEntry};

/// 词库域状态：词条数据源与词库来源信息。
#[derive(Clone, Copy)]
pub struct LexiconState {
    /// 已提交词库；练习页、导出、总结页等都读取这里。
    pub entries: ReadSignal<Vec<WordBankEntry>>,
    pub set_entries: WriteSignal<Vec<WordBankEntry>>,
    /// 外部重载词库版本号（例如导入 CSV、切换内置词库、单条/批量新增）。
    pub data_version: ReadSignal<u64>,
    pub set_data_version: WriteSignal<u64>,
    pub source_name: ReadSignal<String>,
    pub set_source_name: WriteSignal<String>,
}

/// 练习域状态：选题范围、临时统计、历史统计与总结返回目标页。
#[derive(Clone, Copy)]
pub struct PracticeState {
    /// 当前待练习词条 id 列表（会被 shuffle 后用于练习队列）。
    pub selected_word_entry_ids: ReadSignal<Vec<String>>,
    pub set_selected_word_entry_ids: WriteSignal<Vec<String>>,
    /// 历史累计练习结果。
    pub practice_result: ReadSignal<PracticeResult>,
    pub set_practice_result: WriteSignal<PracticeResult>,
    /// 当前轮次临时结果（完成练习时合并进 `practice_result`）。
    pub temp_practice_result: ReadSignal<PracticeResult>,
    pub set_temp_practice_result: WriteSignal<PracticeResult>,
    /// 最近一次完成练习时的快照（总结页“本次结果”读取它）。
    pub last_completed_practice_result: ReadSignal<PracticeResult>,
    pub set_last_completed_practice_result: WriteSignal<PracticeResult>,
    /// 总结页“继续练习/返回”时的目标页面。
    pub summary_return_page: ReadSignal<AppPage>,
    pub set_summary_return_page: WriteSignal<AppPage>,
}

/// UI 域状态：与业务无关的界面配置。
#[derive(Clone, Copy)]
pub struct UiState {
    pub ui_language: ReadSignal<UiLanguage>,
    #[allow(dead_code)]
    pub ui_theme: ReadSignal<UiTheme>,
}
