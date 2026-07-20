//! 词库浏览器结构定义：仅保留浏览模式枚举。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexiconBrowserMode {
    Edit,
    Query,
}
