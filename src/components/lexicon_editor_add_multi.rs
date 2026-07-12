use leptos::ev::SubmitEvent;
use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::utils::i18n::tr;

#[component]
pub fn LexiconEditorAddMulti(
    on_submit: Callback<SubmitEvent>,
    bulk_input: ReadSignal<String>,
    set_bulk_input: WriteSignal<String>,
    bulk_errors: ReadSignal<Vec<String>>,
    set_bulk_errors: WriteSignal<Vec<String>>,
    bulk_success_message: ReadSignal<String>,
    set_bulk_success_message: WriteSignal<String>,
) -> impl IntoView {
    let lang = expect_context::<WordBankState>().ui_language;
    view! {
        <form
            on:submit=move |ev| on_submit.run(ev)
            class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-4"
        >
            <h2 class="mb-3 text-lg font-semibold">
                {move || tr(lang.get(), "多条添加（CSV，多行）", "Bulk Add (CSV, multiple rows)")}
            </h2>
            <textarea
                placeholder=move || {
                    tr(
                        lang.get(),
                        "粘贴 CSV 文本（含表头），数组字段用 | 分隔",
                        "Paste CSV with header; list fields separated by |",
                    )
                }
                prop:value=move || bulk_input.get()
                on:input=move |ev| {
                    set_bulk_input.set(event_target_value(&ev));
                    set_bulk_errors.set(Vec::new());
                    set_bulk_success_message.set(String::new());
                }
                class="min-h-40 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
            ></textarea>
            {move || {
                let errors = bulk_errors.get();
                if errors.is_empty() {
                    view! { <></> }.into_any()
                } else {
                    view! {
                        <ul class="mt-3 list-disc space-y-1 pl-5 text-sm text-red-400">
                            {errors
                                .into_iter()
                                .map(|item| view! { <li>{item}</li> })
                                .collect_view()}
                        </ul>
                    }
                        .into_any()
                }
            }}
            {move || {
                let message = bulk_success_message.get();
                if message.is_empty() {
                    view! { <></> }.into_any()
                } else {
                    view! { <p class="mt-3 text-sm font-medium text-emerald-400">{message}</p> }
                        .into_any()
                }
            }}
            <button
                type="submit"
                class="mt-3 w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium hover:bg-slate-700 sm:w-auto"
            >
                {move || tr(lang.get(), "添加多条", "Add Entries")}
            </button>
        </form>
    }
}
