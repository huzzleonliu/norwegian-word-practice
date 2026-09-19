//! 跟读播放器页：本地音频/歌词与内置音频书。

use leptos::prelude::*;

use crate::components::return_button::ReturnButton;
use crate::components::web_player::WebPlayer;
use crate::pages::AppPage;
use crate::app_state::UiState;
use crate::utils::i18n::tr;

#[component]
pub fn PlayerPage() -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    view! {
        <main class="ui-page">
            <section class="ui-shell">
                <div class="ui-topbar">
                    <div>
                        <p class="ui-eyebrow">
                            {move || tr(lang.get(), "跟读", "listen")}
                        </p>
                        <h1 class="ui-title">
                            {move || tr(lang.get(), "跟读练习", "Read along")}
                        </h1>
                    </div>
                    <ReturnButton target_page=AppPage::PracticeModeSelect/>
                </div>
                <WebPlayer/>
            </section>
        </main>
    }
}
