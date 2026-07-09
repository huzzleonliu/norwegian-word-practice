use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::pages::AppPage;

#[component]
pub fn PracticeModePage() -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let word_bank_state = expect_context::<WordBankState>();

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
            <section class="w-full max-w-2xl p-8 rounded-2xl border border-slate-800 bg-slate-900 shadow-xl">
                <h1 class="text-3xl font-bold tracking-tight">"请选择练习模式"</h1>
                <p class="mt-4 text-slate-300">
                    {move || {
                        let count = word_bank_state.entries.get().len();
                        let source = word_bank_state.source_name.get();
                        if count == 0 {
                            "尚未加载词库，请先回到首页选择词库或导入词库 CSV。".to_string()
                        } else {
                            format!("当前词库：{source}（共 {count} 条词条）。")
                        }
                    }}
                </p>

                <div class="mt-6 flex flex-wrap items-center gap-3">
                    <button
                        type="button"
                        on:click=move |_| set_current_page.set(AppPage::LexiconMode)
                        class="inline-flex items-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                    >
                        "词库模式"
                    </button>
                    <button
                        type="button"
                        class="inline-flex items-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                    >
                        "单词系列模式"
                    </button>
                </div>

                <div class="mt-10 text-right">
                    <button
                        type="button"
                        on:click=move |_| set_current_page.set(AppPage::LocalLexiconEditor)
                        class="text-sm text-slate-400 underline underline-offset-4 hover:text-slate-300"
                    >
                        "本地词库修改器"
                    </button>
                </div>
            </section>
        </main>
    }
}
