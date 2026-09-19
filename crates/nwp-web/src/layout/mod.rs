//! 应用布局：页面母版、全局控件，以及不对应独立路由的使用说明浮层。

mod chrome;
mod help;
mod page_shell;

pub use chrome::MasterChrome;
pub use page_shell::{PageEyebrow, PageHeading, PageShell, PageTitle, PageTopbar};

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::app_state::{NavigateToPage, UiState};
use crate::pages::AppPage;

/// 应用母版根：注入导航，渲染全局控件，并承托各页内容。
#[component]
pub fn MasterLayout(children: Children) -> impl IntoView {
    let ui = expect_context::<UiState>();
    let navigate = use_navigate();
    let navigate_to_page = NavigateToPage(Callback::new({
        let navigate = navigate.clone();
        move |page: AppPage| {
            let lang = ui.ui_language.get_untracked();
            navigate(&page.localized_path(lang), Default::default());
        }
    }));
    provide_context(navigate_to_page);

    view! {
        <div>
            <MasterChrome/>
            {children()}
        </div>
    }
}
