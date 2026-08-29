//! 快速记忆练习页：左右整框对照，随机一种变体，遮罩切换显隐，支持分页。

use leptos::prelude::*;

use crate::app_state::{LexiconState, PracticeState, UiState};
use crate::components::mini_console::MiniConsole;
use crate::components::practice_engine::{
    PracticeSettings, default_answer_fields, entry_field_value, is_answer_field_available,
};
use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;
use crate::structures::field_meta::NONE_FIELD_KEY;
use crate::structures::word_bank_entry::{UiLanguage, WordBankEntry};
use crate::utils::i18n::{field_label, tr};
use crate::utils::shuffle::pick_index;

#[derive(Clone, Debug, PartialEq, Eq)]
struct RememberRow {
    entry_id: String,
    /// (label, value) 提示字段，跳过「无」。
    prompts: Vec<(String, String)>,
    variant_label: String,
    answer_value: String,
}

#[component]
pub fn LexiconRememberPage() -> impl IntoView {
    let lexicon_state = expect_context::<LexiconState>();
    let practice_state = expect_context::<PracticeState>();
    let lang = expect_context::<UiState>().ui_language;

    let (questions_per_page, set_questions_per_page) = signal(10_usize);
    let (prompt_field_a, set_prompt_field_a) = signal("chinese".to_string());
    let (prompt_field_b, set_prompt_field_b) = signal("english".to_string());
    let (prompt_field_c, set_prompt_field_c) = signal("part_of_speech".to_string());
    let (answer_fields, set_answer_fields) = signal(default_answer_fields());
    let (allow_answer_reveal, set_allow_answer_reveal) = signal(true);
    let (status, set_status) = signal(String::new());
    let (page_index, set_page_index) = signal(0_usize);
    let (left_masked, set_left_masked) = signal(false);
    let (right_masked, set_right_masked) = signal(false);
    let (page_rows, set_page_rows) = signal(Vec::<RememberRow>::new());

    let hide_all_revealed_answers = Callback::new(move |_| {
        set_status.set(String::new());
    });

    let total_pages = Signal::derive(move || {
        let total = practice_state.selected_word_entry_ids.get().len();
        let page_size = questions_per_page.get().max(1);
        if total == 0 {
            1
        } else {
            total.div_ceil(page_size)
        }
    });

    // 页大小或选词变化时校正页码；翻页/设置变化时重建当前页行并重置遮罩。
    Effect::new(move |_| {
        let page_size = questions_per_page.get().max(1);
        let selected_ids = practice_state.selected_word_entry_ids.get();
        let total = selected_ids.len();
        let max_page = if total == 0 {
            0
        } else {
            (total - 1) / page_size
        };
        let mut current_page = page_index.get();
        if current_page > max_page {
            current_page = max_page;
            set_page_index.set(max_page);
        }

        let language = lang.get();
        let field_a = prompt_field_a.get();
        let field_b = prompt_field_b.get();
        let field_c = prompt_field_c.get();
        let fields = answer_fields.get();
        let all_entries = lexicon_state.entries.get();
        let start = current_page.saturating_mul(page_size);

        let rows = selected_ids
            .into_iter()
            .skip(start)
            .take(page_size)
            .filter_map(|id| all_entries.iter().find(|entry| entry.id == id).cloned())
            .map(|entry| {
                build_remember_row(entry, &[&field_a, &field_b, &field_c], &fields, language)
            })
            .collect::<Vec<_>>();

        set_page_rows.set(rows);
        set_left_masked.set(false);
        set_right_masked.set(false);
    });

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-3 sm:p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-4 sm:p-6 shadow-xl">
                <ReturnButton target_page=AppPage::LexiconMode/>
                <header class="mb-6 flex items-center gap-4">
                    <h1 class="text-xl sm:text-2xl font-bold tracking-tight">
                        {move || tr(lang.get(), "快速记忆练习", "Quick Memory Practice")}
                    </h1>
                </header>

                <MiniConsole
                    message=Signal::derive(move || {
                        let language = lang.get();
                        let selected_count = practice_state.selected_word_entry_ids.get().len();
                        let current = page_index.get() + 1;
                        let pages = total_pages.get();
                        let overview = format!(
                            "{}：{selected_count} {}，{} {current}/{pages}。",
                            tr(language, "当前可练习词条", "Available entries"),
                            tr(language, "条", "entries"),
                            tr(language, "当前页", "Page")
                        );
                        let status_line = {
                            let s = status.get();
                            if s.trim().is_empty() {
                                tr(
                                    language,
                                    "点击左侧或右侧框可切换遮罩（隐藏/显示整侧内容）。",
                                    "Click left or right panel to toggle the overlay mask.",
                                )
                                .to_string()
                            } else {
                                s
                            }
                        };
                        format!("{overview}\n{status_line}")
                    })
                />

                <section class="rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">
                        {move || tr(lang.get(), "第一部分：练习设置", "Part 1: Practice Settings")}
                    </h2>
                    <PracticeSettings
                        questions_per_page=questions_per_page
                        set_questions_per_page=set_questions_per_page
                        prompt_field_a=prompt_field_a
                        set_prompt_field_a=set_prompt_field_a
                        prompt_field_b=prompt_field_b
                        set_prompt_field_b=set_prompt_field_b
                        prompt_field_c=prompt_field_c
                        set_prompt_field_c=set_prompt_field_c
                        answer_fields=answer_fields
                        set_answer_fields=set_answer_fields
                        allow_answer_reveal=allow_answer_reveal
                        set_allow_answer_reveal=set_allow_answer_reveal
                        hide_all_revealed_answers=hide_all_revealed_answers
                    />
                </section>

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">
                        {move || tr(lang.get(), "第二部分：记忆对照", "Part 2: Memory Cards")}
                    </h2>
                    {move || {
                        if page_rows.get().is_empty() {
                            view! {
                                <p class="mt-4 text-sm text-slate-400">
                                    {move || {
                                        tr(
                                            lang.get(),
                                            "当前没有可练习词条，请先回到选词页勾选词条。",
                                            "No entries to practice. Please select entries first.",
                                        )
                                    }}
                                </p>
                            }
                            .into_any()
                        } else {
                            // 每行左右成对同高；整侧遮罩覆盖左半/右半。
                            view! {
                                <div class="relative mt-4 overflow-hidden rounded-xl border border-slate-700 bg-slate-950/70">
                                    <div>
                                        <For
                                            each=move || page_rows.get()
                                            key=|row| row.entry_id.clone()
                                            children=move |row| {
                                                let prompts = if row.prompts.is_empty() {
                                                    view! { <></> }.into_any()
                                                } else {
                                                    view! {
                                                        <p class="flex flex-wrap items-baseline justify-end gap-x-3 gap-y-1">
                                                            {row.prompts
                                                                .into_iter()
                                                                .map(|(label, value)| {
                                                                    view! {
                                                                        <span>
                                                                            <span class="text-slate-400">{label}{": "}</span>
                                                                            <span>{value}</span>
                                                                        </span>
                                                                    }
                                                                })
                                                                .collect_view()}
                                                        </p>
                                                    }
                                                        .into_any()
                                                };
                                                let variant_label = row.variant_label;
                                                let answer_value = row.answer_value;
                                                view! {
                                                    <div class="grid grid-cols-2 border-b border-slate-800 last:border-b-0">
                                                        <div class="border-r border-slate-800 p-3 text-right text-sm text-slate-200">
                                                            <div class="space-y-1">
                                                                {prompts}
                                                                <p>
                                                                    <span class="text-xs text-slate-400">
                                                                        {move || format!("{}: ", tr(lang.get(), "变体", "Variant"))}
                                                                    </span>
                                                                    <span>{variant_label}</span>
                                                                </p>
                                                            </div>
                                                        </div>
                                                        <div class="flex items-center p-3 text-left text-sm text-slate-200">
                                                            <p class="font-medium tracking-wide">{answer_value}</p>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                    <button
                                        type="button"
                                        aria-label=move || tr(lang.get(), "切换左侧遮罩", "Toggle left mask")
                                        class="absolute inset-y-0 left-0 z-10 w-1/2 border-0 bg-transparent p-0"
                                        on:click=move |_| {
                                            set_left_masked.update(|masked| *masked = !*masked);
                                        }
                                    >
                                        <span
                                            class="pointer-events-none absolute inset-0 bg-slate-950 transition-opacity duration-150"
                                            style=move || {
                                                if left_masked.get() {
                                                    "opacity: 1"
                                                } else {
                                                    "opacity: 0"
                                                }
                                            }
                                        />
                                    </button>
                                    <button
                                        type="button"
                                        aria-label=move || tr(lang.get(), "切换右侧遮罩", "Toggle right mask")
                                        class="absolute inset-y-0 right-0 z-10 w-1/2 border-0 bg-transparent p-0"
                                        on:click=move |_| {
                                            set_right_masked.update(|masked| *masked = !*masked);
                                        }
                                    >
                                        <span
                                            class="pointer-events-none absolute inset-0 bg-slate-950 transition-opacity duration-150"
                                            style=move || {
                                                if right_masked.get() {
                                                    "opacity: 1"
                                                } else {
                                                    "opacity: 0"
                                                }
                                            }
                                        />
                                    </button>
                                </div>
                            }
                            .into_any()
                        }
                    }}
                </section>

                <section class="mt-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                    <p class="text-sm text-slate-400">
                        {move || {
                            format!(
                                "{} {} / {}",
                                tr(lang.get(), "页码", "Page"),
                                page_index.get() + 1,
                                total_pages.get()
                            )
                        }}
                    </p>
                    <div class="flex gap-3">
                        <button
                            type="button"
                            class="inline-flex flex-1 items-center justify-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm text-slate-100 hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-50 sm:flex-none"
                            disabled=move || page_index.get() == 0
                            on:click=move |_| {
                                set_page_index.update(|page| *page = page.saturating_sub(1));
                            }
                        >
                            {move || tr(lang.get(), "上一页", "Previous")}
                        </button>
                        <button
                            type="button"
                            class="inline-flex flex-1 items-center justify-center rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm text-slate-100 hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-50 sm:flex-none"
                            disabled=move || page_index.get() + 1 == total_pages.get()
                            on:click=move |_| {
                                let max_page = total_pages.get_untracked().saturating_sub(1);
                                set_page_index.update(|page| {
                                    if *page < max_page {
                                        *page += 1;
                                    }
                                });
                            }
                        >
                            {move || tr(lang.get(), "下一页", "Next")}
                        </button>
                    </div>
                </section>
            </section>
        </main>
    }
}

fn build_remember_row(
    entry: WordBankEntry,
    prompt_fields: &[&str],
    answer_fields: &[String],
    language: UiLanguage,
) -> RememberRow {
    let prompts = prompt_fields
        .iter()
        .copied()
        .filter(|field| *field != NONE_FIELD_KEY)
        .map(|field| {
            (
                field_label(language, field).to_string(),
                prompt_value(&entry, field, language),
            )
        })
        .collect::<Vec<_>>();

    let available = answer_fields
        .iter()
        .filter(|field| is_answer_field_available(&entry, field))
        .cloned()
        .collect::<Vec<_>>();

    let (variant_label, answer_value) = if available.is_empty() {
        (
            tr(language, "无可显示变体", "No variants available").to_string(),
            tr(language, "无可显示挪威语值", "No Norwegian values available").to_string(),
        )
    } else {
        let field = &available[pick_index(available.len())];
        (
            field_label(language, field).to_string(),
            entry_field_value(&entry, field),
        )
    };

    RememberRow {
        entry_id: entry.id,
        prompts,
        variant_label,
        answer_value,
    }
}

fn prompt_value(entry: &WordBankEntry, field: &str, language: UiLanguage) -> String {
    if field == "part_of_speech" {
        entry.part_of_speech.display_name(language).to_string()
    } else {
        entry_field_value(entry, field)
    }
}
