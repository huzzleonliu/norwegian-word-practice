//! 列分组勾选面板：搜索范围与表格显示列共用同一布局；
//! 表格模式支持组内拖拽列顺序、组间拖拽组顺序，并在拖拽时显示放置提示。

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use super::utils::{header_name, search_column_groups};
use crate::structures::word_bank_entry::UiLanguage;
use crate::utils::i18n::{field_label, tr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragPayload {
    Column(usize),
    Group(&'static str),
}

/// 放置提示：列/组插入到目标之前或之后。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DropHint {
    BeforeColumn(usize),
    AfterColumn(usize),
    BeforeGroup(&'static str),
    AfterGroup(&'static str),
}

#[component]
pub fn ColumnGroupCheckboxes(
    lang: ReadSignal<UiLanguage>,
    columns: ReadSignal<Vec<bool>>,
    set_columns: WriteSignal<Vec<bool>>,
    /// 表格列显示顺序（数据列下标）。搜索模式可不传。
    #[prop(optional)]
    column_order: Option<ReadSignal<Vec<usize>>>,
    #[prop(optional)]
    set_column_order: Option<WriteSignal<Vec<usize>>>,
) -> impl IntoView {
    let reorder_enabled = column_order.is_some() && set_column_order.is_some();
    let (drag_payload, set_drag_payload) = signal(None::<DragPayload>);
    let (drop_hint, set_drop_hint) = signal(None::<DropHint>);

    let clear_drag = move || {
        set_drag_payload.set(None);
        set_drop_hint.set(None);
    };

    view! {
        <div class="space-y-3">
            {move || {
                if reorder_enabled {
                    view! {
                        <p class="text-xs text-slate-500">
                            {move || {
                                tr(
                                    lang.get(),
                                    "拖拽标签可调整组内列顺序；拖拽组标题可调整组顺序",
                                    "Drag tags to reorder columns within a group; drag group titles to reorder groups",
                                )
                            }}
                        </p>
                    }
                        .into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }}
            // 仅订阅 column_order：拖拽提示用子级独立订阅，避免重挂载打断 HTML5 DnD。
            {move || {
                let base_groups = search_column_groups();
                let order = column_order
                    .map(|sig| sig.get())
                    .unwrap_or_else(|| {
                        base_groups
                            .iter()
                            .flat_map(|(_, idxs)| idxs.iter().copied())
                            .collect()
                    });
                let display = display_groups(&order, &base_groups);

                display
                    .into_iter()
                    .map(|(group_name, indices)| {
                        let group_indices = indices.clone();
                        let base_groups_for_drop = base_groups.clone();

                        view! {
                            <div class="relative">
                                {move || {
                                    let show = matches!(
                                        (drag_payload.get(), drop_hint.get()),
                                        (
                                            Some(DragPayload::Group(_)),
                                            Some(DropHint::BeforeGroup(name))
                                        ) if name == group_name
                                    );
                                    if show {
                                        view! {
                                            <div
                                                class="pointer-events-none absolute inset-x-0 -top-1.5 z-10 h-0.5 rounded bg-sky-400"
                                                aria-hidden="true"
                                            />
                                        }
                                            .into_any()
                                    } else {
                                        view! { <></> }.into_any()
                                    }
                                }}
                                {move || {
                                    let show = matches!(
                                        (drag_payload.get(), drop_hint.get()),
                                        (
                                            Some(DragPayload::Group(_)),
                                            Some(DropHint::AfterGroup(name))
                                        ) if name == group_name
                                    );
                                    if show {
                                        view! {
                                            <div
                                                class="pointer-events-none absolute inset-x-0 -bottom-1.5 z-10 h-0.5 rounded bg-sky-400"
                                                aria-hidden="true"
                                            />
                                        }
                                            .into_any()
                                    } else {
                                        view! { <></> }.into_any()
                                    }
                                }}
                                <section
                                    class="rounded border border-slate-800 bg-slate-950/40 p-2 transition-[opacity,border-color]"
                                    class:opacity-50=move || {
                                        matches!(
                                            drag_payload.get(),
                                            Some(DragPayload::Group(name)) if name == group_name
                                        )
                                    }
                                    class:border-sky-500=move || {
                                        matches!(
                                            drag_payload.get(),
                                            Some(DragPayload::Group(name)) if name == group_name
                                        )
                                    }
                                    on:dragover=move |ev| {
                                        if !reorder_enabled {
                                            return;
                                        }
                                        let Some(DragPayload::Group(_)) =
                                            drag_payload.get_untracked()
                                        else {
                                            return;
                                        };
                                        ev.prevent_default();
                                        set_drop_effect_move(&ev);
                                        let hint = group_drop_hint_from_event(&ev, group_name);
                                        if drop_hint.get_untracked() != Some(hint) {
                                            set_drop_hint.set(Some(hint));
                                        }
                                    }
                                    on:drop=move |ev| {
                                        if !reorder_enabled {
                                            return;
                                        }
                                        let Some(DragPayload::Group(from_name)) =
                                            drag_payload.get_untracked()
                                        else {
                                            return;
                                        };
                                        ev.prevent_default();
                                        let hint = drop_hint.get_untracked().unwrap_or_else(|| {
                                            group_drop_hint_from_event(&ev, group_name)
                                        });
                                        if let Some(set_order) = set_column_order {
                                            set_order.update(|ord| {
                                                apply_group_drop_hint(
                                                    ord,
                                                    &base_groups_for_drop,
                                                    from_name,
                                                    hint,
                                                );
                                            });
                                        }
                                        clear_drag();
                                    }
                                >
                                    <p
                                        class="mb-2 inline-flex cursor-grab items-center gap-1.5 text-xs font-semibold text-slate-400 active:cursor-grabbing"
                                        class:cursor-default=!reorder_enabled
                                        draggable=reorder_enabled.to_string()
                                        on:dragstart=move |ev| {
                                            if !reorder_enabled {
                                                return;
                                            }
                                            set_drag_payload
                                                .set(Some(DragPayload::Group(group_name)));
                                            set_drop_hint.set(None);
                                            if let Ok(drag) = ev.dyn_into::<web_sys::DragEvent>() {
                                                if let Some(dt) = drag.data_transfer() {
                                                    let _ = dt.set_data(
                                                        "text/plain",
                                                        &format!("group:{group_name}"),
                                                    );
                                                    dt.set_effect_allowed("move");
                                                }
                                            }
                                        }
                                        on:dragend=move |_| clear_drag()
                                    >
                                        <span class="select-none text-slate-600" aria-hidden="true">
                                            "⠿"
                                        </span>
                                        {move || group_title(lang.get(), group_name)}
                                    </p>
                                    <div class="grid grid-cols-1 gap-2 sm:grid-cols-2 md:grid-cols-4 lg:grid-cols-5">
                                        {indices
                                            .into_iter()
                                            .map(|idx| {
                                                let group_indices_over = group_indices.clone();
                                                let group_indices_drop = group_indices.clone();

                                                view! {
                                                    <div class="relative">
                                                        {move || {
                                                            let show = matches!(
                                                                (drag_payload.get(), drop_hint.get()),
                                                                (
                                                                    Some(DragPayload::Column(_)),
                                                                    Some(DropHint::BeforeColumn(t))
                                                                ) if t == idx
                                                            );
                                                            if show {
                                                                view! {
                                                                    <div
                                                                        class="pointer-events-none absolute -left-1 top-0 z-10 h-full w-0.5 rounded bg-sky-400"
                                                                        aria-hidden="true"
                                                                    />
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! { <></> }.into_any()
                                                            }
                                                        }}
                                                        {move || {
                                                            let show = matches!(
                                                                (drag_payload.get(), drop_hint.get()),
                                                                (
                                                                    Some(DragPayload::Column(_)),
                                                                    Some(DropHint::AfterColumn(t))
                                                                ) if t == idx
                                                            );
                                                            if show {
                                                                view! {
                                                                    <div
                                                                        class="pointer-events-none absolute -right-1 top-0 z-10 h-full w-0.5 rounded bg-sky-400"
                                                                        aria-hidden="true"
                                                                    />
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! { <></> }.into_any()
                                                            }
                                                        }}
                                                        <label
                                                            class="inline-flex w-full cursor-grab items-center gap-2 rounded border border-slate-800 bg-slate-950/60 px-2 py-1 text-xs text-slate-300 transition-[opacity,border-color] active:cursor-grabbing"
                                                            class:opacity-40=move || {
                                                                matches!(
                                                                    drag_payload.get(),
                                                                    Some(DragPayload::Column(src)) if src == idx
                                                                )
                                                            }
                                                            class:border-sky-500=move || {
                                                                matches!(
                                                                    drag_payload.get(),
                                                                    Some(DragPayload::Column(src)) if src == idx
                                                                )
                                                            }
                                                            class:cursor-default=!reorder_enabled
                                                            draggable=reorder_enabled.to_string()
                                                            on:dragstart=move |ev| {
                                                                if !reorder_enabled {
                                                                    return;
                                                                }
                                                                set_drag_payload
                                                                    .set(Some(DragPayload::Column(idx)));
                                                                set_drop_hint.set(None);
                                                                if let Ok(drag) =
                                                                    ev.dyn_into::<web_sys::DragEvent>()
                                                                {
                                                                    if let Some(dt) = drag.data_transfer()
                                                                    {
                                                                        let _ = dt.set_data(
                                                                            "text/plain",
                                                                            &idx.to_string(),
                                                                        );
                                                                        dt.set_effect_allowed("move");
                                                                    }
                                                                }
                                                            }
                                                            on:dragover=move |ev| {
                                                                if !reorder_enabled {
                                                                    return;
                                                                }
                                                                let Some(DragPayload::Column(from_idx)) =
                                                                    drag_payload.get_untracked()
                                                                else {
                                                                    return;
                                                                };
                                                                if !same_group(
                                                                    &group_indices_over,
                                                                    from_idx,
                                                                    idx,
                                                                ) {
                                                                    return;
                                                                }
                                                                ev.prevent_default();
                                                                set_drop_effect_move(&ev);
                                                                let hint = column_drop_hint_from_event(
                                                                    &ev, idx,
                                                                );
                                                                if drop_hint.get_untracked() != Some(hint)
                                                                {
                                                                    set_drop_hint.set(Some(hint));
                                                                }
                                                            }
                                                            on:drop=move |ev| {
                                                                if !reorder_enabled {
                                                                    return;
                                                                }
                                                                let Some(DragPayload::Column(from_idx)) =
                                                                    drag_payload.get_untracked()
                                                                else {
                                                                    return;
                                                                };
                                                                if !same_group(
                                                                    &group_indices_drop,
                                                                    from_idx,
                                                                    idx,
                                                                ) {
                                                                    clear_drag();
                                                                    return;
                                                                }
                                                                ev.prevent_default();
                                                                let hint = drop_hint
                                                                    .get_untracked()
                                                                    .unwrap_or_else(|| {
                                                                        column_drop_hint_from_event(
                                                                            &ev, idx,
                                                                        )
                                                                    });
                                                                if let Some(set_order) = set_column_order {
                                                                    set_order.update(|ord| {
                                                                        apply_column_drop_hint(
                                                                            ord,
                                                                            &group_indices_drop,
                                                                            from_idx,
                                                                            hint,
                                                                        );
                                                                    });
                                                                }
                                                                clear_drag();
                                                            }
                                                            on:dragend=move |_| clear_drag()
                                                        >
                                                            <input
                                                                type="checkbox"
                                                                prop:checked=move || {
                                                                    columns
                                                                        .get()
                                                                        .get(idx)
                                                                        .copied()
                                                                        .unwrap_or(false)
                                                                }
                                                                on:mousedown=move |ev| ev.stop_propagation()
                                                                on:change=move |ev| {
                                                                    let checked = event_target_checked(&ev);
                                                                    set_columns.update(|cols| {
                                                                        if idx < cols.len() {
                                                                            cols[idx] = checked;
                                                                        }
                                                                    });
                                                                }
                                                            />
                                                            <span>
                                                                {move || {
                                                                    field_label(lang.get(), header_name(idx))
                                                                }}
                                                            </span>
                                                        </label>
                                                    </div>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                </section>
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}

fn group_title(lang: UiLanguage, group_name: &str) -> &'static str {
    match group_name {
        "core" => tr(lang, "基础组", "Core"),
        "verb" => tr(lang, "动词变体组", "Verb Forms"),
        "noun" => tr(lang, "名词变体组", "Noun Forms"),
        "adjective" => tr(lang, "形容词变体组", "Adjective Forms"),
        "pronoun" => tr(lang, "代词变体组", "Pronoun Forms"),
        "determinative" => tr(lang, "限定词变体组", "Determinative Forms"),
        "adverb" => tr(lang, "副词变体组", "Adverb Forms"),
        _ => "Group",
    }
}

fn set_drop_effect_move(ev: &leptos::ev::DragEvent) {
    if let Some(drag) = ev.dyn_ref::<web_sys::DragEvent>() {
        if let Some(dt) = drag.data_transfer() {
            dt.set_drop_effect("move");
        }
    }
}

fn column_drop_hint_from_event(ev: &leptos::ev::DragEvent, idx: usize) -> DropHint {
    let Some(drag) = ev.dyn_ref::<web_sys::DragEvent>() else {
        return DropHint::BeforeColumn(idx);
    };
    let Some(target) = drag.target() else {
        return DropHint::BeforeColumn(idx);
    };
    let Ok(el) = target.dyn_into::<web_sys::Element>() else {
        return DropHint::BeforeColumn(idx);
    };
    let box_el = el.closest("label").ok().flatten().unwrap_or(el);
    let Ok(html) = box_el.dyn_into::<web_sys::HtmlElement>() else {
        return DropHint::BeforeColumn(idx);
    };
    let rect = html.get_bounding_client_rect();
    let mid = rect.left() + rect.width() / 2.0;
    if (drag.client_x() as f64) < mid {
        DropHint::BeforeColumn(idx)
    } else {
        DropHint::AfterColumn(idx)
    }
}

fn group_drop_hint_from_event(ev: &leptos::ev::DragEvent, group_name: &'static str) -> DropHint {
    let Some(drag) = ev.dyn_ref::<web_sys::DragEvent>() else {
        return DropHint::BeforeGroup(group_name);
    };
    let Some(target) = drag.target() else {
        return DropHint::BeforeGroup(group_name);
    };
    let Ok(el) = target.dyn_into::<web_sys::Element>() else {
        return DropHint::BeforeGroup(group_name);
    };
    let box_el = el.closest("section").ok().flatten().unwrap_or(el);
    let Ok(html) = box_el.dyn_into::<web_sys::HtmlElement>() else {
        return DropHint::BeforeGroup(group_name);
    };
    let rect = html.get_bounding_client_rect();
    let mid = rect.top() + rect.height() / 2.0;
    if (drag.client_y() as f64) < mid {
        DropHint::BeforeGroup(group_name)
    } else {
        DropHint::AfterGroup(group_name)
    }
}

fn same_group(group_indices: &[usize], a: usize, b: usize) -> bool {
    group_indices.contains(&a) && group_indices.contains(&b)
}

/// 按 `order` 中首次出现位置排列组；组内按 `order` 排序。
pub fn display_groups(
    order: &[usize],
    base_groups: &[(&'static str, Vec<usize>)],
) -> Vec<(&'static str, Vec<usize>)> {
    let mut ranked: Vec<(usize, &'static str, Vec<usize>)> = base_groups
        .iter()
        .map(|(name, indices)| {
            let mut sorted = indices.clone();
            sorted.sort_by_key(|idx| {
                order
                    .iter()
                    .position(|i| i == idx)
                    .unwrap_or(usize::MAX)
            });
            let rank = sorted
                .iter()
                .filter_map(|idx| order.iter().position(|i| i == idx))
                .min()
                .unwrap_or(usize::MAX);
            (rank, *name, sorted)
        })
        .collect();
    ranked.sort_by_key(|(rank, _, _)| *rank);
    ranked
        .into_iter()
        .map(|(_, name, indices)| (name, indices))
        .collect()
}

fn apply_column_drop_hint(
    order: &mut Vec<usize>,
    group_indices: &[usize],
    from_idx: usize,
    hint: DropHint,
) {
    match hint {
        DropHint::BeforeColumn(to_idx) => {
            reorder_column_within_group(order, group_indices, from_idx, Some(to_idx));
        }
        DropHint::AfterColumn(to_idx) => {
            let after = next_in_group_order(order, group_indices, to_idx);
            reorder_column_within_group(order, group_indices, from_idx, after);
        }
        DropHint::BeforeGroup(_) | DropHint::AfterGroup(_) => {}
    }
}

fn apply_group_drop_hint(
    order: &mut Vec<usize>,
    base_groups: &[(&'static str, Vec<usize>)],
    from_group: &'static str,
    hint: DropHint,
) {
    match hint {
        DropHint::BeforeGroup(before) => {
            reorder_group_before(order, base_groups, from_group, before);
        }
        DropHint::AfterGroup(after) => {
            let display = display_groups(order, base_groups);
            let next = display
                .iter()
                .position(|(n, _)| *n == after)
                .and_then(|i| display.get(i + 1).map(|(n, _)| *n));
            match next {
                Some(before) => reorder_group_before(order, base_groups, from_group, before),
                None => reorder_group_to_end(order, base_groups, from_group),
            }
        }
        DropHint::BeforeColumn(_) | DropHint::AfterColumn(_) => {}
    }
}

fn next_in_group_order(order: &[usize], group_indices: &[usize], idx: usize) -> Option<usize> {
    let mut seen = false;
    for &col in order {
        if !group_indices.contains(&col) {
            continue;
        }
        if seen {
            return Some(col);
        }
        if col == idx {
            seen = true;
        }
    }
    None
}

/// 将 `from_idx` 移到 `before_idx` 之前；`before_idx == None` 表示移到组内末尾。
/// 仅允许组内成员之间调整。
pub fn reorder_column_within_group(
    order: &mut Vec<usize>,
    group_indices: &[usize],
    from_idx: usize,
    before_idx: Option<usize>,
) {
    if !group_indices.contains(&from_idx) {
        return;
    }
    if before_idx == Some(from_idx) {
        return;
    }
    if let Some(before) = before_idx {
        if !group_indices.contains(&before) {
            return;
        }
    }

    let members: Vec<usize> = order
        .iter()
        .copied()
        .filter(|i| group_indices.contains(i))
        .collect();
    let Some(from_pos) = members.iter().position(|&i| i == from_idx) else {
        return;
    };
    let mut reordered = members;
    let item = reordered.remove(from_pos);
    let insert_at = match before_idx {
        Some(before) => reordered.iter().position(|&i| i == before).unwrap_or(reordered.len()),
        None => reordered.len(),
    };
    reordered.insert(insert_at, item);

    let mut iter = reordered.into_iter();
    for slot in order.iter_mut() {
        if group_indices.contains(slot) {
            if let Some(next) = iter.next() {
                *slot = next;
            }
        }
    }
}

/// 将 `from_group` 整块移到 `before_group` 之前（保持组内相对顺序）。
pub fn reorder_group_before(
    order: &mut Vec<usize>,
    base_groups: &[(&'static str, Vec<usize>)],
    from_group: &str,
    before_group: &str,
) {
    if from_group == before_group {
        return;
    }
    let Some((_, from_indices)) = base_groups.iter().find(|(n, _)| *n == from_group) else {
        return;
    };
    let Some((_, before_indices)) = base_groups.iter().find(|(n, _)| *n == before_group) else {
        return;
    };

    let block: Vec<usize> = order
        .iter()
        .copied()
        .filter(|i| from_indices.contains(i))
        .collect();
    if block.is_empty() {
        return;
    }

    order.retain(|i| !from_indices.contains(i));

    let insert_at = order
        .iter()
        .position(|i| before_indices.contains(i))
        .unwrap_or(order.len());
    for (offset, col) in block.into_iter().enumerate() {
        order.insert(insert_at + offset, col);
    }
}

pub fn reorder_group_to_end(
    order: &mut Vec<usize>,
    base_groups: &[(&'static str, Vec<usize>)],
    from_group: &str,
) {
    let Some((_, from_indices)) = base_groups.iter().find(|(n, _)| *n == from_group) else {
        return;
    };
    let block: Vec<usize> = order
        .iter()
        .copied()
        .filter(|i| from_indices.contains(i))
        .collect();
    if block.is_empty() {
        return;
    }
    order.retain(|i| !from_indices.contains(i));
    order.extend(block);
}

#[cfg(test)]
mod tests {
    use super::{
        display_groups, reorder_column_within_group, reorder_group_before, reorder_group_to_end,
    };

    #[test]
    fn within_group_moves_earlier_item_before_target() {
        let mut order = vec![0, 1, 2, 3];
        let group = vec![0, 1, 2, 3];
        reorder_column_within_group(&mut order, &group, 0, Some(2));
        assert_eq!(order, vec![1, 0, 2, 3]);
    }

    #[test]
    fn within_group_moves_later_item_before_target() {
        let mut order = vec![0, 1, 2, 3];
        let group = vec![0, 1, 2, 3];
        reorder_column_within_group(&mut order, &group, 3, Some(1));
        assert_eq!(order, vec![0, 3, 1, 2]);
    }

    #[test]
    fn within_group_reorder_preserves_other_groups() {
        let mut order = vec![0, 1, 2, 10, 11, 12];
        let group = vec![10, 11, 12];
        reorder_column_within_group(&mut order, &group, 12, Some(10));
        assert_eq!(order, vec![0, 1, 2, 12, 10, 11]);
    }

    #[test]
    fn within_group_move_to_end() {
        let mut order = vec![0, 1, 2, 10, 11, 12];
        let group = vec![10, 11, 12];
        reorder_column_within_group(&mut order, &group, 10, None);
        assert_eq!(order, vec![0, 1, 2, 11, 12, 10]);
    }

    #[test]
    fn within_group_rejects_cross_group_target() {
        let mut order = vec![0, 1, 10, 11];
        let group = vec![10, 11];
        reorder_column_within_group(&mut order, &group, 10, Some(0));
        assert_eq!(order, vec![0, 1, 10, 11]);
    }

    #[test]
    fn group_reorder_moves_block() {
        let base = vec![
            ("core", vec![0, 1]),
            ("verb", vec![10, 11]),
            ("noun", vec![20, 21]),
        ];
        let mut order = vec![0, 1, 10, 11, 20, 21];
        reorder_group_before(&mut order, &base, "noun", "verb");
        assert_eq!(order, vec![0, 1, 20, 21, 10, 11]);
    }

    #[test]
    fn group_reorder_to_end() {
        let base = vec![
            ("core", vec![0, 1]),
            ("verb", vec![10, 11]),
            ("noun", vec![20, 21]),
        ];
        let mut order = vec![0, 1, 10, 11, 20, 21];
        reorder_group_to_end(&mut order, &base, "core");
        assert_eq!(order, vec![10, 11, 20, 21, 0, 1]);
    }

    #[test]
    fn display_groups_follows_order() {
        let base = vec![("core", vec![0, 1]), ("verb", vec![10, 11])];
        let order = vec![11, 10, 1, 0];
        let display = display_groups(&order, &base);
        assert_eq!(display[0].0, "verb");
        assert_eq!(display[0].1, vec![11, 10]);
        assert_eq!(display[1].0, "core");
        assert_eq!(display[1].1, vec![1, 0]);
    }
}
