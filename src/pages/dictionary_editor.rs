//! 本地词库编辑页：单条新增、批量新增、AI 查询分流、表格编辑与 CSV 导入导出。

use leptos::ev::SubmitEvent;
use leptos::prelude::*;

use crate::app_state::{LexiconState, UiState};
use crate::components::lexicon_browser::{
    LexiconBrowser, LexiconBrowserMode, parse_pipe_list,
};
use crate::components::lexicon_editor_add_multi::LexiconEditorAddMulti;
use crate::components::lexicon_editor_add_single::LexiconEditorAddSingle;
use crate::components::mini_console::MiniConsole;
use crate::components::return_button::ReturnButton;
use crate::components::word_search::{AiResearcher, AiResearcherActions, SingleEntryFormState};
use crate::pages::AppPage;
use crate::structures::word_bank_entry::WordBankEntry;
use crate::utils::dictionary::{
    SingleEntryDraft, draft_from_word_entry, parse_part_of_speech,
    validate_and_prepare_single_entry,
};
use crate::utils::i18n::tr;

#[component]
pub fn LocalLexiconEditorPage() -> impl IntoView {
    let lexicon_state = expect_context::<LexiconState>();
    let lang = expect_context::<UiState>().ui_language;
    let entries = lexicon_state.entries;
    let set_entries = lexicon_state.set_entries;
    let data_version = lexicon_state.data_version;
    let set_data_version = lexicon_state.set_data_version;
    let (status, set_status) = signal({
        let language = lang.get_untracked();
        let count = entries.get_untracked().len();
        let source = lexicon_state.source_name.get_untracked();
        if count == 0 {
            tr(
                language,
                "当前词库为空，请先回到首页加载或导入词库。",
                "Current lexicon is empty. Please load/import lexicon first.",
            )
            .to_string()
        } else {
            format!(
                "{}：{source}（{} {count} {}）。",
                tr(language, "当前词库", "Current Lexicon"),
                tr(language, "共", "total"),
                tr(language, "条", "entries")
            )
        }
    });

    let (single_selected, set_single_selected) = signal(true);
    let (single_pos, set_single_pos) = signal("noun".to_string());
    let (single_norwegian, set_single_norwegian) = signal(String::new());
    let (single_chinese, set_single_chinese) = signal(String::new());
    let (single_english, set_single_english) = signal(String::new());
    let (single_tags, set_single_tags) = signal(String::new());
    let (single_verb_present_tense, set_single_verb_present_tense) = signal(String::new());
    let (single_past_tense, set_single_past_tense) = signal(String::new());
    let (single_imperative, set_single_imperative) = signal(String::new());
    let (single_verb_present_participle, set_single_verb_present_participle) =
        signal(String::new());
    let (single_verb_past_participle, set_single_verb_past_participle) = signal(String::new());
    let (single_verb_passive_infinitive, set_single_verb_passive_infinitive) =
        signal(String::new());
    let (single_verb_passive_present, set_single_verb_passive_present) = signal(String::new());
    let (single_verb_passive_past, set_single_verb_passive_past) = signal(String::new());
    let (single_plural, set_single_plural) = signal(String::new());
    let (single_singular_definite, set_single_singular_definite) = signal(String::new());
    let (single_plural_definite, set_single_plural_definite) = signal(String::new());
    let (single_noun_singular_definite_genitive, set_single_noun_singular_definite_genitive) =
        signal(String::new());
    let (single_noun_plural_definite_genitive, set_single_noun_plural_definite_genitive) =
        signal(String::new());
    let (single_noun_singular_indefinite_genitive, set_single_noun_singular_indefinite_genitive) =
        signal(String::new());
    let (single_noun_plural_indefinite_genitive, set_single_noun_plural_indefinite_genitive) =
        signal(String::new());
    let (single_adjective_feminine_form, set_single_adjective_feminine_form) =
        signal(String::new());
    let (single_neuter_form, set_single_neuter_form) = signal(String::new());
    let (single_plural_form, set_single_plural_form) = signal(String::new());
    let (single_adjective_comparative, set_single_adjective_comparative) = signal(String::new());
    let (single_adjective_superlative_indefinite, set_single_adjective_superlative_indefinite) =
        signal(String::new());
    let (single_adjective_superlative_definite, set_single_adjective_superlative_definite) =
        signal(String::new());
    let (single_pronoun_object, set_single_pronoun_object) = signal(String::new());
    let (single_pronoun_reflexive, set_single_pronoun_reflexive) = signal(String::new());
    let (single_pronoun_plural_subject, set_single_pronoun_plural_subject) = signal(String::new());
    let (single_pronoun_plural_object, set_single_pronoun_plural_object) = signal(String::new());
    let (single_pronoun_plural_reflexive, set_single_pronoun_plural_reflexive) =
        signal(String::new());
    let (single_determinative_feminine_form, set_single_determinative_feminine_form) =
        signal(String::new());
    let (single_determinative_neuter_form, set_single_determinative_neuter_form) =
        signal(String::new());
    let (single_determinative_plural_form, set_single_determinative_plural_form) =
        signal(String::new());
    let (single_adverb_comparative, set_single_adverb_comparative) = signal(String::new());
    let (single_adverb_superlative, set_single_adverb_superlative) = signal(String::new());

    let (bulk_input, set_bulk_input) = signal(String::new());
    let (bulk_errors, set_bulk_errors) = signal(Vec::<String>::new());
    let (bulk_success_message, set_bulk_success_message) = signal(String::new());
    let (editor_reset_version, set_editor_reset_version) = signal(0_u64);

    let clear_editor_forms = Callback::new(move |_| {
        let language = lang.get_untracked();
        set_single_selected.set(true);
        set_single_pos.set("noun".to_string());
        set_single_norwegian.set(String::new());
        set_single_chinese.set(String::new());
        set_single_english.set(String::new());
        set_single_tags.set(String::new());
        set_single_verb_present_tense.set(String::new());
        set_single_past_tense.set(String::new());
        set_single_imperative.set(String::new());
        set_single_verb_present_participle.set(String::new());
        set_single_verb_past_participle.set(String::new());
        set_single_verb_passive_infinitive.set(String::new());
        set_single_verb_passive_present.set(String::new());
        set_single_verb_passive_past.set(String::new());
        set_single_plural.set(String::new());
        set_single_singular_definite.set(String::new());
        set_single_plural_definite.set(String::new());
        set_single_noun_singular_definite_genitive.set(String::new());
        set_single_noun_plural_definite_genitive.set(String::new());
        set_single_noun_singular_indefinite_genitive.set(String::new());
        set_single_noun_plural_indefinite_genitive.set(String::new());
        set_single_adjective_feminine_form.set(String::new());
        set_single_neuter_form.set(String::new());
        set_single_plural_form.set(String::new());
        set_single_adjective_comparative.set(String::new());
        set_single_adjective_superlative_indefinite.set(String::new());
        set_single_adjective_superlative_definite.set(String::new());
        set_single_pronoun_object.set(String::new());
        set_single_pronoun_reflexive.set(String::new());
        set_single_pronoun_plural_subject.set(String::new());
        set_single_pronoun_plural_object.set(String::new());
        set_single_pronoun_plural_reflexive.set(String::new());
        set_single_determinative_feminine_form.set(String::new());
        set_single_determinative_neuter_form.set(String::new());
        set_single_determinative_plural_form.set(String::new());
        set_single_adverb_comparative.set(String::new());
        set_single_adverb_superlative.set(String::new());
        set_bulk_input.set(String::new());
        set_bulk_errors.set(Vec::new());
        set_bulk_success_message.set(String::new());
        set_editor_reset_version.update(|ver| *ver += 1);
        set_status.set(
            tr(
                language,
                "已清空编辑区填写内容。",
                "Editor fields cleared.",
            )
            .to_string(),
        );
    });

    // 单条新增：构造草稿 -> 校验规范化 -> 写入全局词库并 bump 版本。
    let add_single_entry = Callback::new(move |ev: SubmitEvent| {
        let language = lang.get_untracked();
        ev.prevent_default();

        let draft: SingleEntryDraft = SingleEntryDraft {
            id: String::new(),
            selected: single_selected.get(),
            part_of_speech: match parse_part_of_speech(&single_pos.get()) {
                Ok(value) => value,
                Err(err) => {
                    set_status.set(format!(
                        "{} {err}",
                        tr(language, "单条添加失败：", "Single add failed:")
                    ));
                    return;
                }
            },
            tags: parse_csv_list(&single_tags.get()),
            english: parse_csv_list(&single_english.get()),
            chinese: parse_csv_list(&single_chinese.get()),
            base_form: single_norwegian.get(),
            verb_present_tense: parse_optional_input(&single_verb_present_tense.get()),
            verb_past_tense: parse_optional_input(&single_past_tense.get()),
            verb_imperative: parse_optional_input(&single_imperative.get()),
            verb_present_participle: parse_optional_input(&single_verb_present_participle.get()),
            verb_past_participle: parse_optional_input(&single_verb_past_participle.get()),
            verb_passive_infinitive: parse_optional_input(&single_verb_passive_infinitive.get()),
            verb_passive_present: parse_optional_input(&single_verb_passive_present.get()),
            verb_passive_past: parse_optional_input(&single_verb_passive_past.get()),
            noun_plural: parse_optional_input(&single_plural.get()),
            noun_singular_definite: parse_optional_input(&single_singular_definite.get()),
            noun_plural_definite: parse_optional_input(&single_plural_definite.get()),
            noun_singular_definite_genitive: parse_optional_input(
                &single_noun_singular_definite_genitive.get(),
            ),
            noun_plural_definite_genitive: parse_optional_input(
                &single_noun_plural_definite_genitive.get(),
            ),
            noun_singular_indefinite_genitive: parse_optional_input(
                &single_noun_singular_indefinite_genitive.get(),
            ),
            noun_plural_indefinite_genitive: parse_optional_input(
                &single_noun_plural_indefinite_genitive.get(),
            ),
            adjective_feminine_form: parse_optional_input(&single_adjective_feminine_form.get()),
            adjective_neuter_form: parse_optional_input(&single_neuter_form.get()),
            adjective_plural_form: parse_optional_input(&single_plural_form.get()),
            adjective_comparative: parse_optional_input(&single_adjective_comparative.get()),
            adjective_superlative_indefinite: parse_optional_input(
                &single_adjective_superlative_indefinite.get(),
            ),
            adjective_superlative_definite: parse_optional_input(
                &single_adjective_superlative_definite.get(),
            ),
            pronoun_object: parse_optional_input(&single_pronoun_object.get()),
            pronoun_reflexive: parse_optional_input(&single_pronoun_reflexive.get()),
            pronoun_plural_subject: parse_optional_input(&single_pronoun_plural_subject.get()),
            pronoun_plural_object: parse_optional_input(&single_pronoun_plural_object.get()),
            pronoun_plural_reflexive: parse_optional_input(&single_pronoun_plural_reflexive.get()),
            determinative_feminine_form: parse_optional_input(
                &single_determinative_feminine_form.get(),
            ),
            determinative_neuter_form: parse_optional_input(
                &single_determinative_neuter_form.get(),
            ),
            determinative_plural_form: parse_optional_input(
                &single_determinative_plural_form.get(),
            ),
            adverb_comparative: parse_optional_input(&single_adverb_comparative.get()),
            adverb_superlative: parse_optional_input(&single_adverb_superlative.get()),
        };
        let existing_entries = entries.get_untracked();
        let new_entry = match validate_and_prepare_single_entry(draft, &existing_entries) {
            Ok(entry) => entry,
            Err(err) => {
                set_status.set(format!(
                    "{} {err}",
                    tr(language, "单条添加失败：", "Single add failed:")
                ));
                return;
            }
        };

        let generated_id = new_entry.id.clone();
        let mut total_after_add = existing_entries.len();
        set_entries.update(|list| {
            list.push(new_entry);
            total_after_add = list.len();
        });
        set_data_version.update(|ver| *ver += 1);
        set_status.set(format!(
            "{}（id: {generated_id}），{} {total_after_add} {}。",
            tr(
                language,
                "已添加 1 条到本地缓存词库",
                "Added 1 entry to local cache"
            ),
            tr(language, "当前共", "now total"),
            tr(language, "条", "entries")
        ));

        set_single_selected.set(true);
        set_single_pos.set("noun".to_string());
        set_single_norwegian.set(String::new());
        set_single_chinese.set(String::new());
        set_single_english.set(String::new());
        set_single_tags.set(String::new());
        set_single_verb_present_tense.set(String::new());
        set_single_past_tense.set(String::new());
        set_single_imperative.set(String::new());
        set_single_verb_present_participle.set(String::new());
        set_single_verb_past_participle.set(String::new());
        set_single_verb_passive_infinitive.set(String::new());
        set_single_verb_passive_present.set(String::new());
        set_single_verb_passive_past.set(String::new());
        set_single_plural.set(String::new());
        set_single_singular_definite.set(String::new());
        set_single_plural_definite.set(String::new());
        set_single_noun_singular_definite_genitive.set(String::new());
        set_single_noun_plural_definite_genitive.set(String::new());
        set_single_noun_singular_indefinite_genitive.set(String::new());
        set_single_noun_plural_indefinite_genitive.set(String::new());
        set_single_adjective_feminine_form.set(String::new());
        set_single_neuter_form.set(String::new());
        set_single_plural_form.set(String::new());
        set_single_adjective_comparative.set(String::new());
        set_single_adjective_superlative_indefinite.set(String::new());
        set_single_adjective_superlative_definite.set(String::new());
        set_single_pronoun_object.set(String::new());
        set_single_pronoun_reflexive.set(String::new());
        set_single_pronoun_plural_subject.set(String::new());
        set_single_pronoun_plural_object.set(String::new());
        set_single_pronoun_plural_reflexive.set(String::new());
        set_single_determinative_feminine_form.set(String::new());
        set_single_determinative_neuter_form.set(String::new());
        set_single_determinative_plural_form.set(String::new());
        set_single_adverb_comparative.set(String::new());
        set_single_adverb_superlative.set(String::new());
    });

    // 批量新增：按顺序逐条校验（模拟追加）以确保批次内/批次外都不冲突。
    let add_bulk_entries = Callback::new(move |rows: Vec<WordBankEntry>| {
        let language = lang.get_untracked();
        set_bulk_errors.set(Vec::new());
        set_bulk_success_message.set(String::new());

        if rows.is_empty() {
            set_status.set(
                tr(
                    language,
                    "多条添加失败：没有可添加条目。",
                    "Bulk add failed: no entries to add.",
                )
                .to_string(),
            );
            set_bulk_errors.set(vec![
                tr(
                    language,
                    "请先用“查询并分流”生成多条结果。",
                    "Please generate multi-results via Query and Route first.",
                )
                .to_string(),
            ]);
            return;
        }

        let mut simulated_entries = entries.get_untracked();
        let mut validated_entries = Vec::with_capacity(rows.len());
        let mut errors = Vec::new();

        for (index, row) in rows.into_iter().enumerate() {
            let row_no = index + 1;
            let base_form_label = if row.base_form.trim().is_empty() {
                "（原型为空）".to_string()
            } else {
                format!("（原型：{}）", row.base_form.trim())
            };

            let draft = draft_from_word_entry(&row);

            match validate_and_prepare_single_entry(draft, &simulated_entries) {
                Ok(valid_entry) => {
                    simulated_entries.push(valid_entry.clone());
                    validated_entries.push(valid_entry);
                }
                Err(err) => {
                    errors.push(format!("第 {row_no} 条 {base_form_label}：{err}"));
                }
            }
        }

        if !errors.is_empty() {
            set_bulk_errors.set(errors);
            set_status.set(
                tr(
                    language,
                    "批量添加失败：存在不合法条目，请先修正红字错误。",
                    "Bulk add failed: invalid entries found.",
                )
                .to_string(),
            );
            return;
        }

        let add_count = validated_entries.len();
        set_entries.update(|list| list.extend(validated_entries));
        set_data_version.update(|ver| *ver += 1);
        set_bulk_input.set(String::new());
        set_status.set(format!(
            "{} {} {}",
            tr(language, "批量添加成功，新增", "Bulk add succeeded, added"),
            add_count,
            tr(language, "条。", "entries.")
        ));
        set_bulk_success_message.set(tr(language, "成功导入", "Imported").to_string());
    });
    let ai_actions = AiResearcherActions {
        set_status,
        set_bulk_input,
        set_bulk_errors,
        set_bulk_success_message,
    };
    let single_form_state = SingleEntryFormState {
        set_single_pos,
        single_norwegian,
        set_single_norwegian,
        single_chinese,
        set_single_chinese,
        single_english,
        set_single_english,
        single_verb_present_tense,
        set_single_verb_present_tense,
        single_verb_past_tense: single_past_tense,
        set_single_verb_past_tense: set_single_past_tense,
        single_verb_imperative: single_imperative,
        set_single_verb_imperative: set_single_imperative,
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
        single_noun_plural: single_plural,
        set_single_noun_plural: set_single_plural,
        single_noun_singular_definite: single_singular_definite,
        set_single_noun_singular_definite: set_single_singular_definite,
        single_noun_plural_definite: single_plural_definite,
        set_single_noun_plural_definite: set_single_plural_definite,
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
        single_adjective_neuter_form: single_neuter_form,
        set_single_adjective_neuter_form: set_single_neuter_form,
        single_adjective_plural_form: single_plural_form,
        set_single_adjective_plural_form: set_single_plural_form,
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
        set_single_tags,
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-3 sm:p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-4 sm:p-6 shadow-xl">
                <ReturnButton target_page=AppPage::PracticeModeSelect/>
                <header class="mb-6 flex items-center gap-4">
                    <h1 class="text-xl sm:text-2xl font-bold tracking-tight">
                        {move || tr(lang.get(), "本地词库修改器", "Local Lexicon Editor")}
                    </h1>
                </header>

                <MiniConsole
                    message=Signal::derive(move || {
                        let status_line = {
                            let s = status.get();
                            if s.trim().is_empty() {
                                tr(lang.get(), "等待词库编辑操作...", "Waiting for lexicon editing...")
                                    .to_string()
                            } else {
                                s
                            }
                        };
                        format!(
                            "{status_line}\n{}",
                            tr(lang.get(), "编辑后请点击“确认修改”再导出。", "Click \"Confirm Changes\" before exporting.")
                        )
                    })
                />

                <LexiconEditorAddSingle
                    on_submit=add_single_entry
                    on_clear=clear_editor_forms
                    single_selected=single_selected
                    set_single_selected=set_single_selected
                    single_pos=single_pos
                    set_single_pos=set_single_pos
                    single_norwegian=single_norwegian
                    set_single_norwegian=set_single_norwegian
                    single_chinese=single_chinese
                    set_single_chinese=set_single_chinese
                    single_english=single_english
                    set_single_english=set_single_english
                    single_tags=single_tags
                    set_single_tags=set_single_tags
                    single_verb_present_tense=single_verb_present_tense
                    set_single_verb_present_tense=set_single_verb_present_tense
                    single_past_tense=single_past_tense
                    set_single_past_tense=set_single_past_tense
                    single_imperative=single_imperative
                    set_single_imperative=set_single_imperative
                    single_verb_present_participle=single_verb_present_participle
                    set_single_verb_present_participle=set_single_verb_present_participle
                    single_verb_past_participle=single_verb_past_participle
                    set_single_verb_past_participle=set_single_verb_past_participle
                    single_verb_passive_infinitive=single_verb_passive_infinitive
                    set_single_verb_passive_infinitive=set_single_verb_passive_infinitive
                    single_verb_passive_present=single_verb_passive_present
                    set_single_verb_passive_present=set_single_verb_passive_present
                    single_verb_passive_past=single_verb_passive_past
                    set_single_verb_passive_past=set_single_verb_passive_past
                    single_plural=single_plural
                    set_single_plural=set_single_plural
                    single_singular_definite=single_singular_definite
                    set_single_singular_definite=set_single_singular_definite
                    single_plural_definite=single_plural_definite
                    set_single_plural_definite=set_single_plural_definite
                    single_noun_singular_definite_genitive=single_noun_singular_definite_genitive
                    set_single_noun_singular_definite_genitive=set_single_noun_singular_definite_genitive
                    single_noun_plural_definite_genitive=single_noun_plural_definite_genitive
                    set_single_noun_plural_definite_genitive=set_single_noun_plural_definite_genitive
                    single_noun_singular_indefinite_genitive=single_noun_singular_indefinite_genitive
                    set_single_noun_singular_indefinite_genitive=set_single_noun_singular_indefinite_genitive
                    single_noun_plural_indefinite_genitive=single_noun_plural_indefinite_genitive
                    set_single_noun_plural_indefinite_genitive=set_single_noun_plural_indefinite_genitive
                    single_adjective_feminine_form=single_adjective_feminine_form
                    set_single_adjective_feminine_form=set_single_adjective_feminine_form
                    single_neuter_form=single_neuter_form
                    set_single_neuter_form=set_single_neuter_form
                    single_plural_form=single_plural_form
                    set_single_plural_form=set_single_plural_form
                    single_adjective_comparative=single_adjective_comparative
                    set_single_adjective_comparative=set_single_adjective_comparative
                    single_adjective_superlative_indefinite=single_adjective_superlative_indefinite
                    set_single_adjective_superlative_indefinite=set_single_adjective_superlative_indefinite
                    single_adjective_superlative_definite=single_adjective_superlative_definite
                    set_single_adjective_superlative_definite=set_single_adjective_superlative_definite
                    single_pronoun_object=single_pronoun_object
                    set_single_pronoun_object=set_single_pronoun_object
                    single_pronoun_reflexive=single_pronoun_reflexive
                    set_single_pronoun_reflexive=set_single_pronoun_reflexive
                    single_pronoun_plural_subject=single_pronoun_plural_subject
                    set_single_pronoun_plural_subject=set_single_pronoun_plural_subject
                    single_pronoun_plural_object=single_pronoun_plural_object
                    set_single_pronoun_plural_object=set_single_pronoun_plural_object
                    single_pronoun_plural_reflexive=single_pronoun_plural_reflexive
                    set_single_pronoun_plural_reflexive=set_single_pronoun_plural_reflexive
                    single_determinative_feminine_form=single_determinative_feminine_form
                    set_single_determinative_feminine_form=set_single_determinative_feminine_form
                    single_determinative_neuter_form=single_determinative_neuter_form
                    set_single_determinative_neuter_form=set_single_determinative_neuter_form
                    single_determinative_plural_form=single_determinative_plural_form
                    set_single_determinative_plural_form=set_single_determinative_plural_form
                    single_adverb_comparative=single_adverb_comparative
                    set_single_adverb_comparative=set_single_adverb_comparative
                    single_adverb_superlative=single_adverb_superlative
                    set_single_adverb_superlative=set_single_adverb_superlative
                />

                <AiResearcher
                    actions=ai_actions
                    single_form_state=single_form_state
                    reset_version=editor_reset_version
                />

                <LexiconEditorAddMulti
                    on_submit=add_bulk_entries
                    on_clear=clear_editor_forms
                    bulk_input=bulk_input
                    bulk_errors=bulk_errors
                    bulk_success_message=bulk_success_message
                />

                <section class="mt-4">
                    <h2 class="mb-3 text-lg font-semibold">
                        {move || tr(lang.get(), "词库浏览器", "Lexicon Browser")}
                    </h2>
                    <p class="mb-3 text-sm text-slate-400">
                        {move || {
                            tr(
                                lang.get(),
                                "以表格形式查看、直接修改并删除词条（类似 Excel）。",
                                "View, edit and delete entries in table form (Excel-like).",
                            )
                        }}
                    </p>
                    <p class="mb-3 text-xs text-slate-500">
                        {move || {
                            tr(
                                lang.get(),
                                "拖拽表头右侧边界可调整列宽（更像 Excel）。",
                                "Drag header right edge to resize columns.",
                            )
                        }}
                    </p>
                </section>

                <LexiconBrowser
                    entries=entries
                    set_entries=set_entries
                    set_status=set_status
                    data_version=data_version
                    mode=LexiconBrowserMode::Edit
                />
            </section>
        </main>
    }
}

fn parse_csv_list(raw: &str) -> Vec<String> {
    parse_pipe_list(raw)
}

/// 表单输入转 Optional：空串表示未填写。
fn parse_optional_input(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
