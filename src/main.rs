use leptos::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct WordEntry {
    id: String,
    part_of_speech: String,
    norwegian_base: String,
    chinese: Vec<String>,
    english: Vec<String>,
    tags: Vec<String>,
}

fn load_word_bank() -> Result<Vec<WordEntry>, serde_json::Error> {
    serde_json::from_str(include_str!("../data/word-bank.json"))
}

fn main() {
    let word_bank = load_word_bank().unwrap_or_default();

    leptos::mount::mount_to_body(move || {
        view! {
            <main class="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center">
                <section class="w-full max-w-2xl p-8 rounded-2xl border border-slate-800 bg-slate-900 shadow-xl">
                    <h1 class="text-3xl font-bold tracking-tight">"Norwegian Word Practice"</h1>
                    <p class="mt-4 text-slate-300">
                        "纯前端 + JSON 词库框架已就绪。"
                    </p>
                    <p class="mt-2 text-slate-400">
                        {format!("当前词库条目数：{}", word_bank.len())}
                    </p>
                    <ul class="mt-6 space-y-2 text-slate-300">
                        {word_bank
                            .iter()
                            .take(3)
                            .map(|entry| {
                                view! {
                                    <li class="rounded-lg border border-slate-800 px-3 py-2 text-sm">
                                        {format!(
                                            "#{} · {} {} · {} · EN:{} · tag:{}",
                                            entry.id,
                                            entry.part_of_speech,
                                            entry.norwegian_base,
                                            entry.chinese.join(" / "),
                                            entry.english.len(),
                                            entry.tags.len()
                                        )}
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </section>
            </main>
        }
    })
}