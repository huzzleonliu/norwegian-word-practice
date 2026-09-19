//! 页面版面结构：外层画布、内容卡片、页眉标题区。
//! 视觉 token 仍定义在 `style/input.css` 的 `.ui-page` / `.ui-shell` 等规则中。

use leptos::prelude::*;

pub const PAGE_CLASS: &str = "ui-page";
pub const SHELL_CLASS: &str = "ui-shell";
pub const SHELL_NARROW_CLASS: &str = "ui-shell ui-shell-narrow";
pub const TOPBAR_CLASS: &str = "ui-topbar";
pub const EYEBROW_CLASS: &str = "ui-eyebrow";
pub const TITLE_CLASS: &str = "ui-title";

/// 页面外框：居中卡片。`narrow` 用于首页等较窄内容。
#[component]
pub fn PageShell(#[prop(optional)] narrow: bool, children: Children) -> impl IntoView {
    let shell_class = if narrow {
        SHELL_NARROW_CLASS
    } else {
        SHELL_CLASS
    };
    view! {
        <main class=PAGE_CLASS>
            <section class=shell_class>{children()}</section>
        </main>
    }
}

/// 页眉：左侧标题区 + 右侧操作（通常是返回按钮）。
#[component]
pub fn PageTopbar(children: Children) -> impl IntoView {
    view! { <div class=TOPBAR_CLASS>{children()}</div> }
}

/// 页眉左侧：eyebrow + 标题的容器。
#[component]
pub fn PageHeading(children: Children) -> impl IntoView {
    view! { <div>{children()}</div> }
}

#[component]
pub fn PageEyebrow(children: Children) -> impl IntoView {
    view! { <p class=EYEBROW_CLASS>{children()}</p> }
}

#[component]
pub fn PageTitle(children: Children) -> impl IntoView {
    view! { <h1 class=TITLE_CLASS>{children()}</h1> }
}
