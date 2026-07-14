//! 迷你控制台组件：展示状态日志，支持折叠与拖拽改变高度。

#[cfg(target_arch = "wasm32")]
use leptos::ev;
use leptos::prelude::*;

use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
const MIN_PANEL_HEIGHT_PX: i32 = 72;
const DEFAULT_PANEL_HEIGHT_PX: i32 = 72;

#[component]
pub fn MiniConsole(
    #[prop(into)] message: Signal<String>,
    #[prop(optional)] max_entries: Option<usize>,
) -> impl IntoView {
    let (history, set_history) = signal(Vec::<String>::new());
    let panel_ref = NodeRef::<leptos::html::Div>::new();
    let (panel_height_px, set_panel_height) = signal(DEFAULT_PANEL_HEIGHT_PX);
    let (resize_state, set_resize_state) = signal(None::<(i32, i32)>);
    let move_listener = Arc::new(Mutex::new(None::<WindowListenerHandle>));
    let up_listener = Arc::new(Mutex::new(None::<WindowListenerHandle>));
    let limit = max_entries.unwrap_or(200);
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (&set_panel_height, &resize_state);
    }

    Effect::new(move |_| {
        let current = message.get();
        if current.trim().is_empty() {
            return;
        }
        set_history.update(|items| {
            if items.last().is_some_and(|last| last == &current) {
                return;
            }
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

    let stop_resize = {
        let move_listener = Arc::clone(&move_listener);
        let up_listener = Arc::clone(&up_listener);
        move |_: leptos::ev::MouseEvent| {
            set_resize_state.set(None);
            if let Some(handle) = move_listener.lock().expect("move listener lock").take() {
                handle.remove();
            }
            if let Some(handle) = up_listener.lock().expect("up listener lock").take() {
                handle.remove();
            }
        }
    };
    #[cfg(target_arch = "wasm32")]
    let move_listener_for_resize = Arc::clone(&move_listener);
    #[cfg(target_arch = "wasm32")]
    let up_listener_for_resize = Arc::clone(&up_listener);
    let start_resize = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        set_resize_state.set(Some((ev.client_y(), panel_height_px.get_untracked())));

        #[cfg(target_arch = "wasm32")]
        {
            let move_listener = Arc::clone(&move_listener_for_resize);
            let up_listener = Arc::clone(&up_listener_for_resize);
            if let Some(handle) = move_listener.lock().expect("move listener lock").take() {
                handle.remove();
            }
            if let Some(handle) = up_listener.lock().expect("up listener lock").take() {
                handle.remove();
            }

            let move_handle =
                window_event_listener(ev::mousemove, move |ev: web_sys::MouseEvent| {
                    if let Some((start_y, start_height)) = resize_state.get_untracked() {
                        let delta = ev.client_y() - start_y;
                        let max_height = max_panel_height_px();
                        let next = (start_height + delta)
                            .max(MIN_PANEL_HEIGHT_PX)
                            .min(max_height);
                        set_panel_height.set(next);
                    }
                });
            *move_listener.lock().expect("move listener set lock") = Some(move_handle);

            let move_listener_for_up = Arc::clone(&move_listener);
            let up_listener_for_up = Arc::clone(&up_listener);
            let up_handle = window_event_listener(ev::mouseup, move |_ev: web_sys::MouseEvent| {
                set_resize_state.set(None);
                if let Some(handle) = move_listener_for_up
                    .lock()
                    .expect("move listener up lock")
                    .take()
                {
                    handle.remove();
                }
                if let Some(handle) = up_listener_for_up
                    .lock()
                    .expect("up listener up lock")
                    .take()
                {
                    handle.remove();
                }
            });
            *up_listener.lock().expect("up listener set lock") = Some(up_handle);
        }
    };

    on_cleanup({
        let move_listener = Arc::clone(&move_listener);
        let up_listener = Arc::clone(&up_listener);
        move || {
            if let Some(handle) = move_listener.lock().expect("move listener lock").take() {
                handle.remove();
            }
            if let Some(handle) = up_listener.lock().expect("up listener lock").take() {
                handle.remove();
            }
        }
    });

    view! {
        <section
            class="mt-3 rounded-lg border border-emerald-800/70 bg-black px-3 py-2 font-mono text-sm text-emerald-400 shadow-inner shadow-emerald-950/40"
            on:mouseup=stop_resize
        >
            <div
                node_ref=panel_ref
                class="overflow-y-auto overflow-x-hidden whitespace-pre-wrap leading-6 pr-1"
                style:height=move || format!("{}px", panel_height_px.get())
                style:max-height="80vh"
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
            <div
                on:mousedown=start_resize
                class="mt-1 flex h-3 cursor-ns-resize items-center justify-center select-none"
                title="拖拽调整高度"
            >
                <span class="h-1 w-10 rounded-full bg-emerald-800/70"></span>
            </div>
        </section>
    }
}

#[cfg(target_arch = "wasm32")]
fn max_panel_height_px() -> i32 {
    if let Some(window) = web_sys::window() {
        if let Ok(height) = window.inner_height() {
            if let Some(height_px) = height.as_f64() {
                return (height_px * 0.8)
                    .round()
                    .max(f64::from(MIN_PANEL_HEIGHT_PX)) as i32;
            }
        }
    }
    640
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
