# UNRELEASED

- Make `into_owned` available for cloneable reference targets in core-only builds.
- Add shared and mutable folding, mutable views, fallible mapping, and `Option` and `Result` transposition.
- Add on-demand ownership conversion with `make_owned_with` and `try_make_owned_with`.

- Add optional QuickCheck, proptest, arbitrary, Borsh, Schemars, rkyv, and zerocopy integrations.
- Forward Debug formatting to either payload for all pairs with Debug implementations.

- Add `Maybe::fold`, `Maybe::borrow_as`, a const shared accessor on `MaybeMut`, and a consuming conversion from `MaybeMut` to `MaybeRef`.
- Forward `clone_from` to the payload when the variant matches so reusable buffers can retain their allocation.
- Report the caller's location when `unwrap_ref`, `unwrap_owned`, `expect_ref`, or `expect_owned` panics.
- Extend `Clone` and `Copy` to arbitrary `Maybe<R, T>` payload pairs that implement the corresponding trait.
- Extend Serde serialization to both payloads of any serializable pair. The untagged format and deserialization into `Owned` are unchanged.
- Add the `serde_1` feature and forward the selected `alloc` and `std` features to Serde.
- Set the supported Rust version to 1.85 and test the core, alloc, and std configurations on it.

# 0.1.2 (January 6th, 2022)

FEATURES


