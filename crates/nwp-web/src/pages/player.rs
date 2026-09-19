//! 跟读播放器页：本地音频/歌词与内置音频书。

use leptos::prelude::*;

use crate::layout::{PageEyebrow, PageHeading, PageShell, PageTitle, PageTopbar};
use crate::components::return_button::ReturnButton;
use crate::components::web_player::WebPlayer;
use crate::pages::AppPage;
use crate::app_state::UiState;
use crate::utils::i18n::tr;

#[component]
pub fn PlayerPage() -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    view! {
        <PageShell>
            <PageTopbar>
                <PageHeading>
                    <PageEyebrow>
                        {move || tr(lang.get(), "跟读", "listen")}
                    </PageEyebrow>
                    <PageTitle>
                        {move || tr(lang.get(), "跟读练习", "Read along")}
                    </PageTitle>
                </PageHeading>
                <ReturnButton target_page=AppPage::PracticeModeSelect/>
            </PageTopbar>
            <WebPlayer/>
        </PageShell>
    }
}
