//! 练习模式选择页：加载词库、导入字典文件，并分流到词库练习/系列练习/本地编辑器。

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app_state::{LexiconState, NavigateToPage, UiState};
use crate::components::import_dictionary::ImportDictionaryButton;
use crate::layout::{PageEyebrow, PageHeading, PageShell, PageTitle, PageTopbar, ShellAttach};
use crate::components::mini_console::MiniConsole;
use crate::pages::AppPage;
use crate::utils::i18n::tr;
use crate::utils::lexicon_file::{DEFAULT_BUILTIN_WORD_BANK_FILE, load_lexicon_entries_from_url};

include!(concat!(env!("OUT_DIR"), "/lexicon_word_bank_catalog.rs"));

#[component]
pub fn PracticeModePage() -> impl IntoView {
    let set_current_page = expect_context::<NavigateToPage>();
    let lexicon_state = expect_context::<LexiconState>();
    let lang = expect_context::<UiState>().ui_language;
    let default_word_bank = LEXICON_WORD_BANK_FILES
        .iter()
        .copied()
        .find(|file| *file == DEFAULT_BUILTIN_WORD_BANK_FILE)
        .or_else(|| LEXICON_WORD_BANK_FILES.first().copied())
        .unwrap_or("")
        .to_string();
    let (selected_word_bank, set_selected_word_bank) = signal(default_word_bank);
    let (status, set_status) = signal(String::new());

    // 加载内置词库并覆盖当前全局 entries。
    let choose_word_bank_click = move |_| {
        let language = lang.get_untracked();
        let selected_file = selected_word_bank.get_untracked();
        if selected_file.trim().is_empty() {
            set_status.set(
                tr(
                    language,
                    "未检测到可用内置词库，请检查 data/lexicon-word-bank 目录。",
                    "No built-in lexicon detected, please check data/lexicon-word-bank.",
                )
                .to_string(),
            );
            return;
        }
        let lexicon_state = lexicon_state;
        set_status.set(format!(
            "{}：{selected_file} ...",
            tr(language, "正在加载内置词库", "Loading built-in lexicon")
        ));

        spawn_local(async move {
            let path = format!("/data/lexicon-word-bank/{selected_file}");
            let result = load_lexicon_entries_from_url(&path).await;

            match result {
                Ok(word_list) => {
                    let count = word_list.len();
                    lexicon_state.set_entries.set(word_list);
                    lexicon_state.set_data_version.update(|ver| *ver += 1);
                    lexicon_state.set_source_name.set(format!(
                        "{}：{selected_file}",
                        tr(language, "内置词库", "Built-in Lexicon")
                    ));
                    set_status.set(format!(
                        "{} {count} {}",
                        tr(
                            language,
                            "词库切换成功（已覆盖），共",
                            "Lexicon switched (overwritten),"
                        ),
                        tr(language, "条。", "entries.")
                    ));
                }
                Err(err) => {
                    set_status.set(format!(
                        "{} {err}",
                        tr(language, "词库切换失败：", "Lexicon switch failed:")
                    ));
                }
            }
        });
    };
    let current_lexicon_line = Signal::derive(move || {
        let language = lang.get();
        let count = lexicon_state.entries.get().len();
        let source = lexicon_state.source_name.get();
        if count == 0 {
            tr(
                language,
                "尚未加载词库，请先在本页选择词库或导入字典文件。",
                "No lexicon loaded. Select one or import a dictionary file first.",
            )
            .to_string()
        } else {
            format!(
                "{}：{source}（{} {count} {}）。",
                tr(language, "当前词库", "Current Lexicon"),
                tr(language, "共", "total"),
                tr(language, "条词条", "entries")
            )
        }
    });
    let console_status_line = Signal::derive(move || {
        let language = lang.get();
        let s = status.get();
        if s.trim().is_empty() {
            tr(language, "等待词库操作...", "Waiting for lexicon action...").to_string()
        } else {
            s
        }
    });

    view! {
        <PageShell attach=ShellAttach::Fill>
            <PageTopbar>
                <PageHeading>
                    <PageEyebrow>
                        {move || tr(lang.get(), "练习", "practice")}
                    </PageEyebrow>
                    <PageTitle>
                        {move || tr(lang.get(), "请选择练习模式", "Choose Practice Mode")}
                    </PageTitle>
                </PageHeading>
            </PageTopbar>
            <p class="ui-muted">{move || current_lexicon_line.get()}</p>
            <MiniConsole message=console_status_line max_entries=80/>

                <div class="ui-grid ui-grid-3">
                    <section class="ui-card">
                        <p class="ui-eyebrow">
                            {move || tr(lang.get(), "词库", "lexicon")}
                        </p>
                        <h2>{move || tr(lang.get(), "词库练习", "Lexicon Practice")}</h2>
                        <p>
                            {move || {
                                tr(
                                    lang.get(),
                                    "先选择或导入字典，再进入词库模式。",
                                    "Select or import a dictionary first, then enter lexicon mode.",
                                )
                            }}
                        </p>
                        <div class="mt-1 grid grid-cols-1 gap-2">
                            <select
                                prop:value=move || selected_word_bank.get()
                                on:change=move |ev| set_selected_word_bank.set(event_target_value(&ev))
                                class="ui-field"
                            >
                                {if LEXICON_WORD_BANK_FILES.is_empty() {
                                    view! {
                                        <option value="">
                                            {move || tr(lang.get(), "未检测到词库文件", "No lexicon file detected")}
                                        </option>
                                    }
                                        .into_any()
                                } else {
                                    LEXICON_WORD_BANK_FILES
                                        .iter()
                                        .map(|file| view! { <option value=*file>{*file}</option> })
                                        .collect_view()
                                        .into_any()
                                }}
                            </select>
                            <div class="ui-card-actions">
                                <button
                                    type="button"
                                    on:click=choose_word_bank_click
                                    disabled=LEXICON_WORD_BANK_FILES.is_empty()
                                    class="ui-btn-ghost"
                                >
                                    {move || tr(lang.get(), "选择词库", "Load Lexicon")}
                                </button>
                                <ImportDictionaryButton
                                    input_id="practice-mode-import-dictionary-input".to_string()
                                    label=Signal::derive(move || {
                                        tr(lang.get(), "导入字典文件", "Import Dictionary File").to_string()
                                    })
                                    class="ui-btn-ghost cursor-pointer".to_string()
                                    set_entries=lexicon_state.set_entries
                                    set_status=set_status
                                    set_data_version=lexicon_state.set_data_version
                                    ui_language=lang
                                    set_source_name=lexicon_state.set_source_name
                                />
                            </div>
                        </div>
                        <div class="ui-card-actions">
                            <button
                                type="button"
                                on:click=move |_| set_current_page.set(AppPage::LexiconMode)
                                class="ui-btn-primary"
                            >
                                {move || tr(lang.get(), "词库模式", "Lexicon Mode")}
                            </button>
                        </div>
                    </section>

                    <section class="ui-card">
                        <p class="ui-eyebrow">
                            {move || tr(lang.get(), "系列", "series")}
                        </p>
                        <h2>{move || tr(lang.get(), "单词系列练习", "Series Practice")}</h2>
                        <p>
                            {move || {
                                tr(
                                    lang.get(),
                                    "数词、月份、代词、疑问词等固定系列练习。",
                                    "Fixed series practice: number, month, pronoun, interrogative.",
                                )
                            }}
                        </p>
                        <div class="ui-card-actions">
                            <button
                                type="button"
                                on:click=move |_| set_current_page.set(AppPage::SeriseSelect)
                                class="ui-btn-primary"
                            >
                                {move || tr(lang.get(), "单词系列模式", "Series Mode")}
                            </button>
                        </div>
                    </section>

                    <section class="ui-card">
                        <p class="ui-eyebrow">
                            {move || tr(lang.get(), "跟读", "listen")}
                        </p>
                        <h2>{move || tr(lang.get(), "跟读练习", "Read along")}</h2>
                        <p>
                            {move || {
                                tr(
                                    lang.get(),
                                    "音频与歌词按文件名配对，边听边跟读。",
                                    "Audio and lyrics matched by filename. Read along as the track plays.",
                                )
                            }}
                        </p>
                        <div class="ui-card-actions">
                            <button
                                type="button"
                                class="ui-btn-primary"
                                on:click=move |_| set_current_page.set(AppPage::Player)
                            >
                                {move || tr(lang.get(), "打开播放器", "Open player")}
                            </button>
                        </div>
                    </section>
                </div>

                <div class="mt-8 text-left sm:text-right">
                    <button
                        type="button"
                        class="ui-link"
                        on:click=move |_| set_current_page.set(AppPage::LocalLexiconEditor)
                    >
                        {move || tr(lang.get(), "本地词库修改器", "Local Lexicon Editor")}
                    </button>
                </div>
        </PageShell>
    }
}
