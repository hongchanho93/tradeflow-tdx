# Provenance record

This record separates repository evidence from the maintainer's authorship statement.

## Repository evidence

- The Rust implementation was already present in the initial TradeFlow Lite source snapshot, commit `d7568e446cf9da7c51d39b733085a3434fd802bd` dated 2026-09-14.
- The tracked history for the original `src-tauri/src/tdx/` paths contains only TradeFlow Lite maintainer identities.
- The current source tree does not import, depend on, or bundle `pytdx` or another third-party TDX client.
- The crate's only direct third-party dependency is `flate2`; its resolved dependency graph is recorded by the application lockfile.

## Maintainer confirmation

On 2026-09-22, the TradeFlow maintainer confirmed that this Rust TDX implementation was independently authored and was not copied, translated, or adapted from another TDX client implementation. Git history begins with an imported source snapshot, so this statement records the pre-history authorship fact that Git alone cannot establish.

The `MIT OR Apache-2.0` software license covers this client implementation only. It does not grant access to, or rights in, TongdaXin servers, market data, exchange data, or other third-party services.
