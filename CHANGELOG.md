# UNRELEASED

- Extend `Clone` and `Copy` to arbitrary `Maybe<R, T>` payload pairs that implement the corresponding trait.
- Extend Serde serialization to both payloads of any serializable pair. The untagged format and deserialization into `Owned` are unchanged.
- Add the `serde_1` feature and forward the selected `alloc` and `std` features to Serde.
- Set the supported Rust version to 1.85 and test the core, alloc, and std configurations on it.

# 0.1.2 (January 6th, 2022)

FEATURES


