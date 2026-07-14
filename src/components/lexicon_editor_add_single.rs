//! 单条词库新增表单：根据词性动态展示字段并提交到页面层回调。

use leptos::ev::SubmitEvent;
use leptos::prelude::*;

use crate::app_state::UiState;
use crate::structures::word_bank_entry::{PART_OF_SPEECH_OPTIONS, PartOfSpeech};
use crate::utils::i18n::{field_label, tr};

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
    single_verb_present_participle: ReadSignal<String>,
    set_single_verb_present_participle: WriteSignal<String>,
    single_verb_past_participle: ReadSignal<String>,
    set_single_verb_past_participle: WriteSignal<String>,
    single_verb_passive_infinitive: ReadSignal<String>,
    set_single_verb_passive_infinitive: WriteSignal<String>,
    single_verb_passive_present: ReadSignal<String>,
    set_single_verb_passive_present: WriteSignal<String>,
    single_verb_passive_past: ReadSignal<String>,
    set_single_verb_passive_past: WriteSignal<String>,
    single_plural: ReadSignal<String>,
    set_single_plural: WriteSignal<String>,
    single_singular_definite: ReadSignal<String>,
    set_single_singular_definite: WriteSignal<String>,
    single_plural_definite: ReadSignal<String>,
    set_single_plural_definite: WriteSignal<String>,
    single_noun_singular_definite_genitive: ReadSignal<String>,
    set_single_noun_singular_definite_genitive: WriteSignal<String>,
    single_noun_plural_definite_genitive: ReadSignal<String>,
    set_single_noun_plural_definite_genitive: WriteSignal<String>,
    single_noun_singular_indefinite_genitive: ReadSignal<String>,
    set_single_noun_singular_indefinite_genitive: WriteSignal<String>,
    single_noun_plural_indefinite_genitive: ReadSignal<String>,
    set_single_noun_plural_indefinite_genitive: WriteSignal<String>,
    single_adjective_feminine_form: ReadSignal<String>,
    set_single_adjective_feminine_form: WriteSignal<String>,
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
    single_pronoun_object: ReadSignal<String>,
    set_single_pronoun_object: WriteSignal<String>,
    single_pronoun_reflexive: ReadSignal<String>,
    set_single_pronoun_reflexive: WriteSignal<String>,
    single_pronoun_plural_subject: ReadSignal<String>,
    set_single_pronoun_plural_subject: WriteSignal<String>,
    single_pronoun_plural_object: ReadSignal<String>,
    set_single_pronoun_plural_object: WriteSignal<String>,
    single_pronoun_plural_reflexive: ReadSignal<String>,
    set_single_pronoun_plural_reflexive: WriteSignal<String>,
    single_determinative_feminine_form: ReadSignal<String>,
    set_single_determinative_feminine_form: WriteSignal<String>,
    single_determinative_neuter_form: ReadSignal<String>,
    set_single_determinative_neuter_form: WriteSignal<String>,
    single_determinative_plural_form: ReadSignal<String>,
    set_single_determinative_plural_form: WriteSignal<String>,
    single_adverb_comparative: ReadSignal<String>,
    set_single_adverb_comparative: WriteSignal<String>,
    single_adverb_superlative: ReadSignal<String>,
    set_single_adverb_superlative: WriteSignal<String>,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
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
                        "序号会自动由词性与词形字段计算哈希生成。变体字段按所选词性显示。",
                        "ID is auto-generated from part-of-speech and inflection fields. Variant fields follow selected POS.",
                    )
                }}
            </p>

            <section class="rounded-lg border border-slate-800 bg-slate-900/40 p-3">
                <p class="mb-2 text-xs font-semibold text-slate-400">
                    {move || tr(lang.get(), "基础组", "Core")}
                </p>
                <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
                    <label class="flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm">
                        <input
                            type="checkbox"
                            prop:checked=move || single_selected.get()
                            on:change=move |ev| set_single_selected.set(event_target_checked(&ev))
                        />
                        <span>{move || field_label(lang.get(), "selected")}</span>
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
                    <FormField
                        lang=lang
                        field_key="base_form"
                        required=true
                        value=single_norwegian
                        set_value=set_single_norwegian
                    />
                    <FormField
                        lang=lang
                        field_key="chinese"
                        required=true
                        value=single_chinese
                        set_value=set_single_chinese
                        pipe_separated=true
                    />
                    <FormField
                        lang=lang
                        field_key="english"
                        required=false
                        value=single_english
                        set_value=set_single_english
                        pipe_separated=true
                    />
                    <FormField
                        lang=lang
                        field_key="tags"
                        required=false
                        value=single_tags
                        set_value=set_single_tags
                        pipe_separated=true
                    />
                </div>
            </section>

            {move || pos_variant_sections(
                lang,
                single_pos.get(),
                single_verb_present_tense,
                set_single_verb_present_tense,
                single_past_tense,
                set_single_past_tense,
                single_imperative,
                set_single_imperative,
                single_verb_present_participle,
                set_single_verb_present_participle,
                single_verb_past_participle,
                set_single_verb_past_participle,
                single_verb_passive_infinitive,
                set_single_verb_passive_infinitive,
                single_verb_passive_present,
                set_single_verb_passive_present,
                single_verb_passive_past,
                set_single_verb_passive_past,
                single_plural,
                set_single_plural,
                single_singular_definite,
                set_single_singular_definite,
                single_plural_definite,
                set_single_plural_definite,
                single_noun_singular_definite_genitive,
                set_single_noun_singular_definite_genitive,
                single_noun_plural_definite_genitive,
                set_single_noun_plural_definite_genitive,
                single_noun_singular_indefinite_genitive,
                set_single_noun_singular_indefinite_genitive,
                single_noun_plural_indefinite_genitive,
                set_single_noun_plural_indefinite_genitive,
                single_adjective_feminine_form,
                set_single_adjective_feminine_form,
                single_neuter_form,
                set_single_neuter_form,
                single_plural_form,
                set_single_plural_form,
                single_adjective_comparative,
                set_single_adjective_comparative,
                single_adjective_superlative_indefinite,
                set_single_adjective_superlative_indefinite,
                single_adjective_superlative_definite,
                set_single_adjective_superlative_definite,
                single_pronoun_object,
                set_single_pronoun_object,
                single_pronoun_reflexive,
                set_single_pronoun_reflexive,
                single_pronoun_plural_subject,
                set_single_pronoun_plural_subject,
                single_pronoun_plural_object,
                set_single_pronoun_plural_object,
                single_pronoun_plural_reflexive,
                set_single_pronoun_plural_reflexive,
                single_determinative_feminine_form,
                set_single_determinative_feminine_form,
                single_determinative_neuter_form,
                set_single_determinative_neuter_form,
                single_determinative_plural_form,
                set_single_determinative_plural_form,
                single_adverb_comparative,
                set_single_adverb_comparative,
                single_adverb_superlative,
                set_single_adverb_superlative,
            )}

            <button
                type="submit"
                class="mt-3 w-full rounded-lg border border-slate-700 bg-emerald-700 px-4 py-2 text-sm font-medium hover:bg-emerald-600 sm:w-auto"
            >
                {move || tr(lang.get(), "添加单条", "Add Entry")}
            </button>
        </form>
    }
}

