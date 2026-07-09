use leptos::prelude::*;

pub const NONE_FIELD_KEY: &str = "none";

const PROMPT_FIELD_OPTIONS: [(&str, &str); 18] = [
    (NONE_FIELD_KEY, "无"),
    ("part_of_speech", "part_of_speech"),
    ("tags", "tags"),
    ("english", "english"),
    ("chinese", "chinese"),
    ("base_form", "base_form"),
    ("past_tense", "past_tense"),
    ("imperative", "imperative"),
    ("plural", "plural"),
    ("singular_definite", "singular_definite"),
    ("plural_definite", "plural_definite"),
    ("neuter_form", "neuter_form"),
    ("plural_form", "plural_form"),
    ("adjective_comparative", "adjective_comparative"),
    (
        "adjective_superlative_indefinite",
        "adjective_superlative_indefinite",
    ),
    (
        "adjective_superlative_definite",
        "adjective_superlative_definite",
    ),
    ("adverb_comparative", "adverb_comparative"),
    ("adverb_superlative", "adverb_superlative"),
];

const ANSWER_FIELD_OPTIONS: [(&str, &str); 15] = [
    ("english", "english"),
    ("chinese", "chinese"),
    ("base_form", "base_form"),
    ("past_tense", "past_tense"),
    ("imperative", "imperative"),
    ("plural", "plural"),
    ("singular_definite", "singular_definite"),
    ("plural_definite", "plural_definite"),
    ("neuter_form", "neuter_form"),
    ("plural_form", "plural_form"),
    ("adjective_comparative", "adjective_comparative"),
    (
        "adjective_superlative_indefinite",
        "adjective_superlative_indefinite",
    ),
    (
        "adjective_superlative_definite",
        "adjective_superlative_definite",
    ),
    ("adverb_comparative", "adverb_comparative"),
    ("adverb_superlative", "adverb_superlative"),
];

#[component]
pub fn PracticeSettings(
    questions_per_page: ReadSignal<usize>,
    set_questions_per_page: WriteSignal<usize>,
    prompt_field_a: ReadSignal<String>,
    set_prompt_field_a: WriteSignal<String>,
    prompt_field_b: ReadSignal<String>,
    set_prompt_field_b: WriteSignal<String>,
    answer_fields: ReadSignal<Vec<String>>,
    set_answer_fields: WriteSignal<Vec<String>>,
) -> impl IntoView {
    view! {
        <div class="mt-4 grid grid-cols-1 gap-3 lg:grid-cols-3">
            <label class="flex flex-col gap-1 text-sm">
                <span class="text-slate-300">"一页练习（词条数）"</span>
                <input
                    type="number"
                    min="1"
                    prop:value=move || questions_per_page.get().to_string()
                    on:input=move |ev| {
                        let raw = event_target_value(&ev);
                        if let Ok(value) = raw.parse::<usize>() {
                            set_questions_per_page.set(value.max(1));
                        }
                    }
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
            </label>

            <label class="flex flex-col gap-1 text-sm">
                <span class="text-slate-300">"根据（下拉框 1）"</span>
                <select
                    prop:value=move || prompt_field_a.get()
                    on:change=move |ev| set_prompt_field_a.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                >
                    {PROMPT_FIELD_OPTIONS
                        .iter()
                        .map(|(key, label)| view! { <option value=*key>{*label}</option> })
                        .collect_view()}
                </select>
            </label>

            <label class="flex flex-col gap-1 text-sm">
                <span class="text-slate-300">"根据（下拉框 2）"</span>
                <select
                    prop:value=move || prompt_field_b.get()
                    on:change=move |ev| set_prompt_field_b.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                >
                    {PROMPT_FIELD_OPTIONS
                        .iter()
                        .map(|(key, label)| view! { <option value=*key>{*label}</option> })
                        .collect_view()}
                </select>
            </label>
        </div>

        <div class="mt-4">
            <p class="mb-2 text-sm text-slate-300">"回答（勾选要回答的项）"</p>
            <div class="grid grid-cols-2 gap-2 md:grid-cols-3 lg:grid-cols-4">
                {ANSWER_FIELD_OPTIONS
                    .iter()
                    .map(|(key, label)| {
                        let key_for_checked = (*key).to_string();
                        let key_for_change = (*key).to_string();
                        view! {
                            <label class="inline-flex items-center gap-2 rounded border border-slate-800 bg-slate-900 px-2 py-1 text-xs text-slate-300">
                                <input
                                    type="checkbox"
                                    prop:checked=move || {
                                        answer_fields
                                            .get()
                                            .iter()
                                            .any(|field| field == &key_for_checked)
                                    }
                                    on:change=move |ev| {
                                        let checked = event_target_checked(&ev);
                                        let key_to_toggle = key_for_change.clone();
                                        set_answer_fields.update(|fields| {
                                            if checked {
                                                if !fields.iter().any(|field| field == &key_to_toggle) {
                                                    fields.push(key_to_toggle.clone());
                                                }
                                            } else {
                                                fields.retain(|field| field != &key_to_toggle);
                                            }
                                        });
                                    }
                                />
                                <span>{*label}</span>
                            </label>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}
