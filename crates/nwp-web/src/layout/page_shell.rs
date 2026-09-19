//! 页面版面结构：外层画布、内容卡片、页眉标题区。
//! 视觉 token 仍定义在 `style/input.css` 的 `.ui-page` / `.ui-shell` 等规则中。

use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;

pub const PAGE_CLASS: &str = "ui-page";
pub const SHELL_CLASS: &str = "ui-shell";
pub const TOPBAR_CLASS: &str = "ui-topbar";
pub const EYEBROW_CLASS: &str = "ui-eyebrow";
pub const TITLE_CLASS: &str = "ui-title";

/// 卡片如何吸附到视口：随内容收缩，或按屏幕比例铺满。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShellAttach {
    /// 宽高跟随内容（首页）。
    #[default]
    Content,
    /// 宽度按屏幕比例，高度铺满视口可用区域，内容从上往下排；超出在卡片内滚动。
    Fill,
}

impl ShellAttach {
    fn page_class(self) -> String {
        let attach = match self {
            Self::Content => "ui-page-attach-content",
            Self::Fill => "ui-page-attach-fill",
        };
        format!("{PAGE_CLASS} {attach}")
    }
}

/// 页面外框。
///
/// 吸附变量（也可由调用方覆盖 CSS 变量）：
/// - `--ui-shell-width`
/// - `--ui-shell-max-width`
/// - `--ui-shell-min-height`
///
/// 有上一级时，右上角固定显示返回按钮。
#[component]
pub fn PageShell(
    #[prop(optional)] attach: ShellAttach,
    #[prop(optional)] width: Option<&'static str>,
    #[prop(optional)] max_width: Option<&'static str>,
    #[prop(optional)] min_height: Option<&'static str>,
    #[prop(optional)] back_to: Option<AppPage>,
    children: Children,
) -> impl IntoView {
    let location = use_location();
    let back_target = Memo::new(move |_| {
        back_to.or_else(|| {
            AppPage::from_path(&location.pathname.get()).and_then(AppPage::parent)
        })
    });
    let page_class = attach.page_class();
    let shell_style = shell_var_style(width, max_width, min_height);
    view! {
        <main class=page_class>
            <section
                class=SHELL_CLASS
                class:ui-shell-has-back=move || back_target.get().is_some()
                style=shell_style
            >
                {move || {
                    back_target.get().map(|target_page| {
                        view! {
                            <div class="ui-shell-back">
                                <ReturnButton target_page=target_page/>
                            </div>
                        }
                    })
                }}
                {children()}
            </section>
        </main>
    }
}

fn shell_var_style(
    width: Option<&'static str>,
    max_width: Option<&'static str>,
    min_height: Option<&'static str>,
) -> String {
    let mut items = Vec::new();
    if let Some(width) = width {
        items.push(format!("--ui-shell-width:{width}"));
    }
    if let Some(max_width) = max_width {
        items.push(format!("--ui-shell-max-width:{max_width}"));
    }
    if let Some(min_height) = min_height {
        items.push(format!("--ui-shell-min-height:{min_height}"));
    }
    items.join(";")
}

/// 页眉：标题区。返回按钮由 `PageShell` 固定在卡片右上角。
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
