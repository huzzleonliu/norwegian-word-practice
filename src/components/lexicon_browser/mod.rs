//! 词库浏览器模块入口：导出主组件、模式枚举与 CSV 工具函数。

pub mod components;
pub mod search_panel;
pub mod structures;
pub mod table_panel;
pub mod utils;

pub use components::LexiconBrowser;
pub use structures::LexiconBrowserMode;
pub use utils::{parse_pipe_list, parse_word_bank_csv, serialize_word_bank_csv};
