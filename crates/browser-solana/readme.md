# browser-solana

Browser WASM helpers for Solana dApps: wallet connect, page-injected JS bridges, and Candy Machine RPC reads.

This crate is **framework-agnostic** (no Leptos/React). Pair it with [`leptos-solana-gate`](../leptos-solana-gate) for NFT page/component guards.

## Features

| Module | Purpose |
|--------|---------|
| `wallet` | `window.solana` connect / disconnect / `shorten_address` |
| `ownership` | `owns_via_window_fn("myCheck", owner)` → `bool` |
| `mint` | `mint_via_window_fn("myMint")` → `{ signature, mint }` |
| `candy_machine` | Parameterized `items_redeemed` JSON-RPC read |
| `js_bridge` | Generic `window[name](...args)` Promise helper |

## Example

```rust
use browser_solana::{
    connect_wallet, fetch_items_redeemed, mint_via_window_fn, owns_via_window_fn,
    CandyMachineQuery,
};

async fn demo() -> Result<(), browser_solana::BrowserSolanaError> {
    let pubkey = connect_wallet().await?;
    let owns = owns_via_window_fn("__myHasCollectionNft", &pubkey).await?;
    if !owns {
        let minted = mint_via_window_fn("__myMintNft").await?;
        let _ = minted.signature;
    }
    let sold = fetch_items_redeemed(CandyMachineQuery {
        rpc_url: "https://api.devnet.solana.com",
        candy_machine_id: "YourCandyMachine111...",
        items_redeemed_offset: 112,
    })
    .await?;
    let _ = sold;
    Ok(())
}
```

## Design notes

- No hard-coded collection / candy machine / treasury IDs.
- Metaplex Umi (or any mint stack) stays in **page JS**; Rust only calls the bridge you expose on `window`.
- Errors are `BrowserSolanaError` (string message) for easy UI display.
