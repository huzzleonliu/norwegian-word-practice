use leptos::prelude::*;

use crate::pages::AppPage;

#[component]
pub fn HomePage() -> impl IntoView {
    let set_current_page =
        expect_context::<WriteSignal<AppPage>>();

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center p-6">
            <section class="w-full max-w-2xl p-8 rounded-2xl border border-slate-800 bg-slate-900 shadow-xl">
                <h1 class="text-3xl font-bold tracking-tight">"Norwegian Word Practice"</h1>
                <p class="mt-4 text-slate-300">
                    "导入你的练习结果（.pracresult）后开始练习。"
                </p>

                <input
                    id="pracresult-input"
                    type="file"
                    accept=".pracresult"
                    class="hidden"
                />

                <div class="mt-6 flex flex-wrap items-center gap-3">
                    <label
                        for="pracresult-input"
                        class="inline-flex cursor-pointer items-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                    >
                        "导入练习结果"
                    </label>

                    <button
                        type="button"
                        on:click=move |_| set_current_page.set(AppPage::PracticeModeSelect)
                        class="inline-flex items-center rounded-lg border border-slate-700 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600"
                    >
                        "直接开始练习"
                    </button>
                </div>
            </section>
        </main>
    }
}
