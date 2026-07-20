//! Page and button guard components.

use std::rc::Rc;

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::action::{refresh_wallet_ownership, request_gated_route};
use crate::runtime::GateRuntime;
use crate::state::OwnershipStatus;

/// Cloneable view factory for gate fallback UIs.
#[derive(Clone)]
pub struct GateViewFn(SendWrapper<Rc<dyn Fn() -> AnyView>>);

impl GateViewFn {
    /// Build from a closure that returns [`AnyView`].
    pub fn new(f: impl Fn() -> AnyView + 'static) -> Self {
        Self(SendWrapper::new(Rc::new(f)))
    }

    fn render(&self) -> AnyView {
        (self.0)()
    }
}

impl<F> From<F> for GateViewFn
where
    F: Fn() -> AnyView + 'static,
{
    fn from(value: F) -> Self {
        Self::new(value)
    }
}

/// Page-level guard: render `children` only when ownership is [`OwnershipStatus::Owns`].
///
/// Deep-link behaviour: if the user lands here disconnected / missing, the gate
/// requests connect and/or runs `on_access_denied` from [`GateRuntime`].
#[component]
pub fn RequireNftPage<R>(
    /// Route value used when auto-requesting access on mount (deep link).
    route: R,
    /// Shown while ownership is [`OwnershipStatus::Checking`].
    #[prop(into)]
    checking: GateViewFn,
    /// Shown while [`OwnershipStatus::Disconnected`].
    #[prop(into)]
    disconnected: GateViewFn,
    /// Shown while [`OwnershipStatus::Missing`].
    #[prop(into)]
    missing: GateViewFn,
    /// Protected content when the user owns access.
    children: ChildrenFn,
) -> impl IntoView
where
    R: Clone + Copy + PartialEq + Eq + Send + Sync + 'static,
{
    let runtime = expect_context::<GateRuntime<R>>();
    let gate = runtime.gate;
    let wallet = runtime.wallet;
    let navigate = runtime.navigate;
    let on_access_denied = runtime.on_access_denied;
    let (denied_opened, set_denied_opened) = signal(false);

    Effect::new(move |_| {
        let ownership = gate.ownership.get();
        let connected = wallet.address.get().is_some();
        if !connected {
            if gate.pending.get_untracked().is_none() {
                request_gated_route(gate, wallet, navigate, on_access_denied, route);
            }
            return;
        }
        if ownership == OwnershipStatus::Missing && !denied_opened.get_untracked() {
            set_denied_opened.set(true);
            on_access_denied.run(());
        }
    });

    view! {
        {move || {
            match gate.ownership.get() {
                OwnershipStatus::Owns => children().into_any(),
                OwnershipStatus::Checking => checking.render(),
                OwnershipStatus::Disconnected => disconnected.render(),
                OwnershipStatus::Missing => missing.render(),
            }
        }}
    }
}

/// Button that runs the gated navigation flow toward `target`.
#[component]
pub fn GatedNavigateButton<R>(
    /// Destination route once ownership is confirmed.
    target: R,
    /// CSS class string for the `<button>`.
    #[prop(into)]
    class: String,
    children: Children,
) -> impl IntoView
where
    R: Clone + Copy + PartialEq + Eq + Send + Sync + 'static,
{
    let runtime = expect_context::<GateRuntime<R>>();
    let gate = runtime.gate;
    let wallet = runtime.wallet;
    let navigate = runtime.navigate;
    let on_access_denied = runtime.on_access_denied;

    let on_click = move |_| {
        request_gated_route(gate, wallet, navigate, on_access_denied, target);
    };

    view! {
        <button type="button" class=class on:click=on_click>
            {children()}
        </button>
    }
}

/// Helper for “recheck ownership” buttons inside custom missing views.
pub fn recheck_ownership<R>(runtime: &GateRuntime<R>)
where
    R: Clone + Copy + PartialEq + Eq + Send + Sync + 'static,
{
    let Some(pubkey) = runtime.wallet.address.get_untracked() else {
        return;
    };
    refresh_wallet_ownership(runtime.gate, runtime.checker.clone(), pubkey);
}
