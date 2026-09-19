//! 跟读播放器页：本地音频/歌词与内置音频书。

use leptos::prelude::*;

use crate::layout::{PageEyebrow, PageHeading, PageShell, PageTitle, PageTopbar, ShellAttach};
use crate::components::web_player::WebPlayer;
use crate::app_state::UiState;
use crate::utils::i18n::tr;

#[component]
pub fn PlayerPage() -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    view! {
        <PageShell attach=ShellAttach::Fill>
            <PageTopbar>
                <PageHeading>
                    <PageEyebrow>
                        {move || tr(lang.get(), "跟读", "listen")}
                    </PageEyebrow>
                    <PageTitle>
                        {move || tr(lang.get(), "跟读练习", "Read along")}
                    </PageTitle>
                </PageHeading>
            </PageTopbar>
            <WebPlayer/>
        </PageShell>
    }
}
