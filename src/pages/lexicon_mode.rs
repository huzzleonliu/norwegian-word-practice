use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::components::lexicon_browser::{LexiconBrowser, LexiconBrowserMode};
use crate::pages::AppPage;

#[component]
pub fn LexiconModePage() -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let word_bank_state = expect_context::<WordBankState>();
    let entries = word_bank_state.entries;
    let set_entries = word_bank_state.set_entries;
    let data_version = word_bank_state.data_version;
    let (status, set_status) = signal(String::new());

    Effect::new(move |_| {
        let _ = data_version.get();
        let count = entries.get().len();
        let source = word_bank_state.source_name.get();
        if count == 0 {
            set_status.set("当前词库为空，请先回到首页加载或导入词库。".to_string());
        } else {
            set_status.set(format!(
                "请选择想要练习的单词（词库：{source}，共 {count} 条）"
            ));
        }
    });

    let start_practice_click = move |_| {
        set_status.set("开始练习功能建设中。".to_string());
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-6">
            <section class="mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-xl">
                <header class="flex items-center justify-between gap-4">
                    <h1 class="text-2xl font-bold tracking-tight">"请选择想要练习的单词"</h1>
                    <button
                        type="button"
                        on:click=move |_| set_current_page.set(AppPage::PracticeModeSelect)
                        class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-sm hover:bg-slate-700"
                    >
                        "返回"
                    </button>
                </header>
                <p class="mt-3 text-sm text-slate-300">{move || status.get()}</p>

                <LexiconBrowser
                    entries=entries
                    set_entries=set_entries
                    set_status=set_status
                    data_version=data_version
                    mode=LexiconBrowserMode::Query
                />

                <div class="mt-6 flex justify-center">
                    <button
                        type="button"
                        on:click=start_practice_click
                        class="inline-flex items-center rounded-lg border border-emerald-700 bg-emerald-700 px-6 py-2 text-sm font-medium text-white hover:bg-emerald-600"
                    >
                        "开始练习"
                    </button>
                </div>
            </section>
        </main>
    }
}