#[component]
fn FormField(
    lang: ReadSignal<UiLanguage>,
    field_key: &'static str,
    required: bool,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(default = false)] pipe_separated: bool,
) -> impl IntoView {
    let placeholder = move || {
        let l = lang.get();
        let label = field_label(l, field_key);
        let suffix = if pipe_separated {
            tr(l, "（| 分隔）", " (| separated)")
        } else {
            ""
        };
        let req = if required {
            tr(l, "（必填）", " (required)")
        } else {
            tr(l, "（可选）", " (optional)")
        };
        format!("{label}{suffix}{req}")
    };

    view! {
        <input
            type="text"
            prop:placeholder=placeholder
            prop:value=move || value.get()
            on:input=move |ev| set_value.set(event_target_value(&ev))
            class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
        />
    }
}

use crate::structures::word_bank_entry::UiLanguage;

fn pos_variant_sections(
    lang: ReadSignal<UiLanguage>,
    pos: String,
    single_verb_present_tense: ReadSignal<String>,
    set_single_verb_present_tense: WriteSignal<String>,
    single_past_tense: ReadSignal<String>,
    set_single_past_tense: WriteSignal<String>,
    single_imperative: ReadSignal<String>,
    set_single_imperative: WriteSignal<String>,
    single_verb_present_participle: ReadSignal<String>,
    set_single_verb_present_participle: WriteSignal<String>,
    single_verb_past_participle: ReadSignal<String>,
    set_single_verb_past_participle: WriteSignal<String>,
    single_verb_passive_infinitive: ReadSignal<String>,
    set_single_verb_passive_infinitive: WriteSignal<String>,
    single_verb_passive_present: ReadSignal<String>,
    set_single_verb_passive_present: WriteSignal<String>,
    single_verb_passive_past: ReadSignal<String>,
    set_single_verb_passive_past: WriteSignal<String>,
    single_plural: ReadSignal<String>,
    set_single_plural: WriteSignal<String>,
    single_singular_definite: ReadSignal<String>,
    set_single_singular_definite: WriteSignal<String>,
    single_plural_definite: ReadSignal<String>,
    set_single_plural_definite: WriteSignal<String>,
    single_noun_singular_definite_genitive: ReadSignal<String>,
    set_single_noun_singular_definite_genitive: WriteSignal<String>,
    single_noun_plural_definite_genitive: ReadSignal<String>,
    set_single_noun_plural_definite_genitive: WriteSignal<String>,
    single_noun_singular_indefinite_genitive: ReadSignal<String>,
    set_single_noun_singular_indefinite_genitive: WriteSignal<String>,
    single_noun_plural_indefinite_genitive: ReadSignal<String>,
    set_single_noun_plural_indefinite_genitive: WriteSignal<String>,
    single_adjective_feminine_form: ReadSignal<String>,
    set_single_adjective_feminine_form: WriteSignal<String>,
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
    single_pronoun_object: ReadSignal<String>,
    set_single_pronoun_object: WriteSignal<String>,
    single_pronoun_reflexive: ReadSignal<String>,
    set_single_pronoun_reflexive: WriteSignal<String>,
    single_pronoun_plural_subject: ReadSignal<String>,
    set_single_pronoun_plural_subject: WriteSignal<String>,
    single_pronoun_plural_object: ReadSignal<String>,
    set_single_pronoun_plural_object: WriteSignal<String>,
    single_pronoun_plural_reflexive: ReadSignal<String>,
    set_single_pronoun_plural_reflexive: WriteSignal<String>,
    single_determinative_feminine_form: ReadSignal<String>,
    set_single_determinative_feminine_form: WriteSignal<String>,
    single_determinative_neuter_form: ReadSignal<String>,
    set_single_determinative_neuter_form: WriteSignal<String>,
    single_determinative_plural_form: ReadSignal<String>,
    set_single_determinative_plural_form: WriteSignal<String>,
    single_adverb_comparative: ReadSignal<String>,
    set_single_adverb_comparative: WriteSignal<String>,
    single_adverb_superlative: ReadSignal<String>,
    set_single_adverb_superlative: WriteSignal<String>,
) -> AnyView {
    let l = lang.get();
    match pos.as_str() {
        "verb" => variant_section(
            l,
            tr(l, "动词变体组", "Verb Forms"),
            view! {
                <FormField lang=lang field_key="verb_present_tense" required=true value=single_verb_present_tense set_value=set_single_verb_present_tense/>
                <FormField lang=lang field_key="verb_past_tense" required=true value=single_past_tense set_value=set_single_past_tense/>
                <FormField lang=lang field_key="verb_imperative" required=true value=single_imperative set_value=set_single_imperative/>
                <FormField lang=lang field_key="verb_present_participle" required=true value=single_verb_present_participle set_value=set_single_verb_present_participle/>
                <FormField lang=lang field_key="verb_past_participle" required=true value=single_verb_past_participle set_value=set_single_verb_past_participle/>
                <FormField lang=lang field_key="verb_passive_infinitive" required=true value=single_verb_passive_infinitive set_value=set_single_verb_passive_infinitive/>
                <FormField lang=lang field_key="verb_passive_present" required=true value=single_verb_passive_present set_value=set_single_verb_passive_present/>
                <FormField lang=lang field_key="verb_passive_past" required=false value=single_verb_passive_past set_value=set_single_verb_passive_past/>
            },
        ),
        "noun" => variant_section(
            l,
            tr(l, "名词变体组", "Noun Forms"),
            view! {
                <FormField lang=lang field_key="noun_plural" required=true value=single_plural set_value=set_single_plural/>
                <FormField lang=lang field_key="noun_singular_definite" required=true value=single_singular_definite set_value=set_single_singular_definite/>
                <FormField lang=lang field_key="noun_plural_definite" required=true value=single_plural_definite set_value=set_single_plural_definite/>
                <FormField lang=lang field_key="noun_singular_definite_genitive" required=true value=single_noun_singular_definite_genitive set_value=set_single_noun_singular_definite_genitive/>
                <FormField lang=lang field_key="noun_plural_definite_genitive" required=true value=single_noun_plural_definite_genitive set_value=set_single_noun_plural_definite_genitive/>
                <FormField lang=lang field_key="noun_singular_indefinite_genitive" required=false value=single_noun_singular_indefinite_genitive set_value=set_single_noun_singular_indefinite_genitive/>
                <FormField lang=lang field_key="noun_plural_indefinite_genitive" required=false value=single_noun_plural_indefinite_genitive set_value=set_single_noun_plural_indefinite_genitive/>
            },
        ),
        "adjective" => variant_section(
            l,
            tr(l, "形容词变体组", "Adjective Forms"),
            view! {
                <FormField lang=lang field_key="adjective_feminine_form" required=true value=single_adjective_feminine_form set_value=set_single_adjective_feminine_form/>
                <FormField lang=lang field_key="adjective_neuter_form" required=true value=single_neuter_form set_value=set_single_neuter_form/>
                <FormField lang=lang field_key="adjective_plural_form" required=true value=single_plural_form set_value=set_single_plural_form/>
                <FormField lang=lang field_key="adjective_comparative" required=true value=single_adjective_comparative set_value=set_single_adjective_comparative/>
                <FormField lang=lang field_key="adjective_superlative_indefinite" required=true value=single_adjective_superlative_indefinite set_value=set_single_adjective_superlative_indefinite/>
                <FormField lang=lang field_key="adjective_superlative_definite" required=true value=single_adjective_superlative_definite set_value=set_single_adjective_superlative_definite/>
            },
        ),
        "pronoun" => variant_section(
            l,
            tr(l, "代词变体组", "Pronoun Forms"),
            view! {
                <FormField lang=lang field_key="pronoun_object" required=true value=single_pronoun_object set_value=set_single_pronoun_object/>
                <FormField lang=lang field_key="pronoun_reflexive" required=true value=single_pronoun_reflexive set_value=set_single_pronoun_reflexive/>
                <FormField lang=lang field_key="pronoun_plural_subject" required=true value=single_pronoun_plural_subject set_value=set_single_pronoun_plural_subject/>
                <FormField lang=lang field_key="pronoun_plural_object" required=true value=single_pronoun_plural_object set_value=set_single_pronoun_plural_object/>
                <FormField lang=lang field_key="pronoun_plural_reflexive" required=true value=single_pronoun_plural_reflexive set_value=set_single_pronoun_plural_reflexive/>
            },
        ),
        "determinative" => variant_section(
            l,
            tr(l, "限定词变体组", "Determinative Forms"),
            view! {
                <FormField lang=lang field_key="determinative_feminine_form" required=true value=single_determinative_feminine_form set_value=set_single_determinative_feminine_form/>
                <FormField lang=lang field_key="determinative_neuter_form" required=true value=single_determinative_neuter_form set_value=set_single_determinative_neuter_form/>
                <FormField lang=lang field_key="determinative_plural_form" required=true value=single_determinative_plural_form set_value=set_single_determinative_plural_form/>
            },
        ),
        "adverb" => variant_section(
            l,
            tr(l, "副词变体组", "Adverb Forms"),
            view! {
                <p class="md:col-span-3 text-xs text-slate-400">
                    {tr(
                        l,
                        "副词通常只需原型；比较级/最高级仅在少数词中使用，可留空。",
                        "Adverbs usually only need base form; comparative/superlative are optional.",
                    )}
                </p>
                <FormField lang=lang field_key="adverb_comparative" required=false value=single_adverb_comparative set_value=set_single_adverb_comparative/>
                <FormField lang=lang field_key="adverb_superlative" required=false value=single_adverb_superlative set_value=set_single_adverb_superlative/>
            },
        ),
        "preposition" | "conjunction" | "subjunction" | "interjection" => variant_section(
            l,
            tr(l, "不可变词性", "Invariant Part of Speech"),
            view! {
                <p class="md:col-span-3 text-xs text-slate-400">
                    {tr(
                        l,
                        "当前词性只需要基础组字段（选中、词性、中文、原型）。",
                        "This part of speech only needs core fields (selected, POS, Chinese, base form).",
                    )}
                </p>
            },
        ),
        _ => view! { <></> }.into_any(),
    }
}

fn variant_section(_lang: UiLanguage, title: &'static str, fields: impl IntoView) -> AnyView {
    view! {
        <section class="mt-3 rounded-lg border border-slate-800 bg-slate-900/40 p-3">
            <p class="mb-2 text-xs font-semibold text-slate-400">{title}</p>
            <div class="grid grid-cols-1 gap-3 md:grid-cols-3">{fields}</div>
        </section>
    }
    .into_any()
}
