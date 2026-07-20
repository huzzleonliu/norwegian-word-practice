# leptos-solana-gate

Leptos CSR guards for Solana NFT (or any async ownership) gated pages and buttons.

Depends on [`browser-solana`](../browser-solana) only for the optional
`ownership_checker_from_window_fn` helper — the gate state machine itself is
driven by an injected [`OwnershipChecker`].

## Quick start

```rust
use leptos::prelude::*;
use leptos_solana_gate::{
    install_gate_effects, ownership_checker_from_window_fn, provide_gate_runtime,
    GateIntent, GateRuntime, GatedNavigateButton, NftGateState, OwnershipStatus,
    RequireNftPage, WalletGateHandle,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Route { Editor, Mint, Home }

#[component]
fn App() -> impl IntoView {
    let (address, set_address) = signal(None::<String>);
    let (ownership, set_ownership) = signal(OwnershipStatus::Disconnected);
    let (pending, set_pending) = signal(None::<GateIntent<Route>>);

    let gate = NftGateState { ownership, set_ownership, pending, set_pending };
    let wallet = WalletGateHandle {
        address,
        request_connect: Callback::new(move |_| { /* open wallet UI */ }),
    };

    let runtime = GateRuntime {
        gate,
        wallet,
        navigate: Callback::new(|r: Route| { /* router */ let _ = r; }),
        on_access_denied: Callback::new(|_| { /* open mint tab */ }),
        on_fallback: Callback::new(|_| { /* go to mint route */ }),
        checker: ownership_checker_from_window_fn("__myHasCollectionNft"),
    };
    provide_gate_runtime(runtime.clone());
    install_gate_effects(runtime);

    view! {
        <GatedNavigateButton target=Route::Editor class="btn">
            "Open editor"
        </GatedNavigateButton>
        <RequireNftPage
            route=Route::Editor
            checking=|| view! { <p>"Checking…"</p> }.into()
            disconnected=|| view! { <p>"Connect wallet…"</p> }.into()
            missing=|| view! { <p>"Mint required"</p> }.into()
        >
            || view! { <p>"Secret"</p> }.into()
        </RequireNftPage>
    }
}
```

> Note: `ChildrenFn` prop syntax varies slightly by Leptos version; see `nwp-web`
> for a production wiring example.

## What stays in your app

- Collection / Candy Machine IDs
- Mint page UI and `window.__yourMint` script
- i18n / theme
- Router enum (`R`) and path mapping
