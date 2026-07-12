use leptos::ev::SubmitEvent;
use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::structures::word_bank_entry::{PART_OF_SPEECH_OPTIONS, PartOfSpeech};
use crate::utils::i18n::tr;

#[component]
pub fn LexiconEditorAddSingle(
    on_submit: Callback<SubmitEvent>,
    single_selected: ReadSignal<bool>,
    set_single_selected: WriteSignal<bool>,
    single_pos: ReadSignal<String>,
    set_single_pos: WriteSignal<String>,
    single_norwegian: ReadSignal<String>,
    set_single_norwegian: WriteSignal<String>,
    single_chinese: ReadSignal<String>,
    set_single_chinese: WriteSignal<String>,
    single_english: ReadSignal<String>,
    set_single_english: WriteSignal<String>,
    single_tags: ReadSignal<String>,
    set_single_tags: WriteSignal<String>,
    single_verb_present_tense: ReadSignal<String>,
    set_single_verb_present_tense: WriteSignal<String>,
    single_past_tense: ReadSignal<String>,
    set_single_past_tense: WriteSignal<String>,
    single_imperative: ReadSignal<String>,
    set_single_imperative: WriteSignal<String>,
    single_plural: ReadSignal<String>,
    set_single_plural: WriteSignal<String>,
    single_singular_definite: ReadSignal<String>,
    set_single_singular_definite: WriteSignal<String>,
    single_plural_definite: ReadSignal<String>,
    set_single_plural_definite: WriteSignal<String>,
    single_neuter_form: ReadSignal<String>,
    set_single_neuter_form: WriteSignal<String>,
    single_plural_form: ReadSignal<String>,
    set_single_plural_form: WriteSignal<String>,
    single_adjective_comparative: ReadSignal<String>,
    set_single_adjective_comparative: WriteSignal<String>,
    single_adjective_superlative_indefinite: ReadSignal<String>,
    set_single_adjective_superlative_indefinite: WriteSignal<String>,
    single_adjective_superlative_definite: ReadSignal<String>,
    set_single_adjective_superlative_definite: WriteSignal<String>,
    single_adverb_comparative: ReadSignal<String>,
    set_single_adverb_comparative: WriteSignal<String>,
    single_adverb_superlative: ReadSignal<String>,
    set_single_adverb_superlative: WriteSignal<String>,
) -> impl IntoView {
    let lang = expect_context::<WordBankState>().ui_language;
    view! {
        <form
            on:submit=move |ev| on_submit.run(ev)
            class="rounded-xl border border-slate-800 bg-slate-950/50 p-4"
        >
            <h2 class="mb-3 text-lg font-semibold">{move || tr(lang.get(), "单条添加", "Single Add")}</h2>
            <p class="mb-3 text-xs text-slate-400">
                {move || {
                    tr(
                        lang.get(),
                        "序号会自动由词性与词形字段计算哈希生成。",
                        "ID is auto-generated as a hash from part-of-speech and inflection fields.",
                    )
                }}
            </p>
            <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
                <label class="flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm">
                    <input
                        type="checkbox"
                        prop:checked=move || single_selected.get()
                        on:change=move |ev| set_single_selected.set(event_target_checked(&ev))
                    />
                    <span>{move || tr(lang.get(), "选中", "Selected")}</span>
                </label>
                <select
                    prop:value=move || single_pos.get()
                    on:change=move |ev| set_single_pos.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                >
                    {PART_OF_SPEECH_OPTIONS
                        .iter()
                        .map(|option| {
                            view! {
                                <option value=*option>
                                    {move || {
                                        PartOfSpeech::from_key(option)
                                            .map(|pos| pos.display_name(lang.get()).to_string())
                                            .unwrap_or_else(|_| (*option).to_string())
                                    }}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
                <input
                    type="text"
                    placeholder="base_form (norwegian_base)"
                    prop:value=move || single_norwegian.get()
                    on:input=move |ev| set_single_norwegian.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
                <input
                    type="text"
                    placeholder="中文（| 分隔）"
                    prop:value=move || single_chinese.get()
                    on:input=move |ev| set_single_chinese.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
                <input
                    type="text"
                    placeholder="英文（| 分隔）"
                    prop:value=move || single_english.get()
                    on:input=move |ev| set_single_english.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
                <input
                    type="text"
                    placeholder="tags（| 分隔）"
                    prop:value=move || single_tags.get()
                    on:input=move |ev| set_single_tags.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
                {move || {
                    if single_pos.get() == "verb" {
                        view! {
                            <>
                                <input
                                    type="text"
                                    placeholder="verb_present_tense（必填）"
                                    prop:value=move || single_verb_present_tense.get()
                                    on:input=move |ev| set_single_verb_present_tense.set(event_target_value(&ev))
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                                <input
                                    type="text"
                                    placeholder="verb_past_tense（必填）"
                                    prop:value=move || single_past_tense.get()
                                    on:input=move |ev| set_single_past_tense.set(event_target_value(&ev))
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                                <input
                                    type="text"
                                    placeholder="verb_imperative（必填）"
                                    prop:value=move || single_imperative.get()
                                    on:input=move |ev| set_single_imperative.set(event_target_value(&ev))
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                            </>
                        }
                            .into_any()
                    } else if single_pos.get() == "noun" {
                        view! {
                            <>
                                <input
                                    type="text"
                                    placeholder="noun_plural（必填）"
                                    prop:value=move || single_plural.get()
                                    on:input=move |ev| set_single_plural.set(event_target_value(&ev))
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                                <input
                                    type="text"
                                    placeholder="noun_singular_definite（必填）"
                                    prop:value=move || single_singular_definite.get()
                                    on:input=move |ev| {
                                        set_single_singular_definite.set(event_target_value(&ev))
                                    }
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                                <input
                                    type="text"
                                    placeholder="noun_plural_definite（必填）"
                                    prop:value=move || single_plural_definite.get()
                                    on:input=move |ev| set_single_plural_definite.set(event_target_value(&ev))
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                            </>
                        }
                            .into_any()
                    } else if single_pos.get() == "adjective" {
                        view! {
                            <>
                                <input
                                    type="text"
                                    placeholder="adjective_neuter_form（必填）"
                                    prop:value=move || single_neuter_form.get()
                                    on:input=move |ev| set_single_neuter_form.set(event_target_value(&ev))
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                                <input
                                    type="text"
                                    placeholder="adjective_plural_form（必填）"
                                    prop:value=move || single_plural_form.get()
                                    on:input=move |ev| set_single_plural_form.set(event_target_value(&ev))
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                                <input
                                    type="text"
                                    placeholder="adjective_comparative（必填）"
                                    prop:value=move || single_adjective_comparative.get()
                                    on:input=move |ev| {
                                        set_single_adjective_comparative.set(event_target_value(&ev))
                                    }
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                                <input
                                    type="text"
                                    placeholder="adjective_superlative_indefinite（必填）"
                                    prop:value=move || single_adjective_superlative_indefinite.get()
                                    on:input=move |ev| {
                                        set_single_adjective_superlative_indefinite
                                            .set(event_target_value(&ev))
                                    }
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                                <input
                                    type="text"
                                    placeholder="adjective_superlative_definite（必填）"
                                    prop:value=move || single_adjective_superlative_definite.get()
                                    on:input=move |ev| {
                                        set_single_adjective_superlative_definite
                                            .set(event_target_value(&ev))
                                    }
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                            </>
                        }
                            .into_any()
                    } else if single_pos.get() == "adverb" {
                        view! {
                            <>
                                <input
                                    type="text"
                                    placeholder="adverb_comparative（必填）"
                                    prop:value=move || single_adverb_comparative.get()
                                    on:input=move |ev| set_single_adverb_comparative.set(event_target_value(&ev))
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                                <input
                                    type="text"
                                    placeholder="adverb_superlative（必填）"
                                    prop:value=move || single_adverb_superlative.get()
                                    on:input=move |ev| set_single_adverb_superlative.set(event_target_value(&ev))
                                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                                />
                            </>
                        }
                            .into_any()
                    } else {
                        view! {
                            <p class="rounded-lg border border-slate-800 bg-slate-900 px-3 py-2 text-xs text-slate-400 md:col-span-3">
                                {move || {
                                    tr(
                                        lang.get(),
                                        "当前词性只需要基础字段：选中、词性、中文、原型。",
                                        "This part of speech only needs core fields: selected, POS, Chinese, base form.",
                                    )
                                }}
                            </p>
                        }
                            .into_any()
                    }
                }}
            </div>
            <button
                type="submit"
                class="mt-3 rounded-lg border border-slate-700 bg-emerald-700 px-4 py-2 text-sm font-medium hover:bg-emerald-600"
            >
                {move || tr(lang.get(), "添加单条", "Add Entry")}
            </button>
        </form>
    }
}
