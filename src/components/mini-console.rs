use leptos::prelude::*;

#[component]
pub fn MiniConsole(
    #[prop(into)] message: Signal<String>,
    #[prop(optional)] max_entries: Option<usize>,
) -> impl IntoView {
    let (history, set_history) = signal(Vec::<String>::new());
    let panel_ref = NodeRef::<leptos::html::Div>::new();
    let limit = max_entries.unwrap_or(200);

    Effect::new(move |_| {
        let current = message.get();
        if current.trim().is_empty() {
            return;
        }
        set_history.update(|items| {
            items.push(current);
            if items.len() > limit {
                let overflow = items.len() - limit;
                items.drain(0..overflow);
            }
        });
    });

    Effect::new(move |_| {
        let _ = history.get().len();
        #[cfg(target_arch = "wasm32")]
        {
            schedule_scroll_to_bottom(panel_ref.clone());
        }
    });

    view! {
        <section class="mt-3 rounded-lg border border-emerald-800/70 bg-black px-3 py-2 font-mono text-sm text-emerald-400 shadow-inner shadow-emerald-950/40">
            <div
                node_ref=panel_ref
                class="h-[4.5rem] overflow-y-auto whitespace-pre-wrap leading-6 pr-1"
            >
                <div class="min-h-full flex flex-col justify-end">
                    <For
                        each=move || {
                            history
                                .get()
                                .into_iter()
                                .enumerate()
                                .collect::<Vec<(usize, String)>>()
                        }
                        key=|(idx, _)| *idx
                        children=move |(_, line)| {
                            view! { <p>{line}</p> }
                        }
                    />
                </div>
            </div>
        </section>
    }
}

#[cfg(target_arch = "wasm32")]
fn schedule_scroll_to_bottom(panel_ref: NodeRef<leptos::html::Div>) {
    use wasm_bindgen::{JsCast, closure::Closure};

    let callback = Closure::wrap(Box::new(move || {
        if let Some(panel) = panel_ref.get_untracked() {
            panel.set_scroll_top(panel.scroll_height());
        }
    }) as Box<dyn FnMut()>);

    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            0,
        );
    }
    callback.forget();
}
